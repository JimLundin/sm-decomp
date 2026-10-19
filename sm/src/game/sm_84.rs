use crate::game::sm_80::{
    AddGrappleToHudTilemap, AddMissilesToHudTilemap, AddPowerBombsToHudTilemap,
    AddSuperMissilesToHudTilemap, AddXrayToHudTilemap, CheckBossBitForCurArea, CheckEventHappened,
    PrepareBitAccess, QueueMusic_Delayed8, QueueSfx1_Max1, QueueSfx1_Max15, QueueSfx1_Max3,
    QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15, QueueSfx2_Max3,
    QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15, QueueSfx3_Max3,
    QueueSfx3_Max6, QueueSfx3_Max9, SetEventHappened,
};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_85::{DisplayMessageBox, DisplayMessageBox_Async, DisplayMessageBox_Poll};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_90::{CallSomeSamusCode, UpdateBeamTilesAndPalette};
use crate::game::sm_91::{
    GravitySuitPickup, LoadBlockToXrayTilemap, MakeSamusFaceForward, VariaSuitPickup,
};
use crate::game_state::Game;
use crate::rtl::abs16;

use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    CoroutineRet, Func_Y_V, LongPtr, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PlmHeader_Size4 {
    pub func_ptr: VoidP,
    pub instr_list_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct XraySpecialCasing {
    pub x_block: uint8,
    pub y_block: uint8,
    pub level_data_block: uint16,
}
pub use crate::types::{EnemyData, RoomPlmEntry, SamusPose};
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
pub use crate::types::{Consts_60, RoomDefRoomstate, TileTable, TileTables};
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
pub use crate::types::Consts_84;
pub const addr_locret_84D779: Consts_84 = 55161;
pub const addr_kPlmHeader_D6EA: Consts_84 = 55018;
pub const addr_kPlmHeader_D113: Consts_84 = 53523;
pub const addr_kPlmInstrList_C922: Consts_84 = 51490;
pub const addr_kPlmInstrList_C91C: Consts_84 = 51484;
pub const addr_locret_84BBDC: Consts_84 = 48092;
pub const addr_locret_84BB6A: Consts_84 = 47978;
pub const addr_kPlmHeader_B6FF: Consts_84 = 46847;
pub const addr_word_84AD76: Consts_84 = 44406;
pub const addr_kDefaultPlmDrawInstruction: Consts_84 = 36256;
pub const addr_locret_848AE0: Consts_84 = 35552;
pub const addr_PlmPreInstr_Empty4: Consts_84 = 35494;
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
pub use crate::types::SpawnHardcodedPlmArgs;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
pub const kCoroutineNone: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_RAM(mut addr: uint16_t, game: &mut Game) -> *mut uint8 {
    '_c2rust_label: {
        if (addr as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const ::core::ffi::c_char,
                51 as ::core::ffi::c_uint,
                b"uint8 *RomPtr_RAM(uint16_t)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    return (&raw mut game.g_ram as *mut uint8).offset(addr as ::core::ffi::c_int as isize);
}
#[inline]
unsafe fn RomPtr_84(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x840000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_8F(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomBankBase(mut bank: uint8, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize));
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_RoomDefRoomstate(mut a: uint16, game: &mut Game) -> *mut RoomDefRoomstate {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut RoomDefRoomstate;
}
#[inline]
unsafe fn get_RoomPlmEntry(mut a: uint16, game: &mut Game) -> *mut RoomPlmEntry {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut RoomPlmEntry;
}
#[inline]
unsafe fn get_PlmHeader_Size4(mut a: uint16, game: &mut Game) -> *mut PlmHeader_Size4 {
    return RomPtr(
        (0x840000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut PlmHeader_Size4;
}
pub const fnPlmPreInstr_nullsub_60: uint32 = 8684649;
pub const fnPlmPreInstr_Empty2: ::core::ffi::c_int = 0x8484e6;
pub const fnPlmPreInstr_Empty3: ::core::ffi::c_int = 0x84853d;
pub const fnPlmInstr_Sleep: uint32 = 8685236;
pub const fnPlmInstr_Delete: uint32 = 8685244;
pub const fnPlmInstr_PreInstr: uint32 = 8685249;
pub const fnPlmInstr_ClearPreInstr: uint32 = 8685258;
pub const fnPlmPreInstr_Empty: ::core::ffi::c_int = 0x8486d0;
pub const fnPlmInstr_CallFunction: uint32 = 8685323;
pub const fnPlmInstr_Goto: uint32 = 8685348;
pub const fnPlmInstr_DecrementAndBranchNonzero: uint32 = 8685375;
pub const fnPlmInstr_SetTimer: uint32 = 8685390;
pub const fnPlmInstr_LoadItemPlmGfx: uint32 = 8685412;
pub const fnPlmInstr_CopyFromRamToVram: uint32 = 8685541;
pub const fnPlmInstr_GotoIfBossBitSet: uint32 = 8685582;
pub const fnPlmInstr_GotoIfEventSet: uint32 = 8685613;
pub const fnPlmInstr_SetEvent: uint32 = 8685630;
pub const fnPlmInstr_GotoIfChozoSet: uint32 = 8685640;
pub const fnPlmInstr_SetRoomChozoBit: uint32 = 8685669;
pub const fnPlmInstr_GotoIfItemBitSet: uint32 = 8685692;
pub const fnPlmInstr_SetItemBit: uint32 = 8685721;
pub const fnPlmInstr_PickupBeamAndShowMessage: uint32 = 8685744;
pub const fnPlmInstr_PickupEquipmentAndShowMessage: uint32 = 8685811;
pub const fnPlmInstr_PickupEquipmentAddGrappleShowMessage: uint32 = 8685850;
pub const fnPlmInstr_PickupEquipmentAddXrayShowMessage: uint32 = 8685889;
pub const fnPlmInstr_CollectHealthEnergyTank: uint32 = 8685928;
pub const fnPlmInstr_CollectHealthReserveTank: uint32 = 8685958;
pub const fnPlmInstr_CollectAmmoMissileTank: uint32 = 8685993;
pub const fnPlmInstr_CollectAmmoSuperMissileTank: uint32 = 8686034;
pub const fnPlmInstr_CollectAmmoPowerBombTank: uint32 = 8686075;
pub const fnPlmInstr_SetLinkReg: uint32 = 8686116;
pub const fnPlmInstr_Call: uint32 = 8686126;
pub const fnPlmInstr_Return: uint32 = 8686138;
pub const fnPlmInstr_GotoIfDoorBitSet: uint32 = 8686194;
pub const fnPlmInstr_IncrementDoorHitCounterAndJGE: uint32 = 8686225;
pub const fnPlmInstr_IncrementArgumentAndJGE: uint32 = 8686285;
pub const fnlocret_848AE0: uint32 = 8686304;
pub const fnPlmInstr_SetBTS: uint32 = 8686321;
pub const fnPlmInstr_DrawPlmBlock: uint32 = 8686341;
pub const fnPlmInstr_DrawPlmBlock_: uint32 = 8686359;
pub const fnPlmInstr_ProcessAirScrollUpdate: uint32 = 8686421;
pub const fnPlmInstr_ProcessSolidScrollUpdate: uint32 = 8686483;
pub const fnPlmInstr_QueueMusic: uint32 = 8686545;
pub const fnPlmInstr_ClearMusicQueueAndQueueTrack: uint32 = 8686557;
pub const fnPlmInstr_QueueSfx1_Max6: uint32 = 8686599;
pub const fnPlmInstr_QueueSfx2_Max6: uint32 = 8686608;
pub const fnPlmInstr_QueueSfx3_Max6: uint32 = 8686617;
pub const fnPlmInstr_QueueSfx1_Max15: uint32 = 8686626;
pub const fnPlmInstr_QueueSfx2_Max15: uint32 = 8686635;
pub const fnPlmInstr_QueueSfx3_Max15: uint32 = 8686644;
pub const fnPlmInstr_QueueSfx1_Max3: uint32 = 8686653;
pub const fnPlmInstr_QueueSfx2_Max3: uint32 = 8686662;
pub const fnPlmInstr_QueueSfx_Max3: uint32 = 8686671;
pub const fnPlmInstr_QueueSfx1_Max9: uint32 = 8686680;
pub const fnPlmInstr_QueueSfx2_Max9: uint32 = 8686689;
pub const fnPlmInstr_QueueSfx3_Max9: uint32 = 8686698;
pub const fnPlmInstr_QueueSfx1_Max1: uint32 = 8686707;
pub const fnPlmInstr_QueueSfx2_Max1: uint32 = 8686716;
pub const fnPlmInstr_QueueSfx3_Max1: uint32 = 8686725;
pub const fnPlmInstr_ActivateMapStation: uint32 = 8686735;
pub const fnPlmInstr_ActivateEnergyStation: uint32 = 8686767;
pub const fnPlmInstr_ActivateMissileStation: uint32 = 8686800;
pub const fnPlmInstr_ActivateSaveStationAndGotoIfNo: uint32 = 8686833;
pub const fnPlmInstr_GotoIfSamusNear: uint32 = 8686913;
pub const fnPlmInstr_MovePlmDownOneBlock: uint32 = 8694528;
pub const fnnullsub_67: uint32 = 8694567;
pub const fnPlmSetup_CrumbleBotwoonWall: uint32 = 8694568;
pub const fnPlmInstr_Scroll_0_1_Blue: uint32 = 8694609;
pub const fnPlmInstr_MovePlmDownOneBlock_0: uint32 = 8694617;
pub const fnPlmInstr_ABD6: uint32 = 8694742;
pub const fnPlmPreInstr_PositionSamusAndInvincible: uint32 = 8694921;
pub const fnPlmInstr_DealDamage_2: uint32 = 8694941;
pub const fnPlmInstr_GiveInvincibility: uint32 = 8694961;
pub const fnPlmInstr_Draw0x38FramesOfRightTreadmill: uint32 = 8695107;
pub const fnPlmInstr_Draw0x38FramesOfLeftTreadmill: uint32 = 8695128;
pub const fnPlmInstr_GotoIfSamusHealthFull: uint32 = 8695349;
pub const fnPlmInstr_GotoIfMissilesFull: uint32 = 8695487;
pub const fnPlmInstr_PlaceSamusOnSaveStation: uint32 = 8695822;
pub const fnPlmInstr_DisplayGameSavedMessageBox: uint32 = 8695844;
pub const fnPlmInstr_EnableMovementAndSetSaveStationUsed: uint32 = 8695856;
pub const fnPlmSetup_SetrupWreckedShipEntrance: ::core::ffi::c_int = 0x84b04a;
pub const fnPlmSetup_BTS_Brinstar_0x80_Floorplant: uint32 = 8696028;
pub const fnPlmSetup_BTS_Brinstar_0x81_Ceilingplant: uint32 = 8696083;
pub const fnPlmSetup_B6D3_MapStation: uint32 = 8696203;
pub const fnPlmSetup_Bts47_MapStationRightAccess: uint32 = 8696264;
pub const fnPlmSetup_Bts4_MapStationLeftAccess: uint32 = 8696304;
pub const fnPlmSetup_PlmB6DF_EnergyStation: uint32 = 8696349;
pub const fnPlmSetup_PlmB6EB_EnergyStation: uint32 = 8696389;
pub const fnPlmSetup_B6E3_EnergyStationRightAccess: uint32 = 8696429;
pub const fnPlmSetup_B6E7_EnergyStationLeftAccess: uint32 = 8696477;
pub const fnPlmSetup_B6EF_MissileStationRightAccess: uint32 = 8696528;
pub const fnPlmSetup_B6F3_MissileStationLeftAccess: uint32 = 8696576;
pub const fnPlmSetup_B638_Rightwards_Extension: uint32 = 8696634;
pub const fnPlmSetup_B63F_Leftwards_Extension: uint32 = 8696645;
pub const fnPlmSetup_B643_Downwards_Extension: uint32 = 8696656;
pub const fnPlmSetup_B647_Upwards_Extension: uint32 = 8696667;
pub const fnPlmSetup_B703_ScrollPLM: uint32 = 8696689;
pub const fnPlmSetup_B707_SolidScrollPLM: uint32 = 8696706;
pub const fnPlmSetup_B6FF_ScrollBlockTouch: uint32 = 8696723;
pub const fnPlmSetup_DeactivatePlm: uint32 = 8696769;
pub const fnnullsub_290: uint32 = 8696783;
pub const fnPlmSetup_ReturnCarryClear: uint32 = 8696784;
pub const fnPlmSetup_ReturnCarrySet: uint32 = 8696786;
pub const fnPlmSetup_D094_EnemyBreakableBlock: uint32 = 8696788;
pub const fnUNUSED_sub_84B3E3: uint32 = 8696803;
pub const fnPlmSetup_B70F_IcePhysics: uint32 = 8696811;
pub const fnPlmSetup_QuicksandSurface: uint32 = 8696840;
pub const fnPlmSetup_B71F_SubmergingQuicksand: uint32 = 8696983;
pub const fnPlmSetup_B723_SandfallsSlow: uint32 = 8697000;
pub const fnPlmSetup_B727_SandFallsFast: uint32 = 8697014;
pub const fnPlmSetup_QuicksandSurfaceB: uint32 = 8697028;
pub const fnPlmSetup_B737_SubmergingQuicksand: uint32 = 8697153;
pub const fnPlmSetup_B73B_B73F_SandFalls: uint32 = 8697167;
pub const fnPlmSetup_ClearShitroidInvisibleWall: uint32 = 8697169;
pub const fnPlmSetup_B767_ClearShitroidInvisibleWall: uint32 = 8697199;
pub const fnPlmSetup_B76B_SaveStationTrigger: uint32 = 8697232;
pub const fnPlmSetup_B76F_SaveStation: uint32 = 8697326;
pub const fnPlmSetup_MotherBrainRoomEscapeDoor: uint32 = 8697336;
pub const fnPlmSetup_B7EB_EnableSoundsIn32Frames: uint32 = 8697795;
pub const fnPlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm: ::core::ffi::c_int = 0x84b7dd;
pub const fnPlmPreInstr_WakeAndLavaIfBoosterCollected: uint32 = 8697839;
pub const fnPlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft: uint32 = 8697898;
pub const fnPlmPreInstr_AdvanceLavaSamusMovesLeft: uint32 = 8697926;
pub const fnPlmSetup_SpeedBoosterEscape: uint32 = 8698012;
pub const fnPlmPreInstr_ShaktoolsRoom: uint32 = 8698032;
pub const fnPlmSetup_ShaktoolsRoom: uint32 = 8698076;
pub const fnnullsub_68: uint32 = 8698095;
pub const fnPlmPreInstr_OldTourianEscapeShaftEscape: uint32 = 8698151;
pub const fnPlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft: uint32 = 8698184;
pub const fnPlmSetup_B974: uint32 = 8698220;
pub const fnPlmSetup_B9C1_CrittersEscapeBlock: uint32 = 8698232;
pub const fnPlmInstr_SetCrittersEscapedEvent: uint32 = 8698297;
pub const fnPlmSetup_B9ED_CrittersEscapeBlock: uint32 = 8698309;
pub const fnsub_84B9F1: uint32 = 8698353;
pub const fnPlmInstr_JumpIfSamusHasNoBombs: uint32 = 8698479;
pub const fnnullsub_84BAFA: uint32 = 8698618;
pub const fnPlmSetup_BB30_CrateriaMainstreetEscape: uint32 = 8698633;
pub const fnPlmInstr_MovePlmRight4Blocks: uint32 = 8698661;
pub const fnPlmPreInstr_WakePlmIfTriggered: uint32 = 8698706;
pub const fnnullsub_359: uint32 = 8698730;
pub const fnPlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm: uint32 = 8698731;
pub const fnPlmPreInstr_Empty5: ::core::ffi::c_int = 0x84bba3;
pub const fnPlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm: uint32 = 8698788;
pub const fnPlmInstr_ClearTrigger: uint32 = 8698845;
pub const fnPlmInstr_SpawnEproj: uint32 = 8698849;
pub const fnPlmInstr_WakeEprojAtPlmPos: uint32 = 8698864;
pub const fnPlmPreInstr_GoToLinkInstrIfShot: uint32 = 8699151;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithPowerBomb: uint32 = 8699174;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithAnyMissile: uint32 = 8699216;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithSuperMissile: uint32 = 8699272;
pub const fnPlmPreInstr_GoToLinkInstruction: uint32 = 8699314;
pub const fnPlmPreInstr_GotoLinkIfBoss1Dead: uint32 = 8699348;
pub const fnPlmPreInstr_GotoLinkIfMiniBossDead: uint32 = 8699363;
pub const fnPlmPreInstr_GotoLinkIfTorizoDead: uint32 = 8699378;
pub const fnPlmPreInstr_GotoLinkIfEnemyDeathQuotaOk: uint32 = 8699393;
pub const fnPlmPreInstr_PlayDudSound: uint32 = 8699420;
pub const fnPlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing: uint32 = 8699423;
pub const fnPlmPreInstr_GotoLinkIfCrittersEscaped: uint32 = 8699440;
pub const fnPlmInstr_SetGreyDoorPreInstr: uint32 = 8699455;
pub const fnPlmSetup_C806_LeftGreenGateTrigger: uint32 = 8701261;
pub const fnPlmSetup_C80A_RightGreenGateTrigger: uint32 = 8701292;
pub const fnPlmSetup_C80E_LeftRedGateTrigger: uint32 = 8701323;
pub const fnPlmSetup_C812_RightRedGateTrigger: uint32 = 8701359;
pub const fnPlmSetup_C81E_LeftYellowGateTrigger: uint32 = 8701395;
pub const fnPlmSetup_C822_RightYellowGateTrigger: uint32 = 8701425;
pub const fnPlmSetup_C816_LeftBlueGateTrigger: uint32 = 8701456;
pub const fnPlmSetup_C81A_RightBlueGateTrigger: uint32 = 8701479;
pub const fnPlmSetup_C82A_DownwardsClosedGate: uint32 = 8701630;
pub const fnPlmSetup_C832_UpwardsClosedGate: uint32 = 8701643;
pub const fnPlmSetup_C826_DownwardsOpenGate: uint32 = 8701656;
pub const fnPlmSetup_C82E_UpwardsOpenGate: uint32 = 8701660;
pub const fnPlmSetup_C836_DownwardsGateShootblock: uint32 = 8701664;
pub const fnPlmSetup_C73A_UpwardsGateShootblock: uint32 = 8701754;
pub const fnPlmSetup_C794_GreyDoor: uint32 = 8701844;
pub const fnPlmSetup_Door_Colored: uint32 = 8701873;
pub const fnPlmSetup_Door_Blue: uint32 = 8701883;
pub const fnPlmSetup_Door_Strange: uint32 = 8701922;
pub const fnPlmInstr_SetBtsTo1: uint32 = 8703379;
pub const fnPlmSetup_D028_D02C_Unused: uint32 = 8703426;
pub const fnPlmSetup_RespawningSpeedBoostBlock: uint32 = 8703466;
pub const fnPlmSetup_RespawningCrumbleBlock: uint32 = 8703543;
pub const fnPlmSetup_RespawningShotBlock: uint32 = 8703595;
pub const fnPlmSetup_RespawningBombBlock: uint32 = 8703619;
pub const fnPlmSetup_RespawningBombBlock2: uint32 = 8703706;
pub const fnPlmSetup_RespawningPowerBombBlock: uint32 = 8703790;
pub const fnPlmSetup_D08C_SuperMissileBlockRespawning: uint32 = 8703847;
pub const fnPlmSetup_D08C_CrumbleBlock: uint32 = 8703904;
pub const fnPlmSetup_D0DC_BreakableGrappleBlock: uint32 = 8703925;
pub const fnPlmSetup_D0D8_SetVFlag: uint32 = 8703949;
pub const fnPlmSetup_D0D8_ClearVflag: uint32 = 8703953;
pub const fnPlmSetup_D0E8_GiveSamusDamage: uint32 = 8703957;
pub const fnPlmSetup_D113_LowerNorfairChozoRoomPlug: uint32 = 8704264;
pub const fnPlmSetup_D127: uint32 = 8704279;
pub const fnPlmSetup_D138: uint32 = 8704299;
pub const fnPlmInstr_FxBaseYPos_0x2D2: uint32 = 8704341;
pub const fnPlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed: uint32 = 8704348;
pub const fnnullsub_69: uint32 = 8704398;
pub const fnPlmSetup_D6DA_LowerNorfairChozoHandTrigger: uint32 = 8704399;
pub const fnPlmPreInstr_IncrementRoomArgIfShotBySuperMissile: uint32 = 8704486;
pub const fnPlmInstr_GotoIfRoomArgLess: uint32 = 8704761;
pub const fnPlmInstr_SpawnFourMotherBrainGlass: uint32 = 8704779;
pub const fnPlmPreInstr_WakePlmIfSamusHasBombs: uint32 = 8704827;
pub const fnnullsub_351: ::core::ffi::c_int = 0x84d356;
pub const fnPlmInstr_SpawnTorizoStatueBreaking: uint32 = 8704855;
pub const fnPlmInstr_QueueSong1MusicTrack: uint32 = 8704967;
pub const fnPlmInstr_TransferWreckedShipChozoSpikesToSlopes: uint32 = 8704983;
pub const fnPlmInstr_TransferWreckedShipSlopesToChozoSpikes: uint32 = 8705012;
pub const fnPlmPreInstr_WakeOnKeyPress: uint32 = 8705215;
pub const fnPlmInstr_EnableWaterPhysics: uint32 = 8705317;
pub const fnPlmInstr_SpawnN00bTubeCrackEproj: uint32 = 8705324;
pub const fnPlmInstr_DiagonalEarthquake: uint32 = 8705334;
pub const fnPlmInstr_Spawn10shardsAnd6n00bs: uint32 = 8705347;
pub const fnPlmInstr_DisableSamusControls: uint32 = 8705510;
pub const fnPlmInstr_EnableSamusControls: uint32 = 8705518;
pub const fnPlmSetup_MotherBrainGlass: uint32 = 8705526;
pub const fnPlmSetup_DeletePlmIfAreaTorizoDead: uint32 = 8705542;
pub const fnPlmSetup_MakeBllockChozoHandTrigger: uint32 = 8705558;
pub const fnPlmSetup_D6F2_WreckedShipChozoHandTrigger: uint32 = 8705568;
pub const fnPlmSetup_D700_MakePlmAirBlock_Unused: uint32 = 8705663;
pub const fnPlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused: uint32 = 8705673;
pub const fnPlmSetup_D708_LowerNorfairChozoBlockUnused: uint32 = 8705683;
pub const fnPlmSetup_D70C_NoobTube: uint32 = 8705740;
pub const fnnullsub_71: uint32 = 8705782;
pub const fnnullsub_72: uint32 = 8705783;
pub const fnPlmPreInstr_WakePlmIfRoomArgumentDoorIsSet: uint32 = 8705875;
pub const fnPlmInstr_ShootEyeDoorProjectileWithProjectileArg: uint32 = 8705914;
pub const fnPlmInstr_SpawnEyeDoorSweatEproj: uint32 = 8705936;
pub const fnPlmInstr_SpawnTwoEyeDoorSmoke: uint32 = 8705951;
pub const fnPlmInstr_SpawnEyeDoorSmokeProjectile: uint32 = 8705974;
pub const fnPlmInstr_MoveUpAndMakeBlueDoorFacingRight: uint32 = 8705987;
pub const fnPlmInstr_MoveUpAndMakeBlueDoorFacingLeft: uint32 = 8706010;
pub const fnPlmSetup_EyeDoorEye: uint32 = 8706700;
pub const fnPlmSetup_EyeDoor: uint32 = 8706745;
pub const fnnullsub_73: uint32 = 8706773;
pub const fnnullsub_74: uint32 = 8706774;
pub const fnnullsub_75: uint32 = 8706775;
pub const fnnullsub_76: uint32 = 8706776;
pub const fnnullsub_77: uint32 = 8706777;
pub const fnnullsub_78: uint32 = 8706778;
pub const fnnullsub_79: uint32 = 8706779;
pub const fnnullsub_80: uint32 = 8706780;
pub const fnnullsub_81: uint32 = 8706781;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x10: uint32 = 8706782;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x11: uint32 = 8706798;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x12: uint32 = 8706814;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x13: uint32 = 8706830;
pub const fnPlmSetup_SetMetroidRequiredClearState: uint32 = 8706846;
pub const fnPlmPreInstr_GotoLinkIfShotWithSuperMissile: uint32 = 8706916;
pub const fnPlmInstr_DamageDraygonTurret: uint32 = 8706958;
pub const fnPlmInstr_DamageDraygonTurretFacingDownRight: uint32 = 8707000;
pub const fnPlmInstr_DamageDraygonTurretFacingUpRight: uint32 = 8707063;
pub const fnPlmInstr_DamageDraygonTurret2: uint32 = 8707126;
pub const fnPlmInstr_DamageDraygonTurretFacingDownLeft: uint32 = 8707168;
pub const fnPlmInstr_DamageDraygonTurretFacingUpLeft: uint32 = 8707231;
pub const fnPlmSetup_DraygonCannonFacingRight: uint32 = 8707732;
pub const fnPlmSetup_DraygonCannonFacingDiagonalRight: uint32 = 8707769;
pub const fnPlmSetup_DraygonCannonFacingLeft: uint32 = 8707824;
pub const fnPlmSetup_DraygonCannonFacingDiagonalLeft: uint32 = 8707861;
pub const fnPlmSetup_DraygonCannon: uint32 = 8707916;
pub const fnPlmPreInstr_GotoLinkIfTriggered: ::core::ffi::c_int = 0x84df89;
pub const fnPlmPreInstr_nullsub_301: ::core::ffi::c_int = 0x84dfa8;
pub const fnPlmPreInstr_WakeIfTriggered: uint32 = 8708070;
pub const fnPlmInstr_DrawItemFrame0: uint32 = 8708175;
pub const fnPlmInstr_DrawItemFrame1: uint32 = 8708199;
pub const fnPlmInstr_DrawItemFrame_Common: uint32 = 8708223;
pub const fnPlmInstr_ClearChargeBeamCounter: uint32 = 8708765;
pub const fnPlmInstr_E63B: uint32 = 8709691;
pub const fnsub_84EE4D: uint32 = 8711757;
pub const fnsub_84EE52: uint32 = 8711762;
pub const fnsub_84EE57: uint32 = 8711767;
pub const fnsub_84EE5C: uint32 = 8711772;
pub const fnsub_84EE64: uint32 = 8711780;
pub const fnsub_84EE77: uint32 = 8711799;
pub const fnsub_84EE7C: uint32 = 8711804;
pub const fnsub_84EE81: uint32 = 8711809;
pub const fnsub_84EE86: uint32 = 8711814;
pub const fnsub_84EE8E: uint32 = 8711822;
pub const fnsub_84EEAB: uint32 = 8711851;
pub const fnVariaSuitPickup: uint32 = 9557220;
pub const fnGravitySuitPickup: uint32 = 9557434;
pub const fnPlmPreInstr_Empty4: uint32 = 8686246;
pub unsafe fn SetGoldenTorizoPalette(mut a: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = (*(&raw mut a as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        & 0x78 as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v1 = 56 as int16;
    }
    let mut v2: uint16 =
        (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int | 0x1e as ::core::ffi::c_int) as uint16;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v5: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v5 + 160 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0x848132 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x848132 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v4 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v5 + 144 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0x848032 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x848032 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v4 as isize);
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn LoadRoomPlmGfx(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c2d) as *mut uint16) = 0 as uint16;
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1c2f) as *mut uint16)
            .offset((v0 >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            PlmInstr_LoadItemPlmGfx(
                RomPtr_84(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1c2f) as *mut uint16)
                        .offset((v0 >> 1 as ::core::ffi::c_int) as isize)
                        as uint16_t,
                    game,
                ),
                v0 as uint16,
                game,
            );
        }
        v0 += 2 as ::core::ffi::c_int;
        if !(v0 != 8 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn ClearSoundsWhenGoingThroughDoor(game: &mut Game) {
    CallSomeSamusCode(0x1d as uint16, game);
}
pub unsafe fn UNUSED_sub_848258(game: &mut Game) {
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        == 768 as ::core::ffi::c_int
        || (*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            == 5120 as ::core::ffi::c_int
    {
        QueueSfx1_Max15(0x32 as uint16, game);
    }
}
pub unsafe fn PlaySpinJumpSoundIfSpinJumping(game: &mut Game) {
    CallSomeSamusCode(0x1c as uint16, game);
}
pub unsafe fn UNUSED_sub_848278(game: &mut Game) {
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        == 768 as ::core::ffi::c_int
        || (*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            == 5120 as ::core::ffi::c_int
    {
        QueueSfx1_Max15(0x30 as uint16, game);
    }
}
pub unsafe fn CalculatePlmBlockCoords(k: uint16, game: &mut Game) {
    let block_index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
    let block_index = u16::from_le_bytes([
        game.g_ram[block_index_addr],
        game.g_ram[block_index_addr + 1],
    ]) >> 1;
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let y = SnesDivide(block_index, room_width);
    game.g_ram[0x1c2b..0x1c2d].copy_from_slice(&y.to_le_bytes()); // plm_y_block
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let x = SnesModulus(block_index, room_width);
    game.g_ram[0x1c29..0x1c2b].copy_from_slice(&x.to_le_bytes()); // plm_x_block
}
pub fn WriteLevelDataBlockTypeAndBts(k: uint16, a: uint16, game: &mut Game) {
    // The level-data byte uses the exact offset, while BTS rounds down to a word.
    let block_high = 0x10003 + usize::from(k);
    game.g_ram[block_high] = (a >> 8) as u8 | (game.g_ram[block_high] & 0x0f);
    game.g_ram[0x16402 + usize::from(k >> 1)] = a as u8;
}
pub unsafe fn WriteRowOfLevelDataBlockAndBTS(
    mut k: uint16,
    mut arg0: uint16,
    mut arg1: uint16,
    mut arg2: uint16,
    game: &mut Game,
) {
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v8: uint16 = (v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < arg2 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
            (v2 as ::core::ffi::c_int + i * 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                as isize,
        ) = arg0;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset((v8 as ::core::ffi::c_int + i) as isize) = arg1 as uint8;
        i += 1;
    }
}
pub unsafe fn LoadXrayBlocks(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 78 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            >= 0x84df89 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            let mut k: uint16 = i as uint16;
            let mut v2: uint16 =
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xd870)
                    .offset(PrepareBitAccess(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                            .offset(v1 as isize),
                        game,
                    ) as isize) as uint16;
            i = k as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16)
                as ::core::ffi::c_int
                & v2 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                CalculatePlmBlockCoords(k, game);
                let mut v3: *const uint8 = RomPtr_84(
                    *(game.g_rom.offset(
                        (((0x84839d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x84839d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16)
                                .offset(
                                    (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                                ) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as uint16_t,
                    game,
                );
                LoadBlockToXrayTilemap(
                    (*(v3.offset(2) as *mut uint16) as ::core::ffi::c_int
                        & 0xfff as ::core::ffi::c_int) as uint16,
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16),
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16),
                    game,
                );
                i = k as ::core::ffi::c_int;
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
    let mut RS: *mut RoomDefRoomstate = get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb) as *mut uint16),
        game,
    );
    if (*RS).xray_special_casing_ptr != 0 {
        let mut p: *const XraySpecialCasing =
            RomPtr_8F((*RS).xray_special_casing_ptr as uint16_t, game) as *const XraySpecialCasing;
        while (*p).x_block as ::core::ffi::c_int != 0 || (*p).y_block as ::core::ffi::c_int != 0 {
            LoadBlockToXrayTilemap(
                (*p).level_data_block,
                (*p).x_block as uint16,
                (*p).y_block as uint16,
                game,
            );
            p = p.offset(1);
        }
    }
}
pub fn EnablePLMs(game: &mut Game) {
    // plm_flag: enable bit 15 while preserving the other flag bits.
    let flags = u16::from_le_bytes([game.g_ram[0x1c23], game.g_ram[0x1c24]]);
    game.g_ram[0x1c23..0x1c25].copy_from_slice(&(flags | 0x8000).to_le_bytes());
}
pub unsafe fn DisablePLMs(game: &mut Game) {
    let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c23) as *mut uint16);
    *fresh22 = (*fresh22 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub fn ClearPLMs(game: &mut Game) {
    for slot in (0..40).rev() {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        game.g_ram[header_addr..header_addr + 2].copy_from_slice(&0u16.to_le_bytes());
    }
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&0u16.to_le_bytes()); // plm_item_gfx_index
}
pub unsafe fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs, game: &mut Game) {
    spawn_hardcoded_plm(p, core::ptr::null_mut(), game);
}
// Nested setup spawns retain the calling collision's displacement.
unsafe fn spawn_hardcoded_plm(
    mut p: SpawnHardcodedPlmArgs,
    displacement: *mut i32,
    game: &mut Game,
) {
    let mut pp: *mut SpawnHardcodedPlmArgs = &raw mut p;
    let mut v1: uint16 = 78 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut prod: uint16 = Mult8x8(
        (*pp).field_1,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = (2 * (u32::from(prod) + u32::from((*pp).field_0))) as u16;
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let v3 = (*pp).field_2 as u16;
    write_word(game, 0x1c37 + index, v3); // plm_header_ptr
    write_word(game, 0x1dc7 + index, 0); // plm_room_arguments
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x8469); // plm_pre_instrs
    let PH = get_PlmHeader_Size4(v3, game);
    let instruction = (*PH).instr_list_ptr as u16;
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        ((*PH).func_ptr as ::core::ffi::c_int | 0x840000 as ::core::ffi::c_int) as uint32,
        v1,
        displacement,
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
pub unsafe fn SpawnRoomPLM(k: uint16, game: &mut Game) {
    let v1 = match (0..40).rev().find(|&slot| {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        u16::from_le_bytes([game.g_ram[header_addr], game.g_ram[header_addr + 1]]) == 0
    }) {
        Some(slot) => (slot * 2) as u16,
        None => return,
    };
    let entry = if k == 0x12 {
        game.g_ram.as_ptr().add(0x12)
    } else {
        get_RoomPlmEntry(k, game).cast::<u8>()
    };
    let read_entry_word =
        |offset: usize| u16::from_le_bytes([*entry.add(offset), *entry.add(offset + 1)]);
    let y_block = u16::from(*entry.add(3));
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let row_start = y_block * u16::from(room_width);
    let x_block = u16::from(*entry.add(2));
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = row_start.wrapping_add(x_block).wrapping_mul(2);
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let argument = read_entry_word(4);
    write_word(game, 0x1dc7 + index, argument); // plm_room_arguments
    let pp = read_entry_word(0);
    let header = read_entry_word(0);
    write_word(game, 0x1c37 + index, header); // plm_header_ptr
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x84e6); // plm_pre_instrs
    let header_bytes = get_PlmHeader_Size4(pp, game).cast::<u8>();
    let read_header_word = |offset: usize| {
        u16::from_le_bytes([*header_bytes.add(offset), *header_bytes.add(offset + 1)])
    };
    let instruction = read_header_word(2);
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        0x840000 | u32::from(read_header_word(0)),
        v1,
        core::ptr::null_mut(),
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
pub unsafe fn PlmPreInstr_Empty2() {}
/// Prepare a PLM slot at the current collision block and invoke its setup.
///
/// # Safety
/// The selected setup callback requires the legacy game runtime to be initialized.
pub unsafe fn SpawnPLM(header: uint16, game: &mut Game) -> uint8 {
    SpawnPLMWithCollision(header, core::ptr::null_mut(), game)
}
/// Prepare a PLM while retaining the caller's active collision displacement.
///
/// # Safety
/// Setup requires initialized runtime state. A non-null `displacement` must
/// remain live and writable through setup and any nested PLM spawns.
pub unsafe fn SpawnPLMWithCollision(
    header: uint16,
    displacement: *mut i32,
    game: &mut Game,
) -> uint8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    // Allocate from the last slot down, retaining the original full-table result.
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| read_word(game, 0x1c37 + slot * 2) == 0)
    else {
        return 0;
    };
    let index = slot * 2;
    write_word(game, 0x1c87 + index, game.cur_block_index().wrapping_mul(2));
    write_word(game, 0x1c37 + index, header);
    write_word(game, 0x1cd7 + index, 0x853d);
    let instruction = game.read_rom_word(0x840000 | u32::from(header), 2);
    write_word(game, 0x1d27 + index, instruction);
    write_word(game, 0xde1c + index, 1);
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0);
    write_word(game, 0x1dc7 + index, 0);
    write_word(game, 0xdf0c + index, 0);
    write_word(game, 0x1c27, index as u16);
    // Read setup after preparing the slot, at the original callback boundary.
    let setup = game.read_rom_word(0x840000 | u32::from(header), 0);
    CallPlmHeaderFunc(
        0x840000 | u32::from(setup),
        index as u16,
        displacement,
        0,
        game,
    )
}
pub unsafe fn PlmHandler_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x613);
    's_158: {
        let mut base: *const uint8 = ::core::ptr::null::<uint8>();
        let mut v5: *const uint8 = ::core::ptr::null::<uint8>();
        let mut current_block_28: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x1c23) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    return kCoroutineNone as CoroutineRet;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16) = 78 as uint16;
                current_block_28 = 17179679302217393232;
            }
            1 => {
                current_block_28 = 12780425637862113792;
            }
            _ => {
                current_block_28 = 2569451025026770673;
            }
        }
        'c_23315: loop {
            match current_block_28 {
                2569451025026770673 => {
                    break 's_158;
                }
                17179679302217393232 => {
                    if *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        if *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
                            .offset(
                                (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16)
                                    as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                    as isize,
                            )
                            == 0
                        {
                            current_block_28 = 735147466149431745;
                        } else {
                            CallPlmPreInstr(
                                (*((&raw mut game.g_ram as *mut uint8).offset(0x1cd7)
                                    as *mut uint16)
                                    .offset(
                                        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                            as *mut uint16)
                                            as ::core::ffi::c_int
                                            >> 1 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int
                                    | 0x840000 as ::core::ffi::c_int)
                                    as uint32,
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16),
                                game,
                            );
                            let ref mut fresh26 = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xde1c)
                                as *mut uint16)
                                .offset(
                                    (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                        as *mut uint16)
                                        as ::core::ffi::c_int
                                        >> 1 as ::core::ffi::c_int)
                                        as isize,
                                );
                            *fresh26 = (*fresh26).wrapping_sub(1);
                            if *fresh26 != 0 {
                                current_block_28 = 735147466149431745;
                            } else {
                                base = RomBankBase(0x84 as uint8, game);
                                v5 = ::core::ptr::null::<uint8>();
                                v5 = base.offset(
                                    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27)
                                        as *mut uint16)
                                        .offset(
                                            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                                as *mut uint16)
                                                as ::core::ffi::c_int
                                                >> 1 as ::core::ffi::c_int)
                                                as isize,
                                        ) as ::core::ffi::c_int
                                        as isize,
                                );
                                current_block_28 = 11812396948646013369;
                            }
                        }
                    } else {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16) =
                            0 as uint16;
                        current_block_28 = 2569451025026770673;
                        continue;
                    }
                }
                _ => {
                    let mut _coret_: uint8 = DisplayMessageBox_Async(
                        *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16),
                        game,
                    ) as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16) =
                        0 as uint16;
                    base = RomBankBase(0x84 as uint8, game);
                    v5 = base.offset(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(
                            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                                as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int as isize,
                    );
                    current_block_28 = 11812396948646013369;
                }
            }
            loop {
                match current_block_28 {
                    735147466149431745 => {
                        let ref mut fresh27 =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16);
                        *fresh27 =
                            (*fresh27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                        current_block_28 = 17179679302217393232;
                        continue 'c_23315;
                    }
                    _ => {
                        if *(v5 as *mut uint16) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            let mut v7: ::core::ffi::c_int = 0;
                            v7 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                as *mut uint16)
                                as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int;
                            *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
                                .offset(v7 as isize) = *(v5 as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8).offset(0xde6c) as *mut uint16)
                                .offset(v7 as isize) = *(v5.offset(2) as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
                                .offset(v7 as isize) =
                                v5.offset(4).offset_from(base) as ::core::ffi::c_long as uint16;
                            ProcessPlmDrawInstruction(
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16),
                                game,
                            );
                            CalculatePlmBlockCoords(
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16),
                                game,
                            );
                            DrawPlm(
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16),
                                game,
                            );
                            current_block_28 = 735147466149431745;
                        } else {
                            v5 = CallPlmInstr(
                                (*(v5 as *mut uint16) as ::core::ffi::c_int
                                    | 0x840000 as ::core::ffi::c_int)
                                    as uint32,
                                v5.offset(2),
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27)
                                    as *mut uint16),
                                game,
                            );
                            if v5.is_null() {
                                current_block_28 = 735147466149431745;
                                continue;
                            }
                            if (v5 as uintptr_t) < 0x10000 as ::core::ffi::c_int as uintptr_t {
                                v5 = base.offset(v5 as uintptr_t as isize);
                            }
                            if *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16)
                                as ::core::ffi::c_int
                                != 0 as ::core::ffi::c_int
                            {
                                break;
                            } else {
                                current_block_28 = 11812396948646013369;
                            }
                        }
                    }
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = v5.offset_from(base) as ::core::ffi::c_long as uint16;
            current_block_28 = 12780425637862113792;
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
#[inline]
unsafe fn RomPtr_84orRAM(mut addr: uint16_t, game: &mut Game) -> *mut uint8 {
    if addr as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        return RomPtr(
            (0x840000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
            game,
        ) as *mut uint8;
    } else {
        '_c2rust_label: {
            if (addr as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"addr < 0x2000\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/sm_84.c\0" as *const u8 as *const ::core::ffi::c_char,
                    265 as ::core::ffi::c_uint,
                    b"uint8 *RomPtr_84orRAM(uint16_t)\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        };
        return RomPtr_RAM(addr, game);
    };
}
pub unsafe fn ProcessPlmDrawInstruction(k: uint16, game: &mut Game) {
    let slot = usize::from(k >> 1) * 2;
    let draw_instruction =
        u16::from_le_bytes([game.g_ram[0xde6c + slot], game.g_ram[0xde6d + slot]]);
    let mut words = RomPtr_84orRAM(draw_instruction, game) as *const u16;
    let mut destination =
        u16::from_le_bytes([game.g_ram[0x1c87 + slot], game.g_ram[0x1c88 + slot]]);
    let origin = destination;
    loop {
        let command = words.read_unaligned();
        words = words.add(1);
        let stride = if command & 0x8000 != 0 {
            usize::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]])) * 2
        } else {
            2
        };
        for block in 0..usize::from(command as u8) {
            let word = words.read_unaligned();
            words = words.add(1);
            // The original word indexing rounds odd byte destinations down.
            let offset = 0x10002 + ((usize::from(destination) + block * stride) >> 1) * 2;
            game.g_ram[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
        }
        let displacement = words.read_unaligned();
        if displacement == 0 {
            break;
        }
        let x = i32::from(displacement as u8 as i8);
        let y = i32::from((displacement >> 8) as u8 as i8);
        let width = i32::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]));
        destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
        words = words.add(1);
    }
}
pub unsafe fn PlmInstr_Sleep(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let instruction = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_sub(2);
    let offset = 0x1d27 + usize::from(k >> 1) * 2; // plm_instruction_list_ptrs
    game.g_ram[offset..offset + 2].copy_from_slice(&instruction.to_le_bytes());
    ::core::ptr::null()
}
pub fn PlmInstr_Delete(_plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let offset = 0x1c37 + usize::from(k >> 1) * 2; // plm_header_ptr
    game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
    ::core::ptr::null()
}
pub unsafe fn PlmInstr_PreInstr(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let pre_instruction = (plmp as *const u16).read_unaligned();
    let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
    game.g_ram[offset..offset + 2].copy_from_slice(&pre_instruction.to_le_bytes());
    plmp.add(2)
}
pub fn PlmInstr_ClearPreInstr(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
    game.g_ram[offset..offset + 2].copy_from_slice(&0x86d0u16.to_le_bytes());
    plmp
}
pub unsafe fn CallPlmInstrFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        9557220 => {
            VariaSuitPickup(game);
            return;
        }
        9557434 => {
            GravitySuitPickup(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn PlmInstr_CallFunction(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    CallPlmInstrFunc(Load24(plmp as *mut LongPtr), game);
    return plmp.offset(3);
}
pub unsafe fn PlmInstr_Goto(plmp: *const uint8, _k: uint16) -> *const uint8 {
    let target = (plmp as *const u16).read_unaligned();
    usize::from(target) as *const uint8
}
pub unsafe fn PlmInstr_DecrementAndBranchNonzero(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let fresh66 = *fresh65;
    *fresh65 = (*fresh65).wrapping_sub(1);
    if fresh66 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return plmp.offset(2);
    } else {
        return PlmInstr_Goto(plmp, k);
    };
}
pub unsafe fn PlmInstr_SetTimer(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    game.g_ram[0x1d77 + usize::from(k)] = *plmp; // Only one byte of plm_timers.
    plmp.add(1)
}
pub unsafe fn PlmInstr_LoadItemPlmGfx(
    mut plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let gfx_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
    let slot = 0xdf0c + usize::from(k >> 1) * 2; // plm_variables
    let current_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
    game.g_ram[slot..slot + 2].copy_from_slice(&current_index.to_le_bytes());
    let next_index = gfx_index.wrapping_add(2) & 6;
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&next_index.to_le_bytes());
    let table_offset = usize::from(gfx_index >> 1) * 2;
    let vram_destination = game.read_rom_word(0x8487cd, table_offset);
    let tile_offset = usize::from(game.read_rom_word(0x8487d5, table_offset));
    let mut tile_number = game.read_rom_word(0x8487dd, table_offset);
    let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
    let pointer_offset = 0x1c2f + usize::from(gfx_index >> 1) * 2; // plm_item_gfx_ptrs
    game.g_ram[pointer_offset..pointer_offset + 2].copy_from_slice(&instruction.to_le_bytes());
    let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
    game.g_ram[entry..entry + 2].copy_from_slice(&256u16.to_le_bytes());
    let source = (plmp as *const u16).read_unaligned();
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
    game.g_ram[entry + 4] = 0x89;
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&vram_destination.to_le_bytes());
    game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
    plmp = plmp.add(2);
    for tile in 0..8 {
        let attributes = u16::from(*plmp) << 10;
        let word = tile_number.wrapping_add(attributes);
        let destination = 0xa000 + tile_offset + tile * 2; // tile_table.tables[0].top_left
        game.g_ram[destination..destination + 2].copy_from_slice(&word.to_le_bytes());
        tile_number = tile_number.wrapping_add(1);
        plmp = plmp.add(1);
    }
    plmp
}
pub unsafe fn PlmInstr_CopyFromRamToVram(
    plmp: *const uint8,
    _k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
    let size = (plmp as *const u16).read_unaligned();
    game.g_ram[entry..entry + 2].copy_from_slice(&size.to_le_bytes());
    let source = (plmp.add(2) as *const u16).read_unaligned();
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
    game.g_ram[entry + 4] = *plmp.add(4);
    let destination = (plmp.add(5) as *const u16).read_unaligned();
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_le_bytes());
    game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
    plmp.add(7)
}
pub unsafe fn PlmInstr_GotoIfBossBitSet(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if CheckBossBitForCurArea(*plmp.offset(0) as uint16, game) != 0 {
        return PlmInstr_Goto(plmp.offset(1), k);
    } else {
        return plmp.offset(3);
    };
}
pub unsafe fn PlmInstr_GotoIfEventSet(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if CheckEventHappened(*(plmp as *mut uint16), game) != 0 {
        return PlmInstr_Goto(plmp.offset(2), k);
    } else {
        return plmp.offset(4);
    };
}
pub unsafe fn PlmInstr_SetEvent(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetEventHappened(*(plmp as *mut uint16), game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_GotoIfChozoSet(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut idx: ::core::ffi::c_int = PrepareBitAccess(v3, game) as ::core::ffi::c_int;
    if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int
            & *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd830)
                .offset(idx as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return PlmInstr_Goto(plmp, k);
    } else {
        return plmp.offset(2);
    };
}
pub unsafe fn PlmInstr_SetRoomChozoBit(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        let mut idx: ::core::ffi::c_int = PrepareBitAccess(v3, game) as ::core::ffi::c_int;
        let ref mut fresh64 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(idx as isize);
        *fresh64 = (*fresh64 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16)
                as ::core::ffi::c_int) as uint8;
    }
    return plmp;
}
pub unsafe fn PlmInstr_GotoIfItemBitSet(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
    let byte_index = usize::from(PrepareBitAccess(v2, game));
    if v2 & 0x8000 == 0 {
        let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
        if bitmask & u16::from(game.g_ram[0xd870 + byte_index]) != 0 {
            return PlmInstr_Goto(plmp, k);
        }
    }
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_SetItemBit(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
    if v2 & 0x8000 == 0 {
        let byte_index = usize::from(PrepareBitAccess(v2, game));
        let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
        game.g_ram[0xd870 + byte_index] |= bitmask as u8;
    }
    return plmp;
}
pub unsafe fn PlmInstr_PickupBeamAndShowMessage(
    plmp: *const uint8,
    _k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let beam = u16::from_le_bytes([*plmp, *plmp.add(1)]);
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    game.set_collected_beams(read_word(game, 0x9a8) | beam);
    game.set_equipped_beams(read_word(game, 0x9a6) | beam);
    // Spazer and Plasma are mutually exclusive; retain both ordered masks.
    game.set_equipped_beams(read_word(game, 0x9a6) & !((beam << 1) & 8));
    game.set_equipped_beams(read_word(game, 0x9a6) & !((beam >> 1) & 4));
    UpdateBeamTilesAndPalette(game);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    let message = u16::from(*plmp.add(2));
    DisplayMessageBox(message, game);
    plmp.add(3)
}
pub unsafe fn PlmInstr_PickupEquipmentAndShowMessage(
    plmp: *const uint8,
    _k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let equipment = (plmp as *const u16).read_unaligned();
    let equipped = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    game.set_equipped_items(equipped | equipment);
    game.set_collected_items(game.collected_items() | equipment);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    let message = u16::from(*plmp.add(2));
    DisplayMessageBox(message, game);
    plmp.add(3)
}
pub unsafe fn PlmInstr_PickupEquipmentAddGrappleShowMessage(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut t: uint16 = *(plmp as *mut uint16);
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16);
    *fresh55 = (*fresh55 as ::core::ffi::c_int | t as ::core::ffi::c_int) as uint16;
    let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16);
    *fresh56 = (*fresh56 as ::core::ffi::c_int | t as ::core::ffi::c_int) as uint16;
    AddGrappleToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
    DisplayMessageBox(5 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_PickupEquipmentAddXrayShowMessage(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut t: uint16 = *(plmp as *mut uint16);
    let ref mut fresh53 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16);
    *fresh53 = (*fresh53 as ::core::ffi::c_int | t as ::core::ffi::c_int) as uint16;
    let ref mut fresh54 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16);
    *fresh54 = (*fresh54 as ::core::ffi::c_int | t as ::core::ffi::c_int) as uint16;
    AddXrayToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
    DisplayMessageBox(6 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_CollectHealthEnergyTank(
    plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let amount = (plmp as *const u16).read_unaligned();
    game.set_samus_max_health(game.samus_max_health().wrapping_add(amount));
    game.set_samus_health(game.samus_max_health());
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    DisplayMessageBox(1, game);
    plmp.add(2)
}
pub unsafe fn PlmInstr_CollectHealthReserveTank(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16);
    *fresh50 =
        (*fresh50 as ::core::ffi::c_int + *(plmp as *mut uint16) as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) == 0 {
        let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16);
        *fresh51 = (*fresh51).wrapping_add(1);
    }
    PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
    DisplayMessageBox(0x19 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_CollectAmmoMissileTank(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let t = *(plmp as *const uint16);
    game.set_samus_max_missiles(game.samus_max_missiles().wrapping_add(t));
    game.set_samus_missiles(game.samus_missiles().wrapping_add(t));
    AddMissilesToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
    DisplayMessageBox(2 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_CollectAmmoSuperMissileTank(
    plmp: *const uint8,
    _k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let amount = u16::from_le_bytes([*plmp, *plmp.add(1)]);
    game.set_samus_max_super_missiles(game.samus_max_super_missiles().wrapping_add(amount));
    game.set_samus_super_missiles(game.samus_super_missiles().wrapping_add(amount));
    AddSuperMissilesToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    DisplayMessageBox(3, game);
    plmp.add(2)
}
pub unsafe fn PlmInstr_CollectAmmoPowerBombTank(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut t: uint16 = *(plmp as *mut uint16);
    let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x9d0) as *mut uint16);
    *fresh44 = (*fresh44 as ::core::ffi::c_int + t as ::core::ffi::c_int) as uint16;
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16);
    *fresh45 = (*fresh45 as ::core::ffi::c_int + t as ::core::ffi::c_int) as uint16;
    AddPowerBombsToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
    DisplayMessageBox(4 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_SetLinkReg(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let link = (plmp as *const u16).read_unaligned();
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    game.g_ram[offset..offset + 2].copy_from_slice(&link.to_le_bytes());
    plmp.add(2)
}
pub unsafe fn PlmInstr_Call(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let return_address = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_add(2);
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    game.g_ram[offset..offset + 2].copy_from_slice(&return_address.to_le_bytes());
    let target = (plmp as *const u16).read_unaligned();
    usize::from(target) as *const uint8
}
pub fn PlmInstr_Return(_plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    let link = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    usize::from(link) as *const uint8
}
pub unsafe fn PlmInstr_GotoIfDoorBitSet(
    plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    if argument & 0x8000 != 0 {
        return plmp.add(2);
    }
    let bit_index = usize::from(PrepareBitAccess(argument, game));
    let mask = read_word(game, 0x5e7); // bitmask
    if mask & u16::from(game.g_ram[0xd8b0 + bit_index]) != 0 {
        return usize::from((plmp as *const u16).read_unaligned()) as *const uint8;
    }
    plmp.add(2)
}
pub unsafe fn PlmInstr_IncrementDoorHitCounterAndJGE(
    plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let hits = read_word(game, 0xdf0c + index).wrapping_add(1); // plm_variables
    write_word(game, 0xdf0c + index, hits);
    if (hits as u8) < *plmp {
        return plmp.add(3);
    }
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    if argument & 0x8000 == 0 {
        let bit_index = usize::from(PrepareBitAccess(argument, game));
        let mask = read_word(game, 0x5e7); // bitmask
        let door = 0xd8b0 + bit_index; // opened_door_bit_array
        game.g_ram[door] |= mask as u8;
        // Preserve the original word read, including the following bit-array byte.
        let marked_argument = read_word(game, door) | 0x8000;
        write_word(game, 0x1dc7 + index, marked_argument);
    }
    write_word(game, 0x1cd7 + index, addr_PlmPreInstr_Empty4 as u16);
    PlmInstr_Goto(plmp.add(1), k)
}
pub unsafe fn PlmInstr_IncrementArgumentAndJGE(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: uint8 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
    if v2 as ::core::ffi::c_int >= *plmp.offset(0) as ::core::ffi::c_int {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v4 as isize) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v4 as isize) =
            addr_locret_848AE0 as ::core::ffi::c_int as uint16;
        return PlmInstr_Goto(plmp.offset(1), k);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2 as uint16;
        return plmp.offset(3);
    };
}
pub unsafe fn PlmInstr_SetBTS(plmp: *const uint8, k: uint16, game: &mut Game) -> *const uint8 {
    let bts = *plmp;
    let index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
    let block_index = u16::from_le_bytes([game.g_ram[index_addr], game.g_ram[index_addr + 1]]) >> 1;
    game.g_ram[0x16402 + usize::from(block_index)] = bts;
    plmp.add(1)
}
pub unsafe fn PlmInstr_DrawPlmBlock(
    plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block = read_word(game, 0x1e17 + index); // plm_variable
    let block_index = read_word(game, 0x1c87 + index);
    write_word(game, 0x10002 + usize::from(block_index >> 1) * 2, block);
    write_word(game, 0x1e69, block);
    write_word(game, 0x1e67, 1);
    write_word(game, 0x1e6b, 0);
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, 0x1e67); // plm_instruction_draw_ptr
    let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    ProcessPlmDrawInstruction(k, game);
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16);
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16),
        game,
    );
    DrawPlm(v4, game);
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn PlmInstr_ProcessAirScrollUpdate(
    plmp: *const uint8,
    k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    write_word(game, 0x1e17 + index, 0); // plm_variable
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    let mut entries = RomPtr_8F(argument, game);
    while *entries & 0x80 == 0 {
        let value = *entries.add(1);
        game.g_ram[0xcd20 + usize::from(*entries)] = value; // scrolls
        entries = entries.add(2);
    }
    let block_index = read_word(game, 0x1c87 + index); // plm_block_indices
    let block_addr = 0x10002 + usize::from(block_index >> 1) * 2;
    let block = read_word(game, block_addr);
    write_word(game, block_addr, (block & 0x0fff) | 0x3000);
    plmp
}
pub unsafe fn PlmInstr_ProcessSolidScrollUpdate(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    let mut v5: *const uint8 = RomPtr_8F(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as uint16_t,
        game,
    );
    while !(*v5.offset(0) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(*v5.offset(0) as isize) = *v5.offset(1);
        v5 = v5.offset(2);
    }
    let mut v7: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v7 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v7 as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0xb000 as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_QueueMusic(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueMusic_Delayed8(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_ClearMusicQueueAndQueueTrack(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x63b) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x639) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x63f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x63d) as *mut uint16) = 0 as uint16;
    QueueMusic_Delayed8(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx1_Max6(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max6(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx2_Max6(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max6(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx3_Max6(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max6(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx1_Max15(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max15(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx2_Max15(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max15(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx3_Max15(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max15(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx1_Max3(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max3(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx2_Max3(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max3(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx_Max3(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max3(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx1_Max9(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max9(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx2_Max9(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max9(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx3_Max9(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max9(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx1_Max1(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max1(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx2_Max1(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_QueueSfx3_Max1(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max1(*plmp.offset(0) as uint16, game);
    return plmp.offset(1);
}
pub unsafe fn PlmInstr_ActivateMapStation(
    plmp: *const uint8,
    _k: uint16,
    game: &mut Game,
) -> *const uint8 {
    // map_station_byte_array; word OR with 00FF preserves the neighboring byte.
    let station = 0xd908 + usize::from(game.area_index());
    game.g_ram[station] = 0xff;
    DisplayMessageBox(0x14, game);
    game.set_has_area_map(1);
    plmp
}
pub unsafe fn PlmInstr_ActivateEnergyStation(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
    {
        DisplayMessageBox(0x15 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16);
    }
    CallSomeSamusCode(1 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_ActivateMissileStation(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
    {
        DisplayMessageBox(0x16 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16);
    }
    CallSomeSamusCode(1 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_ActivateSaveStationAndGotoIfNo(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut r: ::core::ffi::c_int = DisplayMessageBox_Poll(23 as uint16, game);
    if r < 0 as ::core::ffi::c_int {
        return plmp.offset(-(2 as ::core::ffi::c_int as isize));
    }
    if r == 2 as ::core::ffi::c_int {
        return *(plmp as *mut uint16) as uintptr_t as *const uint8;
    }
    SpawnEprojWithRoomGfx(
        addr_kEproj_SaveStationElectricity as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint16;
    PrepareBitAccess(
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16),
        game,
    );
    let ref mut fresh40 = *(&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
        (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int) as isize,
    );
    *fresh40 = (*fresh40 as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int)
        as uint8;
    SaveToSram(
        *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
        game,
    );
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_GotoIfSamusNear(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    CalculatePlmBlockCoords(k, game);
    let mut v2: uint8 = abs16(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
    ) as uint8;
    let mut v4: uint8 = abs16(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
    ) as uint8;
    if v2 as ::core::ffi::c_int <= *plmp.offset(0) as ::core::ffi::c_int
        && v4 as ::core::ffi::c_int <= *plmp.offset(1) as ::core::ffi::c_int
    {
        return *(plmp.offset(2) as *mut uint16) as uintptr_t as *const uint8;
    } else {
        return plmp.offset(4);
    };
}
unsafe fn PartiallyQueueVramForSingleScreenPlm(
    mut a: uint16,
    mut k: uint16,
    mut r20: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut ent: *mut VramWriteEntry = gVramWriteEntry(k as ::core::ffi::c_int, game);
    (*ent).vram_dst = (r28 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint16;
    let mut size: uint16 = (4 as ::core::ffi::c_int * r20 as ::core::ffi::c_int) as uint16;
    let mut base: VoidP = (*((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
        as ::core::ffi::c_int
        + 0xc6c8 as ::core::ffi::c_int) as VoidP;
    (*ent.offset(0)).size = size;
    (*ent.offset(1)).size = size;
    (*ent.offset(0)).src = LongPtr {
        addr: base,
        bank: 126 as uint8,
    };
    (*ent.offset(1)).src = LongPtr {
        addr: (base as ::core::ffi::c_int + size as ::core::ffi::c_int) as VoidP,
        bank: 126 as uint8,
    };
}
unsafe fn CalculatePlmDrawTilemapVramDst(
    mut k: uint16,
    mut r9: uint16,
    mut r12: uint16,
    mut r20: uint16,
    mut r24: uint16,
    mut r26: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut a: uint16 = 0;
    let mut prod: uint16 = ((r26 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        * 0x40 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (r24 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
    if v1 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        a = (prod as ::core::ffi::c_int
            + r12 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16) as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            a = (a as ::core::ffi::c_int - 1024 as ::core::ffi::c_int) as uint16;
        }
    } else {
        a = (prod as ::core::ffi::c_int
            + r9 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16) as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            a = (a as ::core::ffi::c_int + 1024 as ::core::ffi::c_int) as uint16;
        }
    }
    PartiallyQueueVramForSingleScreenPlm(a, k, r20, r28, game);
}
pub unsafe fn DrawPlm(mut k: uint16, game: &mut Game) {
    let mut v24: uint16 = 0;
    let mut v26: uint16 = 0;
    let mut v27: uint16 = 0;
    let mut v29: uint16 = 0;
    let mut v2: int16 = 0;
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    let mut v10: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v12: VoidP = 0;
    let mut v13: VoidP = 0;
    let mut v15: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v18: int16 = 0;
    let mut v20: int16 = 0;
    let mut v23: bool = false;
    let mut v25: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v28: int16 = 0;
    let mut v31: int16 = 0;
    let mut v35: uint16 = 0;
    let mut r26: uint16 = 0;
    let mut r22: uint16 = 0;
    let mut a: uint16 = 0;
    let mut r24: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r9: uint16 = addr_unk_605000 as ::core::ffi::c_int as uint16;
    let mut r12: uint16 = addr_unk_6053E0 as ::core::ffi::c_int as uint16;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xde6c) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut x: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16);
    let mut y: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16);
    let mut r3: uint16 = 0;
    let mut r0: *mut uint16 = ::core::ptr::null_mut::<uint16>();
    let mut r6: *mut uint16 = ::core::ptr::null_mut::<uint16>();
    '_LABEL_2: loop {
        r26 = (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            + 15 as ::core::ffi::c_int
            - y as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            return;
        }
        v2 = *(RomPtr_84orRAM(v1 as uint16_t, game) as *mut uint16) as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            r20 = (v2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as uint16;
            if !(*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int
                == x as ::core::ffi::c_int
                || (((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int)
                    - x as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
            {
                break;
            }
            v20 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int)
                + 17 as ::core::ffi::c_int) as int16;
            if !(v20 as ::core::ffi::c_int != x as ::core::ffi::c_int
                && (v20 as ::core::ffi::c_int - x as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int)
            {
                break;
            }
            r24 = x;
            r18 = 0 as uint16;
            r22 = (r26 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
            if ((r26 as ::core::ffi::c_int - y as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                r26 = y;
            } else {
                r18 = (r26 as ::core::ffi::c_int - y as ::core::ffi::c_int) as uint16;
                if y as ::core::ffi::c_int + r20 as ::core::ffi::c_int - r26 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    return;
                }
                let mut v21: bool = r20 as ::core::ffi::c_int == r18 as ::core::ffi::c_int;
                let mut v22: bool = ((r20 as ::core::ffi::c_int - r18 as ::core::ffi::c_int)
                    as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int;
                r20 = (r20 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
                if v22 {
                    Unreachable(game);
                }
                if v21 {
                    return;
                }
            }
            r22 = (r20 as ::core::ffi::c_int + r26 as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
            if !(r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                || {
                    v23 = ((r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int;
                    r20 = (r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
                    r20 as ::core::ffi::c_int != 0 as ::core::ffi::c_int && !v23
                })
            {
                break;
            }
            v35 = k;
            v24 = 0;
            v24 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            if !(((*((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int
                - 240 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
                && ((512 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 3 as ::core::ffi::c_int)
                    - r20 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0)
            {
                break;
            }
            CalculatePlmDrawTilemapVramDst(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16),
                r9,
                r12,
                r20,
                r24,
                r26,
                0x8000 as uint16,
                game,
            );
            v25 = gVramWriteEntry(v24 as ::core::ffi::c_int, game);
            (*v25.offset(1)).vram_dst = ((*v25.offset(0)).vram_dst as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
            r0 = (&raw mut game.g_ram as *mut uint8)
                .offset((*v25.offset(0)).src.addr as ::core::ffi::c_int as isize)
                as *mut uint16;
            r6 = (&raw mut game.g_ram as *mut uint8)
                .offset((*v25.offset(1)).src.addr as ::core::ffi::c_int as isize)
                as *mut uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                (v24 as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
            r18 = (r18 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            r3 = (r18 as ::core::ffi::c_int + v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)
                as uint16;
            v26 = 0;
            v26 = 0 as uint16;
            loop {
                x = *(RomPtr_84orRAM(r3 as uint16_t, game) as *mut uint16);
                v27 = 0;
                v29 = 0;
                v27 = (x as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16;
                v28 = (x as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) as int16;
                if x as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                {
                    if v28 as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
                        *r0.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        v29 = (v26 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                    } else if v28 as ::core::ffi::c_int == 2048 as ::core::ffi::c_int {
                        *r0.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        v29 = (v26 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    } else {
                        *r0.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        v29 = (v26 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v27 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    }
                } else {
                    *r0.offset((v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v27 as usize]
                            .top_left;
                    *r6.offset((v26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v27 as usize]
                            .top_right;
                    v29 = (v26 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    *r0.offset((v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_left;
                    *r6.offset((v29 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v27 as usize]
                            .bottom_right;
                }
                v26 = (v29 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                r3 = r3.wrapping_add(1);
                r3 = r3.wrapping_add(1);
                let ref mut fresh23 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16);
                *fresh23 = (*fresh23 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
                    as ::core::ffi::c_int
                    - 512 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
                if r20 == 0 {
                    break;
                }
            }
        } else {
            r20 = (v2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as uint16;
            if y as ::core::ffi::c_int - r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                return;
            }
            r26 = y;
            r18 = 0 as uint16;
            r24 = x;
            r22 = (((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
                + 15 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int)
                - 1 as ::core::ffi::c_int) as uint16;
            if (r22 as ::core::ffi::c_int - x as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
                && r22 as ::core::ffi::c_int != x as ::core::ffi::c_int
            {
                r18 = (r22 as ::core::ffi::c_int - x as ::core::ffi::c_int) as uint16;
                if x as ::core::ffi::c_int + r20 as ::core::ffi::c_int == r22 as ::core::ffi::c_int
                    || ((x as ::core::ffi::c_int + r20 as ::core::ffi::c_int
                        - r22 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                {
                    return;
                }
                r20 = (r20 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
                r24 = r22;
            }
            r22 = (r22 as ::core::ffi::c_int + 17 as ::core::ffi::c_int) as uint16;
            if !(r22 as ::core::ffi::c_int - x as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0)
            {
                break;
            }
            r22 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
            if !(r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                || {
                    r20 = (r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
                    r20 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                })
            {
                break;
            }
            v35 = k;
            let mut v3: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            if !(((*((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int
                - 480 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
                && ((512 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 3 as ::core::ffi::c_int)
                    - r20 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0)
            {
                break;
            }
            let mut prod: uint16 = ((r26 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                * 0x40 as ::core::ffi::c_int) as uint16;
            let mut v4: uint16 = (r24 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
            if v4 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v8 = (r12 as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int * v4 as ::core::ffi::c_int)
                    as int16;
                a = (prod as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x100 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    a = (a as ::core::ffi::c_int - 1024 as ::core::ffi::c_int) as uint16;
                }
            } else {
                v5 = (r9 as ::core::ffi::c_int + 2 as ::core::ffi::c_int * v4 as ::core::ffi::c_int)
                    as int16;
                let mut RegWord: uint16 = prod;
                a = (RegWord as ::core::ffi::c_int + v5 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x100 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    a = (a as ::core::ffi::c_int + 1024 as ::core::ffi::c_int) as uint16;
                }
            }
            x = (2 as ::core::ffi::c_int * r20 as ::core::ffi::c_int) as uint16;
            let mut R34: uint16 = (a as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
            if 2 as ::core::ffi::c_int * r20 as ::core::ffi::c_int
                + (a as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                - 1 as ::core::ffi::c_int
                & 0xffe0 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if (v3 as ::core::ffi::c_int - 228 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                    || ((32 as ::core::ffi::c_int - R34 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                {
                    return;
                }
                let mut v9: uint16 = (2 as ::core::ffi::c_int
                    * (32 as ::core::ffi::c_int - R34 as ::core::ffi::c_int))
                    as uint16;
                v10 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
                (*v10).size = v9;
                (*v10.offset(2)).size = v9;
                (*v10.offset(0)).vram_dst = a;
                (*v10.offset(1)).vram_dst = (a as ::core::ffi::c_int & 0xffe0 as ::core::ffi::c_int
                    ^ 0x400 as ::core::ffi::c_int)
                    as uint16;
                (*v10.offset(3)).vram_dst = ((*v10.offset(1)).vram_dst as ::core::ffi::c_int
                    + 32 as ::core::ffi::c_int)
                    as uint16;
                (*v10.offset(2)).vram_dst =
                    ((*v10).vram_dst as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
                x = (4 as ::core::ffi::c_int * r20 as ::core::ffi::c_int) as uint16;
                let mut v11: uint16 = (4 as ::core::ffi::c_int * r20 as ::core::ffi::c_int
                    - (*v10).size as ::core::ffi::c_int)
                    as uint16;
                (*v10.offset(1)).size = v11;
                (*v10.offset(3)).size = v11;
                v12 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
                    as ::core::ffi::c_int
                    - 14648 as ::core::ffi::c_int) as VoidP;
                (*v10).src.addr = (*((&raw mut game.g_ram as *mut uint8).offset(0x1c25)
                    as *mut uint16) as ::core::ffi::c_int
                    - 14648 as ::core::ffi::c_int) as VoidP;
                r0 = (&raw mut game.g_ram as *mut uint8).offset(v12 as ::core::ffi::c_int as isize)
                    as *mut uint16;
                v13 = ((*v10).size as ::core::ffi::c_int + v12 as ::core::ffi::c_int) as VoidP;
                (*v10.offset(1)).src.addr = v13;
                let mut v14: uint16 = ((*v10.offset(1)).size as ::core::ffi::c_int
                    + v13 as ::core::ffi::c_int) as uint16;
                (*v10.offset(2)).src.addr = v14 as VoidP;
                r6 = (&raw mut game.g_ram as *mut uint8).offset(v14 as ::core::ffi::c_int as isize)
                    as *mut uint16;
                (*v10.offset(3)).src.addr = ((*v10.offset(2)).size as ::core::ffi::c_int
                    + v14 as ::core::ffi::c_int)
                    as VoidP;
                (*v10).src.bank = 126 as uint8;
                (*v10.offset(1)).src.bank = 126 as uint8;
                (*v10.offset(2)).src.bank = 126 as uint8;
                (*v10.offset(3)).src.bank = 126 as uint8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                    (v3 as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
            } else {
                PartiallyQueueVramForSingleScreenPlm(a, v3, r20, 0 as uint16, game);
                v15 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
                (*v15.offset(1)).vram_dst = ((*v15.offset(0)).vram_dst as ::core::ffi::c_int
                    + 32 as ::core::ffi::c_int)
                    as uint16;
                r0 = (&raw mut game.g_ram as *mut uint8)
                    .offset((*v15.offset(0)).src.addr as ::core::ffi::c_int as isize)
                    as *mut uint16;
                r6 = (&raw mut game.g_ram as *mut uint8)
                    .offset((*v15.offset(1)).src.addr as ::core::ffi::c_int as isize)
                    as *mut uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                    (v3 as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
            }
            r18 = (r18 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            r3 = (r18 as ::core::ffi::c_int + v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)
                as uint16;
            let mut v16: uint16 = 0 as uint16;
            loop {
                x = *(RomPtr_84orRAM(r3 as uint16_t, game) as *mut uint16);
                let mut v17: uint16 =
                    (x as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16;
                let mut v19: uint16 = 0;
                v18 = (x as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) as int16;
                if x as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                {
                    if v18 as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
                        *r0.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        v19 = (v16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x4000 as ::core::ffi::c_int) as uint16;
                    } else if v18 as ::core::ffi::c_int == 2048 as ::core::ffi::c_int {
                        *r0.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        v19 = (v16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    } else {
                        *r0.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        v19 = (v16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        *r0.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                        *r6.offset(
                            (v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                        ) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v17 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    }
                } else {
                    *r0.offset((v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v17 as usize]
                            .top_left;
                    *r6.offset((v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_left;
                    v19 = (v16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    *r0.offset((v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v17 as usize]
                            .top_right;
                    *r6.offset((v19 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                            .tables[v17 as usize]
                            .bottom_right;
                }
                v16 = (v19 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                r3 = (r3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                let ref mut fresh24 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16);
                *fresh24 = (*fresh24 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x1c25) as *mut uint16)
                    as ::core::ffi::c_int
                    - 512 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
                if r20 == 0 {
                    break;
                }
            }
        }
        k = v35;
        let mut addr: uint16 = r3;
        if r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            addr = (r3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int * r22 as ::core::ffi::c_int)
                as uint16;
        }
        v31 = *(RomPtr_84orRAM(addr as uint16_t, game) as *mut uint16) as int16;
        if v31 != 0 {
            x = (*((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16)
                as ::core::ffi::c_int
                + v31 as int8 as ::core::ffi::c_int) as uint16;
            let mut v32: uint16 = (addr as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            y = (*((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16)
                as ::core::ffi::c_int
                + *RomPtr_84orRAM(v32 as uint16_t, game) as int8 as ::core::ffi::c_int)
                as uint16;
            v1 = (v32 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        } else {
            return;
        }
    }
}
pub unsafe fn PlmInstr_MovePlmDownOneBlock(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh39 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh39 = (*fresh39 as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmSetup_CrumbleBotwoonWall(mut j: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 64 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmInstr_Scroll_0_1_Blue(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 257 as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_MovePlmDownOneBlock_0(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh38 = (*fresh38 as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_ABD6(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh31 = (*fresh31 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmPreInstr_PositionSamusAndInvincible(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16).offset(v1 as isize);
    let ref mut fresh69 = *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16);
    *fresh69 = (*fresh69 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn PlmInstr_DealDamage_2(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh37 = *((&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16);
    *fresh37 = (*fresh37 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_GiveInvincibility(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 48 as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_Draw0x38FramesOfRightTreadmill(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    WriteRowOfLevelDataBlockAndBTS(k, 0x30ff as uint16, 0x8 as uint16, 0x38 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_Draw0x38FramesOfLeftTreadmill(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    WriteRowOfLevelDataBlockAndBTS(k, 0x30ff as uint16, 0x9 as uint16, 0x38 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_GotoIfSamusHealthFull(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
    {
        return plmp.offset(2);
    }
    CallSomeSamusCode(1 as uint16, game);
    return *(plmp as *mut uint16) as uintptr_t as *const uint8;
}
pub unsafe fn PlmInstr_GotoIfMissilesFull(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
    {
        return plmp.offset(2);
    }
    CallSomeSamusCode(1 as uint16, game);
    return *(plmp as *mut uint16) as uintptr_t as *const uint8;
}
pub unsafe fn PlmInstr_PlaceSamusOnSaveStation(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            & 0xfff0 as ::core::ffi::c_int) as uint16;
    MakeSamusFaceForward(game);
    return plmp;
}
pub unsafe fn PlmInstr_DisplayGameSavedMessageBox(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    DisplayMessageBox(0x18 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_EnableMovementAndSetSaveStationUsed(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    CallSomeSamusCode(1 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e75) as *mut uint16) = 1 as uint16;
    return plmp;
}
pub unsafe fn PlmSetup_SetrupWreckedShipEntrance(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    v2 = 56 as int16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 -= 1;
        if !(v2 != 0) {
            break;
        }
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_BTS_Brinstar_0x80_Floorplant(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v1 as isize);
        *fresh20 = (*fresh20 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v2 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16).offset(v2 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_BTS_Brinstar_0x81_Ceilingplant(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v1 as isize);
        *fresh19 = (*fresh19 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v2 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16).offset(v2 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn ActivateStationIfSamusCannonLinedUp(
    mut a: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    let mut current_block: u64;
    let mut v2: uint16 = 78 as uint16;
    loop {
        if !(a as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int)
        {
            current_block = 16658872821858055392;
            break;
        }
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            current_block = 12991533354927412482;
            break;
        }
    }
    match current_block {
        16658872821858055392 => {
            CalculatePlmBlockCoords(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16),
                game,
            );
            if (16 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                | 0xb as ::core::ffi::c_int
                == *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
            {
                let mut v3: ::core::ffi::c_int =
                    v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
                    .offset(v3 as isize) = *((&raw mut game.g_ram as *mut uint8).offset(0xdebc)
                    as *mut uint16)
                    .offset(v3 as isize);
                *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
                    .offset(v3 as isize) = 1 as uint16;
                CallSomeSamusCode(6 as uint16, game);
                return 1 as uint8;
            }
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmSetup_B6D3_MapStation(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v1 as isize) =
            addr_word_84AD76 as ::core::ffi::c_int as uint16;
    } else {
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16,
            0xb047 as uint16,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 4 as ::core::ffi::c_int) as uint16,
            0xb048 as uint16,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_Bts47_MapStationRightAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_8A_FaceL_Ranintowall as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return ActivateStationIfSamusCannonLinedUp(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16,
            j,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmSetup_Bts4_MapStationLeftAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_89_FaceR_Ranintowall as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return ActivateStationIfSamusCannonLinedUp(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int) as uint16,
            j,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmSetup_PlmB6DF_EnergyStation(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16,
        0xb049 as uint16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16,
        (0x84b04a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_PlmB6EB_EnergyStation(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16,
        0xb04b as uint16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16,
        0xb04c as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_B6E3_EnergyStationRightAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            != kPose_8A_FaceL_Ranintowall as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)
                as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    } else {
        return ActivateStationIfSamusCannonLinedUp(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16,
            j,
            game,
        );
    };
}
pub unsafe fn PlmSetup_B6E7_EnergyStationLeftAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_89_FaceR_Ranintowall as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)
                as ::core::ffi::c_int
    {
        return ActivateStationIfSamusCannonLinedUp(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16,
            j,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmSetup_B6EF_MissileStationRightAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            != kPose_8A_FaceL_Ranintowall as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16)
                as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    }
    return ActivateStationIfSamusCannonLinedUp(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16,
        j,
        game,
    );
}
pub unsafe fn PlmSetup_B6F3_MissileStationLeftAccess(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_89_FaceR_Ranintowall as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16)
                as ::core::ffi::c_int
    {
        return ActivateStationIfSamusCannonLinedUp(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16,
            j,
            game,
        );
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    };
}
pub fn DeletePlm(j: uint16, game: &mut Game) {
    let header_offset = 0x1c37 + usize::from(j & !1);
    game.g_ram[header_offset..header_offset + 2].fill(0);
}
pub fn PlmSetup_B638_Rightwards_Extension(j: uint16, game: &mut Game) -> uint8 {
    setup_extension_block(j, 0x50ff, game)
}
pub fn PlmSetup_B63F_Leftwards_Extension(j: uint16, game: &mut Game) -> uint8 {
    setup_extension_block(j, 0x5001, game)
}
pub fn PlmSetup_B643_Downwards_Extension(j: uint16, game: &mut Game) -> uint8 {
    setup_extension_block(j, 0xd0ff, game)
}
pub fn PlmSetup_B647_Upwards_Extension(j: uint16, game: &mut Game) -> uint8 {
    setup_extension_block(j, 0xd001, game)
}
fn setup_extension_block(j: uint16, block_type_and_bts: u16, game: &mut Game) -> uint8 {
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    DeletePlm(j, game);
    0
}
pub fn SkipDebugDrawInstructionsForScrollPlms(j: uint16, game: &mut Game) {
    let instruction_offset = 0x1d27 + usize::from(j & !1);
    let instruction = u16::from_le_bytes([
        game.g_ram[instruction_offset],
        game.g_ram[instruction_offset + 1],
    ]);
    game.g_ram[instruction_offset..instruction_offset + 2]
        .copy_from_slice(&instruction.wrapping_add(4).to_le_bytes());
}
pub fn PlmSetup_B703_ScrollPLM(j: uint16, game: &mut Game) -> uint8 {
    setup_scroll_block(j, 0x3046, game)
}
pub fn PlmSetup_B707_SolidScrollPLM(j: uint16, game: &mut Game) -> uint8 {
    setup_scroll_block(j, 0xb046, game)
}
fn setup_scroll_block(j: uint16, block_type_and_bts: u16, game: &mut Game) -> uint8 {
    let slot_index = usize::from(j & !1);
    let block_offset = 0x1c87 + slot_index;
    let block_index = u16::from_le_bytes([game.g_ram[block_offset], game.g_ram[block_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    let trigger_offset = 0x1e17 + slot_index;
    game.g_ram[trigger_offset..trigger_offset + 2].fill(0);
    // Read the live instruction after the level-data/BTS and trigger writes.
    SkipDebugDrawInstructionsForScrollPlms(j, game);
    0
}
pub fn PlmSetup_B6FF_ScrollBlockTouch(j: uint16, game: &mut Game) -> uint8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_offset = 0x1c87 + usize::from(j & !1);
    let block_index = read_word(game, block_offset);
    write_word(game, block_offset, 0);
    // Search after clearing our block index, including our own slot as before.
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| read_word(game, 0x1c87 + slot * 2) == block_index)
    else {
        // The original handler hangs when no matching scroll PLM exists.
        loop {}
    };
    let index = slot * 2;
    if read_word(game, 0x1e17 + index) & 0x8000 == 0 {
        write_word(game, 0x1e17 + index, 0x8000);
        // Retain both separate instruction writes before setting its timer.
        let instruction = read_word(game, 0x1d27 + index).wrapping_add(1);
        write_word(game, 0x1d27 + index, instruction);
        let instruction = read_word(game, 0x1d27 + index).wrapping_add(1);
        write_word(game, 0x1d27 + index, instruction);
        write_word(game, 0xde1c + index, 1);
    }
    0
}
pub fn PlmSetup_DeactivatePlm(j: uint16, game: &mut Game) -> uint8 {
    retain_plm_block_bits(j, 0x8fff, game);
    0
}
pub fn PlmSetup_ReturnCarryClear(_j: uint16) -> uint8 {
    0
}
pub fn PlmSetup_ReturnCarrySet(_j: uint16) -> uint8 {
    1
}
pub fn PlmSetup_D094_EnemyBreakableBlock(j: uint16, game: &mut Game) -> uint8 {
    // Turn the spike into air, retaining its tile and flip bits.
    retain_plm_block_bits(j, 0x0fff, game);
    0
}
fn retain_plm_block_bits(j: uint16, mask: u16, game: &mut Game) {
    // Both indices are byte offsets; the original word indexing rounds them down.
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    let block_offset = 0x10002 + usize::from(block_index & !1);
    let block = u16::from_le_bytes([game.g_ram[block_offset], game.g_ram[block_offset + 1]]);
    game.g_ram[block_offset..block_offset + 2].copy_from_slice(&(block & mask).to_le_bytes());
}
pub unsafe fn UNUSED_sub_84B3E3(mut j: uint16) -> uint8 {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/sm_84.c\0" as *const u8 as *const ::core::ffi::c_char,
            1370 as ::core::ffi::c_uint,
            b"uint8 UNUSED_sub_84B3E3(uint16)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return 0 as uint8;
}
pub fn PlmSetup_B70F_IcePhysics(_j: uint16, game: &mut Game) -> uint8 {
    let y_radius = u16::from_le_bytes([game.g_ram[0xb00], game.g_ram[0xb01]]);
    let y_pos = u16::from_le_bytes([game.g_ram[0xafa], game.g_ram[0xafb]]);
    let bottom = y_pos.wrapping_add(y_radius).wrapping_sub(1);
    if bottom & 0x0f == 7 || bottom & 0x0f == 15 {
        game.g_ram[0xb4c..0xb4e].copy_from_slice(&16u16.to_le_bytes());
    }
    0
}
static g_word_84B48B: [uint16; 2] = [0x200, 0x200];
static g_word_84B48F: [uint16; 2] = [0x120, 0x100];
static g_word_84B493: [uint16; 2] = [0x280, 0x380];
pub fn PlmSetup_QuicksandSurface(_j: uint16, game: &mut Game) -> uint8 {
    // Clear momentum, boost and extra running speed in the original order.
    for offset in [0xb3c, 0xb3e, 0xb40, 0xb44, 0xb42] {
        game.g_ram[offset..offset + 2].fill(0);
    }
    game.g_ram[0xb49] &= 0x7f;
    game.g_ram[0xb46..0xb48].fill(0);
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    let physics_index = if equipped_items & 0x20 != 0 { 2 } else { 0 };
    let reaction_point = u16::from_le_bytes([game.g_ram[0x1e73], game.g_ram[0x1e74]]);
    if reaction_point == 0 {
        let y_direction = u16::from_le_bytes([game.g_ram[0xb36], game.g_ram[0xb37]]);
        match y_direction & 3 {
            0 | 3 => PlmSetup_QuicksandSurface_0(physics_index, game),
            1 => PlmSetup_QuicksandSurface_1(physics_index, game),
            2 => PlmSetup_QuicksandSurface_2(physics_index, game),
            _ => unreachable!(),
        }
    }
    0
}
pub fn PlmSetup_QuicksandSurface_0(j: uint16, game: &mut Game) {
    game.g_ram[0xb2c..0xb2e].fill(0);
    game.g_ram[0xb2e..0xb30].fill(0);
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    // Write the middle word of the extra 16.16 Y displacement.
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48F[usize::from(j >> 1)].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_1(j: uint16, game: &mut Game) {
    let physics_index = usize::from(j >> 1);
    let y_speed_middle = u16::from_le_bytes([game.g_ram[0xb2d], game.g_ram[0xb2e]]);
    if g_word_84B493[physics_index] < y_speed_middle {
        game.g_ram[0xb2c..0xb2e].fill(0);
        game.g_ram[0xb2e..0xb30].fill(0);
        game.g_ram[0xb2d..0xb2f].copy_from_slice(&g_word_84B493[physics_index].to_le_bytes());
    }
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[physics_index].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_2(j: uint16, game: &mut Game) {
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[usize::from(j >> 1)].to_le_bytes());
    game.g_ram[0xaf4..0xaf6].fill(0);
}
pub fn PlmSetup_B71F_SubmergingQuicksand(_j: uint16, game: &mut Game) -> uint8 {
    game.g_ram[0xaf4..0xaf6].fill(0);
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x2000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
pub fn PlmSetup_B723_SandfallsSlow(_j: uint16, game: &mut Game) -> uint8 {
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x4000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
pub fn PlmSetup_B727_SandFallsFast(_j: uint16, game: &mut Game) -> uint8 {
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0xc000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
/// Apply the collision-side quicksand reaction to the active displacement.
///
/// # Safety
/// A non-null `displacement` must point to a live, writable i32 for the
/// duration of this call, as passed through collision PLM setup.
pub unsafe fn PlmSetup_QuicksandSurfaceB(
    _j: uint16,
    displacement: *mut i32,
    game: &mut Game,
) -> uint8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    if read_word(game, 0xb02) & 2 == 0 {
        return 0;
    }
    if displacement.is_null() {
        printf(
            b"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        Unreachable(game);
        return 0 as uint8;
    }
    match read_word(game, 0xb36) & 3 {
        0 | 3 => {
            if read_word(game, 0xb02) & 0x0f == 3 {
                if read_word(game, 0xa6e) == 1 {
                    *displacement = 0;
                    return 1;
                }
                // Compare the unsigned middle word; retain both outside bytes.
                if (*displacement >> 8) as u16 > 0x30 {
                    *displacement = ((*displacement as u32 & 0xff0000ff) | 0x3000) as i32;
                }
                let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
                game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
            }
        }
        2 => {
            if read_word(game, 0xa6e) == 1 {
                *displacement = 0;
                return 1;
            }
            let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
            game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
        }
        _ => {}
    }
    0
}
pub fn PlmSetup_B737_SubmergingQuicksand(_j: uint16, game: &mut Game) -> uint8 {
    game.g_ram[0xb2c..0xb2e].fill(0);
    game.g_ram[0xb2e..0xb30].fill(0);
    game.g_ram[0xb32..0xb34].fill(0);
    game.g_ram[0xb34..0xb36].fill(0);
    0
}
pub fn PlmSetup_B73B_B73F_SandFalls(_j: uint16) -> uint8 {
    0
}
pub unsafe fn PlmSetup_ClearShitroidInvisibleWall(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    v2 = 10 as int16;
    loop {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh10 = (*fresh10 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        v2 -= 1;
        if !(v2 != 0) {
            break;
        }
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_B767_ClearShitroidInvisibleWall(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    v2 = 10 as int16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        v2 -= 1;
        if !(v2 != 0) {
            break;
        }
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_B76B_SaveStationTrigger(j: uint16, game: &mut Game) -> uint8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    if read_word(game, 0x592) == 0 // power_bomb_explosion_status
        && (game.samus_pose() == kPose_01_FaceR_Normal as u16
            || game.samus_pose() == kPose_02_FaceL_Normal as u16)
        && read_word(game, 0x1e75) == 0 // save_station_lockout_flag
        && game.samus_collision_direction() & 0xf == 3
    {
        CalculatePlmBlockCoords(j, game);
        let samus_block = game.samus_x_pos().wrapping_sub(8) >> 4;
        let station_block = read_word(game, 0x1c29); // plm_x_block
        if samus_block == station_block {
            let trigger_offset = usize::from(j >> 1) * 2;
            let block_index = read_word(game, 0x1c87 + trigger_offset);
            // Remove the temporary trigger before finding the station's PLM.
            game.g_ram[0x1c87 + trigger_offset..0x1c89 + trigger_offset]
                .copy_from_slice(&0u16.to_le_bytes());
            game.g_ram[0x1c37 + trigger_offset..0x1c39 + trigger_offset]
                .copy_from_slice(&0u16.to_le_bytes());
            for slot in (0..40).rev() {
                let slot_offset = slot * 2;
                if read_word(game, 0x1c87 + slot_offset) == block_index {
                    let instruction_offset = 0x1d27 + slot_offset;
                    let instruction = read_word(game, instruction_offset).wrapping_add(1);
                    game.g_ram[instruction_offset..instruction_offset + 2]
                        .copy_from_slice(&instruction.to_le_bytes());
                    let instruction = read_word(game, instruction_offset).wrapping_add(1);
                    game.g_ram[instruction_offset..instruction_offset + 2]
                        .copy_from_slice(&instruction.to_le_bytes());
                    game.g_ram[0xde1c + slot_offset..0xde1e + slot_offset]
                        .copy_from_slice(&1u16.to_le_bytes());
                    break;
                }
            }
        }
    }
    1
}
pub unsafe fn PlmSetup_B76F_SaveStation(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0xb04d as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_MotherBrainRoomEscapeDoor(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v1, 0x9001 as uint16, game);
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v1 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0ff as uint16, game);
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ff as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v3 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_B7EB_EnableSoundsIn32Frames(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 6 as ::core::ffi::c_int
    {
        v1 = 32 as uint16;
    } else {
        v1 = 240 as uint16;
    }
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) = v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v2 as isize) =
        (0x84b7dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh84 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize);
    let fresh85 = *fresh84;
    *fresh84 = (*fresh84).wrapping_sub(1);
    if fresh85 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5f5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(v2 as isize) =
            0 as uint16;
    }
}
pub unsafe fn PlmPreInstr_WakeAndLavaIfBoosterCollected(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 0x2000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
                -(128 as ::core::ffi::c_int) as uint16;
            let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh82 = (*fresh82).wrapping_add(1);
            let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh83 = (*fresh83).wrapping_add(1);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    };
}
pub unsafe fn PlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        <= 0xae0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 1 as uint16;
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh80 = (*fresh80).wrapping_add(1);
        let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh81 = (*fresh81).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
pub unsafe fn PlmPreInstr_AdvanceLavaSamusMovesLeft(mut k: uint16, game: &mut Game) {
    static g_word_84B876: [uint16; 10] = [
        0x72b as ::core::ffi::c_int as uint16,
        0x1bf as ::core::ffi::c_int as uint16,
        0xff50 as ::core::ffi::c_int as uint16,
        0x50a as ::core::ffi::c_int as uint16,
        0x167 as ::core::ffi::c_int as uint16,
        0xff20 as ::core::ffi::c_int as uint16,
        0x244 as ::core::ffi::c_int as uint16,
        0x100 as ::core::ffi::c_int as uint16,
        0xff20 as ::core::ffi::c_int as uint16,
        0x8000 as ::core::ffi::c_int as uint16,
    ];
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize);
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = g_word_84B876[v3 as usize];
    if v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        SetEventHappened(0x15 as uint16, game);
    } else if v4 as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
    {
        if (g_word_84B876[(v3 + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
                as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) =
                g_word_84B876[(v3 + 1 as ::core::ffi::c_int) as usize];
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
            g_word_84B876[(v3 + 2 as ::core::ffi::c_int) as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize) =
            (v2 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn PlmSetup_SpeedBoosterEscape(mut j: uint16, game: &mut Game) -> uint8 {
    if CheckEventHappened(0x15 as uint16, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_ShaktoolsRoom(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 257 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8
            as *mut uint16) = 257 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        > 0x348 as ::core::ffi::c_int
    {
        SetEventHappened(0xd as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn PlmSetup_ShaktoolsRoom(mut j: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8 as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn WakePlmIfSamusIsBelowAndRightOfTarget(
    mut k: uint16,
    mut x_r18: uint16,
    mut y_r20: uint16,
    game: &mut Game,
) -> uint8 {
    if (x_r18 as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        && (y_r20 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v2 as isize);
        *fresh21 = (*fresh21 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        return false_0 as uint8;
    }
    return true_0 as uint8;
}
pub unsafe fn PlmPreInstr_OldTourianEscapeShaftEscape(mut k: uint16, game: &mut Game) {
    if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240 as uint16, 2080 as uint16, game) == 0 {
        SpawnEprojWithRoomGfx(0xb4b1 as uint16, 0 as uint16, game);
    }
}
pub unsafe fn PlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft(mut k: uint16, game: &mut Game) {
    if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240 as uint16, 1344 as uint16, game) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
            -(104 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 16 as uint16;
    }
}
pub unsafe fn PlmSetup_B974(mut j: uint16) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn PlmSetup_B9C1_CrittersEscapeBlock(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) != 0
    {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0x9f as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
            (v3 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmInstr_SetCrittersEscapedEvent(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetEventHappened(0xf as uint16, game);
    return plmp;
}
pub unsafe fn PlmSetup_B9ED_CrittersEscapeBlock(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v1, 0xc04f as uint16, game);
    let mut v2: uint16 = (v1 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0ff as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn sub_84B9F1(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    let mut v4: ::core::ffi::c_int =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v3 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v4 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v4 as isize)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmInstr_JumpIfSamusHasNoBombs(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 0x1000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return plmp.offset(2);
    } else {
        return *(plmp as *mut uint16) as uintptr_t as *const uint8;
    };
}
pub unsafe fn PlmSetup_BB30_CrateriaMainstreetEscape(mut j: uint16, game: &mut Game) -> uint8 {
    if CheckEventHappened(0xf as uint16, game) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmInstr_MovePlmRight4Blocks(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh36 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh36 = (*fresh36 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggered(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize) != 0
    {
        let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh78 = (*fresh78).wrapping_add(1);
        let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh79 = (*fresh79).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v1 as isize) =
            addr_locret_84BB6A as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm(mut k: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16)
            as ::core::ffi::c_int
        && (((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
            < 5 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh76 = (*fresh76).wrapping_add(1);
        let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh77 = (*fresh77).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v1 as isize) =
            (0x84bba3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm(mut k: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x1c29) as *mut uint16)
            as ::core::ffi::c_int
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b) as *mut uint16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >= 0xfffc as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh74 = (*fresh74).wrapping_add(1);
        let ref mut fresh75 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh75 = (*fresh75).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v1 as isize) =
            addr_locret_84BBDC as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn PlmInstr_ClearTrigger(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_SpawnEproj(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(*(plmp as *mut uint16), 0 as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_WakeEprojAtPlmPos(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut i: ::core::ffi::c_int = 0;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    i = 34 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if v2 as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
        {
            break;
        }
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v4: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v4 as isize) =
        1 as uint16;
    let ref mut fresh35 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v4 as isize);
    *fresh35 = (*fresh35 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    return plmp.offset(2);
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShot(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v2 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xdebc) as *mut uint16)
                .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
    }
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithPowerBomb(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize);
    if v3 != 0 {
        if v3 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int == 768 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
                .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8).offset(0xdebc)
                as *mut uint16)
                .offset(v2 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            return;
        }
        QueueSfx2_Max6(0x57 as uint16, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithAnyMissile(k: uint16, game: &mut Game) {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let shot = read_word(game, 0x1d77 + index); // plm_timers
    if shot != 0 {
        match shot & 0x0f00 {
            0x0200 => write_word(game, 0xdf0c + index, 119), // plm_variables
            0x0100 => {}
            _ => {
                QueueSfx2_Max6(0x57, game);
                write_word(game, 0x1d77 + index, 0);
                return;
            }
        }
        write_word(game, 0x1d77 + index, 0);
        let link = read_word(game, 0xdebc + index); // plm_instruction_list_link_reg
        write_word(game, 0x1d27 + index, link); // plm_instr_list_ptrs
        write_word(game, 0xde1c + index, 1); // plm_instruction_timer
        return;
    }
    write_word(game, 0x1d77 + index, 0);
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithSuperMissile(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize);
    if v3 != 0 {
        if v3 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int == 512 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
                .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8).offset(0xdebc)
                as *mut uint16)
                .offset(v2 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            return;
        }
        QueueSfx2_Max6(0x57 as uint16, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
pub unsafe fn PlmPreInstr_GoToLinkInstruction(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdebc) as *mut uint16).offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
}
pub unsafe fn PlmPreInstr_PlayDudSound(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) != 0
    {
        QueueSfx2_Max6(0x57 as uint16, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
pub unsafe fn PlmPreInstr_GotoLinkIfBoss1Dead(mut k: uint16, game: &mut Game) {
    if CheckBossBitForCurArea(1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfMiniBossDead(mut k: uint16, game: &mut Game) {
    if CheckBossBitForCurArea(2 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfTorizoDead(mut k: uint16, game: &mut Game) {
    if CheckBossBitForCurArea(4 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfEnemyDeathQuotaOk(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xe50) as *mut uint16) as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0xe52) as *mut uint16) as ::core::ffi::c_int
    {
        PlmPreInstr_PlayDudSound(k, game);
    } else {
        SetEventHappened(0 as uint16, game);
        PlmPreInstr_GoToLinkInstruction(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing(
    mut k: uint16,
    game: &mut Game,
) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1e6d) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        PlmPreInstr_PlayDudSound(k, game);
    } else {
        PlmPreInstr_GoToLinkInstruction(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfCrittersEscaped(mut k: uint16, game: &mut Game) {
    if CheckEventHappened(0xf as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmInstr_SetGreyDoorPreInstr(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(game.g_rom.offset(
            (((0x84be4b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84be4b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    return plmp;
}
pub unsafe fn PlmSetup_C806_LeftGreenGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        == 512 as ::core::ffi::c_int
    {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C80A_RightGreenGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        == 512 as ::core::ffi::c_int
    {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C80E_LeftRedGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int == 256 as ::core::ffi::c_int
        || v1 as ::core::ffi::c_int == 512 as ::core::ffi::c_int
    {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C812_RightRedGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int == 256 as ::core::ffi::c_int
        || v1 as ::core::ffi::c_int == 512 as ::core::ffi::c_int
    {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C822_RightYellowGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        != 768 as ::core::ffi::c_int
    {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C81E_LeftYellowGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        == 768 as ::core::ffi::c_int
    {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57 as uint16, game);
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    return PlmSetup_C822_RightYellowGateTrigger(j, game);
}
pub unsafe fn PlmSetup_C81A_RightBlueGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        != 768 as ::core::ffi::c_int
    {
        return sub_84C647(j, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C816_LeftBlueGateTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        != 768 as ::core::ffi::c_int
    {
        return sub_84C63F(j, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return PlmSetup_C81A_RightBlueGateTrigger(j, game);
}
pub unsafe fn sub_84C63F(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84C64C(
        j,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn sub_84C647(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84C64C(
        j,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn sub_84C64C(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    let mut v3: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut v2: uint16 = 78 as uint16;
    loop {
        if !(a as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int)
        {
            current_block = 16658872821858055392;
            break;
        }
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            current_block = 5208286747870620093;
            break;
        }
    }
    match current_block {
        16658872821858055392 => {
            v3 = 0;
            v3 = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
                .offset(v3 as isize)
                == 0
            {
                let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d77)
                    as *mut uint16)
                    .offset(v3 as isize);
                *fresh7 = (*fresh7).wrapping_add(1);
            }
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn SetBts0x10FiveStepsDown(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = SetBtsTo0x10AdvanceRow(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    let mut v2: uint16 = SetBtsTo0x10AdvanceRow(v1, game);
    let mut v3: uint16 = SetBtsTo0x10AdvanceRow(v2, game);
    let mut v4: uint16 = SetBtsTo0x10AdvanceRow(v3, game);
    SetBtsTo0x10AdvanceRow(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRow(mut k: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as *mut uint8 as *mut uint16) = ((*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset((k as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        | 0x10 as ::core::ffi::c_int)
        as uint16;
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        + k as ::core::ffi::c_int) as uint16;
}
pub unsafe fn SetBts0x10FiveStepsUp(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = SetBtsTo0x10AdvanceRowUp(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    let mut v2: uint16 = SetBtsTo0x10AdvanceRowUp(v1, game);
    let mut v3: uint16 = SetBtsTo0x10AdvanceRowUp(v2, game);
    let mut v4: uint16 = SetBtsTo0x10AdvanceRowUp(v3, game);
    SetBtsTo0x10AdvanceRowUp(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRowUp(mut k: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as *mut uint8 as *mut uint16) = ((*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset((k as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        | 0x10 as ::core::ffi::c_int)
        as uint16;
    return (k as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn PlmSetup_C82A_DownwardsClosedGate(
    mut j: uint16,
    setup_a: uint16,
    game: &mut Game,
) -> uint8 {
    SpawnEprojWithRoomGfx(0xe659 as uint16, setup_a, game);
    SetBts0x10FiveStepsDown(j, game);
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C832_UpwardsClosedGate(mut j: uint16, game: &mut Game) -> uint8 {
    SpawnEprojWithRoomGfx(0xe675 as uint16, 0 as uint16, game);
    SetBts0x10FiveStepsUp(j, game);
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C826_DownwardsOpenGate(mut j: uint16, game: &mut Game) -> uint8 {
    SetBts0x10FiveStepsDown(j, game);
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C82E_UpwardsOpenGate(mut j: uint16, game: &mut Game) -> uint8 {
    SetBts0x10FiveStepsUp(j, game);
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C836_DownwardsGateShootblock(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(
            (((0x84c70a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84c70a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    let mut v2: uint16 = *(game.g_rom.offset(
        (((0x84c71a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x84c71a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v2 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16,
            v2,
            game,
        );
    }
    let mut v3: uint16 = *(game.g_rom.offset(
        (((0x84c72a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x84c72a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v3 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16,
            v3,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C73A_UpwardsGateShootblock(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(
            (((0x84c764 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84c764 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    let mut v2: uint16 = *(game.g_rom.offset(
        (((0x84c774 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x84c774 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v2 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16,
            v2,
            game,
        );
    }
    let mut v3: uint16 = *(game.g_rom.offset(
        (((0x84c784 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x84c784 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v3 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16,
            v3,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_C794_GreyDoor(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 0x7c as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
    let ref mut fresh6 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *fresh6 = (*fresh6 as ::core::ffi::c_int & !(0x7c00 as ::core::ffi::c_int)) as uint16;
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize),
        0xc044 as uint16,
        game,
    );
    return 0 as uint8;
}
pub fn PlmSetup_Door_Colored(j: uint16, game: &mut Game) -> uint8 {
    let index = usize::from(j >> 1) * 2;
    let block_index = u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
    WriteLevelDataBlockTypeAndBts(block_index, 0xc044, game);
    0
}
pub unsafe fn PlmSetup_Door_Blue(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 768 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let index = usize::from(j >> 1) * 2;
        let v1 = u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int) as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_Door_Strange(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let mut v3: uint16 = 78 as uint16;
    while v2 as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
    {
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 0 as uint8;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as int16
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
            & 0x1fff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmInstr_SetBtsTo1(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x16402).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = 1 as uint8;
    return plmp;
}
pub unsafe fn PlmSetup_D028_D02C_Unused(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
            (v3 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn PlmSetup_RespawningSpeedBoostBlock(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3e) as *mut uint16) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_C9_FaceR_Shinespark_Horiz as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CA_FaceL_Shinespark_Horiz as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CB_FaceR_Shinespark_Vert as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CC_FaceL_Shinespark_Vert as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CD_FaceR_Shinespark_Diag as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CE_FaceL_Shinespark_Diag as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0xb6 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
            (v3 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn PlmSetup_RespawningCrumbleBlock(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0xbc as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
            (v3 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            4 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn PlmSetup_RespawningShotBlock(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int
        | 0x52 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v2 as isize) =
        (v3 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_RespawningBombBlock(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3e) as *mut uint16) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_C9_FaceR_Shinespark_Horiz as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CA_FaceL_Shinespark_Horiz as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CB_FaceR_Shinespark_Vert as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CC_FaceL_Shinespark_Vert as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CD_FaceR_Shinespark_Diag as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_CE_FaceL_Shinespark_Diag as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0x58 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v2 as isize) =
            v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v3 as isize) =
            (v4 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn PlmSetup_RespawningBombBlock2(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int == 1280 as ::core::ffi::c_int {
        let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v5 as isize);
        *fresh5 = (*fresh5 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
        let mut v6: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v5 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v7: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v6 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0x58 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v5 as isize) =
            v7;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v6 as isize) =
            (v7 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
    } else if v1 as ::core::ffi::c_int == 768 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0x58 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v2 as isize) =
            v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v3 as isize) =
            (v4 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub fn PlmSetup_RespawningPowerBombBlock(j: uint16, game: &mut Game) -> uint8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let projectile_index = usize::from(read_word(game, 0xdde) >> 1) * 2;
    let projectile_type = read_word(game, 0xc18 + projectile_index) & 0x0f00;
    let index = usize::from(j >> 1) * 2;
    match projectile_type {
        0x0500 => write_word(game, 0x1d27 + index, addr_kPlmInstrList_C91C as u16),
        0x0300 => {
            let block_index = usize::from(read_word(game, 0x1c87 + index) >> 1) * 2;
            let block = read_word(game, 0x10002 + block_index);
            let revealed = (block & 0xf000) | 0x0057;
            write_word(game, 0x1e17 + index, revealed);
            // Keep the original reveal mask and variable-before-level write order.
            write_word(game, 0x10002 + block_index, revealed & 0x8fff);
        }
        _ => write_word(game, 0x1c37 + index, 0),
    }
    0
}
pub unsafe fn PlmSetup_D08C_SuperMissileBlockRespawning(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int == 1280 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            addr_kPlmInstrList_C922 as ::core::ffi::c_int as uint16;
    } else if v1 as ::core::ffi::c_int == 512 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            | 0x9f as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v2 as isize) =
            v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(v3 as isize) =
            (v4 as ::core::ffi::c_int & 0x8fff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D08C_CrumbleBlock(mut j: uint16, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        != 1280 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D0DC_BreakableGrappleBlock(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = ((*(&raw mut game.g_ram as *mut uint8).offset(0x16402).offset(
        ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    return 0x41 as uint8;
}
pub unsafe fn PlmSetup_D0D8_SetVFlag(mut j: uint16) -> uint8 {
    return 0x41 as uint8;
}
pub unsafe fn PlmSetup_D0D8_ClearVflag(mut j: uint16) -> uint8 {
    return 1 as uint8;
}
pub unsafe fn PlmSetup_D0E8_GiveSamusDamage(mut j: uint16, game: &mut Game) -> uint8 {
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16);
    *fresh4 = (*fresh4).wrapping_add(1);
    return 0x41 as uint8;
}
pub unsafe fn PlmSetup_D113_LowerNorfairChozoRoomPlug(mut j: uint16, game: &mut Game) -> uint8 {
    let mut a: uint16 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = (a as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D127(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0xc000 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D138(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0xe000 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmInstr_FxBaseYPos_0x2D2(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = 722 as uint16;
    return plmp;
}
pub unsafe fn PlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed(mut k: uint16, game: &mut Game) {
    let mut prod: uint16 = (8 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8
            as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + 4 as ::core::ffi::c_int))
        as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        == 255 as ::core::ffi::c_int
    {
        WriteLevelDataBlockTypeAndBts(v1, 0xb083 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = 0 as uint16;
    }
}
pub unsafe fn PlmSetup_D6DA_LowerNorfairChozoHandTrigger(
    mut j: uint16,
    displacement: *mut i32,
    game: &mut Game,
) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
            as ::core::ffi::c_int
            == kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                == kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                == kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int)
    {
        SetEventHappened(0xc as uint16, game);
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .parameter_1 = 1 as uint16;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v2 as isize);
        *fresh3 = (*fresh3 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        CallSomeSamusCode(0 as uint16, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs {
                field_0: 0xc as uint8,
                field_1: 0x1d as uint8,
                field_2: 0xd113 as VoidP,
            },
            displacement,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmPreInstr_IncrementRoomArgIfShotBySuperMissile(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize);
    if v2 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int == 512 as ::core::ffi::c_int
        || v2 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int == 256 as ::core::ffi::c_int
    {
        let ref mut fresh73 = *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset(v1 as isize);
        *fresh73 = (*fresh73).wrapping_add(1);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn PlmInstr_GotoIfRoomArgLess(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >= *(plmp as *mut uint16) as ::core::ffi::c_int
    {
        return plmp.offset(4);
    } else {
        return *(plmp.offset(2) as *mut uint16) as uintptr_t as *const uint8;
    };
}
pub unsafe fn PlmInstr_SpawnFourMotherBrainGlass(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max15(0x2e as uint16, game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(0) as *mut uint16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(2) as *mut uint16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(4) as *mut uint16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(6) as *mut uint16), game);
    return plmp.offset(8);
}
pub unsafe fn SpawnMotherBrainGlassShatteringShard(mut a: uint16, game: &mut Game) {
    SpawnEprojWithRoomGfx(0xcefc as uint16, a, game);
}
pub unsafe fn PlmPreInstr_WakePlmIfSamusHasBombs(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 0x1000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        let ref mut fresh72 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh72 = (*fresh72 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v1 as isize) =
            (0x84d356 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn PlmInstr_SpawnTorizoStatueBreaking(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(0xa993 as uint16, *(plmp as *mut uint16), game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_QueueSong1MusicTrack(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    QueueMusic_Delayed8(6 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_TransferWreckedShipChozoSpikesToSlopes(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    WriteLevelDataBlockTypeAndBts(0x1608 as uint16, 0x1012 as uint16, game);
    WriteLevelDataBlockTypeAndBts(0x160a as uint16, 0x1013 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_TransferWreckedShipSlopesToChozoSpikes(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    WriteLevelDataBlockTypeAndBts(0x1608 as uint16, 0xa000 as uint16, game);
    WriteLevelDataBlockTypeAndBts(0x160a as uint16, 0xa000 as uint16, game);
    return plmp;
}
pub unsafe fn PlmPreInstr_WakeOnKeyPress(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_B as ::core::ffi::c_int
            | kButton_Y as ::core::ffi::c_int
            | kButton_Left as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int
            | kButton_A as ::core::ffi::c_int
            | kButton_X as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v2 as isize);
        *fresh67 = (*fresh67).wrapping_add(1);
        let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v2 as isize);
        *fresh68 = (*fresh68).wrapping_add(1);
    }
}
pub unsafe fn PlmInstr_EnableWaterPhysics(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16);
    *fresh34 = (*fresh34 as ::core::ffi::c_int & !(4 as ::core::ffi::c_int)) as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_SpawnN00bTubeCrackEproj(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(0xd904 as uint16, 0 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_DiagonalEarthquake(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 11 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 64 as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_Spawn10shardsAnd6n00bs(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        2 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        4 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        6 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        8 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0xa as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0xc as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0xe as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0x10 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeShards as ::core::ffi::c_int as uint16,
        0x12 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        2 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        4 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        6 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        8 as uint16,
        game,
    );
    SpawnEprojWithRoomGfx(
        addr_kEproj_N00bTubeReleasedAirBubbles as ::core::ffi::c_int as uint16,
        0xa as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DisableSamusControls(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    CallSomeSamusCode(0 as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_EnableSamusControls(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    CallSomeSamusCode(1 as uint16, game);
    return plmp;
}
pub unsafe fn PlmSetup_MotherBrainGlass(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize),
        0x8044 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_DeletePlmIfAreaTorizoDead(mut j: uint16, game: &mut Game) -> uint8 {
    if CheckBossBitForCurArea(4 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_MakeBllockChozoHandTrigger(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0xb080 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D6F2_WreckedShipChozoHandTrigger(
    mut j: uint16,
    displacement: *mut i32,
    game: &mut Game,
) -> uint8 {
    if CheckBossBitForCurArea(1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xb02) as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
            as ::core::ffi::c_int
            == kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                == kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                == kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int)
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .parameter_1 = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) as *mut uint8
            as *mut uint16) = 514 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(13) as *mut uint8 as *mut uint16) = 257 as uint16;
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset(v1 as isize);
        *fresh2 = (*fresh2 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as uint16;
        CallSomeSamusCode(0 as uint16, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs {
                field_0: 0x17 as uint8,
                field_1: 0x1d as uint8,
                field_2: 0xd6f8 as VoidP,
            },
            displacement,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn PlmSetup_D700_MakePlmAirBlock_Unused(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0x44 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused(
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0x8044 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D708_LowerNorfairChozoBlockUnused(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize),
        0x8044 as uint16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16,
        0x50ff as uint16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_D70C_NoobTube(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0x8044 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_WakePlmIfRoomArgumentDoorIsSet(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = PrepareBitAccess(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v2 as isize),
        game,
    ) as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(idx as isize) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v3 as isize) =
            addr_locret_84D779 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v3 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xdebc) as *mut uint16)
                .offset(v3 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v3 as isize) =
            1 as uint16;
    }
}
pub unsafe fn PlmInstr_ShootEyeDoorProjectileWithProjectileArg(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_EyeDoorProjectile as ::core::ffi::c_int as uint16,
        *(plmp as *mut uint16),
        game,
    );
    QueueSfx2_Max6(0x4c as uint16, game);
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_SpawnEyeDoorSweatEproj(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_EyeDoorSweat as ::core::ffi::c_int as uint16,
        *(plmp as *mut uint16),
        game,
    );
    return plmp.offset(2);
}
pub unsafe fn PlmInstr_SpawnTwoEyeDoorSmoke(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(0xe517 as uint16, 0x30a as uint16, game);
    SpawnEprojWithRoomGfx(0xe517 as uint16, 0x30a as uint16, game);
    return plmp;
}
pub unsafe fn PlmInstr_SpawnEyeDoorSmokeProjectile(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_EyeDoorSmoke as ::core::ffi::c_int as uint16,
        0xb as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_MoveUpAndMakeBlueDoorFacingRight(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh33 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v2 as isize);
    *fresh33 = (*fresh33 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int) as uint16;
    let mut blk: uint16 = *fresh33;
    WriteLevelDataBlockTypeAndBts(blk, 0xc041 as uint16, game);
    sub_84D7EF(blk, game);
    return plmp;
}
pub unsafe fn PlmInstr_MoveUpAndMakeBlueDoorFacingLeft(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh32 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v2 as isize);
    *fresh32 = (*fresh32 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int) as uint16;
    let mut blk: uint16 = *fresh32;
    WriteLevelDataBlockTypeAndBts(blk, 0xc040 as uint16, game);
    sub_84D7EF(blk, game);
    return plmp;
}
pub unsafe fn sub_84D7EF(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + k as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v1, 0xd0ff as uint16, game);
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v1 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0fe as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v2 as ::core::ffi::c_int) as uint16,
        0xd0fd as uint16,
        game,
    );
}
pub unsafe fn PlmSetup_EyeDoorEye(mut j: uint16, game: &mut Game) -> uint8 {
    let mut idx: ::core::ffi::c_int = PrepareBitAccess(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        game,
    ) as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(idx as isize) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        WriteLevelDataBlockTypeAndBts(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize),
            0xc044 as uint16,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            0xd0ff as uint16,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn PlmSetup_EyeDoor(mut j: uint16, game: &mut Game) -> uint8 {
    let mut idx: ::core::ffi::c_int = PrepareBitAccess(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        game,
    ) as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(idx as isize) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        WriteLevelDataBlockTypeAndBts(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xa000 as uint16,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x10(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe50) as *mut uint16) as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xe52) as *mut uint16) as ::core::ffi::c_int
    {
        SetEventHappened(0x10 as uint16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x11(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe50) as *mut uint16) as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xe52) as *mut uint16) as ::core::ffi::c_int
    {
        SetEventHappened(0x11 as uint16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x12(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe50) as *mut uint16) as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xe52) as *mut uint16) as ::core::ffi::c_int
    {
        SetEventHappened(0x12 as uint16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x13(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe50) as *mut uint16) as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xe52) as *mut uint16) as ::core::ffi::c_int
    {
        SetEventHappened(0x13 as uint16, game);
    }
}
pub unsafe fn PlmSetup_SetMetroidRequiredClearState(mut j: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(game.g_rom.offset(
            (((0x84db28 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84db28 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
                    .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_GotoLinkIfShotWithSuperMissile(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int == 512 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 119 as uint16;
    } else if v1 as ::core::ffi::c_int != 256 as ::core::ffi::c_int {
        return;
    }
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdebc) as *mut uint16).offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
}
pub unsafe fn SetPlmVarPtr(mut k: uint16, mut a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as isize,
    ) as *mut uint8 as *mut uint16) = a;
}
pub unsafe fn PlmInstr_DamageDraygonTurret(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingDownRight(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v3 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingUpRight(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0 as uint16,
        game,
    );
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DamageDraygonTurret2(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingDownLeft(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingUpLeft(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    SetPlmVarPtr(k, 1 as uint16, game);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0 as uint16,
        game,
    );
    let mut v4: uint16 = (v2 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0xa003 as uint16,
        game,
    );
    return plmp;
}
pub unsafe fn PlmSetup_DraygonCannonFacingRight(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_DraygonCannonFacingDiagonalRight(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0x50ff as uint16,
        game,
    );
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ff as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_DraygonCannonFacingLeft(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_DraygonCannonFacingDiagonalLeft(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044 as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0x5001 as uint16,
        game,
    );
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ff as uint16, game);
    WriteLevelDataBlockTypeAndBts(
        (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16,
        0xd0ff as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn PlmSetup_DraygonCannon(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(v1 as isize) =
        3 as uint16;
    return 0 as uint8;
}
pub unsafe fn PlmPreInstr_GotoLinkIfTriggered(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *(((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize)
        as *mut uint16 as *mut uint8)
        .offset(0) as ::core::ffi::c_int
        == 255 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1cd7) as *mut uint16).offset(v1 as isize) =
            (0x84dfa8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xdebc) as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn PlmPreInstr_WakeIfTriggered(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize);
    if v2 as ::core::ffi::c_int != 768 as ::core::ffi::c_int
        && v2 as uint8 as ::core::ffi::c_int == 255 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        let ref mut fresh70 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh70 = (*fresh70).wrapping_add(1);
        let ref mut fresh71 = *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16)
            .offset(v1 as isize);
        *fresh71 = (*fresh71).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn PlmInstr_DrawItemFrame_Common(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v2 as isize) =
        4 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v2 as isize) =
        plmp.offset_from(RomBankBase(0x84 as uint8, game)) as ::core::ffi::c_long as uint16;
    ProcessPlmDrawInstruction(k, game);
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16);
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27) as *mut uint16),
        game,
    );
    DrawPlm(v3, game);
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn PlmInstr_DrawItemFrame0(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xde6c) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(game.g_rom.offset(
            (((0x84e05f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84e05f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    return PlmInstr_DrawItemFrame_Common(plmp, k, game);
}
pub unsafe fn PlmInstr_DrawItemFrame1(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xde6c) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(game.g_rom.offset(
            (((0x84e077 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x84e077 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    return PlmInstr_DrawItemFrame_Common(plmp, k, game);
}
pub unsafe fn PlmInstr_ClearChargeBeamCounter(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
    return plmp;
}
pub unsafe fn PlmInstr_E63B(
    mut plmp: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
        -(32 as ::core::ffi::c_int) as uint16;
    return plmp;
}
pub unsafe fn sub_84EE4D(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE5F(j, 8 as uint16, game);
}
pub unsafe fn sub_84EE52(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE5F(j, 0xa as uint16, game);
}
pub unsafe fn sub_84EE57(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE5F(j, 0xc as uint16, game);
}
pub unsafe fn sub_84EE5C(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE5F(j, 0xe as uint16, game);
}
pub unsafe fn sub_84EE5F(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = a;
    return sub_84EE64(j, game);
}
pub unsafe fn sub_84EE64(mut j: uint16, game: &mut Game) -> uint8 {
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        0x45 as uint16,
        game,
    );
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xd91a) as *mut uint16);
    *fresh1 = (*fresh1).wrapping_add(1);
    return 0 as uint8;
}
pub unsafe fn sub_84EE77(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE89(j, 8 as uint16, game);
}
pub unsafe fn sub_84EE7C(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE89(j, 0xa as uint16, game);
}
pub unsafe fn sub_84EE81(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE89(j, 0xc as uint16, game);
}
pub unsafe fn sub_84EE86(mut j: uint16, game: &mut Game) -> uint8 {
    return sub_84EE89(j, 0xe as uint16, game);
}
pub unsafe fn sub_84EE89(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf0c) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = a;
    return sub_84EE8E(j, game);
}
pub unsafe fn sub_84EE8E(mut j: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    WriteLevelDataBlockTypeAndBts(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize),
        0xc045 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e17) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xd91a) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_add(1);
    return 0 as uint8;
}
pub unsafe fn sub_84EEAB(mut v0: uint16, game: &mut Game) -> uint8 {
    let mut i: ::core::ffi::c_int = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        i = 78 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            if v2 as ::core::ffi::c_int
                == *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
            {
                break;
            }
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1d77) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
    }
    return 0 as uint8;
}
/// Dispatch setup with the active collision displacement, or null outside collision.
/// `setup_a` preserves the spawner accumulator for setup routines that consume it.
///
/// # Safety
/// Legacy setup callbacks require initialized runtime state. A non-null
/// `displacement` must remain live and writable throughout the setup call.
pub unsafe fn CallPlmHeaderFunc(
    ea: uint32,
    j: uint16,
    displacement: *mut i32,
    setup_a: uint16,
    game: &mut Game,
) -> uint8 {
    match ea {
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 => {
            return 0 as uint8;
        }
        8694568 => return PlmSetup_CrumbleBotwoonWall(j, game),
        8695882 => return PlmSetup_SetrupWreckedShipEntrance(j, game),
        8696028 => return PlmSetup_BTS_Brinstar_0x80_Floorplant(j, game),
        8696083 => return PlmSetup_BTS_Brinstar_0x81_Ceilingplant(j, game),
        8696203 => return PlmSetup_B6D3_MapStation(j, game),
        8696264 => return PlmSetup_Bts47_MapStationRightAccess(j, game),
        8696304 => return PlmSetup_Bts4_MapStationLeftAccess(j, game),
        8696349 => return PlmSetup_PlmB6DF_EnergyStation(j, game),
        8696389 => return PlmSetup_PlmB6EB_EnergyStation(j, game),
        8696429 => return PlmSetup_B6E3_EnergyStationRightAccess(j, game),
        8696477 => return PlmSetup_B6E7_EnergyStationLeftAccess(j, game),
        8696528 => return PlmSetup_B6EF_MissileStationRightAccess(j, game),
        8696576 => return PlmSetup_B6F3_MissileStationLeftAccess(j, game),
        8696634 => return PlmSetup_B638_Rightwards_Extension(j, game),
        8696645 => return PlmSetup_B63F_Leftwards_Extension(j, game),
        8696656 => return PlmSetup_B643_Downwards_Extension(j, game),
        8696667 => return PlmSetup_B647_Upwards_Extension(j, game),
        8696689 => return PlmSetup_B703_ScrollPLM(j, game),
        8696706 => return PlmSetup_B707_SolidScrollPLM(j, game),
        8696723 => return PlmSetup_B6FF_ScrollBlockTouch(j, game),
        8696769 => return PlmSetup_DeactivatePlm(j, game),
        8696784 => return PlmSetup_ReturnCarryClear(j),
        8696786 => return PlmSetup_ReturnCarrySet(j),
        8696788 => return PlmSetup_D094_EnemyBreakableBlock(j, game),
        8696803 => return UNUSED_sub_84B3E3(j),
        8696811 => return PlmSetup_B70F_IcePhysics(j, game),
        8696840 => return PlmSetup_QuicksandSurface(j, game),
        8696983 => return PlmSetup_B71F_SubmergingQuicksand(j, game),
        8697000 => return PlmSetup_B723_SandfallsSlow(j, game),
        8697014 => return PlmSetup_B727_SandFallsFast(j, game),
        8697028 => return PlmSetup_QuicksandSurfaceB(j, displacement, game),
        8697153 => return PlmSetup_B737_SubmergingQuicksand(j, game),
        8697167 => return PlmSetup_B73B_B73F_SandFalls(j),
        8697169 => return PlmSetup_ClearShitroidInvisibleWall(j, game),
        8697199 => return PlmSetup_B767_ClearShitroidInvisibleWall(j, game),
        8697232 => return PlmSetup_B76B_SaveStationTrigger(j, game),
        8697326 => return PlmSetup_B76F_SaveStation(j, game),
        8697336 => return PlmSetup_MotherBrainRoomEscapeDoor(j, game),
        8697795 => return PlmSetup_B7EB_EnableSoundsIn32Frames(j, game),
        8698012 => return PlmSetup_SpeedBoosterEscape(j, game),
        8698076 => return PlmSetup_ShaktoolsRoom(j, game),
        8698220 => return PlmSetup_B974(j),
        8698232 => return PlmSetup_B9C1_CrittersEscapeBlock(j, game),
        8698309 => return PlmSetup_B9ED_CrittersEscapeBlock(j, game),
        8698353 => return sub_84B9F1(j, game),
        8698633 => return PlmSetup_BB30_CrateriaMainstreetEscape(j, game),
        8701261 => return PlmSetup_C806_LeftGreenGateTrigger(j, game),
        8701292 => return PlmSetup_C80A_RightGreenGateTrigger(j, game),
        8701323 => return PlmSetup_C80E_LeftRedGateTrigger(j, game),
        8701359 => return PlmSetup_C812_RightRedGateTrigger(j, game),
        8701395 => return PlmSetup_C81E_LeftYellowGateTrigger(j, game),
        8701425 => return PlmSetup_C822_RightYellowGateTrigger(j, game),
        8701456 => return PlmSetup_C816_LeftBlueGateTrigger(j, game),
        8701479 => return PlmSetup_C81A_RightBlueGateTrigger(j, game),
        8701630 => return PlmSetup_C82A_DownwardsClosedGate(j, setup_a, game),
        8701643 => return PlmSetup_C832_UpwardsClosedGate(j, game),
        8701656 => return PlmSetup_C826_DownwardsOpenGate(j, game),
        8701660 => return PlmSetup_C82E_UpwardsOpenGate(j, game),
        8701664 => return PlmSetup_C836_DownwardsGateShootblock(j, game),
        8701754 => return PlmSetup_C73A_UpwardsGateShootblock(j, game),
        8701844 => return PlmSetup_C794_GreyDoor(j, game),
        8701873 => return PlmSetup_Door_Colored(j, game),
        8701883 => return PlmSetup_Door_Blue(j, game),
        8701922 => return PlmSetup_Door_Strange(j, game),
        8703426 => return PlmSetup_D028_D02C_Unused(j, game),
        8703466 => return PlmSetup_RespawningSpeedBoostBlock(j, game),
        8703543 => return PlmSetup_RespawningCrumbleBlock(j, game),
        8703595 => return PlmSetup_RespawningShotBlock(j, game),
        8703619 => return PlmSetup_RespawningBombBlock(j, game),
        8703706 => return PlmSetup_RespawningBombBlock2(j, game),
        8703790 => return PlmSetup_RespawningPowerBombBlock(j, game),
        8703847 => return PlmSetup_D08C_SuperMissileBlockRespawning(j, game),
        8703904 => return PlmSetup_D08C_CrumbleBlock(j, game),
        8703925 => return PlmSetup_D0DC_BreakableGrappleBlock(j, game),
        8703949 => return PlmSetup_D0D8_SetVFlag(j),
        8703953 => return PlmSetup_D0D8_ClearVflag(j),
        8703957 => return PlmSetup_D0E8_GiveSamusDamage(j, game),
        8704264 => return PlmSetup_D113_LowerNorfairChozoRoomPlug(j, game),
        8704279 => return PlmSetup_D127(j, game),
        8704299 => return PlmSetup_D138(j, game),
        8704399 => return PlmSetup_D6DA_LowerNorfairChozoHandTrigger(j, displacement, game),
        8705526 => return PlmSetup_MotherBrainGlass(j, game),
        8705542 => return PlmSetup_DeletePlmIfAreaTorizoDead(j, game),
        8705558 => return PlmSetup_MakeBllockChozoHandTrigger(j, game),
        8705568 => return PlmSetup_D6F2_WreckedShipChozoHandTrigger(j, displacement, game),
        8705663 => return PlmSetup_D700_MakePlmAirBlock_Unused(j, game),
        8705673 => return PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused(j, game),
        8705683 => return PlmSetup_D708_LowerNorfairChozoBlockUnused(j, game),
        8705740 => return PlmSetup_D70C_NoobTube(j, game),
        8706700 => return PlmSetup_EyeDoorEye(j, game),
        8706745 => return PlmSetup_EyeDoor(j, game),
        8706846 => return PlmSetup_SetMetroidRequiredClearState(j, game),
        8707732 => return PlmSetup_DraygonCannonFacingRight(j, game),
        8707769 => return PlmSetup_DraygonCannonFacingDiagonalRight(j, game),
        8707824 => return PlmSetup_DraygonCannonFacingLeft(j, game),
        8707861 => return PlmSetup_DraygonCannonFacingDiagonalLeft(j, game),
        8707916 => return PlmSetup_DraygonCannon(j, game),
        8711757 => return sub_84EE4D(j, game),
        8711762 => return sub_84EE52(j, game),
        8711767 => return sub_84EE57(j, game),
        8711772 => return sub_84EE5C(j, game),
        8711780 => return sub_84EE64(j, game),
        8711799 => return sub_84EE77(j, game),
        8711804 => return sub_84EE7C(j, game),
        8711809 => return sub_84EE81(j, game),
        8711814 => return sub_84EE86(j, game),
        8711822 => return sub_84EE8E(j, game),
        8711851 => return sub_84EEAB(j, game),
        _ => return Unreachable(game) as uint8,
    };
}
pub unsafe fn CallPlmPreInstr(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8684649 => return,
        8708008 => return,
        8685264 => return,
        8684774 => return,
        8684861 => return,
        8686246 => return,
        8698787 => return,
        8704854 => return,
        8698618 => return,
        8696028 => {
            PlmSetup_BTS_Brinstar_0x80_Floorplant(k, game);
            return;
        }
        8696083 => {
            PlmSetup_BTS_Brinstar_0x81_Ceilingplant(k, game);
            return;
        }
        8697821 => {
            PlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm(k, game);
            return;
        }
        8697839 => {
            PlmPreInstr_WakeAndLavaIfBoosterCollected(k, game);
            return;
        }
        8697898 => {
            PlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft(k, game);
            return;
        }
        8697926 => {
            PlmPreInstr_AdvanceLavaSamusMovesLeft(k, game);
            return;
        }
        8698032 => {
            PlmPreInstr_ShaktoolsRoom(k, game);
            return;
        }
        8698151 => {
            PlmPreInstr_OldTourianEscapeShaftEscape(k, game);
            return;
        }
        8698184 => {
            PlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft(k, game);
            return;
        }
        8698706 => {
            PlmPreInstr_WakePlmIfTriggered(k, game);
            return;
        }
        8698731 => {
            PlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm(k, game);
            return;
        }
        8698788 => {
            PlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm(k, game);
            return;
        }
        8704348 => {
            PlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed(k, game);
            return;
        }
        8704486 => {
            PlmPreInstr_IncrementRoomArgIfShotBySuperMissile(k, game);
            return;
        }
        8704827 => {
            PlmPreInstr_WakePlmIfSamusHasBombs(k, game);
            return;
        }
        8705875 => {
            PlmPreInstr_WakePlmIfRoomArgumentDoorIsSet(k, game);
            return;
        }
        8706916 => {
            PlmPreInstr_GotoLinkIfShotWithSuperMissile(k, game);
            return;
        }
        8707977 => {
            PlmPreInstr_GotoLinkIfTriggered(k, game);
            return;
        }
        8708070 => {
            PlmPreInstr_WakeIfTriggered(k, game);
            return;
        }
        8699151 => {
            PlmPreInstr_GoToLinkInstrIfShot(k, game);
            return;
        }
        8699174 => {
            PlmPreInstr_GoToLinkInstrIfShotWithPowerBomb(k, game);
            return;
        }
        8699216 => {
            PlmPreInstr_GoToLinkInstrIfShotWithAnyMissile(k, game);
            return;
        }
        8699272 => {
            PlmPreInstr_GoToLinkInstrIfShotWithSuperMissile(k, game);
            return;
        }
        8699314 => {
            PlmPreInstr_GoToLinkInstruction(k, game);
            return;
        }
        8699420 => {
            PlmPreInstr_PlayDudSound(k, game);
            return;
        }
        8699348 => {
            PlmPreInstr_GotoLinkIfBoss1Dead(k, game);
            return;
        }
        8699363 => {
            PlmPreInstr_GotoLinkIfMiniBossDead(k, game);
            return;
        }
        8699378 => {
            PlmPreInstr_GotoLinkIfTorizoDead(k, game);
            return;
        }
        8699393 => {
            PlmPreInstr_GotoLinkIfEnemyDeathQuotaOk(k, game);
            return;
        }
        8699423 => {
            PlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing(k, game);
            return;
        }
        8699440 => {
            PlmPreInstr_GotoLinkIfCrittersEscaped(k, game);
            return;
        }
        8694921 => {
            PlmPreInstr_PositionSamusAndInvincible(k, game);
            return;
        }
        8705215 => {
            PlmPreInstr_WakeOnKeyPress(k, game);
            return;
        }
        8698730 => return,
        8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778 | 8706779 | 8706780 | 8706781
        | 8686304 => return,
        8706782 => {
            PlmPreInstr_SetMetroidsClearState_Ev0x10(k, game);
            return;
        }
        8706798 => {
            PlmPreInstr_SetMetroidsClearState_Ev0x11(k, game);
            return;
        }
        8706814 => {
            PlmPreInstr_SetMetroidsClearState_Ev0x12(k, game);
            return;
        }
        8706830 => {
            PlmPreInstr_SetMetroidsClearState_Ev0x13(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallPlmInstr(
    mut ea: uint32,
    mut j: *const uint8,
    mut k: uint16,
    game: &mut Game,
) -> *const uint8 {
    match ea {
        8685236 => return PlmInstr_Sleep(j, k, game),
        8685244 => return PlmInstr_Delete(j, k, game),
        8685249 => return PlmInstr_PreInstr(j, k, game),
        8685258 => return PlmInstr_ClearPreInstr(j, k, game),
        8685323 => return PlmInstr_CallFunction(j, k, game),
        8685348 => return PlmInstr_Goto(j, k),
        8685375 => return PlmInstr_DecrementAndBranchNonzero(j, k, game),
        8685390 => return PlmInstr_SetTimer(j, k, game),
        8685412 => return PlmInstr_LoadItemPlmGfx(j, k, game),
        8685541 => return PlmInstr_CopyFromRamToVram(j, k, game),
        8685582 => return PlmInstr_GotoIfBossBitSet(j, k, game),
        8685613 => return PlmInstr_GotoIfEventSet(j, k, game),
        8685630 => return PlmInstr_SetEvent(j, k, game),
        8685640 => return PlmInstr_GotoIfChozoSet(j, k, game),
        8685669 => return PlmInstr_SetRoomChozoBit(j, k, game),
        8685692 => return PlmInstr_GotoIfItemBitSet(j, k, game),
        8685721 => return PlmInstr_SetItemBit(j, k, game),
        8685744 => return PlmInstr_PickupBeamAndShowMessage(j, k, game),
        8685811 => return PlmInstr_PickupEquipmentAndShowMessage(j, k, game),
        8685850 => return PlmInstr_PickupEquipmentAddGrappleShowMessage(j, k, game),
        8685889 => return PlmInstr_PickupEquipmentAddXrayShowMessage(j, k, game),
        8685928 => return PlmInstr_CollectHealthEnergyTank(j, k, game),
        8685958 => return PlmInstr_CollectHealthReserveTank(j, k, game),
        8685993 => return PlmInstr_CollectAmmoMissileTank(j, k, game),
        8686034 => return PlmInstr_CollectAmmoSuperMissileTank(j, k, game),
        8686075 => return PlmInstr_CollectAmmoPowerBombTank(j, k, game),
        8686116 => return PlmInstr_SetLinkReg(j, k, game),
        8686126 => return PlmInstr_Call(j, k, game),
        8686138 => return PlmInstr_Return(j, k, game),
        8686194 => return PlmInstr_GotoIfDoorBitSet(j, k, game),
        8686225 => return PlmInstr_IncrementDoorHitCounterAndJGE(j, k, game),
        8686285 => return PlmInstr_IncrementArgumentAndJGE(j, k, game),
        8686321 => return PlmInstr_SetBTS(j, k, game),
        8686341 => return PlmInstr_DrawPlmBlock(j, k, game),
        8686359 => return PlmInstr_DrawPlmBlock(j, k, game),
        8686421 => return PlmInstr_ProcessAirScrollUpdate(j, k, game),
        8686483 => return PlmInstr_ProcessSolidScrollUpdate(j, k, game),
        8686545 => return PlmInstr_QueueMusic(j, k, game),
        8686557 => return PlmInstr_ClearMusicQueueAndQueueTrack(j, k, game),
        8686599 => return PlmInstr_QueueSfx1_Max6(j, k, game),
        8686608 => return PlmInstr_QueueSfx2_Max6(j, k, game),
        8686617 => return PlmInstr_QueueSfx3_Max6(j, k, game),
        8686626 => return PlmInstr_QueueSfx1_Max15(j, k, game),
        8686635 => return PlmInstr_QueueSfx2_Max15(j, k, game),
        8686644 => return PlmInstr_QueueSfx3_Max15(j, k, game),
        8686653 => return PlmInstr_QueueSfx1_Max3(j, k, game),
        8686662 => return PlmInstr_QueueSfx2_Max3(j, k, game),
        8686671 => return PlmInstr_QueueSfx_Max3(j, k, game),
        8686680 => return PlmInstr_QueueSfx1_Max9(j, k, game),
        8686689 => return PlmInstr_QueueSfx2_Max9(j, k, game),
        8686698 => return PlmInstr_QueueSfx3_Max9(j, k, game),
        8686707 => return PlmInstr_QueueSfx1_Max1(j, k, game),
        8686716 => return PlmInstr_QueueSfx2_Max1(j, k, game),
        8686725 => return PlmInstr_QueueSfx3_Max1(j, k, game),
        8686735 => return PlmInstr_ActivateMapStation(j, k, game),
        8686767 => return PlmInstr_ActivateEnergyStation(j, k, game),
        8686800 => return PlmInstr_ActivateMissileStation(j, k, game),
        8686833 => return PlmInstr_ActivateSaveStationAndGotoIfNo(j, k, game),
        8686913 => return PlmInstr_GotoIfSamusNear(j, k, game),
        8694528 => return PlmInstr_MovePlmDownOneBlock(j, k, game),
        8694609 => return PlmInstr_Scroll_0_1_Blue(j, k, game),
        8694617 => return PlmInstr_MovePlmDownOneBlock_0(j, k, game),
        8694941 => return PlmInstr_DealDamage_2(j, k, game),
        8694961 => return PlmInstr_GiveInvincibility(j, k, game),
        8695107 => return PlmInstr_Draw0x38FramesOfRightTreadmill(j, k, game),
        8695128 => return PlmInstr_Draw0x38FramesOfLeftTreadmill(j, k, game),
        8695349 => return PlmInstr_GotoIfSamusHealthFull(j, k, game),
        8695487 => return PlmInstr_GotoIfMissilesFull(j, k, game),
        8695822 => return PlmInstr_PlaceSamusOnSaveStation(j, k, game),
        8695844 => return PlmInstr_DisplayGameSavedMessageBox(j, k, game),
        8695856 => return PlmInstr_EnableMovementAndSetSaveStationUsed(j, k, game),
        8698297 => return PlmInstr_SetCrittersEscapedEvent(j, k, game),
        8698479 => return PlmInstr_JumpIfSamusHasNoBombs(j, k, game),
        8698661 => return PlmInstr_MovePlmRight4Blocks(j, k, game),
        8698845 => return PlmInstr_ClearTrigger(j, k, game),
        8698849 => return PlmInstr_SpawnEproj(j, k, game),
        8698864 => return PlmInstr_WakeEprojAtPlmPos(j, k, game),
        8699455 => return PlmInstr_SetGreyDoorPreInstr(j, k, game),
        8704341 => return PlmInstr_FxBaseYPos_0x2D2(j, k, game),
        8704761 => return PlmInstr_GotoIfRoomArgLess(j, k, game),
        8704779 => return PlmInstr_SpawnFourMotherBrainGlass(j, k, game),
        8704855 => return PlmInstr_SpawnTorizoStatueBreaking(j, k, game),
        8704967 => return PlmInstr_QueueSong1MusicTrack(j, k, game),
        8704983 => return PlmInstr_TransferWreckedShipChozoSpikesToSlopes(j, k, game),
        8705012 => return PlmInstr_TransferWreckedShipSlopesToChozoSpikes(j, k, game),
        8705317 => return PlmInstr_EnableWaterPhysics(j, k, game),
        8705324 => return PlmInstr_SpawnN00bTubeCrackEproj(j, k, game),
        8705334 => return PlmInstr_DiagonalEarthquake(j, k, game),
        8705347 => return PlmInstr_Spawn10shardsAnd6n00bs(j, k, game),
        8705914 => return PlmInstr_ShootEyeDoorProjectileWithProjectileArg(j, k, game),
        8705936 => return PlmInstr_SpawnEyeDoorSweatEproj(j, k, game),
        8705951 => return PlmInstr_SpawnTwoEyeDoorSmoke(j, k, game),
        8705974 => return PlmInstr_SpawnEyeDoorSmokeProjectile(j, k, game),
        8705987 => return PlmInstr_MoveUpAndMakeBlueDoorFacingRight(j, k, game),
        8706010 => return PlmInstr_MoveUpAndMakeBlueDoorFacingLeft(j, k, game),
        8706958 => return PlmInstr_DamageDraygonTurret(j, k, game),
        8707000 => return PlmInstr_DamageDraygonTurretFacingDownRight(j, k, game),
        8707063 => return PlmInstr_DamageDraygonTurretFacingUpRight(j, k, game),
        8707126 => return PlmInstr_DamageDraygonTurret2(j, k, game),
        8707168 => return PlmInstr_DamageDraygonTurretFacingDownLeft(j, k, game),
        8707231 => return PlmInstr_DamageDraygonTurretFacingUpLeft(j, k, game),
        8708175 => return PlmInstr_DrawItemFrame0(j, k, game),
        8708199 => return PlmInstr_DrawItemFrame1(j, k, game),
        8708223 => return PlmInstr_DrawItemFrame_Common(j, k, game),
        8708765 => return PlmInstr_ClearChargeBeamCounter(j, k, game),
        8694742 => return PlmInstr_ABD6(j, k, game),
        8709691 => return PlmInstr_E63B(j, k, game),
        8703379 => return PlmInstr_SetBtsTo1(j, k, game),
        8705510 => return PlmInstr_DisableSamusControls(j, k, game),
        8705518 => return PlmInstr_EnableSamusControls(j, k, game),
        _ => {
            Unreachable(game);
            return ::core::ptr::null::<uint8>();
        }
    };
}
