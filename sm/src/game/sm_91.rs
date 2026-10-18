use crate::game::sm_80::{
    InvalidInterrupt_Crash, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max6, QueueSfx3_Max6,
    QueueSfx3_Max9,
};
use crate::game::sm_84::{DisablePLMs, EnablePLMs, LoadXrayBlocks};
use crate::game::sm_86::{DisableEprojs, EnableEprojs};
use crate::game::sm_87::{DisableAnimtiles, EnableAnimtiles};
use crate::game::sm_88::SpawnHdmaObject;
use crate::game::sm_8d::{DisablePaletteFx, EnablePaletteFx, SpawnPalfxObject};
use crate::game::sm_90::{
    CallSomeSamusCode, LoadProjectilePalette, Projectile_Func7_Shinespark,
    Samus_AlignBottomWithPrevPose, Samus_GetBottom_R18, Samus_GetTop_R20, Samus_InitJump,
    Samus_InitWallJump, Samus_SetSpeedForKnockback_Y, UpdateBeamTilesAndPalette,
    WriteBeamPalette_A,
};
use crate::game::sm_94::{Samus_CollDetectChangedPose, Samus_MoveRight_NoSolidColl};
use crate::game::sm_9b::{GrappleBeamFunc_BD95, GrappleBeamFunc_BEEB};
use crate::game::sm_a0::Samus_CheckSolidEnemyColl;
use crate::game_state::Game;
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, Func_U8,
    Func_V, LongPtr, SamusPose, SpawnHdmaObject_Args, VoidP, VramWriteEntry,
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
pub use crate::types::{SamusMovementType, SamusPoseParams};
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
pub struct DemoInputObject {
    pub ptr: VoidP,
    pub pre_instr: VoidP,
    pub instr_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DemoSetDef {
    pub items: uint16,
    pub missiles: uint16,
    pub super_missiles: uint16,
    pub power_bombs: uint16,
    pub health: uint16,
    pub beams: uint16,
    pub equipped_beams_: uint16,
    pub demo_obj: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PoseEntry {
    pub new_input: uint16,
    pub cur_input: uint16,
    pub new_pose: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct XrayBlockData {
    pub value: uint16,
    pub addr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusCrystalFlashPalTable {
    pub ptr: VoidP,
    pub timer: uint16,
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
pub use crate::types::{
    Consts_8D, Ram4000, Ram4000_Backups, TileTable, TileTables, VramReadQueueEnt,
};
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
pub use crate::types::Consts_8F;
pub const addr_kDoorClosingPlmIds: Consts_8F = 59018;
pub const addr_kRoom_dede: Consts_8F = 57054;
pub const addr_kRoom_dd58: Consts_8F = 56664;
pub const addr_kRoom_cefb: Consts_8F = 52987;
pub const addr_kRoom_b457: Consts_8F = 46167;
pub const addr_kRoom_b32e: Consts_8F = 45870;
pub const addr_kRoom_a66a: Consts_8F = 42602;
pub const addr_kRoom_9804: Consts_8F = 38916;
pub const addr_kRoom_96ba: Consts_8F = 38586;
pub use crate::types::Consts_91;
pub const addr_stru_919346: Consts_91 = 37702;
pub const addr_stru_918784: Consts_91 = 34692;
pub const addr_stru_91877E: Consts_91 = 34686;
pub const addr_kDemoInstrs_LeaveDemo: Consts_91 = 34678;
pub const addr_off_91864B: Consts_91 = 34379;
pub use crate::types::Consts_9B;
pub const addr_word_9BA3A0: Consts_9B = 41888;
pub const addr_word_9BA380: Consts_9B = 41856;
pub const addr_word_9BA120: Consts_9B = 41248;
pub const addr_kSamusPalette_GravitySuit: Consts_9B = 38912;
pub const addr_kSamusPalette_VariaSuit: Consts_9B = 38176;
pub const addr_kSamusPalette_PowerSuit: Consts_9B = 37888;
pub use crate::types::{CheckEnemyColl_Result, CheckSolidEnemyColl_Result};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct XrayHdmaCtx {
    pub r22: uint16,
    pub r24: uint16,
    pub r26: uint16,
    pub r28: uint16,
    pub r30: uint16,
    pub r32: uint16,
    pub r18_avoid: uint16,
    pub dest: *mut uint16,
}
pub type XrayHdma0Func = unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ();
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Pair_R18_R20 {
    pub r18: uint16,
    pub r20: uint16,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0")
};
#[inline]
unsafe fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe fn RomPtr_RAM(mut addr: uint16_t, game: &mut Game) -> *mut uint8 {
    '_c2rust_label: {
        if (addr as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const ::core::ffi::c_char,
                51 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    return (&raw mut game.g_ram as *mut uint8).offset(addr as ::core::ffi::c_int as isize);
}
#[inline]
unsafe fn RomPtr_91(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x910000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_9B(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x9b0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_PoseEntry(mut a: uint16, game: &mut Game) -> *mut PoseEntry {
    return RomPtr(
        (0x910000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut PoseEntry;
}
#[inline]
unsafe fn get_DemoInputObject(mut a: uint16, game: &mut Game) -> *mut DemoInputObject {
    return RomPtr(
        (0x910000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut DemoInputObject;
}
#[inline]
unsafe fn get_DemoSetDef(mut a: uint16, game: &mut Game) -> *mut DemoSetDef {
    return RomPtr(
        (0x910000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut DemoSetDef;
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
pub const fnSamus_MovementHandler_Normal: ::core::ffi::c_int = 0x90a337 as ::core::ffi::c_int;
pub const fnProjPreInstr_Empty: ::core::ffi::c_int = 0x90b169 as ::core::ffi::c_int;
pub const fnSamus_MoveHandlerVerticalShinespark: ::core::ffi::c_int =
    0x90d0ab as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_Shinespark_Diag: ::core::ffi::c_int = 0x90d0d7 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_Shinespark_Horiz: ::core::ffi::c_int = 0x90d106 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_Knockback: ::core::ffi::c_int = 0x90df38 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_BombJumpStart: ::core::ffi::c_int = 0x90e025 as ::core::ffi::c_int;
pub const fnSamus_Func3: ::core::ffi::c_int = 0x90e0e6 as ::core::ffi::c_int;
pub const fnSamus_Func9: ::core::ffi::c_int = 0x90e41b as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerAlfa_Func11: ::core::ffi::c_int = 0x90e695 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerAlfa_Func12: ::core::ffi::c_int = 0x90e6c9 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerAlfa_Func13: ::core::ffi::c_int = 0x90e713 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerBeta_Func17: ::core::ffi::c_int = 0x90e725 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerBeta_Func14: ::core::ffi::c_int = 0x90e7f5 as ::core::ffi::c_int;
pub const fnSamus_Func16: ::core::ffi::c_int = 0x90e86a as ::core::ffi::c_int;
pub const fnEmptyFunction: ::core::ffi::c_int = 0x90e8cd as ::core::ffi::c_int;
pub const fnj_HandleDemoRecorder_2_0: ::core::ffi::c_int = 0x90e8d9 as ::core::ffi::c_int;
pub const fnSetContactDamageIndexAndUpdateMinimap: ::core::ffi::c_int =
    0x90e8dc as ::core::ffi::c_int;
pub const fnnullsub_152: ::core::ffi::c_int = 0x90e90e as ::core::ffi::c_int;
pub const fnSamus_InputHandler_E913: ::core::ffi::c_int = 0x90e913 as ::core::ffi::c_int;
pub const fnSamus_Func20_: ::core::ffi::c_int = 0x90e918 as ::core::ffi::c_int;
pub const fnSamus_InputHandler_E91D: ::core::ffi::c_int = 0x90e91d as ::core::ffi::c_int;
pub const fnHandleAutoJumpHack: ::core::ffi::c_int = 0x90e926 as ::core::ffi::c_int;
pub const fnSamusMovementType_Xray: ::core::ffi::c_int = 0x90e94f as ::core::ffi::c_int;
pub const fnSamusDrawHandler_Default: ::core::ffi::c_int = 0x90eb52 as ::core::ffi::c_int;
pub const fnSamus_InputHandler: ::core::ffi::c_int = 0x918000 as ::core::ffi::c_int;
pub const fnDemoPreInstr_nullsub_162: ::core::ffi::c_int = 0x9183bf as ::core::ffi::c_int;
pub const fnDemoInstr_Finish: uint32 = 9536551 as uint32;
pub const fnDemoInstr_SetPreInstr: uint32 = 9536564 as uint32;
pub const fnDemoInstr_ClearPreInstr: uint32 = 9536575 as uint32;
pub const fnDemoInstr_Goto: uint32 = 9536584 as uint32;
pub const fnDemoInstr_DecTimerAndGoto: uint32 = 9536591 as uint32;
pub const fnDemoInstr_SetTimer: uint32 = 9536601 as uint32;
pub const fnDemoInstr_Func2: uint32 = 9537020 as uint32;
pub const fnDemoPreInstr_864F: uint32 = 9537103 as uint32;
pub const fnDemoPreInstr_866A: ::core::ffi::c_int = 0x91866a as ::core::ffi::c_int;
pub const fnDemoInstr_Disable: uint32 = 9537154 as uint32;
pub const fnDemoInstr_Func3: uint32 = 9537337 as uint32;
pub const fnDemoPreInstr_CheckLeaveDemo: ::core::ffi::c_int = 0x918a9b as ::core::ffi::c_int;
pub const fnDemoPreInstr_8AB0: uint32 = 9538224 as uint32;
pub const fnXray_Func6: uint32 = 9555577 as uint32;
pub const fnXray_Func6B: uint32 = 9555643 as uint32;
pub const fnXray_Func7: uint32 = 9555766 as uint32;
pub const fnXray_Func8: uint32 = 9555774 as uint32;
pub const fnXray_Func9: ::core::ffi::c_int = 0x91cf4e as ::core::ffi::c_int;
pub const fnXray_Func10: uint32 = 9555810 as uint32;
pub const fnXray_Func11: ::core::ffi::c_int = 0x91cf6f as ::core::ffi::c_int;
pub const fnGrappleBeamFunc_Inactive: ::core::ffi::c_int = 0x9bc4f0 as ::core::ffi::c_int;
static kSamusInputHandlers: [Option<Func_V>; 28] = unsafe {
    [
        Some(Samus_Input_00_Standing as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_01_Running as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_02_NormalJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_03_SpinJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_04_MorphBallOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_05_Crouching as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_06_Falling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_07_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_08_MorphBallFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_09_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0A_KnockbackOrCrystalFlashEnding as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0B_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0C_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0D_Unused as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0E_TurningAroundOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_0F_CrouchingEtcTransition as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_10_Moonwalking as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_11_SpringBallOnGround as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_12_SpringBallInAir as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_13_SpringBallFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_14_WallJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_15_RanIntoWall as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_16_Grappling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_17_TurningAroundJumping as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_18_TurningAroundFalling as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_19_DamageBoost as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_1A_GrabbedByDraygon as unsafe fn(&mut Game) -> ()),
        Some(Samus_Input_1B_ShinesparkEtc as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_InputHandler(game: &mut Game) {
    kSamusInputHandlers[*(&raw mut game.g_ram as *mut uint8)
        .offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Samus_Input_00_Standing(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        Samus_LookupTransitionTable(game);
    }
}
pub unsafe fn Samus_Input_01_Running(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_02_NormalJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_03_SpinJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_04_MorphBallOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_07_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_05_Crouching(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) == 0 {
            let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh45 = (*fresh45 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
            let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh46 = (*fresh46 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn Samus_Input_06_Falling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_08_MorphBallFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_09_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0A_KnockbackOrCrystalFlashEnding(game: &mut Game) {
    Samus_LookupTransitionTable(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_0A_KnockbackOrCrystalFlashEnding as ::core::ffi::c_int
    {
        Samus_InitJump(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_Input_0B_Unused(game: &mut Game) {}
pub unsafe fn Samus_Input_0C_Unused(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_0D_Unused(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_0E_TurningAroundOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_0F_CrouchingEtcTransition(game: &mut Game) {}
pub unsafe fn Samus_Input_10_Moonwalking(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_11_SpringBallOnGround(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_12_SpringBallInAir(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_13_SpringBallFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_14_WallJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_15_RanIntoWall(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        Samus_Func20(game);
    } else {
        Samus_LookupTransitionTable(game);
    };
}
pub unsafe fn Samus_Input_16_Grappling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_17_TurningAroundJumping(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_18_TurningAroundFalling(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_19_DamageBoost(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_1A_GrabbedByDraygon(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_Input_1B_ShinesparkEtc(game: &mut Game) {
    Samus_LookupTransitionTable(game);
}
pub unsafe fn Samus_LookupTransitionTable(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let mut pair: Pair_R18_R20 = TranslateCustomControllerBindingsToDefault(game);
        let mut pe: *mut PoseEntry = get_PoseEntry(
            *(game.g_rom.offset(
                (((0x919ee2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x919ee2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ),
            game,
        );
        if (*pe).new_input as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            return;
        }
        loop {
            if pair.r18 as ::core::ffi::c_int & (*pe).new_input as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && pair.r20 as ::core::ffi::c_int & (*pe).cur_input as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                if (*pe).new_pose as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa28 as ::core::ffi::c_int as isize)
                        as *mut uint16) = (*pe).new_pose;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa56 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
                return;
            }
            pe = pe.offset(1);
            if !((*pe).new_input as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int) {
                break;
            }
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa18 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_Pose_CancelGrapple(game);
}
pub unsafe fn TranslateCustomControllerBindingsToDefault(game: &mut Game) -> Pair_R18_R20 {
    let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Left as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)) as uint16;
    let mut r20: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Left as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = (r18 as ::core::ffi::c_int | kButton_X as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = (r18 as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = (r18 as ::core::ffi::c_int | kButton_B as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = (r18 as ::core::ffi::c_int | kButton_Y as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_L as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            r18 = (r18 as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int) as uint16;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_L as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
    {
        r18 = (r18 as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int) as uint16;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r20 = (r20 as ::core::ffi::c_int | kButton_X as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r20 = (r20 as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r20 = (r20 as ::core::ffi::c_int | kButton_B as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r20 = (r20 as ::core::ffi::c_int | kButton_Y as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_L as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            r20 = (r20 as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int) as uint16;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & v1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_L as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
    {
        r20 = (r20 as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int) as uint16;
    }
    return Pair_R18_R20 {
        r18: !(r18 as ::core::ffi::c_int) as uint16,
        r20: !(r20 as ::core::ffi::c_int) as uint16,
    };
}
pub unsafe fn Samus_Pose_CancelGrapple(game: &mut Game) {
    if Samus_Pose_Func2(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .new_pose_unless_buttons as ::core::ffi::c_int
            == 255 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .new_pose_unless_buttons as uint16;
    };
}
static kSamus_Pose_Func2_Tab: [uint8; 28] = [
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Samus_Pose_Func2(game: &mut Game) -> uint8 {
    let mut v0: uint16 = kSamus_Pose_Func2_Tab
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        as uint16;
    if !(v0 as ::core::ffi::c_int != 1 as ::core::ffi::c_int) {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        {
            v0 = 2 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            return 1 as uint8;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
    return 0 as uint8;
}
pub unsafe fn EnableDemoInput(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa88 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh32 = (*fresh32 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableDemoInput(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa88 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh1 = (*fresh1 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ResetDemoData(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn LoadDemoInputObject(mut a: uint16, mut j: uint16, game: &mut Game) {
    let mut dio: *mut DemoInputObject = ::core::ptr::null_mut::<DemoInputObject>();
    *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    dio = get_DemoInputObject(j, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = (*dio).pre_instr as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
        as *mut uint16) = (*dio).instr_ptr as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn DemoObjectInputHandler(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            ProcessDemoInputObject(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa8c as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa8e as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa86 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa86 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    }
}
pub unsafe fn CallDemoPreInstr(mut ea: uint32, game: &mut Game) {
    match ea {
        9536447 => return,
        9537103 => {
            DemoPreInstr_864F(game);
            return;
        }
        9537130 => {
            DemoPreInstr_866A(game);
            return;
        }
        9538203 => {
            DemoPreInstr_CheckLeaveDemo(game);
            return;
        }
        9538224 => {
            DemoPreInstr_8AB0(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallDemoInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        9536551 => return DemoInstr_Finish(k, j, game),
        9536564 => return DemoInstr_SetPreInstr(k, j, game),
        9536575 => return DemoInstr_ClearPreInstr(k, j, game),
        9536584 => return DemoInstr_Goto(k, j, game),
        9536591 => return DemoInstr_DecTimerAndGoto(k, j, game),
        9536601 => return DemoInstr_SetTimer(k, j, game),
        9537020 => return DemoInstr_Func2(k, j, game),
        9537154 => return DemoInstr_Disable(k, j, game),
        9537337 => return DemoInstr_Func3(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn ProcessDemoInputObject(game: &mut Game) {
    CallDemoPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x910000 as ::core::ffi::c_int) as uint32,
        game,
    );
    let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7c as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh30 = (*fresh30).wrapping_sub(1);
    if *fresh30 == 0 {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v1: *mut uint16 = ::core::ptr::null_mut::<uint16>();
        loop {
            v1 = RomPtr_91(v0 as uint16_t, game) as *mut uint16;
            let mut v2: uint16 = *v1;
            if *v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            v0 = CallDemoInstr(
                (v2 as ::core::ffi::c_int | 0x910000 as ::core::ffi::c_int) as uint32,
                0 as uint16,
                (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                game,
            );
            if v0 == 0 {
                return;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = *v1;
        let mut v3: *const uint8 = RomPtr_91(v0 as uint16_t, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(v3.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(v3.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DemoInstr_Finish(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 0 as uint16;
}
pub unsafe fn DemoInstr_SetPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = *(RomPtr_91(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DemoInstr_ClearPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0x8447 as uint16;
    return j;
}
pub unsafe fn DemoInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_91(j as uint16_t, game) as *mut uint16);
}
pub unsafe fn DemoInstr_DecTimerAndGoto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa80 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_sub(1);
    if *fresh0 != 0 {
        return DemoInstr_Goto(k, j, game);
    } else {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn DemoInstr_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
        as *mut uint16) = *(RomPtr_91(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DemoInstr_Func2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    DisableDemoInput(game);
    return j;
}
pub unsafe fn DemoPreInstr_864F(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 178 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x91866a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x8623 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
}
pub unsafe fn DemoPreInstr_866A(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize)
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9183bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = addr_off_91864B as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
}
pub unsafe fn DemoInstr_Disable(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    DisableDemoInput(game);
    return j;
}
pub unsafe fn UNUSED_DemoInstr_Func4(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
    DisableDemoInput(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DemoInstr_Func3(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
    DisableDemoInput(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
static kDemoSetFuncPtrs_0: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_3 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_1: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_4 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_2: [Option<Func_V>; 6] = unsafe {
    [
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_5 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs_3: [Option<Func_V>; 5] = unsafe {
    [
        Some(DemoSetFunc_6 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_2 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_1 as unsafe fn(&mut Game) -> ()),
        Some(DemoSetFunc_7 as unsafe fn(&mut Game) -> ()),
    ]
};
static kDemoSetFuncPtrs: [&'static [Option<Func_V>]; 4] = [
    &kDemoSetFuncPtrs_0,
    &kDemoSetFuncPtrs_1,
    &kDemoSetFuncPtrs_2,
    &kDemoSetFuncPtrs_3,
];
pub unsafe fn LoadDemoData(game: &mut Game) {
    let mut DemoSetDef: *mut DemoSetDef = ::core::ptr::null_mut::<DemoSetDef>();
    let mut v0: uint16 = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1f57 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0x918885 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x918885 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1f55 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int) as uint16;
    DemoSetDef = get_DemoSetDef(v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).items;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9a4 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).missiles;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c8 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).super_missiles;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9cc as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).power_bombs;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9d0 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).health;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c4 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a8 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).beams;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*DemoSetDef).equipped_beams_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    ResetDemoData(game);
    EnableDemoInput(game);
    let mut demo_obj: uint16 = (*get_DemoSetDef(v0, game)).demo_obj as uint16;
    LoadDemoInputObject(demo_obj, demo_obj, game);
    (*kDemoSetFuncPtrs[*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1f55 as ::core::ffi::c_int as isize) as *mut uint16)
        as usize]
        .as_ptr()
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f57 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .expect("non-null function pointer")(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e6c9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e7f5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_LoadSuitPalette(game);
    UpdateBeamTilesAndPalette(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9aa as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Up as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ac as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Down as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ae as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Left as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b0 as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Right as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_X as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_A as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_B as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Y as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ba as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_Select as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_R as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize)
        as *mut uint16) = kButton_L as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdfa as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdfc as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn DemoSetFunc_0(game: &mut Game) {
    MakeSamusFaceForward(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DemoSetFunc_3(game: &mut Game) {
    DemoSetFunc_Common(0x1f as uint16, game);
}
pub unsafe fn DemoSetFunc_7(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) = 20 as uint16;
    DemoSetFunc_2(game);
}
pub unsafe fn DemoSetFunc_2(game: &mut Game) {
    DemoSetFunc_Common(2 as uint16, game);
}
pub unsafe fn DemoSetFunc_4(game: &mut Game) {
    DemoSetFunc_Common(0x2a as uint16, game);
}
pub unsafe fn DemoSetFunc_1(game: &mut Game) {
    DemoSetFunc_Common(1 as uint16, game);
}
pub unsafe fn DemoSetFunc_Common(mut a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DemoSetFunc_5(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Projectile_Func7_Shinespark(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = kPose_CD_FaceR_Shinespark_Diag as ::core::ffi::c_int as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}
pub unsafe fn DemoSetFunc_6(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Projectile_Func7_Shinespark(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = kPose_CA_FaceL_Shinespark_Horiz as ::core::ffi::c_int as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
}
pub unsafe fn DemoPreInstr_CheckLeaveDemo(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kGameState_44_TransitionFromDemo as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = addr_kDemoInstrs_LeaveDemo as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
}
pub unsafe fn DemoPreInstr_8AB0(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_1A_GrabbedByDraygon as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x918a9b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = addr_stru_919346 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
}
unsafe fn XrayFillUp(mut dest: *mut uint16, mut fill_with: uint16, mut v1: uint16) {
    loop {
        *dest.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = fill_with;
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !(v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0) {
            break;
        }
    }
}
unsafe fn XrayFillDown(mut dest: *mut uint16, mut fill_with: uint16, mut v1: uint16) {
    loop {
        *dest.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = fill_with;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !((v1 as ::core::ffi::c_int) < 460 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn XrayFillUpRamped(
    mut dest: *mut uint16,
    mut pos: ::core::ffi::c_int,
    mut left: uint32,
    mut left_step: uint32,
    mut right: uint32,
    mut right_step: uint32,
    mut adjust: ::core::ffi::c_int,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let mut left_val: uint32 = if left & !(0xffff as ::core::ffi::c_int) as uint32 != 0 {
            (if (left as int32) < 0 as int32 {
                0 as ::core::ffi::c_int
            } else {
                0xffff as ::core::ffi::c_int
            }) as uint32
        } else {
            left
        };
        let mut right_val: uint32 = if right & !(0xffff as ::core::ffi::c_int) as uint32 != 0 {
            (if (right as int32) < 0 as int32 {
                0 as ::core::ffi::c_int
            } else {
                0xffff as ::core::ffi::c_int
            }) as uint32
        } else {
            right
        };
        let mut finished: bool = left & !(0xffff as ::core::ffi::c_int) as uint32 != 0
            && right & !(0xffff as ::core::ffi::c_int) as uint32 != 0
            && (left_step ^ left | right_step ^ right) >> 31 as ::core::ffi::c_int == 0 as uint32;
        *dest.offset((pos >> 1 as ::core::ffi::c_int) as isize) =
            (if (right as int32) < 0 as int32 || left as int32 >= 0x10000 as int32 {
                0xff as uint32
            } else {
                left_val >> 8 as ::core::ffi::c_int
                    | right_val & !(0xff as ::core::ffi::c_int) as uint32
            }) as uint16;
        if finished {
            pos += adjust;
            loop {
                pos -= 2 as ::core::ffi::c_int;
                if !(pos >= 0 as ::core::ffi::c_int) {
                    break;
                }
                *dest.offset((pos >> 1 as ::core::ffi::c_int) as isize) = 0xff as uint16;
            }
            return;
        }
        pos -= 2 as ::core::ffi::c_int;
        if !(pos >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn XrayFillDownRamped(
    mut dest: *mut uint16,
    mut pos: ::core::ffi::c_int,
    mut left: uint32,
    mut left_step: uint32,
    mut right: uint32,
    mut right_step: uint32,
    mut adjust: ::core::ffi::c_int,
) {
    loop {
        left = left.wrapping_add(left_step);
        right = right.wrapping_add(right_step);
        let mut left_val: uint32 = if left & !(0xffff as ::core::ffi::c_int) as uint32 != 0 {
            (if (left as int32) < 0 as int32 {
                0 as ::core::ffi::c_int
            } else {
                0xffff as ::core::ffi::c_int
            }) as uint32
        } else {
            left
        };
        let mut right_val: uint32 = if right & !(0xffff as ::core::ffi::c_int) as uint32 != 0 {
            (if (right as int32) < 0 as int32 {
                0 as ::core::ffi::c_int
            } else {
                0xffff as ::core::ffi::c_int
            }) as uint32
        } else {
            right
        };
        let mut finished: bool = left & !(0xffff as ::core::ffi::c_int) as uint32 != 0
            && right & !(0xffff as ::core::ffi::c_int) as uint32 != 0
            && (left_step ^ left | right_step ^ right) >> 31 as ::core::ffi::c_int == 0 as uint32;
        *dest.offset((pos >> 1 as ::core::ffi::c_int) as isize) =
            (if (right as int32) < 0 as int32 || left as int32 >= 0x10000 as int32 {
                0xff as uint32
            } else {
                left_val >> 8 as ::core::ffi::c_int
                    | right_val & !(0xff as ::core::ffi::c_int) as uint32
            }) as uint16;
        if finished {
            pos += adjust;
            loop {
                pos += 2 as ::core::ffi::c_int;
                if !(pos < 460 as ::core::ffi::c_int) {
                    break;
                }
                *dest.offset((pos >> 1 as ::core::ffi::c_int) as isize) = 0xff as uint16;
            }
            return;
        }
        pos += 2 as ::core::ffi::c_int;
        if !(pos < 460 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn XrayHdmaFunc_BeamAimedR(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut p: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 2 as ::core::ffi::c_int,
        ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
        (*ctx).r30 as uint32,
        65535 as uint32,
        1 as uint32,
        2 as ::core::ffi::c_int,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
        (*ctx).r32 as uint32,
        65535 as uint32,
        1 as uint32,
        -(2 as ::core::ffi::c_int),
    );
}
unsafe fn XrayHdmaFunc_BeamAimedL(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut t: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    XrayFillUpRamped(
        (*ctx).dest,
        t - 2 as ::core::ffi::c_int,
        0 as uint32,
        -(1 as ::core::ffi::c_int) as uint32,
        ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
        -((*ctx).r32 as ::core::ffi::c_int) as uint32,
        2 as ::core::ffi::c_int,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        t,
        0 as uint32,
        -(1 as ::core::ffi::c_int) as uint32,
        ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
        -((*ctx).r30 as ::core::ffi::c_int) as uint32,
        -(2 as ::core::ffi::c_int),
    );
}
unsafe fn XrayHdmaFunc_BeamAimedU(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut p: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    if (*ctx).r26 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as ::core::ffi::c_int,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r30 as uint32,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r32 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else if (*ctx).r28 as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as ::core::ffi::c_int,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r32 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            p - 2 as ::core::ffi::c_int,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            2 as ::core::ffi::c_int,
        );
    }
    XrayFillDown((*ctx).dest, 0xff as uint16, p as uint16);
}
unsafe fn XrayHdmaFunc_BeamAimedD(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut p: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    if (*ctx).r28 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillDownRamped(
            (*ctx).dest,
            p,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r32 as uint32,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r30 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else if (*ctx).r26 as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillDownRamped(
            (*ctx).dest,
            p,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            ((*ctx).r22 as ::core::ffi::c_int - 0x10000 as ::core::ffi::c_int) as uint32,
            (*ctx).r30 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else {
        XrayFillDownRamped(
            (*ctx).dest,
            p,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            ((*ctx).r22 as ::core::ffi::c_int + 0x10000 as ::core::ffi::c_int) as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            0 as ::core::ffi::c_int,
        );
    }
    XrayFillUp(
        (*ctx).dest,
        0xff as uint16,
        (p - 2 as ::core::ffi::c_int) as uint16,
    );
}
unsafe fn XrayHdmaFunc_BeamHoriz(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * ((*ctx).r24 as ::core::ffi::c_int - 1 as ::core::ffi::c_int))
        as uint16;
    *(*ctx)
        .dest
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0xff00 as uint16;
    XrayFillUp(
        (*ctx).dest,
        0xff as uint16,
        (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
    );
    XrayFillDown(
        (*ctx).dest,
        0xff as uint16,
        (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedR(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut p: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    *(*ctx)
        .dest
        .offset((p - 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = ((*ctx).r22
        as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int
        | 0xff00 as ::core::ffi::c_int)
        as uint16;
    XrayFillUpRamped(
        (*ctx).dest,
        p - 4 as ::core::ffi::c_int,
        (*ctx).r22 as uint32,
        (*ctx).r30 as uint32,
        65535 as uint32,
        1 as uint32,
        2 as ::core::ffi::c_int,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        p,
        (*ctx).r22 as uint32,
        (*ctx).r32 as uint32,
        65535 as uint32,
        1 as uint32,
        -(2 as ::core::ffi::c_int),
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedL(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    *(*ctx)
        .dest
        .offset((v0 - 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (((*ctx).r22 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint16;
    XrayFillUpRamped(
        (*ctx).dest,
        v0 - 4 as ::core::ffi::c_int,
        0 as uint32,
        -(1 as ::core::ffi::c_int) as uint32,
        (*ctx).r22 as uint32,
        -((*ctx).r32 as ::core::ffi::c_int) as uint32,
        2 as ::core::ffi::c_int,
    );
    XrayFillDownRamped(
        (*ctx).dest,
        v0,
        0 as uint32,
        -(1 as ::core::ffi::c_int) as uint32,
        (*ctx).r22 as uint32,
        -((*ctx).r30 as ::core::ffi::c_int) as uint32,
        -(2 as ::core::ffi::c_int),
    );
}
unsafe fn XrayHdmaOnScreen_BeamAimedU(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    *(*ctx)
        .dest
        .offset((v0 - 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (((*ctx).r22 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
            * 0x101 as ::core::ffi::c_int) as uint16;
    if (*ctx).r26 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as ::core::ffi::c_int,
            (*ctx).r22 as uint32,
            (*ctx).r30 as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r32 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else if (*ctx).r28 as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as ::core::ffi::c_int,
            (*ctx).r22 as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r32 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else {
        XrayFillUpRamped(
            (*ctx).dest,
            v0 - 4 as ::core::ffi::c_int,
            (*ctx).r22 as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            (*ctx).r22 as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            2 as ::core::ffi::c_int,
        );
    }
    XrayFillDown((*ctx).dest, 0xff as uint16, v0 as uint16);
}
unsafe fn XrayHdmaOnScreen_BeamAimedD(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*ctx).r24 as ::core::ffi::c_int;
    *(*ctx)
        .dest
        .offset((v0 - 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (((*ctx).r22 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
            * 0x101 as ::core::ffi::c_int) as uint16;
    if (*ctx).r28 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillDownRamped(
            (*ctx).dest,
            v0,
            (*ctx).r22 as uint32,
            (*ctx).r32 as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r30 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else if (*ctx).r26 as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        XrayFillDownRamped(
            (*ctx).dest,
            v0,
            (*ctx).r22 as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            (*ctx).r22 as uint32,
            (*ctx).r30 as uint32,
            0 as ::core::ffi::c_int,
        );
    } else {
        XrayFillDownRamped(
            (*ctx).dest,
            v0,
            (*ctx).r22 as uint32,
            -((*ctx).r32 as ::core::ffi::c_int) as uint32,
            (*ctx).r22 as uint32,
            -((*ctx).r30 as ::core::ffi::c_int) as uint32,
            0 as ::core::ffi::c_int,
        );
    }
    XrayFillUp(
        (*ctx).dest,
        0xff as uint16,
        (v0 - 4 as ::core::ffi::c_int) as uint16,
    );
}
unsafe fn XrayHdmaOnScreen_BeamHoriz(mut ctx: *mut XrayHdmaCtx, game: &mut Game) {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * ((*ctx).r24 as ::core::ffi::c_int - 1 as ::core::ffi::c_int))
        as uint16;
    let mut t: uint32 = ((*ctx).r22 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint32;
    *(*ctx)
        .dest
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (if (*ctx).r18_avoid as ::core::ffi::c_int == 64 as ::core::ffi::c_int {
            t | 0xff00 as uint32
        } else {
            t << 8 as ::core::ffi::c_int
        }) as uint16;
    XrayFillUp(
        (*ctx).dest,
        0xff as uint16,
        (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
    );
    XrayFillDown(
        (*ctx).dest,
        0xff as uint16,
        (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
    );
}
pub unsafe fn CalculateXrayHdmaTableInner(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut off_screen: bool,
    mut dest_addr: *mut uint16,
    game: &mut Game,
) {
    let mut v3: ::core::ffi::c_int = 0;
    let mut ctx: [XrayHdmaCtx; 1] = [XrayHdmaCtx {
        r22: 0,
        r24: 0,
        r26: 0,
        r28: 0,
        r30: 0,
        r32: 0,
        r18_avoid: 0,
        dest: ::core::ptr::null_mut::<uint16>(),
    }; 1];
    let ref mut fresh28 = (*(&raw mut ctx as *mut XrayHdmaCtx)).dest;
    *fresh28 = dest_addr;
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r18_avoid = r18;
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r22 =
        ((k as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r24 = j;
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 =
        (r18 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
    let ref mut fresh29 = (*(&raw mut ctx as *mut XrayHdmaCtx)).r26;
    *fresh29 = (*fresh29 as ::core::ffi::c_int
        + if (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            0x100 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 =
        (r20 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    if r20 as ::core::ffi::c_int + r18 as ::core::ffi::c_int - 257 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 =
            (r20 as ::core::ffi::c_int + r18 as ::core::ffi::c_int - 256 as ::core::ffi::c_int)
                as uint16;
    }
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r30 = *(game.g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (if (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
            } else {
                (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int
            }) as isize,
        );
    (*(&raw mut ctx as *mut XrayHdmaCtx)).r32 = *(game.g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (if (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 as ::core::ffi::c_int
            } else {
                (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int
            }) as isize,
        );
    if r20 == 0
        && (r18 as ::core::ffi::c_int == 64 as ::core::ffi::c_int
            || r18 as ::core::ffi::c_int == 192 as ::core::ffi::c_int)
    {
        v3 = 8 as ::core::ffi::c_int;
    } else if (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 4 as ::core::ffi::c_int;
        } else if (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 0 as ::core::ffi::c_int;
        } else {
            v3 = 2 as ::core::ffi::c_int;
        }
    } else if (*(&raw mut ctx as *mut XrayHdmaCtx)).r26 as ::core::ffi::c_int
        - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v3 = 2 as ::core::ffi::c_int;
    } else if (*(&raw mut ctx as *mut XrayHdmaCtx)).r28 as ::core::ffi::c_int
        - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v3 = 6 as ::core::ffi::c_int;
    } else {
        v3 = 4 as ::core::ffi::c_int;
    }
    if off_screen {
        static kXrayHdmaFuncs: [Option<XrayHdma0Func>; 5] = unsafe {
            [
                Some(XrayHdmaFunc_BeamAimedR as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaFunc_BeamAimedU as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaFunc_BeamAimedD as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaFunc_BeamAimedL as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaFunc_BeamHoriz as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
            ]
        };
        kXrayHdmaFuncs[(v3 >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ctx as *mut XrayHdmaCtx, game);
    } else {
        static kXrayHdmaOnScreen_Funcs: [Option<XrayHdma0Func>; 5] = unsafe {
            [
                Some(XrayHdmaOnScreen_BeamAimedR as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaOnScreen_BeamAimedU as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaOnScreen_BeamAimedD as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaOnScreen_BeamAimedL as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
                Some(XrayHdmaOnScreen_BeamHoriz as unsafe fn(*mut XrayHdmaCtx, &mut Game) -> ()),
            ]
        };
        kXrayHdmaOnScreen_Funcs[(v3 >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ctx as *mut XrayHdmaCtx, game);
    };
}
pub unsafe fn XrayRunHandler(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        if Xray_Initialize(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            SpawnHdmaObject(
                0x91 as uint8,
                game.g_rom.offset(
                    (((0x91caf2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91caf2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut SpawnHdmaObject_Args
                    as *const ::core::ffi::c_void,
                game,
            );
        }
    }
}
pub unsafe fn Xray_SetupStage1_FreezeTimeBackup(mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as uint8 as ::core::ffi::c_int as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as uint8 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as uint8 as ::core::ffi::c_int as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as uint8 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as uint8 as ::core::ffi::c_int as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize);
}
pub unsafe fn Xray_SetupStage2_ReadBg1_2ndScreen(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x360 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .vram_target as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
        (((*(&raw mut game.g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .addr as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut VoidP) =
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 0x2800 as ::core::ffi::c_long) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .bank)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .size as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v0 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Xray_SetupStage3_ReadBg1_1stScreen(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x360 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .vram_target as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .addr as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut VoidP) =
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 0x2000 as ::core::ffi::c_long) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .bank)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .size as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v0 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Xray_SetupStage4(game: &mut Game) {
    let mut r24: uint16 = (4 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x91f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0xf0 as ::core::ffi::c_int)) as uint16;
    r24 = (r24 as ::core::ffi::c_int
        + ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x91d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0xf0 as ::core::ffi::c_int)
            >> 3 as ::core::ffi::c_int)) as uint16;
    r24 = (r24 as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x91d as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                & 0x100 as ::core::ffi::c_int)) as uint16;
    let mut r22: uint16 = 0 as uint16;
    let mut m: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        let mut r26: uint16 = (r24 as ::core::ffi::c_int & 0x7e0 as ::core::ffi::c_int) as uint16;
        let mut r28: uint16 = (r24 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
        let mut r30: uint16 = 0 as uint16;
        loop {
            if r30 as ::core::ffi::c_int + r28 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                r26 = (r26 as ::core::ffi::c_int + 1024 as ::core::ffi::c_int
                    & 0x7e0 as ::core::ffi::c_int) as uint16;
                r28 = 0 as uint16;
                r30 = 0 as uint16;
            }
            let mut v0: uint16 = (r30 as ::core::ffi::c_int
                + r28 as ::core::ffi::c_int
                + r26 as ::core::ffi::c_int) as uint16;
            let mut v11: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as ::core::ffi::c_int + 4096 as ::core::ffi::c_int) as usize];
            let mut v10: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as ::core::ffi::c_int + 4097 as ::core::ffi::c_int) as usize];
            let mut v9: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as ::core::ffi::c_int + 4128 as ::core::ffi::c_int) as usize];
            let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[(v0 as ::core::ffi::c_int + 4129 as ::core::ffi::c_int) as usize];
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(33 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(r22 as ::core::ffi::c_int as isize) as *mut uint16) = v1;
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(32 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(r22 as ::core::ffi::c_int as isize) as *mut uint16) = v9;
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(r22 as ::core::ffi::c_int as isize) as *mut uint16) = v10;
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps as *mut uint16 as *mut uint8)
                .offset(r22 as ::core::ffi::c_int as isize) as *mut uint16) = v11;
            r22 = (r22 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            r30 = (r30 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        Xray_SetupStage4_Func1(r22, r26, r28, r30, game);
        r24 = ((r24 as ::core::ffi::c_int & 0x400 as ::core::ffi::c_int)
            + (r24 as ::core::ffi::c_int + 64 as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int))
            as uint16;
        r22 = (r22 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        m -= 1;
        if !(m != 0) {
            break;
        }
    }
    '_c2rust_label: {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 12 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"(uint16)(layer1_y_pos >> 12) == 0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_91.c\0" as *const u8 as *const ::core::ffi::c_char,
                921 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut r34: uint16 = (Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    ) as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)) as uint16;
    r22 = 0 as uint16;
    m = 16 as ::core::ffi::c_int;
    loop {
        Xray_SetupStage4_Func2(r22, r34, game);
        let mut n_0: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        let mut r36: uint16 = r34;
        loop {
            Xray_SetupStage4_Func3(n_0 as uint16, r22, r36, game);
            r22 = (r22 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            r36 = (r36 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            n_0 -= 1;
            if !(n_0 != 0) {
                break;
            }
        }
        let mut v12: uint16 = r22;
        r22 = (r22 as ::core::ffi::c_int + 1984 as ::core::ffi::c_int) as uint16;
        Xray_SetupStage4_Func3(0 as uint16, r22, r36, game);
        r22 = (v12 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        r34 = (r34 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        m -= 1;
        if !(m != 0) {
            break;
        }
    }
    LoadXrayBlocks(game);
    let mut v7: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x360 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .vram_target as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .addr as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut VoidP) = 0x5000 as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .bank)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .size as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v7 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Xray_SetupStage4_Func1(
    mut dst_r22: uint16,
    mut r26: uint16,
    mut r28: uint16,
    mut r30: uint16,
    game: &mut Game,
) {
    let mut v0: int16 = 0;
    let mut R32: uint16 = r26;
    v0 = (r30 as ::core::ffi::c_int + r28 as ::core::ffi::c_int) as int16;
    if r30 as ::core::ffi::c_int + r28 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        R32 = (R32 as ::core::ffi::c_int + 1024 as ::core::ffi::c_int & 0x7e0 as ::core::ffi::c_int)
            as uint16;
        v0 = 0 as int16;
    }
    let mut v1: uint16 = (R32 as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut v6: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000))
        .xray_tilemaps[(v1 as ::core::ffi::c_int + 4096 as ::core::ffi::c_int) as usize];
    let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000))
        .xray_tilemaps[(v1 as ::core::ffi::c_int + 4097 as ::core::ffi::c_int) as usize];
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000))
        .xray_tilemaps[(v1 as ::core::ffi::c_int + 4128 as ::core::ffi::c_int) as usize];
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000))
        .xray_tilemaps[(v1 as ::core::ffi::c_int + 4129 as ::core::ffi::c_int) as usize];
    let mut v2: uint16 = (dst_r22 as ::core::ffi::c_int + 1984 as ::core::ffi::c_int) as uint16;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = v3;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = v4;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = v5;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000))
        .xray_tilemaps as *mut uint16 as *mut uint8)
        .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = v6;
}
unsafe fn Xray_GetXrayedBlock(mut k: uint16, game: &mut Game) -> *const uint8 {
    let mut value: uint16 = 0;
    let mut bts: uint16 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as uint16;
    let mut r40: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x10002 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(k as isize) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int) as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        value = (*(game.g_rom.offset(
            (((0x91d2d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91d2d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut XrayBlockData)
            .offset(i as isize))
        .value;
        if value as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            break;
        }
        if value as ::core::ffi::c_int == r40 as ::core::ffi::c_int {
            let mut p: *const uint8 = RomPtr_91(
                (*(game.g_rom.offset(
                    (((0x91d2d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91d2d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut XrayBlockData)
                    .offset(i as isize))
                .addr as uint16_t,
                game,
            );
            loop {
                value = *(p as *mut uint16);
                if value as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
                    break;
                }
                if value as ::core::ffi::c_int == 0xff00 as ::core::ffi::c_int
                    || value as ::core::ffi::c_int == bts as ::core::ffi::c_int
                {
                    return RomPtr_91(
                        *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16),
                        game,
                    );
                }
                p = p.offset(4 as ::core::ffi::c_int as isize);
            }
            return ::core::ptr::null::<uint8>();
        }
        i += 1;
    }
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn Xray_SetupStage4_Func2(mut dst_r22: uint16, mut r34: uint16, game: &mut Game) {
    if r34 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return;
    }
    let mut jp: *const uint8 = Xray_GetXrayedBlock(
        (r34 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    let mut dst: *mut uint16 =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16 as *mut uint8)
            .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16;
    if jp.is_null() {
        return;
    }
    if *(jp as *mut uint16) as ::core::ffi::c_int
        != 0x91cf4e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if *(jp as *mut uint16) as ::core::ffi::c_int
            != 0x91cf6f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            return;
        }
        let mut src: *mut TileTable = (&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa000 as ::core::ffi::c_int as isize)
            as *mut TileTables))
            .tables as *mut TileTable)
            .offset(
                *(jp.offset(8 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
                    as isize,
            );
        *dst.offset(64 as ::core::ffi::c_int as isize) = (*src).top_left;
        *dst.offset(65 as ::core::ffi::c_int as isize) = (*src).top_right;
        *dst.offset(96 as ::core::ffi::c_int as isize) = (*src).bottom_left;
        *dst.offset(97 as ::core::ffi::c_int as isize) = (*src).bottom_right;
    }
    let mut src_0: *mut TileTable = (&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables as *mut TileTable)
        .offset(
            *(jp.offset(4 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
                as isize,
        );
    *dst.offset(0 as ::core::ffi::c_int as isize) = (*src_0).top_left;
    *dst.offset(1 as ::core::ffi::c_int as isize) = (*src_0).top_right;
    *dst.offset(32 as ::core::ffi::c_int as isize) = (*src_0).bottom_left;
    *dst.offset(33 as ::core::ffi::c_int as isize) = (*src_0).bottom_right;
}
unsafe fn Xray_CombinedMove(
    mut dst_r22: uint16,
    mut r36: uint16,
    mut which_dir: bool,
    game: &mut Game,
) {
    let mut y: uint16 = (r36 as ::core::ffi::c_int
        / *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut x: uint16 = (r36 as ::core::ffi::c_int
        % *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut r48: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x10002 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(r36 as isize) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int) as uint16;
    let mut step: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(r36 as isize) as int8 as ::core::ffi::c_int;
    if step != 0 {
        let mut variable: *mut uint16 = if which_dir as ::core::ffi::c_int != 0 {
            &raw mut y
        } else {
            &raw mut x
        };
        loop {
            if ((*variable as ::core::ffi::c_int + step) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                Xray_Func13(dst_r22, 0xff as uint16, game);
                return;
            }
            *variable = (*variable as ::core::ffi::c_int + step) as uint16;
            let mut pos: ::core::ffi::c_int = y as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + x as ::core::ffi::c_int;
            step = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(pos as isize) as int8 as ::core::ffi::c_int;
            r48 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x10002 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(pos as isize) as ::core::ffi::c_int
                & 0xf000 as ::core::ffi::c_int) as uint16;
            if r48 as ::core::ffi::c_int == 0xd000 as ::core::ffi::c_int {
                variable = &raw mut y;
            } else {
                if r48 as ::core::ffi::c_int != 0x5000 as ::core::ffi::c_int {
                    break;
                }
                if step < 0 as ::core::ffi::c_int {
                    variable = &raw mut x;
                }
            }
        }
    }
    if (*(game.g_rom.offset(
        (((0x91d2d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91d2d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut XrayBlockData)
        .offset(1 as ::core::ffi::c_int as isize))
    .value as ::core::ffi::c_int
        == r48 as ::core::ffi::c_int
    {
        let mut t: *mut uint16 = RomPtr_91(
            (*(game.g_rom.offset(
                (((0x91d2d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91d2d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut XrayBlockData)
                .offset(1 as ::core::ffi::c_int as isize))
            .addr as uint16_t,
            game,
        ) as *mut uint16;
        while *t.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0xffff as ::core::ffi::c_int
        {
            if *t.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xff00 as ::core::ffi::c_int
                || *t.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == step & 0xff as ::core::ffi::c_int
            {
                Xray_Func12(
                    dst_r22,
                    RomPtr_91(
                        (*t.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int) as uint16_t,
                        game,
                    ),
                    game,
                );
                return;
            }
            t = t.offset(2 as ::core::ffi::c_int as isize);
        }
    }
}
unsafe fn Xray_Func7(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
}
unsafe fn Xray_Func8(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        Xray_Func12(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
    }
}
unsafe fn Xray_Func9(mut r18: uint16, mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
    if r18 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        Xray_Func14(dst_r22, jp.offset(4 as ::core::ffi::c_int as isize), game);
    }
}
unsafe fn Xray_Func10(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
    Xray_Func15(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
}
unsafe fn Xray_Func11(mut r18: uint16, mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    Xray_Func12(dst_r22, jp.offset(2 as ::core::ffi::c_int as isize), game);
    if r18 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        Xray_Func14(dst_r22, jp.offset(4 as ::core::ffi::c_int as isize), game);
    }
    Xray_Func15(dst_r22, jp.offset(6 as ::core::ffi::c_int as isize), game);
    if r18 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        let mut v5: uint16 = *(jp.offset(8 as ::core::ffi::c_int as isize) as *mut uint16);
        let mut top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa000 as ::core::ffi::c_int as isize)
            as *mut TileTables))
            .tables[v5 as usize]
            .top_left;
        let mut top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa000 as ::core::ffi::c_int as isize)
            as *mut TileTables))
            .tables[v5 as usize]
            .top_right;
        let mut bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa000 as ::core::ffi::c_int as isize)
            as *mut TileTables))
            .tables[v5 as usize]
            .bottom_left;
        let mut bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa000 as ::core::ffi::c_int as isize)
            as *mut TileTables))
            .tables[v5 as usize]
            .bottom_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(99 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(98 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_left;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(67 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_right;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16)
            .offset(66 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_left;
    }
}
unsafe fn CallXrayFunc(
    mut ea: uint32,
    mut jp: *const uint8,
    mut r18: uint16,
    mut r22: uint16,
    mut r36: uint16,
    game: &mut Game,
) {
    match ea {
        9555577 => {
            Xray_CombinedMove(r22, r36, true_0 != 0, game);
            return;
        }
        9555643 => {
            Xray_CombinedMove(r22, r36, false_0 != 0, game);
            return;
        }
        9555766 => {
            Xray_Func7(r22, jp, game);
            return;
        }
        9555774 => {
            Xray_Func8(r22, jp, game);
            return;
        }
        9555790 => {
            Xray_Func9(r18, r22, jp, game);
            return;
        }
        9555810 => {
            Xray_Func10(r22, jp, game);
            return;
        }
        9555823 => {
            Xray_Func11(r18, r22, jp, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Xray_SetupStage4_Func3(
    mut r18: uint16,
    mut r22: uint16,
    mut r36: uint16,
    game: &mut Game,
) {
    let mut jp: *const uint8 = Xray_GetXrayedBlock(r36, game);
    if !jp.is_null() {
        CallXrayFunc(
            (*(jp as *mut uint16) as ::core::ffi::c_int | 0x910000 as ::core::ffi::c_int) as uint32,
            jp,
            r18,
            r22,
            r36,
            game,
        );
    }
}
unsafe fn Xray_Func12(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    Xray_Func13(dst_r22, *(jp as *mut uint16), game);
}
unsafe fn Xray_Func13(mut dst_r22: uint16, mut a: uint16, game: &mut Game) {
    let mut top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let mut top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let mut bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let mut bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func14(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    let mut a: uint16 = *(jp as *mut uint16);
    let mut top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let mut top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let mut bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let mut bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(35 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(34 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func15(mut dst_r22: uint16, mut jp: *const uint8, game: &mut Game) {
    let mut a: uint16 = *(jp as *mut uint16);
    let mut top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let mut top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let mut bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let mut bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(97 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(96 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(65 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(64 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_left;
}
unsafe fn Xray_Func16(mut dst_r22: uint16, mut a: uint16, game: &mut Game) {
    let mut top_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_left;
    let mut top_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .top_right;
    let mut bottom_left: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_left;
    let mut bottom_right: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa000 as ::core::ffi::c_int as isize)
        as *mut TileTables))
        .tables[a as usize]
        .bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = bottom_left;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(33 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_right;
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
        .xray_tilemaps as *mut uint16)
        .offset(32 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(dst_r22 as ::core::ffi::c_int as isize) as *mut uint16) = top_left;
}
pub unsafe fn LoadBlockToXrayTilemap(mut a: uint16, mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut R24: uint16 = (k as ::core::ffi::c_int
        - (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)) as uint16;
    let mut R26: uint16 = (j as ::core::ffi::c_int
        - (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)) as uint16;
    if R24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        && R24 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        && R26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        && R26 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let mut R22: uint16 = (4 as ::core::ffi::c_int
            * (R24 as ::core::ffi::c_int + 32 as ::core::ffi::c_int * R26 as ::core::ffi::c_int))
            as uint16;
        if a as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            Xray_Func16(
                R22,
                (a as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16,
                game,
            );
        } else {
            Xray_Func13(
                R22,
                (a as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16,
                game,
            );
        }
    }
}
pub unsafe fn Xray_SetupStage5(game: &mut Game) {
    let mut v1: ::core::ffi::c_uint = 0;
    if CanXrayShowBlocks(game) {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x91d as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x91f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) =
            73 as uint8;
    }
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x360 as ::core::ffi::c_int as isize) as *mut uint16);
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_uint;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .vram_target as *mut uint8)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        ) as *mut uint16) = (((*(&raw mut game.g_ram as *mut uint8)
        .offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + 1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .dma_parameters as *mut uint8)
        .offset(v1 as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .addr as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut VoidP) = 0x5800 as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .src
    .bank)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
        as *mut VramReadQueueEnt)
        .offset(0 as ::core::ffi::c_int as isize))
    .size as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v0 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CanXrayShowBlocks(game: &mut Game) -> bool {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == addr_kRoom_a66a as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_cefb as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x24 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 6 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 8 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 10 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe fn Xray_SetupStage6(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v1 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        );
        (*v1).size = 2048 as uint16;
        (*v1).src.addr = ((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as VoidP;
        *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
        (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Xray_SetupStage7(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if CanXrayShowBlocks(game) {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v1 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        );
        (*v1).size = 2048 as uint16;
        (*v1).src.addr = 0x4800 as VoidP;
        *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
        (*v1).vram_dst = (((*(&raw mut game.g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 1024 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    }
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x9800e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8b as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x98c8e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x999098 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xa91 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) = 192 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) = 64 as uint16;
    };
}
pub unsafe fn HdmaobjPreInstr_XraySetup(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: uint16 = 0;
    v1 = 4096 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 36 as ::core::ffi::c_int
    {
        if (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            - 7 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block = 5035836855854403712;
        } else {
            current_block = 4263953485050074566;
        }
    } else {
        v1 = 0x2000 as uint16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as uint16;
            current_block = 4263953485050074566;
        } else {
            current_block = 5035836855854403712;
        }
    }
    match current_block {
        4263953485050074566 => {
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x27 as uint16;
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x47 as uint16;
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x87 as uint16;
        }
        _ => {}
    }
    let ref mut fresh37 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh37 = (*fresh37 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Xray_SetupStage8_SetBackdropColor(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 3171 as uint16;
}
pub unsafe fn GameState_28_Unused_(game: &mut Game) {
    Unreachable(game);
}
pub unsafe fn VariaSuitPickup(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize) = 80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize) =
        0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf3 as ::core::ffi::c_int as isize) = 0 as uint8;
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
        as *mut uint16) = 256 as uint16;
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        QueueSfx1_Max9(0x32 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 120 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 120 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 136 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 136 as ::core::ffi::c_int) as uint16;
    QueueSfx2_Max6(0x56 as uint16, game);
    static unk_91D59B: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x41 as uint8,
        field_1: 0x26 as uint8,
        addr: 0xd5a2 as VoidP,
    };
    SpawnHdmaObject(
        0x91 as uint8,
        &raw const unk_91D59B as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn GravitySuitPickup(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize) = 73 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize) =
        -(112 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xdf3 as ::core::ffi::c_int as isize) = 1 as uint8;
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
        as *mut uint16) = 256 as uint16;
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        QueueSfx1_Max9(0x32 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    CallSomeSamusCode(0x15 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 120 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 120 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 136 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 136 as ::core::ffi::c_int) as uint16;
    QueueSfx2_Max6(0x56 as uint16, game);
    static unk_91D673: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x41 as uint8,
        field_1: 0x26 as uint8,
        addr: 0xd67a as VoidP,
    };
    SpawnHdmaObject(
        0x91 as uint8,
        &raw const unk_91D673 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn InitializeSuitPickupHdma(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 51 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x9800e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8b as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x98c8e4 as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x999098 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xa91 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
static off_91D72D: [Option<Func_U8>; 11] = unsafe {
    [
        Some(Samus_HandleScrewAttackSpeedBoostingPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_SpeedBoosterShinePals as unsafe fn(&mut Game) -> uint8),
        ::core::mem::transmute::<Option<unsafe fn(&mut Game) -> ()>, Option<Func_U8>>(Some(
            HandleMiscSamusPalette as unsafe fn(&mut Game) -> (),
        )),
        None,
        None,
        None,
        Some(Samus_HandleShinesparkingPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleCrystalFlashPals as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleXrayPals as unsafe fn(&mut Game) -> uint8),
        Some(HandleVisorPalette as unsafe fn(&mut Game) -> uint8),
        Some(nullsub_164 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandlePalette(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && (HandleBeamChargePalettes(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
            || off_91D72D[*((&raw mut game.g_ram as *mut uint8)
                .offset(0xacc as ::core::ffi::c_int as isize)
                as *mut uint16) as usize]
                .expect("non-null function pointer")(game) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0)
    {
        CopyToSamusSuitPalette(
            *(game.g_rom.offset(
                (((0x91d727 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91d727 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa74 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ),
            game,
        );
    }
    HandleMiscSamusPalette(game);
}
pub unsafe fn HandleBeamChargePalettes(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb18 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb18 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb18 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x1e as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb18 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    return 1 as uint8;
                }
                CopyToSamusSuitPalette(
                    *(game.g_rom.offset(
                        (((0x91d829 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x91d829 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(
                            ((*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xb18 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                as ::core::ffi::c_int
                                & 0x1e as ::core::ffi::c_int)
                                >> 1 as ::core::ffi::c_int) as isize,
                        ),
                    game,
                );
            }
            let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb18 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh3 = (*fresh3).wrapping_sub(1);
            return 0 as uint8;
        }
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb18 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_sub(1);
        if *fresh4 != 0 {
            let mut i: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        ((i >> 1 as ::core::ffi::c_int) + 193 as ::core::ffi::c_int) as isize,
                    ) = 1023 as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
            return 0 as uint8;
        } else {
            return 1 as uint8;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 60 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        let mut R36: uint16 = 0;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            R36 = *(game.g_rom.offset(
                (((0x91d7ff as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91d7ff as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa74 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                );
        } else {
            R36 = *(game.g_rom.offset(
                (((0x91d7d5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91d7d5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa74 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                );
        }
        CopyToSamusSuitPalette(
            *(RomPtr_91(
                (R36 as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb62 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16),
            game,
        );
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xb62 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return HandleVisorPalette(game);
    };
}
pub unsafe fn HandleVisorPalette(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 40 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 42 as ::core::ffi::c_int
    {
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa72 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
        if v1 as uint8 != 0 {
            return 0 as uint8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v1 as ::core::ffi::c_int | 5 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(196 as ::core::ffi::c_int as isize) = *(game.g_rom.offset(
            (((0x9ba3c0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9ba3c0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(
                (*(&raw mut v1 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        let mut v2: ::core::ffi::c_int = *(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int;
        if v2 - 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
                as *mut uint16) = (swap16(v2 as uint16) as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int)
                as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa72 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                | 0x600 as ::core::ffi::c_int) as uint16;
        }
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa72 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1537 as uint16;
        return 0 as uint8;
    };
}
pub unsafe fn HandleMiscSamusPalette(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            return;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                && (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa44 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != 0x90e8d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
                    || *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != kPose_54_FaceL_Knockback as ::core::ffi::c_int)
            {
                QueueSfx1_Max6(0x35 as uint16, game);
                current_block = 4767789648742619593;
            } else {
                v0 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa48 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                current_block = 15619007995458559411;
            }
        } else {
            current_block = 15619007995458559411;
        }
        match current_block {
            15619007995458559411 => {
                if v0 as ::core::ffi::c_int - 7 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    current_block = 11050875288958768710;
                } else if v0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    CopyToSamusSuitPalette(addr_word_9BA380 as ::core::ffi::c_int as uint16, game);
                    current_block = 11050875288958768710;
                } else {
                    current_block = 4767789648742619593;
                }
            }
            _ => {}
        }
        match current_block {
            4767789648742619593 => {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1f51 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    != 0
                {
                    CopyToSamusSuitPalette(addr_word_9BA3A0 as ::core::ffi::c_int as uint16, game);
                } else {
                    Samus_LoadSuitPalette(game);
                }
            }
            _ => {}
        }
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
        if v1 as ::core::ffi::c_int == 40 as ::core::ffi::c_int {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == kMovementType_03_SpinJumping as ::core::ffi::c_int
                    || *(&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1f as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == kMovementType_14_WallJumping as ::core::ffi::c_int
                {
                    CallSomeSamusCode(0x1c as uint16, game);
                } else if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd0 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 16 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                    && *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9b2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8b as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc0 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 1 as uint16;
                }
            } else if *((&raw mut game.g_ram as *mut uint8)
                .offset(0xd32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 0x37aa as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                QueueSfx1_Max9(6 as uint16, game);
            }
        } else if v1 as ::core::ffi::c_int - 60 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        return;
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            CopyToSamusSuitPalette(
                *(game.g_rom.offset(
                    (((0x91d99e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91d99e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xb62 as ::core::ffi::c_int as isize)
                            as *mut uint16) as isize,
                    ),
                game,
            );
            let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh34 = (*fresh34).wrapping_sub(1);
            let mut v2: bool = *fresh34 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                || v2 as ::core::ffi::c_int != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16);
                let ref mut fresh35 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb62 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh35 = (*fresh35).wrapping_add(1);
                if *fresh35 as ::core::ffi::c_int - 10 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb62 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
            }
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                CopyToSamusSuitPalette(
                    *(game.g_rom.offset(
                        (((0x91d998 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x91d998 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xa74 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ),
                    game,
                );
            } else {
                Samus_LoadSuitPalette(game);
            }
            let ref mut fresh36 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa4a as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh36 = (*fresh36).wrapping_add(1);
        }
        return;
    };
}
pub unsafe fn Samus_HandleScrewAttackSpeedBoostingPals(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r20: uint16 = Samus_GetTop_R20(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r20 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                return 1 as uint8;
            }
        } else if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            current_block = 14068947459426067103;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 27 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                return 0 as uint8;
            }
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block = 8831408221741692167;
        } else {
            current_block = 7431622773599773796;
        }
    } else {
        current_block = 14068947459426067103;
    }
    match current_block {
        8831408221741692167 => {
            let mut R36: uint16 = *(game.g_rom.offset(
                (((0x91da4a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91da4a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa74 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                );
            let mut v1: uint16 = *(RomPtr_91(
                (R36 as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xace as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16);
            CopyToSamusSuitPalette(v1, game);
            let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >= 10 as ::core::ffi::c_int
            {
                v2 = 0 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = v2;
            return 1 as uint8;
        }
        7431622773599773796 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            return 1 as uint8;
        }
        _ => {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int
                != 1024 as ::core::ffi::c_int
            {
                return 1 as uint8;
            }
            let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh17 = (*fresh17).wrapping_sub(1);
            let mut v3: bool = *fresh17 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                || v3 as ::core::ffi::c_int != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 4 as uint16;
                let mut R36_0: uint16 = *(game.g_rom.offset(
                    (((0x91daa9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91daa9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa74 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    );
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    > 6 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xace as ::core::ffi::c_int as isize)
                        as *mut uint16) = 6 as uint16;
                }
                let mut v4: uint16 = *(RomPtr_91(
                    (R36_0 as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xace as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16_t,
                    game,
                ) as *mut uint16);
                CopyToSamusSuitPalette(v4, game);
                let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >= 6 as ::core::ffi::c_int
                {
                    v5 = 6 as uint16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5;
            }
            return 1 as uint8;
        }
    };
}
pub unsafe fn Samus_SpeedBoosterShinePals(game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 170 as ::core::ffi::c_int
    {
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16);
        QueueSfx3_Max9(0xc as uint16, game);
        v0 = v4;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    if (v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        <= 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 0 as uint8;
    } else {
        let mut R36: uint16 = *(game.g_rom.offset(
            (((0x91db10 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91db10 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        let mut v1: uint16 = *(RomPtr_91(
            (R36 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        return 1 as uint8;
    };
}
pub unsafe fn Samus_HandleShinesparkingPals(game: &mut Game) -> uint8 {
    let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh19 = (*fresh19).wrapping_sub(1);
    let mut v0: bool =
        *fresh19 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
        || v0 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 0 as uint8;
    } else {
        let mut R36: uint16 = *(game.g_rom.offset(
            (((0x91db75 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91db75 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        let mut v1: uint16 = *(RomPtr_91(
            (R36 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16);
        CopyToSamusSuitPalette(v1, game);
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 6 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        return 1 as uint8;
    };
}
pub unsafe fn Samus_HandleCrystalFlashPals(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        WriteBeamPalette_A(
            *((&raw mut game.g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 0 as uint8;
    } else {
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh13 = (*fresh13).wrapping_sub(1);
        if *fresh13 as int16 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
            Samus_Copy6PalColors(
                *(game.g_rom.offset(
                    (((0x91dc28 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91dc28 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xace as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ),
                game,
            );
            let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v0 = 0 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = v0;
        }
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xdf2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh15 = (*fresh15).wrapping_sub(1);
        let mut v1: bool = (*fresh15 as int16 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            || v1 as ::core::ffi::c_int != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut (*(game.g_rom.offset(
                (((0x91dc00 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91dc00 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut SamusCrystalFlashPalTable)
                .offset(0 as ::core::ffi::c_int as isize))
            .timer as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xad0 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int as isize,
                ) as *mut uint16);
            Samus_Copy10PalColors(
                *((&raw mut (*(game.g_rom.offset(
                    (((0x91dc00 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91dc00 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut SamusCrystalFlashPalTable)
                    .offset(0 as ::core::ffi::c_int as isize))
                .ptr as *mut uint8)
                    .offset(
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xad0 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int as isize,
                    ) as *mut VoidP),
                game,
            );
            let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 36 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v2 = 0 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) = v2;
        }
        return 1 as uint8;
    };
}
pub unsafe fn Samus_Copy10PalColors(mut v0: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(224 as ::core::ffi::c_int as isize) as *mut uint16
            as *mut ::core::ffi::c_void,
        RomPtr_9B(v0 as uint16_t, game) as *const ::core::ffi::c_void,
        20 as size_t,
    );
}
pub unsafe fn Samus_Copy6PalColors(mut j: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(234 as ::core::ffi::c_int as isize) as *mut uint16
            as *mut ::core::ffi::c_void,
        RomPtr_9B(j as uint16_t, game) as *const ::core::ffi::c_void,
        12 as size_t,
    );
}
pub unsafe fn Samus_HandleXrayPals(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 0 as uint8;
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xad0 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh21 = (*fresh21).wrapping_sub(1);
                let mut v0: bool = *fresh21 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xad0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
                    || v0 as ::core::ffi::c_int != 0
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xad0 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 5 as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc000 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(196 as ::core::ffi::c_int as isize) = *(game.g_rom.offset(
                        (((0x9ba3c0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x9ba3c0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    )
                        as *const uint8
                        as *mut uint16)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xace as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        );
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xace as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 4 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xace as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh22 =
                            (*fresh22 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    }
                }
                return 1 as uint8;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = 6 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
        let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh24 = (*fresh24).wrapping_sub(1);
        let mut v2: bool = *fresh24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && !v2
        {
            return 1 as uint8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(196 as ::core::ffi::c_int as isize) = *(game.g_rom.offset(
            (((0x9ba3c0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9ba3c0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 6 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = v3;
        return 1 as uint8;
    };
}
pub unsafe fn nullsub_164(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn CopyToSamusSuitPalette(mut k: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(192 as ::core::ffi::c_int as isize) as *mut uint16
            as *mut ::core::ffi::c_void,
        RomPtr_9B(k as uint16_t, game) as *mut uint16 as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn CopyToSamusSuitTargetPalette(mut k: uint16, game: &mut Game) {
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(192 as ::core::ffi::c_int as isize) as *mut uint16
            as *mut ::core::ffi::c_void,
        RomPtr_9B(k as uint16_t, game) as *mut uint16 as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn Samus_CancelSpeedBoost(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            CopyToSamusSuitPalette(
                addr_kSamusPalette_GravitySuit as ::core::ffi::c_int as uint16,
                game,
            );
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            CopyToSamusSuitPalette(
                addr_kSamusPalette_VariaSuit as ::core::ffi::c_int as uint16,
                game,
            );
        } else {
            CopyToSamusSuitPalette(
                addr_kSamusPalette_PowerSuit as ::core::ffi::c_int as uint16,
                game,
            );
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) = -(8 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) = -(8 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) = 8 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) = 8 as uint16;
        }
    }
}
pub unsafe fn Samus_LoadSuitPalette(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CopyToSamusSuitPalette(
            addr_kSamusPalette_GravitySuit as ::core::ffi::c_int as uint16,
            game,
        );
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CopyToSamusSuitPalette(
            addr_kSamusPalette_VariaSuit as ::core::ffi::c_int as uint16,
            game,
        );
    } else {
        CopyToSamusSuitPalette(
            addr_kSamusPalette_PowerSuit as ::core::ffi::c_int as uint16,
            game,
        );
    };
}
pub unsafe fn Samus_LoadSuitTargetPalette(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CopyToSamusSuitTargetPalette(
            addr_kSamusPalette_GravitySuit as ::core::ffi::c_int as uint16,
            game,
        );
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CopyToSamusSuitTargetPalette(
            addr_kSamusPalette_VariaSuit as ::core::ffi::c_int as uint16,
            game,
        );
    } else {
        CopyToSamusSuitTargetPalette(
            addr_kSamusPalette_PowerSuit as ::core::ffi::c_int as uint16,
            game,
        );
    };
}
pub fn Samus_RestoreHealth(amount: u16, game: &mut Game) {
    let health = game.samus_health().wrapping_add(amount);
    game.set_samus_health(health);
    if (health.wrapping_sub(game.samus_max_health()) as i16) >= 0 {
        let mut reserve = game
            .samus_reserve_health()
            .wrapping_add(health)
            .wrapping_sub(game.samus_max_health());
        if (reserve.wrapping_sub(game.samus_max_reserve_health()) as i16) >= 0 {
            reserve = game.samus_max_reserve_health();
        }
        game.set_samus_reserve_health(reserve);
        if reserve != 0 && game.reserve_health_mode() == 0 {
            game.set_reserve_health_mode(1);
        }
        game.set_samus_health(game.samus_max_health());
    }
}
pub unsafe fn Samus_DealDamage(amount: u16, game: &mut Game) {
    if amount & 0x8000 == 0 {
        if amount != 300 && game.time_is_frozen_flag() == 0 {
            let health = game.samus_health().wrapping_sub(amount);
            game.set_samus_health(health);
            if game.samus_health() & 0x8000 != 0 {
                game.set_samus_health(0);
            }
        }
    } else {
        InvalidInterrupt_Crash(game);
    }
}
pub fn Samus_RestoreMissiles(amount: u16, game: &mut Game) {
    let missiles = game.samus_missiles().wrapping_add(amount);
    game.set_samus_missiles(missiles);
    if (missiles.wrapping_sub(game.samus_max_missiles()) as i16) >= 0 {
        if game.samus_max_missiles().wrapping_sub(99) & 0x8000 != 0 {
            let reserve = game
                .samus_reserve_missiles()
                .wrapping_add(missiles.wrapping_sub(game.samus_max_missiles()));
            game.set_samus_reserve_missiles(reserve);
            if (game
                .samus_reserve_missiles()
                .wrapping_sub(game.samus_max_missiles()) as i16)
                >= 0
            {
                game.set_samus_reserve_missiles(game.samus_max_missiles());
            }
        } else {
            let reserve = game
                .samus_reserve_missiles()
                .wrapping_add(missiles.wrapping_sub(game.samus_max_missiles()));
            game.set_samus_reserve_missiles(reserve);
            if game.samus_reserve_missiles().wrapping_sub(99) & 0x8000 == 0 {
                game.set_samus_reserve_missiles(99);
            }
        }
        game.set_samus_missiles(game.samus_max_missiles());
    }
}
pub fn Samus_RestoreSuperMissiles(amount: u16, game: &mut Game) {
    let ammo = game.samus_super_missiles().wrapping_add(amount);
    game.set_samus_super_missiles(ammo);
    if (ammo.wrapping_sub(game.samus_max_super_missiles()) as i16) >= 0
        && ammo != game.samus_max_super_missiles()
    {
        game.set_samus_super_missiles(game.samus_max_super_missiles());
    }
}
pub fn Samus_RestorePowerBombs(amount: u16, game: &mut Game) {
    let ammo = game.samus_power_bombs().wrapping_add(amount);
    game.set_samus_power_bombs(ammo);
    if (ammo.wrapping_sub(game.samus_max_power_bombs()) as i16) >= 0
        && ammo != game.samus_max_power_bombs()
    {
        game.set_samus_power_bombs(game.samus_max_power_bombs());
    }
}
pub unsafe fn Samus_Initialize(game: &mut Game) {
    let debug_invincibility = game.debug_invincibility();
    // Clear the original Samus workspace from its last byte to its first.
    for address in (0xa02..=0xe0b).rev() {
        game.g_ram[address] = 0;
    }
    if game.game_state() != kGameState_40_TransitionToDemo as u16 {
        game.set_frame_handler_alfa(fnEmptyFunction as u16);
        if game.loading_game_state() == kGameState_34_CeresGoesBoom as u16 {
            game.set_frame_handler_beta(fnSetContactDamageIndexAndUpdateMinimap as u16);
            game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
            game.set_samus_momentum_routine_index(u16::MAX);
            game.set_samus_special_transgfx_index(0);
            game.set_samus_hurt_switch_index(0);
            Samus_LoadSuitPalette(game);
            game.set_samus_input_handler(fnSamus_InputHandler_E913 as u16);
        } else {
            game.set_frame_handler_beta(fnSamus_Func16 as u16);
            game.set_samus_draw_handler(fnSamusDrawHandler_Default as u16);
            game.set_samus_momentum_routine_index(0);
            game.set_samus_special_transgfx_index(0);
            game.set_samus_hurt_switch_index(0);
            game.set_samus_input_handler(fnSamus_InputHandler_E913 as u16);
            game.set_debug_invincibility(debug_invincibility);
        }
    }
    game.set_samus_new_pose(u16::MAX);
    game.set_samus_new_pose_interrupted(u16::MAX);
    game.set_samus_new_pose_transitional(u16::MAX);
    if game.area_index() == 6 {
        game.set_frame_handler_gamma(fnSamus_Func3 as u16);
    } else {
        game.set_frame_handler_gamma(fnnullsub_152 as u16);
    }
    game.set_samus_movement_handler(fnSamus_MovementHandler_Normal as u16);
    game.set_unused_word_7e0a5e(0u16.wrapping_sub(2764));
    game.set_samus_prev_health_for_flash(50);
    game.set_samus_visor_palette_timer_index(1537);
    for slot in 0..10 {
        game.set_projectile_bomb_pre_instruction(slot, fnProjPreInstr_Empty as u16);
    }
    game.set_grapple_beam_function(fnGrappleBeamFunc_Inactive as u16);
    game.set_enable_horiz_slope_coll(3);
    game.set_samus_hurt_flash_counter(0);
    game.set_samus_special_super_palette_flags(0);
    game.set_absolute_moved_last_frame_x_fract(0);
    game.set_absolute_moved_last_frame_x(1);
    game.set_absolute_moved_last_frame_y_fract(0);
    game.set_absolute_moved_last_frame_y(1);
    // Initialize the hdma_table_1 from its last word to its first.
    for word in game.g_ram[0x9800..0x9a00].chunks_exact_mut(2).rev() {
        word.copy_from_slice(&255u16.to_le_bytes());
    }
    game.set_samus_y_subaccel(0x1c00);
    game.set_samus_y_accel(0);
    game.set_fx_y_pos(u16::MAX);
    game.set_lava_acid_y_pos(u16::MAX);
    UpdateBeamTilesAndPalette(game);
    game.set_cinematic_function(0);
    game.set_samus_pose(kPose_00_FaceF_Powersuit as u16);
    game.set_samus_pose_direction_and_movement_type(0, 0);
    game.set_samus_prev_pose(0);
    game.set_samus_prev_pose_direction_and_movement_type(0, 0);
    game.set_samus_last_different_pose(0);
    game.set_samus_last_different_pose_direction_and_movement_type(0, 0);
    game.set_enemy_index_to_shake(u16::MAX);
    game.set_hud_item_index(0);
    game.set_samus_auto_cancel_hud_item_index(0);
    game.set_samus_invincibility_timer(0);
    game.set_samus_knockback_timer(0);
    game.set_samus_hurt_flash_counter(0);
    game.set_debug_invincibility(0);
    if game.game_state() == kGameState_40_TransitionToDemo as u16 {
        LoadDemoData(game);
    }
    game.set_samus_prev_health_for_flash(game.samus_health());
}
pub unsafe fn Xray_Initialize(game: &mut Game) -> uint8 {
    static byte_91E291: [uint8; 28] = [
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
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
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 7 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 5 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_A4_FaceR_LandJump as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - kPose_A8_FaceR_Grappling as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                || *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - kPose_E0_FaceR_LandJump_AimU as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                    && *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0)
        || *((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 8 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || byte_91E291[*(&raw mut game.g_ram as *mut uint8)
            .offset(0xa11 as ::core::ffi::c_int as isize) as usize]
            == 0
    {
        return 0 as uint8;
    }
    if byte_91E291
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_D6_FaceL_Xray_Stand as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_D5_FaceR_Xray_Stand as ::core::ffi::c_int as uint16;
        }
    } else {
        if byte_91E291[*(&raw mut game.g_ram as *mut uint8)
            .offset(0xa1f as ::core::ffi::c_int as isize) as usize] as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
        {
            return 0 as uint8;
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_DA_FaceL_Xray_Crouch as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_D9_FaceR_Xray_Crouch as ::core::ffi::c_int as uint16;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    DisableEprojs(game);
    DisablePLMs(game);
    DisableAnimtiles(game);
    DisablePaletteFx(game);
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xa8a as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize)
        as *mut uint16) = -(26424 as ::core::ffi::c_int) as uint16;
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0x999098 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xa91 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) = 192 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) = 64 as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn ResponsibleForXrayStandupGlitch(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_05_Crouching as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_28_FaceL_Crouch as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_27_FaceR_Crouch as ::core::ffi::c_int as uint16;
        }
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kPose_04_FaceL_AimU as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_01_FaceR_Normal as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = ((*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh38 = (*fresh38 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    EnableEprojs(game);
    EnablePLMs(game);
    EnableAnimtiles(game);
    EnablePaletteFx(game);
}
pub unsafe fn MakeSamusFaceForward(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 24 as ::core::ffi::c_int
    {
        let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh31 = (*fresh31 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e8dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    CallSomeSamusCode(0x1f as uint16, game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_LoadSuitPalette(game);
}
static kSomeMotherBrainScripts: [Option<Func_U8>; 5] = unsafe {
    [
        Some(SomeMotherBrainScripts_0 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_1 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_2 as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_3_EnableHyperBeam as unsafe fn(&mut Game) -> uint8),
        Some(SomeMotherBrainScripts_4 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SomeMotherBrainScripts(mut a: uint16, game: &mut Game) {
    if kSomeMotherBrainScripts[a as usize].expect("non-null function pointer")(game)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn SomeMotherBrainScripts_0(game: &mut Game) -> uint8 {
    let ref mut fresh27 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh27 = (*fresh27 as ::core::ffi::c_int
        - (21 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_LoadSuitPalette(game);
    return 1 as uint8;
}
pub unsafe fn SomeMotherBrainScripts_1(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_EB_FaceL_Drained_Stand as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_EA_FaceR_Drained_Stand as ::core::ffi::c_int as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SomeMotherBrainScripts_2(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 13 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_EA_FaceR_Drained_Stand as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_EB_FaceL_Drained_Stand as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    return 1 as uint8;
}
pub unsafe fn SomeMotherBrainScripts_3_EnableHyperBeam(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 4105 as uint16;
    UpdateBeamTilesAndPalette(game);
    SpawnPalfxObject(addr_stru_8DE1F0 as ::core::ffi::c_int as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x918000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SomeMotherBrainScripts_4(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn nullsub_17(game: &mut Game) {}
static off_91E6E1: [Option<Func_V>; 28] = unsafe {
    [
        Some(SamusFunc_E633_0 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_3 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_4 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_4 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_17 as unsafe fn(&mut Game) -> ()),
        Some(SamusFunc_E633_20 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_17 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn SamusFunc_E633(game: &mut Game) {
    off_91E6E1[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as usize]
        .expect("non-null function pointer")(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x2000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91b61f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b61f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize);
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                QueueSfx1_Max6(0x41 as uint16, game);
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else {
        LoadProjectilePalette(2 as uint16, game);
        QueueSfx1_Max6(6 as uint16, game);
    }
    Samus_LoadSuitPalette(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 31 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        QueueSfx3_Max6(2 as uint16, game);
    }
}
pub unsafe fn Samus_UpdatePreviousPose_0(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn SamusFunc_E633_0(game: &mut Game) {
    let mut current_block_4: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x20 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
            current_block_4 = 5176481699505674939;
        } else {
            current_block_4 = 7351195479953500246;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
        current_block_4 = 5176481699505674939;
    } else {
        current_block_4 = 7351195479953500246;
    }
    match current_block_4 {
        5176481699505674939 => {
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
            Samus_UpdatePreviousPose_0(game);
        }
        _ => {}
    };
}
pub unsafe fn SamusFunc_E633_3(game: &mut Game) {
    let mut current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 260 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 264 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_82_FaceL_Screwattack as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
        {
            current_block = 14317573491659972910;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_82_FaceL_Screwattack as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_81_FaceR_Screwattack as ::core::ffi::c_int as uint16;
            }
            current_block = 14317573491659972910;
        } else {
            current_block = 5564999883274278710;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        current_block = 5564999883274278710;
    } else {
        current_block = 14317573491659972910;
    }
    match current_block {
        5564999883274278710 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_1A_FaceL_SpinJump as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_19_FaceR_SpinJump as ::core::ffi::c_int as uint16;
            }
        }
        _ => {}
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 772 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 776 as uint16;
    }
    Samus_UpdatePreviousPose_0(game);
}
pub unsafe fn SamusFunc_E633_4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int as uint16;
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_17(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_41_FaceL_Morphball_Ground as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int as uint16;
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        Samus_UpdatePreviousPose_0(game);
    }
}
pub unsafe fn SamusFunc_E633_20(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 23 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
    };
}
pub unsafe fn nullsub_18(game: &mut Game) {}
static kSamus_HandleTransFromBlockColl: [Option<Func_V>; 6] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransFromBlockColl_5 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransFromBlockColl(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        kSamus_HandleTransFromBlockColl[*((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_3(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
}
pub unsafe fn Samus_HandleTransFromBlockColl_4(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
}
pub unsafe fn nullsub_18_U8(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
static off_91E951: [Option<Func_U8>; 6] = unsafe {
    [
        Some(Samus_HandleTransFromBlockColl_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_1 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_2 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_3 as unsafe fn(&mut Game) -> uint8),
        Some(nullsub_18_U8 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransFromBlockColl_1_5 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransFromBlockColl_2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        != 4 as ::core::ffi::c_int
    {
        let mut v0: uint16 = (4 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91e921 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91e921 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int) as isize,
                );
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91e921 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91e921 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        != 4 as ::core::ffi::c_int
    {
        if off_91E951[*((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as usize]
            .expect("non-null function pointer")(game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
        }
    }
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_0(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_A7_FaceL_LandSpinJump as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_A6_FaceR_LandSpinJump as ::core::ffi::c_int as uint16;
        }
        return 0 as uint8;
    } else {
        v0 = (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .direction_shots_fired as int16;
        if v0 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_A5_FaceL_LandJump as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_A4_FaceR_LandJump as ::core::ffi::c_int as uint16;
            }
            return 0 as uint8;
        } else {
            if !(v0 as ::core::ffi::c_int != 2 as ::core::ffi::c_int
                && v0 as ::core::ffi::c_int != 7 as ::core::ffi::c_int)
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x8b as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v0 = (*(game.g_rom.offset(
                        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut SamusPoseParams)
                        .offset(
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xa1c as ::core::ffi::c_int as isize)
                                as *mut uint16) as isize,
                        ))
                    .direction_shots_fired as int16;
                } else {
                    if *(&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1e as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == 4 as ::core::ffi::c_int
                    {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa28 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kPose_E7_FaceL_LandJump_Fire as ::core::ffi::c_int as uint16;
                    } else {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa28 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kPose_E6_FaceR_LandJump_Fire as ::core::ffi::c_int as uint16;
                    }
                    return 0 as uint8;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91e9f3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91e9f3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize);
            return 0 as uint8;
        }
    };
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_1(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    v0 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if 2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0
    {
        if v0 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
            return 0 as uint8;
        }
        if v0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
            Unreachable(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_41_FaceL_Morphball_Ground as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_2(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = 66 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = 32 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_3(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    }
    v1 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int) as int16;
    if v1 != 0 {
        if v1 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
            return 0 as uint8;
        }
        if v1 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
            Unreachable(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = 121 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransFromBlockColl_1_5(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransFromBlockColl_5(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_84_FaceL_Walljump as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_83_FaceR_Walljump as ::core::ffi::c_int as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
}
pub unsafe fn Samus_CheckWalkedIntoSomething(game: &mut Game) {
    let mut cres_0: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let mut current_block: u64;
    let mut amt: int32 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(game.g_rom.offset(
            (((0x91eb74 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91eb74 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *(&raw mut (*(game.g_rom.offset(
                    (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut SamusPoseParams)
                    .offset(0 as ::core::ffi::c_int as isize))
                .direction_shots_fired)
                    .offset(
                        (8 as ::core::ffi::c_int
                            * *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xa1c as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int)
                            as isize,
                    ) as isize,
            );
        *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
        || (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .movement_type as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        amt = ((1 as ::core::ffi::c_int as int16 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
            as int32;
        cres = Samus_CheckSolidEnemyColl(amt, game);
        if cres.collision == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            amt = ((1 as ::core::ffi::c_int as int16 as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int) as int32;
            current_block = 12513788902857388954;
        } else {
            current_block = 8208193451455325075;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        amt = ((1 as ::core::ffi::c_int as int16 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
            as int32;
        cres_0 = Samus_CheckSolidEnemyColl(amt, game);
        if cres_0.collision != 0 {
            current_block = 8208193451455325075;
        } else {
            amt = ((-(1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int) as int32;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            current_block = 12513788902857388954;
        }
    }
    match current_block {
        12513788902857388954 => {
            Samus_MoveRight_NoSolidColl(amt, game);
            if !(*((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0)
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                return;
            }
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
        as *mut uint16) = *(game.g_rom.offset(
        (((0x91eb74 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91eb74 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(0 as ::core::ffi::c_int as isize))
            .direction_shots_fired)
                .offset(
                    (8 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa28 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as isize,
                ) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn nullsub_19(game: &mut Game) {}
pub unsafe fn nullsub_20(game: &mut Game) {}
pub unsafe fn nullsub_21(game: &mut Game) {}
pub unsafe fn nullsub_22(game: &mut Game) {}
static kSamus_HandleTransitionsA: [Option<Func_V>; 9] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_2 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_21 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_22 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_6 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_7 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_8 as unsafe fn(&mut Game) -> ()),
    ]
};
static kSamus_HandleTransitionsB: [Option<Func_V>; 11] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_5 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_19 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_20 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_8 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_9 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsB_10 as unsafe fn(&mut Game) -> ()),
    ]
};
static kSamus_HandleTransitionsC: [Option<Func_V>; 9] = unsafe {
    [
        Some(nullsub_18 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_2 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_6 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_7 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsC_8 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransitions(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            {
                current_block = 5743517142682009007;
            } else {
                current_block = 6038634297008360266;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 9 as ::core::ffi::c_int
        {
            current_block = 6038634297008360266;
        } else {
            current_block = 13536709405535804910;
        }
        match current_block {
            13536709405535804910 => {}
            _ => {
                match current_block {
                    6038634297008360266 => {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa1c as ::core::ffi::c_int as isize)
                            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa2c as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        SamusFunc_F433(game);
                        Samus_SetAnimationFrameIfPoseChanged(game);
                    }
                    _ => {}
                }
                kSamus_HandleTransitionsC[*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa32 as ::core::ffi::c_int as isize)
                    as *mut uint16) as usize]
                    .expect("non-null function pointer")(game);
                current_block = 10476341332908994874;
            }
        }
    } else {
        current_block = 13536709405535804910;
    }
    match current_block {
        13536709405535804910 => {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa2a as ::core::ffi::c_int as isize)
                    as *mut uint16);
                if SamusFunc_F404(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                    kSamus_HandleTransitionsB[*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa30 as ::core::ffi::c_int as isize)
                        as *mut uint16) as usize]
                        .expect("non-null function pointer")(game);
                }
                current_block = 10476341332908994874;
            } else {
                Samus_CheckWalkedIntoSomething(game);
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa28 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa28 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    if SamusFunc_F404(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                        kSamus_HandleTransitionsA[*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa2e as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            as usize]
                            .expect("non-null function pointer")(game);
                    }
                    current_block = 10476341332908994874;
                } else {
                    current_block = 10093773643351532501;
                }
            }
        }
        _ => {}
    }
    match current_block {
        10476341332908994874 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa22 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1e as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn Samus_HandleTransitionsA_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        Samus_CancelSpeedBoost(game);
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16,
            0 as uint32,
        );
        SamusFunc_EC80(game);
    } else {
        Samus_HandleTransitionsA_2(game);
    };
}
pub unsafe fn Samus_HandleTransitionsA_6(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_HandleTransitionsA_8(game);
}
pub unsafe fn Samus_HandleTransitionsA_8(game: &mut Game) {
    Samus_CancelSpeedBoost(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    SamusFunc_EC80(game);
}
pub unsafe fn Samus_HandleTransitionsA_2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_CancelSpeedBoost(game);
}
static word_91ED36: [uint16; 12] = [
    5 as ::core::ffi::c_int as uint16,
    5 as ::core::ffi::c_int as uint16,
    9 as ::core::ffi::c_int as uint16,
    9 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
pub unsafe fn Samus_HandleTransitionsA_7(game: &mut Game) {
    let mut amt: int32 = 0;
    let mut current_block_11: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - kPose_DB as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        amt = ((word_91ED36[(*((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 53 as ::core::ffi::c_int) as usize] as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as int32;
        if amt != 0 {
            current_block_11 = 7426089221157146073;
        } else {
            current_block_11 = 735147466149431745;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_F7_FaceR_StandTrans_AimU as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        amt = ((5 as ::core::ffi::c_int as int16 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
            as int32;
        current_block_11 = 7426089221157146073;
    } else {
        current_block_11 = 1856101646708284338;
    }
    match current_block_11 {
        7426089221157146073 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .y_radius as uint16;
            amt = Samus_CollDetectChangedPose(amt, game);
            current_block_11 = 735147466149431745;
        }
        _ => {}
    }
    match current_block_11 {
        735147466149431745 => {
            let ref mut fresh40 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh40 = (*fresh40 as ::core::ffi::c_int
                + (amt >> 16 as ::core::ffi::c_int) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        }
        _ => {}
    };
}
static kSamus_HandleTransitionsB_1: [Option<Func_U8>; 28] = unsafe {
    [
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_6 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_7 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_10 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_4 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_10 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsB_1_11 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransitionsB_1(game: &mut Game) {
    kSamus_HandleTransitionsB_1[*(&raw mut game.g_ram as *mut uint8)
        .offset(0xa23 as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")(game);
    Samus_SetSpeedForKnockback_Y(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
}
pub unsafe fn Samus_HandleTransitionsB_1_10(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsB_1_11(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsB_1_6(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x90e41b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return 0 as uint8;
    } else {
        return Samus_HandleTransitionsB_1_0(game);
    };
}
pub unsafe fn Samus_HandleTransitionsB_1_0(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 5 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 2 as uint16;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90df38 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn Samus_HandleTransitionsB_1_4(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90df38 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsB_1_7(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90df38 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn Samus_HandleTransitionsB_2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Samus_AlignBottomWithPrevPose(game);
}
pub unsafe fn Samus_HandleTransitionsB_3(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8 as ::core::ffi::c_int
        | 0x800 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e025 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_HandleTransitionsB_4(game: &mut Game) {
    Samus_InitJump(game);
}
pub unsafe fn Samus_HandleTransitionsB_5(game: &mut Game) {
    let mut current_block_21: u64;
    match *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
    {
        5 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 192 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 64 as uint16;
            }
            current_block_21 = 619521094565907383;
        }
        0 | 21 | 1 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 192 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 64 as uint16;
            }
            current_block_21 = 619521094565907383;
        }
        _ => {
            current_block_21 = 5948590327928692120;
        }
    }
    match current_block_21 {
        619521094565907383 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = 63 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90e94f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90e918 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
                as *mut uint16) = 8 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            QueueSfx1_Max6(9 as uint16, game);
        }
        _ => {}
    };
}
pub unsafe fn Samus_HandleTransitionsB_8(game: &mut Game) {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh41 = (*fresh41 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_HandleTransitionsB_9(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    GrappleBeamFunc_BD95(game);
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v0 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v0 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v1 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v1 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
    Samus_CancelSpeedBoost(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
}
pub unsafe fn Samus_HandleTransitionsB_10(game: &mut Game) {
    GrappleBeamFunc_BEEB(game);
    Samus_HandleTransitionsB_9B(game);
}
pub unsafe fn Samus_HandleTransitionsB_9B(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v0 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v0 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v1 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v1 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
    Samus_CancelSpeedBoost(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
}
pub unsafe fn nullsub_23(game: &mut Game) {}
static kSamus_HandleTransitionsA_5: [Option<Func_V>; 7] = unsafe {
    [
        Some(sub_91EFC3 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_1 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_2 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_23 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_4 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_5 as unsafe fn(&mut Game) -> ()),
        Some(Samus_HandleTransitionsA_5_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Samus_HandleTransitionsA_5(game: &mut Game) {
    kSamus_HandleTransitionsA_5[*((&raw mut game.g_ram as *mut uint8)
        .offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Samus_HandleTransitionsA_5_4(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
}
pub unsafe fn Samus_HandleTransitionsA_5_2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa18 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn sub_91EFC3(game: &mut Game) {}
pub unsafe fn sub_91EFC3_rv(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
static kSamus_HandleTransitionsA_5_1: [Option<Func_U8>; 6] = unsafe {
    [
        Some(Samus_HandleTransitionsA_5_1_0 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphBallBounceNoSpringballTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsA_5_1_2 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphBallBounceSpringballTrans as unsafe fn(&mut Game) -> uint8),
        Some(sub_91EFC3_rv as unsafe fn(&mut Game) -> uint8),
        Some(Samus_HandleTransitionsA_5_1_5 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Samus_HandleTransitionsA_5_1(game: &mut Game) {
    HandleLandingSoundEffectsAndGfx(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        SamusFunc_F1D3(game);
    } else if kSamus_HandleTransitionsA_5_1[*((&raw mut game.g_ram as *mut uint8)
        .offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as usize]
        .expect("non-null function pointer")(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa18 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        SamusFunc_F1D3(game);
    }
}
pub unsafe fn HandleLandingSoundEffectsAndGfx(game: &mut Game) {
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_14_WallJumping as ::core::ffi::c_int)
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x34 as uint16, game);
        } else {
            QueueSfx1_Max6(0x32 as uint16, game);
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            QueueSfx3_Max6(4 as uint16, game);
        }
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            return;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            QueueSfx3_Max6(5 as uint16, game);
        }
    }
    HandleLandingGraphics(game);
}
static off_91F0AE: [Option<Func_V>; 8] = unsafe {
    [
        Some(HandleLandingGraphics_Crateria as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Brinstar as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Norfair as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Norfair as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Maridia as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Tourian as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Ceres as unsafe fn(&mut Game) -> ()),
        Some(HandleLandingGraphics_Ceres as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleLandingGraphics(game: &mut Game) {
    off_91F0AE[*((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn HandleLandingGraphics_Ceres(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
}
static g_byte_91F0F3: [uint8; 17] = [
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
    0 as ::core::ffi::c_int as uint8,
];
pub unsafe fn HandleLandingGraphics_Crateria(game: &mut Game) {
    let mut current_block: u64;
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0)
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 28 as ::core::ffi::c_int
        {
            HandleLandingGraphics_Norfair(game);
            return;
        }
        if !((*((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
        {
            if g_byte_91F0F3[*((&raw mut game.g_ram as *mut uint8)
                .offset(0x79d as ::core::ffi::c_int as isize)
                as *mut uint16) as usize] as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if g_byte_91F0F3[*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x79d as ::core::ffi::c_int as isize)
                    as *mut uint16) as usize] as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 944 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        current_block = 18265313692946425332;
                    } else {
                        current_block = 4562832191328891483;
                    }
                } else if g_byte_91F0F3[*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x79d as ::core::ffi::c_int as isize)
                    as *mut uint16) as usize] as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block = 18265313692946425332;
                } else {
                    current_block = 4562832191328891483;
                }
            } else if *((&raw mut game.g_ram as *mut uint8)
                .offset(0x196e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 10 as ::core::ffi::c_int
            {
                current_block = 4562832191328891483;
            } else {
                current_block = 18265313692946425332;
            }
            match current_block {
                4562832191328891483 => {}
                _ => {
                    HandleLandingGraphics_Maridia(game);
                    return;
                }
            }
        }
    }
    HandleLandingGraphics_Ceres(game);
}
pub unsafe fn HandleLandingGraphics_Maridia(game: &mut Game) {
    let mut bottom: uint16 = Samus_GetBottom_R18(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - bottom as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            return;
        }
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - bottom as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0)
    {
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) =
        (bottom as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) =
        (bottom as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn HandleLandingGraphics_Norfair(game: &mut Game) {
    let mut bottom: uint16 = Samus_GetBottom_R18(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - bottom as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            return;
        }
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - bottom as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0)
    {
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 1536 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 1536 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = bottom;
    *((&raw mut game.g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = bottom;
}
pub unsafe fn HandleLandingGraphics_Brinstar(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        HandleLandingGraphics_Norfair(game);
    } else {
        HandleLandingGraphics_Tourian(game);
    };
}
pub unsafe fn HandleLandingGraphics_Tourian(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 11 as ::core::ffi::c_int)
    {
        HandleLandingGraphics_Norfair(game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
    };
}
pub unsafe fn SamusFunc_F1D3(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb22 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb38 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn Samus_HandleTransitionsA_5_1_0(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e926 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return 0 as uint8;
}
static g_word_909EB5: uint16 = 1 as uint16;
static g_word_909EB7: uint16 = 0 as uint16;
pub unsafe fn Samus_MorphBallBounceNoSpringballTrans(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    v0 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if 2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0
    {
        if v0 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb20 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh25 = (*fresh25).wrapping_add(1);
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) = g_word_909EB7;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (g_word_909EB5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            return 1 as uint8;
        }
        if v0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
            Unreachable(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh26 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh26 = (*fresh26).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = g_word_909EB7;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = g_word_909EB5;
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsA_5_1_2(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 3 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_MorphBallBounceSpringballTrans(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_InitJump(game);
        return 1 as uint8;
    }
    v1 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int) as int16;
    if v1 != 0 {
        if v1 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1538 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) = g_word_909EB7;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (g_word_909EB5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            return 1 as uint8;
        }
        if v1 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
            Unreachable(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1537 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = g_word_909EB7;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = g_word_909EB5;
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsA_5_1_5(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HandleTransitionsA_5_5(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb22 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa18 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    QueueSfx3_Max6(5 as uint16, game);
}
pub unsafe fn Samus_HandleTransitionsA_5_6(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 9 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa46 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    != 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa18 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn Samus_HandleTransitionsC_1(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb22 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    Samus_AlignBottomWithPrevPose(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_HandleTransitionsC_2(game: &mut Game) {
    Samus_AlignBottomWithPrevPose(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kGameState_42_PlayingDemo as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Samus_HandleTransitionsC_3(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh42 = (*fresh42 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_HandleTransitionsC_4(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_HandleTransitionsC_5(game: &mut Game) {
    let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh43 = (*fresh43 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn Samus_HandleTransitionsC_6(game: &mut Game) {
    HandleJumpTransition(game);
}
pub unsafe fn Samus_HandleTransitionsC_7(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v0 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v0 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v1 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v1 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_HandleTransitionsC_8(game: &mut Game) {
    Samus_HandleTransitionsC_1(game);
    Samus_HandleTransitionsC_3(game);
}
pub unsafe fn SamusFunc_F404(game: &mut Game) -> uint8 {
    let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as SamusPose;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        HandleCollDueToChangedPose(game);
        SamusFunc_F433(game);
        HandleJumpTransition(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return (v1 as ::core::ffi::c_uint
        != *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_uint) as ::core::ffi::c_int as uint8;
}
pub unsafe fn SamusFunc_F433(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(0 as ::core::ffi::c_int as isize))
    .pose_x_dir)
        .offset(
            (8 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as isize,
        ) as *mut uint16);
    SamusFunc_F468(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_14_WallJumping as ::core::ffi::c_int)
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        Samus_LoadSuitPalette(game);
    }
}
static off_91F4A2: [Option<Func_U8>; 28] = unsafe {
    [
        Some(SamusFunc_F468_Standing as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Running as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_NormalJump as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_SpinJump as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_MorphBall as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Crouching as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Falling as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_MorphBall as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurningAroundOnGround as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_CrouchTransEtc as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Moonwalking as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Springball as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_WallJumping as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurnAroundJumping as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_TurnAroundFalling as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_DamageBoost as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Unused as unsafe fn(&mut Game) -> uint8),
        Some(SamusFunc_F468_Shinespark as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SamusFunc_F468(game: &mut Game) {
    if off_91F4A2
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(0 as ::core::ffi::c_int as isize))
        .pose_x_dir)
            .offset(
                (8 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as isize,
            ) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 3584 as ::core::ffi::c_int
        {
            off_91F4A2[14 as ::core::ffi::c_int as usize].expect("non-null function pointer")(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut SamusPoseParams)
                .offset(0 as ::core::ffi::c_int as isize))
            .pose_x_dir)
                .offset(
                    (8 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa1c as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as isize,
                ) as *mut uint16);
        }
    }
}
pub unsafe fn SamusFunc_F468_Unused(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_Standing(game: &mut Game) -> uint8 {
    if ((*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .direction_shots_fired
        == 0
        || (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .direction_shots_fired as ::core::ffi::c_int
            == 9 as ::core::ffi::c_int)
        && ((*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .direction_shots_fired
            == 0
            || (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa20 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .direction_shots_fired as ::core::ffi::c_int
                == 9 as ::core::ffi::c_int)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_Running(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x918000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdf8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        return 0 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
            | kPose_01_FaceR_Normal as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_25_FaceR_Turn_Stand as ::core::ffi::c_int as uint16;
        return 1 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
            | kPose_02_FaceL_Normal as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_26_FaceL_Turn_Stand as ::core::ffi::c_int as uint16;
        return 1 as uint8;
    }
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_NormalJump(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_15_FaceR_Jump_AimU as ::core::ffi::c_int
        {
            current_block = 10566234459686315488;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_16_FaceL_Jump_AimU as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_6A_FaceL_Jump_AimUL as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_69_FaceR_Jump_AimUR as ::core::ffi::c_int
            {
                current_block = 10164294398894088689;
            } else {
                current_block = 10566234459686315488;
            }
        } else {
            current_block = 1917311967535052937;
        }
        match current_block {
            10164294398894088689 => {}
            1917311967535052937 => {}
            _ => {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa68 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    != 0
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) =
                        kPose_C7_FaceR_ShinesparkWindup_Vert as ::core::ffi::c_int as uint16;
                    current_block = 13140864810214251092;
                } else {
                    current_block = 10164294398894088689;
                }
            }
        }
    } else {
        current_block = 1917311967535052937;
    }
    match current_block {
        1917311967535052937 => {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) =
                    kPose_C8_FaceL_ShinesparkWindup_Vert as ::core::ffi::c_int as uint16;
                current_block = 13140864810214251092;
            } else {
                current_block = 10164294398894088689;
            }
        }
        _ => {}
    }
    match current_block {
        13140864810214251092 => {
            Projectile_Func7_Shinespark(game);
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == kMovementType_02_NormalJumping as ::core::ffi::c_int
            {
                let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh2 = (*fresh2).wrapping_sub(1);
                *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *fresh2;
            }
            return 1 as uint8;
        }
        _ => {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
                || *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb44 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 2 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_15_FaceR_Jump_AimU as ::core::ffi::c_int
                || *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_16_FaceL_Jump_AimU as ::core::ffi::c_int)
                && (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa20 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_55_FaceR_Jumptrans_AimU as ::core::ffi::c_int
                    || *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa20 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        == kPose_56_FaceL_Jumptrans_AimU as ::core::ffi::c_int)
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize)
                    as *mut uint16) = ((*(game.g_rom.offset(
                    (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8
                    as *mut SamusPoseParams)
                    .offset(
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa1c as ::core::ffi::c_int as isize)
                            as *mut uint16) as isize,
                    ))
                .direction_shots_fired as ::core::ffi::c_int
                    | 0x8000 as ::core::ffi::c_int) as uint16;
            }
            return 0 as uint8;
        }
    };
}
pub unsafe fn SamusFunc_F468_Crouching(game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_85_FaceR_Crouch_AimU as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_86_FaceL_Crouch_AimU as ::core::ffi::c_int)
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_F2_FaceL_CrouchTrans_AimU as ::core::ffi::c_int)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_Falling(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_SpinJump(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_03_SpinJumping as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 8 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 772 as ::core::ffi::c_int
            {
                current_block = 12119026790781926931;
            } else {
                current_block = 820271813250567934;
            }
        } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            != 4 as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 776 as ::core::ffi::c_int
        {
            current_block = 12119026790781926931;
        } else {
            current_block = 820271813250567934;
        }
        match current_block {
            12119026790781926931 => {}
            _ => {
                AddToHiLo(
                    (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xb44 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                );
                Samus_CancelSpeedBoost(game);
                SetHiLo(
                    (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    0 as uint32,
                );
                *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
            }
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut r20: uint16 = Samus_GetTop_R20(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                    && *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1962 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - r20 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                {
                    return 0 as uint8;
                }
            } else if *((&raw mut game.g_ram as *mut uint8)
                .offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                return 0 as uint8;
            }
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_82_FaceL_Screwattack as ::core::ffi::c_int as uint16;
            current_block = 131178307184372633;
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x3e as uint16, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int as uint16;
                return 0 as uint8;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1f51 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
            {
                current_block = 807399161326926732;
            } else {
                return 0 as uint8;
            }
        }
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut r20_0: uint16 = Samus_GetTop_R20(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                    && *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1962 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - r20_0 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                {
                    return 0 as uint8;
                }
            } else if *((&raw mut game.g_ram as *mut uint8)
                .offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r20_0 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                return 0 as uint8;
            }
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_81_FaceR_Screwattack as ::core::ffi::c_int as uint16;
            current_block = 131178307184372633;
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa9a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != 0
                    || *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1f51 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 0
                {
                    return 0 as uint8;
                }
            } else {
                QueueSfx1_Max6(0x3e as uint16, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int as uint16;
                return 0 as uint8;
            }
            current_block = 807399161326926732;
        }
    }
    match current_block {
        807399161326926732 => {
            QueueSfx1_Max6(0x31 as uint16, game);
            return 0 as uint8;
        }
        _ => {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                QueueSfx1_Max6(0x33 as uint16, game);
            }
            return 0 as uint8;
        }
    };
}
static off_91F790: [Option<Func_U8>; 12] = unsafe {
    [
        Some(Samus_CrouchTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_CrouchTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(MaybeUnused_sub_91F7F4 as unsafe fn(&mut Game) -> uint8),
        Some(MaybeUnused_sub_91F840 as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
    ]
};
static off_91F7A8: [Option<Func_U8>; 4] = unsafe {
    [
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_MorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
        Some(Samus_StandOrUnmorphTrans as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn SamusFunc_F468_CrouchTransEtc(game: &mut Game) -> uint8 {
    let mut v0: uint16 = 0;
    's_43: {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - kPose_DB as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = (2 as ::core::ffi::c_int
                    * (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 53 as ::core::ffi::c_int)) as uint16;
            } else {
                return off_91F7A8[(*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 219 as ::core::ffi::c_int) as usize]
                    .expect("non-null function pointer")(game);
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_F7_FaceR_StandTrans_AimU as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 0 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 7 as uint16;
            break 's_43;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 7 as uint16;
        return off_91F790[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(game);
    }
    return 0 as uint8;
}
pub unsafe fn Samus_CrouchTrans(game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn Samus_StandOrUnmorphTrans(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn Samus_MorphTrans(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_03_SpinJumping as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 1 as uint8;
    };
}
pub unsafe fn MaybeUnused_sub_91F7F4(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_08_MorphBallFalling as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_13_SpringBallFalling as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_7D_FaceR_Springball_Fall as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_31_FaceR_Morphball_Air as ::core::ffi::c_int as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn MaybeUnused_sub_91F840(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_08_MorphBallFalling as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_13_SpringBallFalling as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_7E_FaceL_Springball_Fall as ::core::ffi::c_int as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = kPose_32_FaceL_Morphball_Air as ::core::ffi::c_int as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_41_FaceL_Morphball_Ground as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_Moonwalking(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_25_FaceR_Turn_Stand as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_26_FaceL_Turn_Stand as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_DamageBoost(game: &mut Game) -> uint8 {
    return SamusFunc_F468_DamageBoost_(game);
}
pub unsafe fn MaybeUnused_sub_91F8B0(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_54_FaceL_Knockback as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_53_FaceR_Knockback as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_DamageBoost_(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_TurningAroundOnGround(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
    {
        let mut v0: uint16 = (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .direction_shots_fired as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_10_Moonwalking as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (v0 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = *(game.g_rom.offset(
                    (((0x91f9ea as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91f9ea as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v0 as isize) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = *(game.g_rom.offset(
                    (((0x91f9c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91f9c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v0 as isize) as uint16;
            }
        } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 5 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91f9cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91f9cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v0 as isize) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) = *(game.g_rom.offset(
                (((0x91f9c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91f9c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v0 as isize) as uint16;
        }
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_TurnAroundJumping(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = *(game.g_rom.offset(
        (((0x91f9d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91f9d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset(
            (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa20 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .direction_shots_fired as isize,
        ) as uint16;
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_TurnAroundFalling(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = *(game.g_rom.offset(
        (((0x91f9e0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91f9e0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset(
            (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa20 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .direction_shots_fired as isize,
        ) as uint16;
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16,
        0 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusFunc_F468_MorphBall(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_04_MorphBallOnGround as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_08_MorphBallFalling as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x918000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    SamusFunc_FA0A(game);
    return 0 as uint8;
}
pub unsafe fn SamusFunc_FA0A(game: &mut Game) {
    let mut current_block: u64;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 4 as ::core::ffi::c_int
        {
            return;
        }
        current_block = 17179679302217393232;
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        current_block = 17179679302217393232;
    } else {
        current_block = 7351195479953500246;
    }
    match current_block {
        17179679302217393232 => {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb44 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            );
            Samus_CancelSpeedBoost(game);
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                0 as uint32,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
        _ => {}
    };
}
pub unsafe fn SamusFunc_F468_Springball(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_11_SpringBallOnGround as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_12_SpringBallInAir as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_13_SpringBallFalling as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x918000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    SamusFunc_FA0A(game);
    return 0 as uint8;
}
pub unsafe fn SamusFunc_F468_WallJumping(game: &mut Game) -> uint8 {
    let mut bottom: uint16 = Samus_GetBottom_R18(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - bottom as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            return 0 as uint8;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - bottom as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 1536 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = bottom;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 8 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 6 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 6 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
static kSamusFunc_F468_Shinespark: [uint16; 6] = [
    fnSamus_MoveHandler_Shinespark_Horiz as uint16,
    fnSamus_MoveHandler_Shinespark_Horiz as uint16,
    fnSamus_MoveHandlerVerticalShinespark as uint16,
    fnSamus_MoveHandlerVerticalShinespark as uint16,
    fnSamus_MoveHandler_Shinespark_Diag as uint16,
    fnSamus_MoveHandler_Shinespark_Diag as uint16,
];
pub unsafe fn SamusFunc_F468_Shinespark(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - kPose_CF_FaceR_Ranintowall_AimUR as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            kSamusFunc_F468_Shinespark[(*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 201 as ::core::ffi::c_int) as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        QueueSfx3_Max9(0xf as uint16, game);
    }
    return 0 as uint8;
}
static kSamusPhys_AnimDelayInWater: uint16 = 3 as uint16;
static kSamusPhys_AnimDelayInAcid: uint16 = 2 as uint16;
pub unsafe fn Samus_SetAnimationFrameIfPoseChanged(game: &mut Game) {
    let mut t: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        t = *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16);
    } else {
        let mut r: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .y_radius as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            t = (if *((&raw mut game.g_ram as *mut uint8)
                .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                - r as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                kSamusPhys_AnimDelayInWater as ::core::ffi::c_int
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            }) as uint16;
        } else if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            t = *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                as *mut uint16);
        } else {
            t = kSamusPhys_AnimDelayInAcid;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa9a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = (t as ::core::ffi::c_int
            + *RomPtr_91(
                (*(game.g_rom.offset(
                    (((0x91b010 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b010 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa1c as ::core::ffi::c_int as isize)
                            as *mut uint16) as isize,
                    ) as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa9a as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16_t,
                game,
            ) as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SamusFunc_EC80(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_06_Falling as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_06_Falling as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn nullsub_24(game: &mut Game) {}
pub unsafe fn nullsub_25(game: &mut Game) {}
static kHandleJumpTrans: [Option<Func_V>; 28] = unsafe {
    [
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_NormalJump as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_SpinJump as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(UNUSED_sub_91FC42 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_SpringBallInAir as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(HandleJumpTransition_WallJump as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_25 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_24 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleJumpTransition(game: &mut Game) {
    kHandleJumpTrans[*(&raw mut game.g_ram as *mut uint8)
        .offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn HandleJumpTransition_WallJump(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        Samus_InitWallJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpringBallInAir(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_7F_FaceR_Springball_Air as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != kMovementType_11_SpringBallOnGround as ::core::ffi::c_int
        {
            return;
        }
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_80_FaceL_Springball_Air as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kMovementType_11_SpringBallOnGround as ::core::ffi::c_int)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn UNUSED_sub_91FC42(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
            | kPose_01_FaceR_Normal as ::core::ffi::c_int
            | 0x20 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 100 as ::core::ffi::c_int
        {
            return;
        }
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
            | kPose_02_FaceL_Normal as ::core::ffi::c_int
            | 0x20 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 99 as ::core::ffi::c_int)
    {
        return;
    }
    Samus_InitJump(game);
}
pub unsafe fn HandleJumpTransition_NormalJump(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_4B_FaceR_Jumptrans as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_4C_FaceL_Jumptrans as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_55_FaceR_Jumptrans_AimU as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - kPose_5B as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_27_FaceR_Crouch as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_28_FaceL_Crouch as ::core::ffi::c_int
        {
            let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh10 = (*fresh10 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as uint16;
        }
        Samus_InitJump(game);
    }
}
pub unsafe fn HandleJumpTransition_SpinJump(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_03_SpinJumping as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa23 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        Samus_InitJump(game);
    }
}
pub unsafe fn Samus_Func20(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 14 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
        {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_25_FaceR_Turn_Stand as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) = kPose_D6_FaceL_Xray_Stand as ::core::ffi::c_int as uint16;
                } else {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) =
                        kPose_DA_FaceL_Xray_Crouch as ::core::ffi::c_int as uint16;
                }
            } else if *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_26_FaceL_Turn_Stand as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_D5_FaceR_Xray_Stand as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_D9_FaceR_Xray_Crouch as ::core::ffi::c_int as uint16;
            }
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa22 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa1e as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    } else {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9b0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) = (256 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 5 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_26_FaceL_Turn_Stand as ::core::ffi::c_int as uint16;
            }
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9ae as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) = (256 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == kMovementType_05_Crouching as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_43_FaceR_Turn_Crouch as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_25_FaceR_Turn_Stand as ::core::ffi::c_int as uint16;
            }
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    };
}
static off_91FE8A: [Option<Func_U8>; 4] = unsafe {
    [
        Some(HandleCollDueToChangedPose_Solid_NoColl as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollAbove as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollBelow as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Solid_CollBoth as unsafe fn(&mut Game) -> uint8),
    ]
};
static off_91FE92: [Option<Func_U8>; 4] = unsafe {
    [
        Some(HandleCollDueToChangedPose_Block_NoColl as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollAbove as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollBelow as unsafe fn(&mut Game) -> uint8),
        Some(HandleCollDueToChangedPose_Block_CollBoth as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn HandleCollDueToChangedPose(game: &mut Game) {
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    let mut amt: int32 = 0;
    let mut v0: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
    {
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    v0 = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as int16;
    if v0 as ::core::ffi::c_int
        - (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_radius as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
        as *mut uint16) = ((*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as ::core::ffi::c_int
        - v0 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    cres = Samus_CheckSolidEnemyColl(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = cres.collision;
    amt = cres.amt;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa3e as ::core::ffi::c_int as isize)
        as *mut uint16) = (amt >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 3 as uint16;
    cres = Samus_CheckSolidEnemyColl(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = cres.collision;
    amt = cres.amt;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa34 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh11 = (*fresh11 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa40 as ::core::ffi::c_int as isize)
        as *mut uint16) = (amt >> 16 as ::core::ffi::c_int) as uint16;
    if off_91FE8A[*((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        return;
    }
    amt = Samus_CollDetectChangedPose(
        (-(*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa38 as ::core::ffi::c_int as isize)
        as *mut uint16) = (amt >> 16 as ::core::ffi::c_int) as uint16;
    amt = Samus_CollDetectChangedPose(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa36 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh12 = (*fresh12 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa3c as ::core::ffi::c_int as isize)
        as *mut uint16) = (amt >> 16 as ::core::ffi::c_int) as uint16;
    if off_91FE92[*((&raw mut game.g_ram as *mut uint8).offset(0xa36 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
}
pub unsafe fn HandleCollDueToChangedPose_Solid_NoColl(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollBoth(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollAbove(game: &mut Game) -> uint8 {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb00 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 3 as uint16;
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xa3e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = cres.collision;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa3e as ::core::ffi::c_int as isize)
        as *mut uint16) = (cres.amt >> 16 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Solid_CollBelow(game: &mut Game) -> uint8 {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb00 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xa40 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16) = cres.collision;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa40 as ::core::ffi::c_int as isize)
        as *mut uint16) = (cres.amt >> 16 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollAbove(game: &mut Game) -> uint8 {
    let mut amt: int32 = Samus_CollDetectChangedPose(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xa38 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 1 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh5 = (*fresh5 as ::core::ffi::c_int
        + (amt >> 16 as ::core::ffi::c_int) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBelow(game: &mut Game) -> uint8 {
    let mut amt: int32 = Samus_CollDetectChangedPose(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa3c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xa3a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 1 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return HandleCollDueToChangedPose_Block_CollBoth(game);
    }
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int
        - (amt >> 16 as ::core::ffi::c_int) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    return 0 as uint8;
}
pub unsafe fn HandleCollDueToChangedPose_Block_NoColl(game: &mut Game) -> uint8 {
    match *((&raw mut game.g_ram as *mut uint8).offset(0xa34 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
    {
        0 => return 0 as uint8,
        1 => {
            let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh8 = (*fresh8 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xa3e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            return 0 as uint8;
        }
        2 => {
            let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh9 = (*fresh9 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xa40 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            return 0 as uint8;
        }
        3 => return HandleCollDueToChangedPose_Block_CollBoth(game),
        _ => {
            Unreachable(game);
            return 0 as uint8;
        }
    };
}
pub unsafe fn HandleCollDueToChangedPose_Block_CollBoth(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) = (if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        kPose_28_FaceL_Crouch as ::core::ffi::c_int
    } else {
        kPose_27_FaceR_Crouch as ::core::ffi::c_int
    }) as uint16;
    let mut r18: uint16 = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh6 = (*fresh6 as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    return 0 as uint8;
}
