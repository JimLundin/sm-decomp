extern "C" {
    static mut g_ram: [uint8; 131072];
    static mut g_rom: *const uint8;
    fn Unreachable() -> bool;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    static kSinCosTable8bit_Sext: [int16; 320];
    fn CallEnemyPreInstr(ea: uint32);
    fn CalculateBlockContainingPixelPos(xpos: uint16, ypos: uint16);
    fn NextRandom() -> uint16;
    fn CheckBossBitForCurArea(a: uint16) -> uint8;
    fn QueueMusic_Delayed8(a: uint16);
    fn QueueSfx2_Max6(a: uint16);
    fn SetBossBitForCurArea(a: uint16);
    fn sub_82DAF7(a: uint16) -> uint8;
    fn SpawnPLM(a: uint16) -> uint8;
    fn SetGoldenTorizoPalette(a: uint16);
    fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs);
    fn SpawnEprojWithGfx(a: uint16, k: uint16, j: uint16);
    fn SpawnEprojWithRoomGfx(j: uint16, a: uint16);
    fn SpawnBombTorizoHaze();
    fn SpawnPalfxObject(j: uint16);
    fn CallSomeSamusCode(a: uint16) -> uint16;
    fn CalculateAngleOfEnemyXfromEnemyY(k: uint16, j: uint16) -> uint16;
    fn CompareDistToSamus_X(k: uint16, a: uint16) -> uint8;
    fn EnemyFunc_C8AD(k: uint16) -> uint8;
    fn Enemy_MoveDown(k: uint16, amt: int32) -> uint8;
    fn Enemy_MoveRight_IgnoreSlopes(k: uint16, amt: int32) -> uint8;
    fn Enemy_ItemDrop_BombTorizo(k: uint16);
    fn Enemy_ItemDrop_GoldenTorizo(k: uint16);
    fn Enemy_SwitchToFrozenAi();
    fn NormalEnemyFrozenAI();
    fn NormalEnemyShotAi();
    fn NormalEnemyShotAiSkipDeathAnim_CurEnemy();
    fn NormalEnemyTouchAi();
    fn NormalEnemyTouchAiSkipDeathAnim_CurEnemy();
}
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyData {
    pub enemy_ptr: uint16,
    pub x_pos: uint16,
    pub x_subpos: uint16,
    pub y_pos: uint16,
    pub y_subpos: uint16,
    pub x_width: uint16,
    pub y_height: uint16,
    pub properties: uint16,
    pub extra_properties: uint16,
    pub ai_handler_bits: uint16,
    pub health: uint16,
    pub spritemap_pointer: uint16,
    pub timer: uint16,
    pub current_instruction: uint16,
    pub instruction_timer: uint16,
    pub palette_index: uint16,
    pub vram_tiles_index: uint16,
    pub layer: uint16,
    pub flash_timer: uint16,
    pub frozen_timer: uint16,
    pub invincibility_timer: uint16,
    pub shake_timer: uint16,
    pub frame_counter: uint16,
    pub bank: uint8,
    pub field_2F: uint8,
    pub ai_var_A: uint16,
    pub ai_var_B: uint16,
    pub ai_var_C: uint16,
    pub ai_var_D: uint16,
    pub ai_var_E: uint16,
    pub ai_preinstr: uint16,
    pub parameter_1: uint16,
    pub parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyBase {
    pub enemy_ptr: uint16,
    pub x_pos: uint16,
    pub x_subpos: uint16,
    pub y_pos: uint16,
    pub y_subpos: uint16,
    pub x_width: uint16,
    pub y_height: uint16,
    pub properties: uint16,
    pub extra_properties: uint16,
    pub ai_handler_bits: uint16,
    pub health: uint16,
    pub spritemap_pointer: uint16,
    pub timer: uint16,
    pub current_instruction: uint16,
    pub instruction_timer: uint16,
    pub palette_index: uint16,
    pub vram_tiles_index: uint16,
    pub layer: uint16,
    pub flash_timer: uint16,
    pub frozen_timer: uint16,
    pub invincibility_timer: uint16,
    pub shake_timer: uint16,
    pub frame_counter: uint16,
    pub bank: uint8,
    pub field_2F: uint8,
}
pub use crate::types::SamusPose;
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
pub use crate::types::EnemyProps;
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
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
pub use crate::types::Consts_AA;
pub const addr_kShaktool_Ilist_E461: Consts_AA = 58465;
pub const addr_kShaktool_Ilist_E3A7: Consts_AA = 58279;
pub const addr_kTorizo_Ilist_D2BF: Consts_AA = 53951;
pub const addr_kTorizo_Ilist_D2AD: Consts_AA = 53933;
pub const addr_kTorizo_Ilist_D203: Consts_AA = 53763;
pub const addr_kTorizo_Ilist_D1F1: Consts_AA = 53745;
pub const addr_kTorizo_Ilist_CEFF: Consts_AA = 52991;
pub const addr_kTorizo_Ilist_CEA5: Consts_AA = 52901;
pub const addr_kTorizo_Ilist_CE43: Consts_AA = 52803;
pub const addr_kTorizo_Ilist_CDE1: Consts_AA = 52705;
pub const addr_locret_AAC95E: Consts_AA = 51550;
pub const addr_kTorizo_Ilist_C188: Consts_AA = 49544;
pub const addr_off_AAC0F2: Consts_AA = 49394;
pub const addr_kTorizo_Ilist_BDD8: Consts_AA = 48600;
pub const addr_kTorizo_Ilist_BD0E: Consts_AA = 48398;
pub const addr_off_AABC78: Consts_AA = 48248;
pub const addr_kTorizo_Ilist_B962: Consts_AA = 47458;
pub const addr_kTorizo_Ilist_B1C8: Consts_AA = 45512;
pub const addr_kTorizo_Ilist_B155: Consts_AA = 45397;
pub const addr_kTorizo_Ilist_B0E5: Consts_AA = 45285;
pub const addr_kTorizo_ExtSprmap_87D0: Consts_AA = 34768;
pub const addr_kSpritemap_Nothing_AA: Consts_AA = 32845;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SpawnHardcodedPlmArgs {
    pub field_0: uint8,
    pub field_1: uint8,
    pub field_2: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Shaktool {
    pub base: EnemyBase,
    pub shakt_var_A: uint16,
    pub shakt_var_B: uint16,
    pub shakt_var_C: uint16,
    pub shakt_var_D: uint16,
    pub shakt_var_E: uint16,
    pub shakt_var_F: uint16,
    pub shakt_parameter_1: uint16,
    pub shakt_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Torizo {
    pub base: EnemyBase,
    pub toriz_var_A: uint16,
    pub toriz_var_B: uint16,
    pub toriz_var_C: uint16,
    pub toriz_var_D: uint16,
    pub toriz_var_E: uint16,
    pub toriz_var_F: uint16,
    pub toriz_parameter_1: uint16,
    pub toriz_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub toriz_var_00: uint16,
    pub toriz_var_01: uint16,
    pub toriz_var_02: uint16,
    pub toriz_var_03: uint16,
    pub toriz_var_04: uint16,
    pub toriz_var_05: uint16,
    pub toriz_var_06: uint16,
    pub toriz_var_07: uint16,
    pub toriz_var_08: uint16,
    pub toriz_var_09: uint16,
    pub toriz_var_0A: uint16,
    pub toriz_var_0B: uint16,
    pub toriz_var_0C: uint16,
    pub toriz_var_0D: uint16,
    pub toriz_var_0E: uint16,
    pub toriz_var_0F: uint16,
    pub toriz_var_10: uint16,
    pub toriz_var_11: uint16,
    pub toriz_var_12: uint16,
    pub toriz_var_13: uint16,
    pub toriz_var_14: uint16,
    pub toriz_var_15: uint16,
    pub toriz_var_16: uint16,
    pub toriz_var_17: uint16,
    pub toriz_var_18: uint16,
    pub toriz_var_19: uint16,
    pub toriz_var_1A: uint16,
    pub toriz_var_1B: uint16,
    pub toriz_var_1C: uint16,
    pub toriz_var_1D: uint16,
    pub toriz_var_1E: uint16,
    pub toriz_var_1F: uint16,
}
#[inline]
unsafe extern "C" fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe extern "C" fn RomBankBase(mut bank: uint8) -> *const uint8 {
    return RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize));
}
#[inline]
unsafe extern "C" fn AddToHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut v: uint32) {
    let mut t: uint32 = ((*lo as ::core::ffi::c_int
        + ((*hi as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
        as uint32)
        .wrapping_add(v);
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe extern "C" fn SetHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut t: uint32) {
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub const fnnullsub_171_AA: ::core::ffi::c_int = 0xaa807b as ::core::ffi::c_int;
pub const fnnullsub_270: ::core::ffi::c_int = 0xaac6ab as ::core::ffi::c_int;
pub const fnTorizo_Func_3: ::core::ffi::c_int = 0xaac6bf as ::core::ffi::c_int;
pub const fnTorizo_Func_4: uint32 = 11192006 as uint32;
pub const fnTorizo_Func_1: uint32 = 11192063 as uint32;
pub const fnTorizo_Func_9: uint32 = 11195842 as uint32;
pub const fnTorizo_Func_10: uint32 = 11195871 as uint32;
pub const fnTorizo_D5E6: uint32 = 11195878 as uint32;
pub const fnnullsub_274: ::core::ffi::c_int = 0xaadcaa as ::core::ffi::c_int;
pub const fnnullsub_276: ::core::ffi::c_int = 0xaae7a6 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe extern "C" fn Get_Shaktool(mut o: ::core::ffi::c_int) -> *mut Enemy_Shaktool {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Shaktool;
}
#[inline]
unsafe extern "C" fn Get_Torizo(mut o: ::core::ffi::c_int) -> *mut Enemy_Torizo {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Torizo;
}
static mut g_word_AAB096: uint16 = 6 as uint16;
static mut g_word_AAB098: uint16 = 5 as uint16;
static mut g_word_AAB09A: uint16 = 3 as uint16;
static mut g_word_AAC3EE: [int16; 16] = [
    -(9 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    5 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    9 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    16 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC40E: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC440: [int16; 16] = [
    -(9 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    5 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    9 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    16 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC460: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC4BD: [int16; 20] = [
    -(5 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(7 as ::core::ffi::c_int) as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    5 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    19 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC532: [int16; 20] = [
    -(5 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(0x13 as ::core::ffi::c_int) as int16,
    -(0x10 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(7 as ::core::ffi::c_int) as int16,
    -(0x11 as ::core::ffi::c_int) as int16,
    -(0x12 as ::core::ffi::c_int) as int16,
    5 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    0x13 as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0x11 as ::core::ffi::c_int as int16,
    0x12 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC95F: [int16; 2] = [
    0xdb as ::core::ffi::c_int as int16,
    0x1a8 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC963: [int16; 2] = [
    0xb3 as ::core::ffi::c_int as int16,
    0x90 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC96B: [int16; 2] = [
    0x2800 as ::core::ffi::c_int as int16,
    0x2800 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC96F: [int16; 2] = [
    0x12 as ::core::ffi::c_int as int16,
    0x12 as ::core::ffi::c_int as int16,
];
static mut g_word_AAC973: [int16; 2] = [
    0x30 as ::core::ffi::c_int as int16,
    0x29 as ::core::ffi::c_int as int16,
];
static mut kTorizo_Palette: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x3ff as ::core::ffi::c_int as uint16,
    0x33b as ::core::ffi::c_int as uint16,
    0x216 as ::core::ffi::c_int as uint16,
    0x113 as ::core::ffi::c_int as uint16,
    0x6b1e as ::core::ffi::c_int as uint16,
    0x4a16 as ::core::ffi::c_int as uint16,
    0x3591 as ::core::ffi::c_int as uint16,
    0x20e9 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_1: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x2df as ::core::ffi::c_int as uint16,
    0x1d7 as ::core::ffi::c_int as uint16,
    0xac as ::core::ffi::c_int as uint16,
    0x5a73 as ::core::ffi::c_int as uint16,
    0x41ad as ::core::ffi::c_int as uint16,
    0x2d08 as ::core::ffi::c_int as uint16,
    0x1863 as ::core::ffi::c_int as uint16,
    0x1486 as ::core::ffi::c_int as uint16,
    0x145 as ::core::ffi::c_int as uint16,
    0x145 as ::core::ffi::c_int as uint16,
    0x145 as ::core::ffi::c_int as uint16,
    0x7fff as ::core::ffi::c_int as uint16,
    0x145 as ::core::ffi::c_int as uint16,
    0x145 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_2: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x679f as ::core::ffi::c_int as uint16,
    0x5299 as ::core::ffi::c_int as uint16,
    0x252e as ::core::ffi::c_int as uint16,
    0x14aa as ::core::ffi::c_int as uint16,
    0x5efc as ::core::ffi::c_int as uint16,
    0x4657 as ::core::ffi::c_int as uint16,
    0x35b2 as ::core::ffi::c_int as uint16,
    0x2d70 as ::core::ffi::c_int as uint16,
    0x5b7f as ::core::ffi::c_int as uint16,
    0x3df8 as ::core::ffi::c_int as uint16,
    0x2d0e as ::core::ffi::c_int as uint16,
    0x5f5f as ::core::ffi::c_int as uint16,
    0x5e1a as ::core::ffi::c_int as uint16,
    0x5d35 as ::core::ffi::c_int as uint16,
    0xc63 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_3: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x4aba as ::core::ffi::c_int as uint16,
    0x35b2 as ::core::ffi::c_int as uint16,
    0x847 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    0x4215 as ::core::ffi::c_int as uint16,
    0x2970 as ::core::ffi::c_int as uint16,
    0x18cb as ::core::ffi::c_int as uint16,
    0x1089 as ::core::ffi::c_int as uint16,
    0x463a as ::core::ffi::c_int as uint16,
    0x28b3 as ::core::ffi::c_int as uint16,
    0x1809 as ::core::ffi::c_int as uint16,
    0x6f7f as ::core::ffi::c_int as uint16,
    0x51fd as ::core::ffi::c_int as uint16,
    0x4113 as ::core::ffi::c_int as uint16,
    0xc63 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_4: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x56ba as ::core::ffi::c_int as uint16,
    0x41b2 as ::core::ffi::c_int as uint16,
    0x1447 as ::core::ffi::c_int as uint16,
    0x403 as ::core::ffi::c_int as uint16,
    0x4e15 as ::core::ffi::c_int as uint16,
    0x3570 as ::core::ffi::c_int as uint16,
    0x24cb as ::core::ffi::c_int as uint16,
    0x1868 as ::core::ffi::c_int as uint16,
    0x6f7f as ::core::ffi::c_int as uint16,
    0x51f8 as ::core::ffi::c_int as uint16,
    0x410e as ::core::ffi::c_int as uint16,
    0x31f as ::core::ffi::c_int as uint16,
    0x1da as ::core::ffi::c_int as uint16,
    0xf5 as ::core::ffi::c_int as uint16,
    0xc63 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_5: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x4215 as ::core::ffi::c_int as uint16,
    0x2d0d as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x3970 as ::core::ffi::c_int as uint16,
    0x20cb as ::core::ffi::c_int as uint16,
    0xc26 as ::core::ffi::c_int as uint16,
    0x403 as ::core::ffi::c_int as uint16,
    0x463a as ::core::ffi::c_int as uint16,
    0x28b3 as ::core::ffi::c_int as uint16,
    0x1809 as ::core::ffi::c_int as uint16,
    0x6f7f as ::core::ffi::c_int as uint16,
    0x51fd as ::core::ffi::c_int as uint16,
    0x4113 as ::core::ffi::c_int as uint16,
    0xc63 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_6: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x6ab5 as ::core::ffi::c_int as uint16,
    0x49b0 as ::core::ffi::c_int as uint16,
    0x1c45 as ::core::ffi::c_int as uint16,
    0xc01 as ::core::ffi::c_int as uint16,
    0x5613 as ::core::ffi::c_int as uint16,
    0x416d as ::core::ffi::c_int as uint16,
    0x2cc9 as ::core::ffi::c_int as uint16,
    0x2066 as ::core::ffi::c_int as uint16,
    0x5714 as ::core::ffi::c_int as uint16,
    0x31cc as ::core::ffi::c_int as uint16,
    0x14e3 as ::core::ffi::c_int as uint16,
    0x5630 as ::core::ffi::c_int as uint16,
    0x3569 as ::core::ffi::c_int as uint16,
    0x1883 as ::core::ffi::c_int as uint16,
    0xc66 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_7: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x5610 as ::core::ffi::c_int as uint16,
    0x350b as ::core::ffi::c_int as uint16,
    0x800 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x416e as ::core::ffi::c_int as uint16,
    0x2cc8 as ::core::ffi::c_int as uint16,
    0x1823 as ::core::ffi::c_int as uint16,
    0xc01 as ::core::ffi::c_int as uint16,
    0x6a31 as ::core::ffi::c_int as uint16,
    0x4caa as ::core::ffi::c_int as uint16,
    0x2406 as ::core::ffi::c_int as uint16,
    0x7f7b as ::core::ffi::c_int as uint16,
    0x75f4 as ::core::ffi::c_int as uint16,
    0x4d10 as ::core::ffi::c_int as uint16,
    0xc63 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_8: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x4bbe as ::core::ffi::c_int as uint16,
    0x6b9 as ::core::ffi::c_int as uint16,
    0xa8 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x173a as ::core::ffi::c_int as uint16,
    0x276 as ::core::ffi::c_int as uint16,
    0x1f2 as ::core::ffi::c_int as uint16,
    0x14d as ::core::ffi::c_int as uint16,
    0x73e0 as ::core::ffi::c_int as uint16,
    0x4f20 as ::core::ffi::c_int as uint16,
    0x2a20 as ::core::ffi::c_int as uint16,
    0x7fe0 as ::core::ffi::c_int as uint16,
    0x5aa0 as ::core::ffi::c_int as uint16,
    0x5920 as ::core::ffi::c_int as uint16,
    0x43 as ::core::ffi::c_int as uint16,
];
static mut kTorizo_Palettes_10: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x3719 as ::core::ffi::c_int as uint16,
    0x214 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x295 as ::core::ffi::c_int as uint16,
    0x1d1 as ::core::ffi::c_int as uint16,
    0x14d as ::core::ffi::c_int as uint16,
    0xa8 as ::core::ffi::c_int as uint16,
    0x4b40 as ::core::ffi::c_int as uint16,
    0x25e0 as ::core::ffi::c_int as uint16,
    0xe0 as ::core::ffi::c_int as uint16,
    0x6b40 as ::core::ffi::c_int as uint16,
    0x4600 as ::core::ffi::c_int as uint16,
    0x4480 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut g_word_AAD59A: [int16; 20] = [
    -(5 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(7 as ::core::ffi::c_int) as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    5 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    19 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
];
static mut g_word_AADEE9: [uint16; 7] = [
    0 as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
    0x60 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0x140 as ::core::ffi::c_int as uint16,
    0x1a0 as ::core::ffi::c_int as uint16,
    0x1e0 as ::core::ffi::c_int as uint16,
];
static mut kChozoStatue_Palette: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x633f as ::core::ffi::c_int as uint16,
    0x4a9f as ::core::ffi::c_int as uint16,
    0x2ddf as ::core::ffi::c_int as uint16,
    0x6739 as ::core::ffi::c_int as uint16,
    0x4e73 as ::core::ffi::c_int as uint16,
    0x318c as ::core::ffi::c_int as uint16,
    0x18c6 as ::core::ffi::c_int as uint16,
    0x27ff as ::core::ffi::c_int as uint16,
    0x1af7 as ::core::ffi::c_int as uint16,
    0xdce as ::core::ffi::c_int as uint16,
    0xc6 as ::core::ffi::c_int as uint16,
    0x3fff as ::core::ffi::c_int as uint16,
    0x2b39 as ::core::ffi::c_int as uint16,
    0x7fff as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kChozoStatue_Palettes: [int16; 16] = [
    0x3800 as ::core::ffi::c_int as int16,
    0x633f as ::core::ffi::c_int as int16,
    0x4a9f as ::core::ffi::c_int as int16,
    0x2ddf as ::core::ffi::c_int as int16,
    0x4210 as ::core::ffi::c_int as int16,
    0x318c as ::core::ffi::c_int as int16,
    0x2108 as ::core::ffi::c_int as int16,
    0x1084 as ::core::ffi::c_int as int16,
    0x27ff as ::core::ffi::c_int as int16,
    0x1af7 as ::core::ffi::c_int as int16,
    0xdce as ::core::ffi::c_int as int16,
    0xc6 as ::core::ffi::c_int as int16,
    0x3fff as ::core::ffi::c_int as int16,
    0x2b39 as ::core::ffi::c_int as int16,
    0x5294 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut kChozoStatue_Palettes2: [int16; 16] = [
    0x3800 as ::core::ffi::c_int as int16,
    0x633f as ::core::ffi::c_int as int16,
    0x4a9f as ::core::ffi::c_int as int16,
    0x2ddf as ::core::ffi::c_int as int16,
    0x2f7c as ::core::ffi::c_int as int16,
    0x2295 as ::core::ffi::c_int as int16,
    0x118d as ::core::ffi::c_int as int16,
    0x8e8 as ::core::ffi::c_int as int16,
    0x27ff as ::core::ffi::c_int as int16,
    0x1af7 as ::core::ffi::c_int as int16,
    0xdce as ::core::ffi::c_int as int16,
    0xc6 as ::core::ffi::c_int as int16,
    0x3fff as ::core::ffi::c_int as int16,
    0x2b39 as ::core::ffi::c_int as int16,
    0x73df as ::core::ffi::c_int as int16,
    0x43 as ::core::ffi::c_int as int16,
];
static mut kChozoStatue_Palettes3: [int16; 16] = [
    0x3800 as ::core::ffi::c_int as int16,
    0x633f as ::core::ffi::c_int as int16,
    0x4a9f as ::core::ffi::c_int as int16,
    0x2ddf as ::core::ffi::c_int as int16,
    0x2295 as ::core::ffi::c_int as int16,
    0x118d as ::core::ffi::c_int as int16,
    0x8e8 as ::core::ffi::c_int as int16,
    0x85 as ::core::ffi::c_int as int16,
    0x27ff as ::core::ffi::c_int as int16,
    0x1af7 as ::core::ffi::c_int as int16,
    0xdce as ::core::ffi::c_int as int16,
    0xc6 as ::core::ffi::c_int as int16,
    0x3fff as ::core::ffi::c_int as int16,
    0x2b39 as ::core::ffi::c_int as int16,
    0x5294 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
];
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_CancelBeam_AA() {
    Enemy_SwitchToFrozenAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalFrozenAI_AA() {
    NormalEnemyFrozenAI();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_ClearAiPreInstr_AA(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int)).ai_preinstr =
        (0xaa807b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_SetAiPreInstr_AA(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int)).ai_preinstr =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_E =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_31(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_stru_86A95B as ::core::ffi::c_int as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_33(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_36(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_37(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_35(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 160 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 144 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_38(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SetBossBitForCurArea(4 as uint16);
    QueueMusic_Delayed8(g_word_AAB09A);
    if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Enemy_ItemDrop_GoldenTorizo(k);
    } else {
        Enemy_ItemDrop_BombTorizo(k);
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    sub_82DAF7(0x600 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Torizo_C268();
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueMusic_Delayed8(g_word_AAB098);
    SpawnPalfxObject(addr_stru_8DF759 as ::core::ffi::c_int as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C20A(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = 512 as int16;
    } else {
        v2 = -(512 as ::core::ffi::c_int) as int16;
    }
    (*E).toriz_var_A = v2 as uint16;
    (*E).toriz_var_B = -(1472 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_C = 40 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C22D(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = -(768 as ::core::ffi::c_int) as int16;
    } else {
        v2 = 768 as int16;
    }
    (*E).toriz_var_A = v2 as uint16;
    (*E).toriz_var_B = -(1152 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_C = 40 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C250() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_3[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_2[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C268() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_5[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_4[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C280() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_7[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_6[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C298() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_10[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_8[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C2B0() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_5[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_4[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_04 = 30583 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_04 = 0 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_25(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if (*Get_Torizo(k as ::core::ffi::c_int)).toriz_parameter_2 as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        return *jp.offset(1 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    } else {
        return jp.offset(2 as ::core::ffi::c_int as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_22(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_00 =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_19(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_00 as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_32(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_01 as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_30(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut a: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    let mut i: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        SpawnEprojWithRoomGfx(
            addr_kEproj_BombTorizoLowHealthExplode as ::core::ffi::c_int as uint16,
            a,
        );
        i -= 1;
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.current_instruction = (jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    (*E).base.flash_timer = 40 as uint16;
    (*E).base.instruction_timer = 40 as uint16;
    return ::core::ptr::null::<uint16>();
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_34(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_BombTorizoDeathExplosion as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.current_instruction = (jp as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    (*E).base.flash_timer = 1 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    return ::core::ptr::null::<uint16>();
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_24(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianLandingDustCloudsRightFoot as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    SpawnEprojWithRoomGfx(
        addr_stru_86AFF3 as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut health: uint16 = (*Get_Torizo(k as ::core::ffi::c_int)).base.health;
    if (health as ::core::ffi::c_int) < 0x15e as ::core::ffi::c_int {
        SpawnEprojWithGfx(
            health,
            k,
            addr_kEproj_BombTorizoLowHealthInitialDrool as ::core::ffi::c_int as uint16,
        );
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_1 =
        ((*E).toriz_parameter_1 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_1 =
        ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_29(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0x8000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0x2000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_28(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0xa000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + g_word_AAC3EE[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + g_word_AAC40E[((v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_40(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        - g_word_AAC440[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        - g_word_AAC460[((v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_16(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_A = g_word_AAC4BD[(*jp.offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize] as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return addr_kTorizo_Ilist_B962 as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            return addr_kTorizo_Ilist_BDD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
    } else {
        EnemyFunc_C8AD(k);
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72 as uint16;
            }
            return jp.offset(1 as ::core::ffi::c_int as isize);
        } else {
            return jp.offset(1 as ::core::ffi::c_int as isize);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_27(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_A = g_word_AAC532[(*jp.offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize] as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return addr_kTorizo_Ilist_BD0E as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            return addr_kTorizo_Ilist_C188 as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
    } else {
        EnemyFunc_C8AD(k);
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72 as uint16;
            }
            return jp.offset(1 as ::core::ffi::c_int as isize);
        } else {
            return jp.offset(1 as ::core::ffi::c_int as isize);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_23(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_B as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    } else {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x38 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if ((*E).toriz_parameter_1 as ::core::ffi::c_int
        ^ *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*E).toriz_var_00 = (jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_15(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x20 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    Torizo_C22D(k);
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_26(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_00 = (jp.offset(2 as ::core::ffi::c_int as isize) as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 5 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    } else {
        return *jp.offset(1 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_18(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_BombTorizosChozoOrbs as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_BombTorizosChozoOrbs as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_BombTorizosChozoOrbs as ::core::ffi::c_int as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_20(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp.offset(0 as ::core::ffi::c_int as isize),
        k,
        addr_kEproj_BombTorizoSonicBoom as ::core::ffi::c_int as uint16,
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_44(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp.offset(0 as ::core::ffi::c_int as isize),
        k,
        addr_kEproj_GoldenTorizoSonicBoom as ::core::ffi::c_int as uint16,
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_21(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_BombTorizoExplosiveSwipe as ::core::ffi::c_int as uint16,
        *jp.offset(0 as ::core::ffi::c_int as isize),
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_17(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x27 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x4b as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C620(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8142 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut health: uint16 = (*Get_Torizo(k as ::core::ffi::c_int)).base.health;
        if health as ::core::ffi::c_int != 0
            && (health as ::core::ffi::c_int) < 0x15e as ::core::ffi::c_int
        {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as ::core::ffi::c_int as uint16,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_C643(mut k: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if Enemy_MoveDown(
        k,
        ((*E).toriz_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
    ) != 0
    {
        let mut v3: int16 = (*E).toriz_var_B as int16;
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && v3 as ::core::ffi::c_int != 256 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
                as *mut uint16) = 32 as uint16;
            (*E).toriz_var_B = 256 as uint16;
        }
    } else {
        (*E).toriz_var_B =
            ((*E).toriz_var_B as ::core::ffi::c_int + 40 as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Hurt() {
    Torizo_C620(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if (*Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 160 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 144 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        Torizo_C2B0();
    };
}
#[no_mangle]
pub unsafe extern "C" fn CallTorizoFunc(mut ea: uint32, mut k: uint16) {
    match ea {
        11191979 => return,
        11191999 => {
            Torizo_Func_3(k);
            return;
        }
        11192006 => {
            Torizo_Func_4(k);
            return;
        }
        11192063 => {
            Torizo_Func_1(k);
            return;
        }
        11195842 => {
            Torizo_Func_9(k);
            return;
        }
        11195871 => {
            Torizo_Func_10(k);
            return;
        }
        11195878 => {
            Torizo_D5E6(k);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Main() {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallTorizoFunc(
        ((*E).toriz_var_E as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_2(mut k: uint16) {
    Torizo_C643(k);
    if sub_82DAF7(0x600 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_E =
            (0xaac6ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_3(mut k: uint16) {
    Torizo_C620(k);
    Torizo_C643(k);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_4(mut k: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = 78 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0x1c37 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        != addr_kPlmHeader_D6EA as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            QueueMusic_Delayed8(g_word_AAB096);
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            return;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_1(mut k: uint16) {
    Torizo_C620(k);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).toriz_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || (*E).base.health as ::core::ffi::c_int >= 0x15e as ::core::ffi::c_int
    {
        if (*E).toriz_parameter_2 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || (*E).base.health as ::core::ffi::c_int >= 0x64 as ::core::ffi::c_int
        {
            CallEnemyPreInstr(
                ((*E).toriz_var_F as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
            );
        } else {
            let mut v2: uint16 = 0;
            if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v2 = addr_kTorizo_Ilist_BD0E as ::core::ffi::c_int as uint16;
            } else {
                v2 = addr_kTorizo_Ilist_C188 as ::core::ffi::c_int as uint16;
            }
            (*E).toriz_var_00 = v2;
            (*E).base.current_instruction = addr_kTorizo_Ilist_B155 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        (*E).toriz_var_01 = (*E).base.current_instruction;
        (*E).base.current_instruction = addr_kTorizo_Ilist_B0E5 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_5(mut k: uint16) {
    let mut v3: uint16 = 0;
    let mut v8: uint16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).toriz_parameter_2 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut torizo_var_03: uint16 = (*E).toriz_var_03;
        if torizo_var_03 == 0 || {
            v8 = (torizo_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).toriz_var_03 = v8;
            (*E).toriz_var_03 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        } {
            let mut v10: uint16 =
                (abs16((*E).toriz_var_A) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v10 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v10 = 15 as uint16;
            }
            if Enemy_MoveDown(k, (v10 as int16 as int32) << 16 as ::core::ffi::c_int) == 0 {
                let mut v11: uint16 = 0;
                if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v11 = addr_off_AAC0F2 as ::core::ffi::c_int as uint16;
                } else {
                    v11 = addr_off_AABC78 as ::core::ffi::c_int as uint16;
                }
                (*E).base.current_instruction = v11;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_var_B = 256 as uint16;
                (*E).toriz_var_A = 0 as uint16;
            }
        } else {
            (*E).toriz_var_03 = 0 as uint16;
            let mut v9: uint16 = 0;
            if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v9 = addr_kTorizo_Ilist_BD0E as ::core::ffi::c_int as uint16;
            } else {
                v9 = addr_kTorizo_Ilist_C188 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v9;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v2: uint16 = (*E).toriz_var_03;
        if v2 == 0 || {
            v3 = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).toriz_var_03 = v3;
            (*E).toriz_var_03 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        } {
            let mut v5: uint16 =
                (abs16((*E).toriz_var_A) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v5 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v5 = 15 as uint16;
            }
            if Enemy_MoveDown(k, (v5 as int16 as int32) << 16 as ::core::ffi::c_int) == 0 {
                let mut v6: uint16 = 0;
                if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v6 = addr_off_AAC0F2 as ::core::ffi::c_int as uint16;
                } else {
                    v6 = addr_off_AABC78 as ::core::ffi::c_int as uint16;
                }
                (*E).base.current_instruction = v6;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_var_B = 256 as uint16;
                (*E).toriz_var_A = 0 as uint16;
            }
        } else {
            (*E).toriz_var_03 = 0 as uint16;
            let mut v4: uint16 = 0;
            if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v4 = addr_kTorizo_Ilist_B962 as ::core::ffi::c_int as uint16;
            } else {
                v4 = addr_kTorizo_Ilist_BDD8 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v4;
            (*E).base.instruction_timer = 1 as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_6(mut k: uint16) {
    Torizo_C643(k);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_7(mut k: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
    );
    EnemyFunc_C8AD(k);
    if Enemy_MoveDown(
        k,
        ((*E).toriz_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).base.current_instruction = (*E).toriz_var_00;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).toriz_var_B = 256 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = 32 as uint16;
    } else {
        (*E).toriz_var_B = ((*E).toriz_var_B as ::core::ffi::c_int
            + (*E).toriz_var_C as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Init() {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if CheckBossBitForCurArea(4 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        let mut v2: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | g_word_AAC96B[v2 as usize] as ::core::ffi::c_int)
            as uint16;
        (*E).base.extra_properties =
            ((*E).base.extra_properties as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        (*E).base.x_width = g_word_AAC96F[v2 as usize] as uint16;
        (*E).base.y_height = g_word_AAC973[v2 as usize] as uint16;
        (*E).toriz_var_E =
            (0xaac6bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
        (*E).toriz_var_F = addr_locret_AAC95E as ::core::ffi::c_int as uint16;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xaac967 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaac967 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        (*E).base.spritemap_pointer = addr_kTorizo_ExtSprmap_87D0 as ::core::ffi::c_int as uint16;
        (*E).base.x_pos = g_word_AAC95F[v2 as usize] as uint16;
        (*E).base.y_pos = g_word_AAC963[v2 as usize] as uint16;
        (*E).toriz_var_A = 0 as uint16;
        (*E).toriz_var_B = 256 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut v4: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 240 as ::core::ffi::c_int) as isize) =
                kTorizo_Palettes_1[v4 as usize];
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 176 as ::core::ffi::c_int) as isize) = kTorizo_Palette[v4 as usize];
            i -= 2 as ::core::ffi::c_int;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            Torizo_C280();
            if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 0xc0c0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 700 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9c4 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 700 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9d4 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 300 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 300 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 100 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 100 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
                    as *mut uint16) = 20 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
                    as *mut uint16) = 20 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
                    as *mut uint16) = 20 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 20 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = -(3273 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9a4 as ::core::ffi::c_int as isize)
                    as *mut uint16) = -(3273 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 4111 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9a8 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 4111 as uint16;
            }
        } else {
            Torizo_C250();
            SpawnBombTorizoHaze();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn GoldTorizo_Touch() {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy();
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Shot() {
    if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        GoldTorizo_Shot();
    } else {
        let mut E: *mut Enemy_Torizo = Get_Torizo(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        if (*E).toriz_var_04 as ::core::ffi::c_int | (*E).base.flash_timer as ::core::ffi::c_int
            == 0
        {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy();
            if (*E).base.health == 0 {
                (*E).toriz_var_E =
                    (0xaac6ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).base.current_instruction =
                    addr_kTorizo_Ilist_B1C8 as ::core::ffi::c_int as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_parameter_2 = ((*E).toriz_parameter_2 as ::core::ffi::c_int
                    | 0xc000 as ::core::ffi::c_int)
                    as uint16;
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int)
                    as uint16;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_8() {
    if *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Torizo_D658();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_39(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if (*Get_Torizo(k as ::core::ffi::c_int)).base.y_pos as ::core::ffi::c_int
        == 375 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    } else {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_41(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Torizo_C298();
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_42(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueMusic_Delayed8(5 as uint16);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).base.x_width = 18 as uint16;
    (*E).base.y_height = 48 as uint16;
    SpawnPalfxObject(addr_stru_8DF75D as ::core::ffi::c_int as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_48(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int & !(0x1000 as ::core::ffi::c_int)) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_57(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_stru_86B1C0 as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_58(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: int16 = 0;
    v2 = 34 as int16;
    while *(((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v2 as uint16 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0xb1c0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            return jp.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_59(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_06 =
        ((*E).toriz_var_06 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_62(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_06 =
        ((*E).toriz_var_06 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_63(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int & !(0x2000 as ::core::ffi::c_int)) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn GoldTorizo_Main() {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
    {
        (*E).toriz_var_07 = (*E).toriz_var_07.wrapping_add(1);
    } else {
        (*E).toriz_var_07 = 0 as uint16;
    }
    CallTorizoFunc(
        ((*E).toriz_var_E as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_56(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x34 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_60(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x67 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_46(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x48 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_12(mut k: uint16) -> uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    return ((*E).toriz_parameter_1 as ::core::ffi::c_int
        ^ (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_11(mut k: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    SetGoldenTorizoPalette((*E).base.health);
}
#[no_mangle]
pub unsafe extern "C" fn GoldTorizo_Hurt() {
    Torizo_C620(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if (*gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .flash_timer as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 160 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 144 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        Torizo_Func_11(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_47(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_GoldenTorizoSuperMissile as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_49(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        || CompareDistToSamus_X(k, 4 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
        || CompareDistToSamus_X(k, 0x28 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_1E_MoveR_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_1F_MoveL_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_7B_MoveR_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_7C_MoveL_Springball_Ground as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_09 = 0 as uint16;
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_61(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_GoldenTorizoEyeBeam as ::core::ffi::c_int as uint16,
        *jp.offset(0 as ::core::ffi::c_int as isize),
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_53(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || CompareDistToSamus_X(k, 0x20 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            == 0
        || CompareDistToSamus_X(k, 0x60 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        || NextRandom() as ::core::ffi::c_int & 0x110 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_00 =
        (jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint8)
            .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
            as ::core::ffi::c_long as uint16;
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_55(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).base.health as ::core::ffi::c_int > 0x788 as ::core::ffi::c_int
        || NextRandom() as ::core::ffi::c_int & 0x102 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    }
    (*E).toriz_var_09 = 0 as uint16;
    (*E).toriz_var_00 = (jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_52(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).base.health as ::core::ffi::c_int <= 0x2a30 as ::core::ffi::c_int
        || (*E).toriz_parameter_2 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*E).toriz_var_00 = (jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long as uint16;
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_50(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x70 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    if Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if (*E).toriz_var_07 as ::core::ffi::c_int <= 0x168 as ::core::ffi::c_int
        && (*((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x300 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || NextRandom() as ::core::ffi::c_int & 0x101 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*E).toriz_var_09 = 0 as uint16;
    Torizo_C20A(k);
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_43(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_GoldenTorizosChozoOrbs as ::core::ffi::c_int as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_51(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    if ((*E).toriz_var_09 as ::core::ffi::c_int) < 8 as ::core::ffi::c_int
        && (CompareDistToSamus_X(k, 0x20 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
            || Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    (*E).toriz_var_09 = 0 as uint16;
    Torizo_C22D(k);
    return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_45(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int)).toriz_var_00 =
        (jp.offset(2 as ::core::ffi::c_int as isize) as *mut uint8)
            .offset_from(RomBankBase((*gEnemyData(k as ::core::ffi::c_int)).bank))
            as ::core::ffi::c_long as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x20 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    } else {
        return *jp.offset(1 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Instr_54(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_A = g_word_AAD59A[(*jp.offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize] as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return addr_kTorizo_Ilist_D203 as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            return addr_kTorizo_Ilist_D2BF as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
    } else {
        EnemyFunc_C8AD(k);
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 16 as uint16;
            }
            return jp.offset(1 as ::core::ffi::c_int as isize);
        } else {
            return jp.offset(1 as ::core::ffi::c_int as isize);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_9(mut k: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        > 0x140 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            > 0x170 as ::core::ffi::c_int
    {
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_Func_10(mut k: uint16) {
    Torizo_C620(k);
    Torizo_C643(k);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D5E6(mut k: uint16) {
    Torizo_C620(k);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    CallEnemyPreInstr(
        ((*E).toriz_var_F as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D5ED(mut k: uint16) {
    Torizo_C643(k);
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D5F1(mut k: uint16) {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    let mut torizo_var_03: uint16 = (*E).toriz_var_03;
    if torizo_var_03 == 0 || {
        v3 = (torizo_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).toriz_var_03 = v3;
        (*E).toriz_var_03 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    } {
        let mut v5: uint16 =
            (abs16((*E).toriz_var_A) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if v5 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
            v5 = 15 as uint16;
        }
        if Enemy_MoveDown(k, (v5 as int16 as int32) << 16 as ::core::ffi::c_int) == 0 {
            let mut v6: uint16 = 0;
            if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v6 = addr_off_AAC0F2 as ::core::ffi::c_int as uint16;
            } else {
                v6 = addr_off_AABC78 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v6;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).toriz_var_B = 256 as uint16;
            (*E).toriz_var_A = 0 as uint16;
        }
    } else {
        (*E).toriz_var_03 = 0 as uint16;
        let mut v4: uint16 = 0;
        if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v4 = addr_kTorizo_Ilist_D203 as ::core::ffi::c_int as uint16;
        } else {
            v4 = addr_kTorizo_Ilist_D2BF as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D658() {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.flash_timer == 0 && (*E).toriz_var_04 == 0 {
        Torizo_D6A6();
    }
}
#[no_mangle]
pub unsafe extern "C" fn GoldTorizo_Shot() {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.flash_timer == 0 {
        let mut v2: uint16 = 0;
        let mut v3: uint16 = 0;
        let mut current_block_14: u64;
        if (*E).toriz_var_04 != 0 {
            Torizo_D6A6();
            return;
        }
        if (*E).toriz_parameter_2 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            current_block_14 = 17149359649428287120;
        } else {
            v2 = 0;
            v3 = 0;
            v2 = (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            v3 = (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int
                & 0xf00 as ::core::ffi::c_int) as uint16;
            (*E).toriz_var_05 = v3;
            if v3 as ::core::ffi::c_int == 256 as ::core::ffi::c_int {
                Torizo_D6D1(
                    *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                        as *mut uint16),
                    v2,
                );
                return;
            }
            if v3 as ::core::ffi::c_int != 512 as ::core::ffi::c_int {
                current_block_14 = 17149359649428287120;
            } else {
                Torizo_D6F7(
                    *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                        as *mut uint16),
                    v2,
                );
                current_block_14 = 3512920355445576850;
            }
        }
        match current_block_14 {
            17149359649428287120 => {
                (*E).toriz_parameter_2 = ((*E).toriz_parameter_2 as ::core::ffi::c_int
                    | 0x2000 as ::core::ffi::c_int)
                    as uint16;
                Torizo_D6A6();
            }
            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D6A6() {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy();
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.health == 0 {
        (*E).base.current_instruction = addr_kTorizo_Ilist_B1C8 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).toriz_parameter_2 =
            ((*E).toriz_parameter_2 as ::core::ffi::c_int | 0xc000 as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D6D1(mut k: uint16, mut j: uint16) {
    let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh1 = (*fresh1 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int);
    (*E).toriz_var_E = (0xaac6ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    let mut v3: uint16 = 0;
    if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v3 = addr_kTorizo_Ilist_D2AD as ::core::ffi::c_int as uint16;
    } else {
        v3 = addr_kTorizo_Ilist_D1F1 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v3;
}
#[no_mangle]
pub unsafe extern "C" fn Torizo_D6F7(mut k: uint16, mut j: uint16) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if Torizo_Func_12(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).toriz_parameter_2 =
            ((*E).toriz_parameter_2 as ::core::ffi::c_int | 0x1000 as ::core::ffi::c_int) as uint16;
        (*E).toriz_var_E =
            (0xaac6ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = 0;
        if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v3 = addr_kTorizo_Ilist_CEFF as ::core::ffi::c_int as uint16;
            } else {
                v3 = addr_kTorizo_Ilist_CE43 as ::core::ffi::c_int as uint16;
            }
        } else if (*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v3 = addr_kTorizo_Ilist_CEA5 as ::core::ffi::c_int as uint16;
        } else {
            v3 = addr_kTorizo_Ilist_CDE1 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v3;
    } else {
        Torizo_D6A6();
    };
}
#[no_mangle]
pub unsafe extern "C" fn TourianEntranceStatue_Init() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).palette_index = 0 as uint16;
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    let mut v1: uint16 = *(g_rom.offset(
        (((0xaad810 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaad810 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*v0).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    (*v0).current_instruction = v1;
    if (*v0).parameter_1 == 0 {
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueBaseDecoration as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueRidley as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatuePhantoon as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
    }
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v5: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 240 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
            (((0xaad785 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaad785 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v5 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 160 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
            (((0xaad765 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaad765 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v5 as isize);
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut EX: *mut Enemy_Shaktool =
        Get_Shaktool((*E).shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int);
    return Shaktool_D956(
        k,
        jp,
        ((*EX).shakt_var_D as ::core::ffi::c_int ^ 0x80 as ::core::ffi::c_int) as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut EX: *mut Enemy_Shaktool =
        Get_Shaktool((*E).shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int);
    return Shaktool_D956(k, jp, (*EX).shakt_var_D);
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    return Shaktool_D956(
        k,
        jp,
        (*((&raw mut (*E).shakt_var_A as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            ^ 0x80 as ::core::ffi::c_int) as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    return Shaktool_D956(
        k,
        jp,
        *((&raw mut (*E).shakt_var_A as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_D956(
    mut k: uint16,
    mut jp: *const uint16,
    mut a: uint16,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut v3: ::core::ffi::c_int = a as uint8 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((kSinCosTable8bit_Sext[(v3 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint32,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Func_2(mut k: uint16) {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackFrontCircle as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackMiddleCircle as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackBackCircle as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut shakt_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int)).shakt_var_F = *(g_rom.offset(
        (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(5 as ::core::ffi::c_int as isize);
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int)).shakt_var_F =
        *(g_rom.offset(
            (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(6 as ::core::ffi::c_int as isize);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DAE5(mut k: uint16) {
    if NextRandom() as ::core::ffi::c_int & 0x8431 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
            (*E).shakt_var_F =
                (0xaadcaa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.current_instruction = *(g_rom.offset(
                (((0xaadf21 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xaadf21 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
            (*E).base.instruction_timer = 1 as uint16;
            k = (k as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DB0E(mut k: uint16, mut a: uint16) {
    let mut shakt_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int))
        .shakt_parameter_1 = a;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DB27(mut k: uint16, mut a: uint16) {
    let mut shakto_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int)).shakt_var_A = a;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DB40(mut k: uint16, mut a: uint16) {
    let mut shakto_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int)).shakt_var_B = a;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DB59(mut k: uint16) {
    let mut shakto_var_E: uint16 = 0;
    let mut shakto_var_B: uint16 = 0;
    let mut v7: uint16 = 0;
    let mut v10: uint16 = 0;
    let mut x_subpos: uint16 = 0;
    let mut y_subpos: uint16 = 0;
    let mut x_pos: uint16 = 0;
    let mut y_pos: uint16 = 0;
    shakto_var_E = (*Get_Shaktool(k as ::core::ffi::c_int)).shakt_var_E;
    let mut Shaktool: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int);
    (*Shaktool).shakt_var_D =
        ((*Shaktool).shakt_var_D as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
    let mut v3: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int);
    shakto_var_B = (*v3).shakt_var_B;
    let mut v5: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int);
    (*v3).shakt_var_B = (((*v5).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v5).shakt_var_B = ((shakto_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v6: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int);
    v7 = (*v6).shakt_var_B;
    let mut v8: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int);
    (*v6).shakt_var_B = (((*v8).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v8).shakt_var_B = ((v7 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v9: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int);
    v10 = (*v9).shakt_var_B;
    let mut v11: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int);
    (*v9).shakt_var_B = (((*v11).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v11).shakt_var_B = ((v10 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v12: *mut Enemy_Shaktool =
        Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int);
    x_subpos = (*v12).base.x_subpos;
    let mut v14: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as ::core::ffi::c_int);
    (*v12).base.x_subpos = (*v14).base.x_subpos;
    (*v14).base.x_subpos = x_subpos;
    y_subpos = (*v12).base.y_subpos;
    (*v12).base.y_subpos = (*v14).base.y_subpos;
    (*v14).base.y_subpos = y_subpos;
    x_pos = (*v12).base.x_pos;
    (*v12).base.x_pos = (*v14).base.x_pos;
    (*v14).base.x_pos = x_pos;
    y_pos = (*v12).base.y_pos;
    (*v12).base.y_pos = (*v14).base.y_pos;
    (*v14).base.y_pos = y_pos;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int))
        .base
        .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int))
        .base
        .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int))
        .base
        .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
        .base
        .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int))
        .base
        .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int))
        .base
        .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int))
        .base
        .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int))
        .base
        .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
        .base
        .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int))
        .base
        .y_subpos = 0x8000 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DC07(mut k: uint16) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut v2: uint16 = 0;
    if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = ((*E).shakt_var_B as ::core::ffi::c_int - (*E).shakt_var_A as ::core::ffi::c_int)
            as uint16;
    } else {
        v2 = ((*E).shakt_var_A as ::core::ffi::c_int - (*E).shakt_var_B as ::core::ffi::c_int)
            as uint16;
    }
    (*E).shakt_var_C = (4 as ::core::ffi::c_int
        * *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DC2A(mut k: uint16) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut v3: ::core::ffi::c_int = *(&raw mut (*E).shakt_var_B as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    SetHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (((*(g_rom.offset(
            (((0xaae0bd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaae0bd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32)
            .wrapping_add(
                (*((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 91 as ::core::ffi::c_int) as isize) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 + 92 as ::core::ffi::c_int) as isize)
                        as uint32,
            ),
    );
    SetHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (((*(g_rom.offset(
            (((0xaae03d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaae03d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32)
            .wrapping_add(
                (*((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 93 as ::core::ffi::c_int) as isize) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 + 94 as ::core::ffi::c_int) as isize)
                        as uint32,
            ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DC6F(mut k: uint16) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    Shaktool_DB40(k, (*E).shakt_var_A);
    let mut shakto_var_E: uint16 = (*E).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[0 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[1 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[2 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[3 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[4 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[5 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int)).shakt_var_C =
        g_word_AADEE9[6 as ::core::ffi::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Hurt() {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).shakt_var_F as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DCAC(mut k: uint16) {
    Shaktool_DC2A(k);
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut shakto_var_C: uint16 = 0;
    if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shakt_var_A =
            ((*E).shakt_var_A as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
        shakto_var_C = 256 as uint16;
    } else {
        shakto_var_C = (*E).shakt_var_C;
    }
    (*E).shakt_var_B = ((*E).shakt_var_B as ::core::ffi::c_int
        + if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(shakto_var_C as ::core::ffi::c_int)
        } else {
            shakto_var_C as ::core::ffi::c_int
        }) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DCD7(mut k: uint16) {
    Shaktool_DCAC(k);
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut r18: uint16 =
        ((*E).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
    let mut v2: int16 = (r18 as ::core::ffi::c_int
        + (((*Get_Shaktool(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).shakt_var_B
            as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as int16;
    if (*E).shakt_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh2 = *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh2 = (*fresh2 as ::core::ffi::c_int ^ 0x80 as ::core::ffi::c_int) as uint8;
    }
    let mut v3: uint16 = (*(&raw mut v2 as *mut uint8).offset(
        (::core::mem::size_of::<int16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
        & 0xe0 as ::core::ffi::c_int) as uint16;
    *(&raw mut (*E).shakt_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
        v3 as uint8;
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xaadd15 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadd15 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((v3 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    (*E).base.instruction_timer = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_DD25(mut k: uint16) {
    let mut v7: int16 = 0;
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut x_pos: uint16 = (*E).base.x_pos;
    Shaktool_DCAC(k);
    let mut v12: uint16 = (*E).base.y_pos;
    let mut v11: uint16 = (*E).base.x_pos;
    (*E).base.x_pos = x_pos;
    (*E).base.y_pos = y_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((v11 as ::core::ffi::c_int - x_pos as ::core::ffi::c_int) as int16 as int32)
            << 16 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int
        != 0
        || {
            (*E).base.y_pos = y_pos;
            Enemy_MoveDown(
                k,
                ((v12 as ::core::ffi::c_int - y_pos as ::core::ffi::c_int) as int16 as int32)
                    << 16 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        }
    {
        if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            Shaktool_DB0E(
                k,
                (((*E).shakt_parameter_1 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
                    & 0x8fff as ::core::ffi::c_int) as uint16,
            );
        } else {
            (*E).base.x_pos = x_pos;
            (*E).base.y_pos = y_pos;
            Shaktool_DB59(k);
            Shaktool_DB0E(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                ((*E).shakt_parameter_1 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int)
                    as uint16,
            );
            Shaktool_DB0E(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0xbfff as ::core::ffi::c_int)
                    as uint16,
            );
        }
        (*E).shakt_var_D = 0 as uint16;
        let mut v5: uint16 = CalculateAngleOfEnemyXfromEnemyY(k, (*E).shakt_var_E);
        v7 = ((v5 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as int16;
        let mut v8: uint16 = 0;
        if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v8 = (v7 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int) as uint16;
        } else {
            v8 = (v7 as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int) as uint16;
        }
        Shaktool_DB27(k, v8);
        let mut i: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            Shaktool_DC07(k);
            let mut EK: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
            (*EK).shakt_var_F =
                (0xaadcaa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*EK).base.current_instruction = *(g_rom.offset(
                (((0xaadf13 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xaadf13 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
            (*EK).base.instruction_timer = 1 as uint16;
            k = (k as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        (*E).base.x_pos = v11;
        (*E).base.y_pos = v12;
        if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).shakt_var_A =
                ((*E).shakt_var_A as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
        } else {
            if ((*E).shakt_var_A as ::core::ffi::c_int ^ (*E).shakt_var_B as ::core::ffi::c_int)
                & 0xff00 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                Shaktool_DC6F(k);
                (*E).shakt_var_D = 30720 as uint16;
                Shaktool_DB0E(
                    k,
                    ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0xdfff as ::core::ffi::c_int)
                        as uint16,
                );
                (*E).shakt_var_D = ((*(&raw mut (*E).shakt_var_D as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint16;
            }
            let mut v2: uint16 = ((*E).shakt_var_D as ::core::ffi::c_int
                + (*E).shakt_var_C as ::core::ffi::c_int)
                as uint16;
            (*E).shakt_var_D = v2;
            if v2 as ::core::ffi::c_int >= 0xf000 as ::core::ffi::c_int {
                Shaktool_DB0E(
                    k,
                    ((*E).shakt_parameter_1 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int)
                        as uint16,
                );
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Init() {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).shakt_var_A = 0 as uint16;
    (*E).shakt_var_D = 0 as uint16;
    let mut shakto_parameter_2: uint16 = (*E).shakt_parameter_2;
    let mut v3: ::core::ffi::c_int =
        shakto_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | *(g_rom.offset(
            (((0xaade95 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaade95 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int) as uint16;
    (*E).shakt_var_E = (*((&raw mut g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *(g_rom.offset(
            (((0xaadea3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadea3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int) as uint16;
    (*E).shakt_var_F = *((g_rom.offset(
        (((0xaadedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(shakto_parameter_2 as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*E).shakt_var_C = (*((&raw const g_word_AADEE9 as *const uint16 as *mut uint8)
        .offset(shakto_parameter_2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *(g_rom.offset(
            (((0xaadef7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaadef7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int) as uint16;
    (*E).shakt_var_B = *(g_rom.offset(
        (((0xaadeb1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadeb1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xaadeb1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadeb1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 7 as ::core::ffi::c_int) as isize);
    (*E).base.layer = *(g_rom.offset(
        (((0xaadecd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaadecd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    if shakto_parameter_2 != 0 {
        Shaktool_DC2A(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Touch() {
    NormalEnemyTouchAi();
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Shot() {
    NormalEnemyShotAi();
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.health == 0 {
        let mut shakto_var_E: uint16 = (*E).shakt_var_E;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int))
            .base
            .properties = 512 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x1980 as ::core::ffi::c_int as isize)
        as *mut uint16) = 32 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) = 64 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
        as *mut uint16) = 722 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    CallSomeSamusCode(1 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn sub_AAE445(mut k: uint16) {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int);
    if (*v1).parameter_1 != 0 {
        (*v1).current_instruction = addr_kShaktool_Ilist_E3A7 as ::core::ffi::c_int as uint16;
        (*v1).instruction_timer = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x1c as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x4b as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut e: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    let mut arg: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    CalculateBlockContainingPixelPos(
        ((*e).base.x_pos as ::core::ffi::c_int + arg as ::core::ffi::c_int) as uint16,
        ((*e).base.y_pos as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x10002 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int
        == 0xa000 as ::core::ffi::c_int
    {
        SpawnPLM(addr_kPlmHeader_D113 as ::core::ffi::c_int as uint16);
        SpawnEprojWithRoomGfx(
            addr_kEproj_WreckedShipChozoSpikeFootsteps_1 as ::core::ffi::c_int as uint16,
            arg,
        );
    }
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
    (*E).shakt_var_C = *jp.offset(0 as ::core::ffi::c_int as isize);
    Enemy_MoveRight_IgnoreSlopes(
        k,
        (*(g_rom.offset(
            (((0xaae630 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaae630 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int16 as int32)
            << 8 as ::core::ffi::c_int,
    );
    Enemy_MoveDown(
        k,
        (abs16(
            *(g_rom.offset(
                (((0xaae630 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xaae630 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ),
        ) as int16 as int32)
            << 8 as ::core::ffi::c_int,
    );
    EnemyFunc_C8AD(k);
    let mut v6: ::core::ffi::c_int =
        (*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*(g_rom.offset(
            (((0xaae670 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaae670 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v6 as isize) as ::core::ffi::c_int
            + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16) =
        (*(g_rom.offset(
            (((0xaae6b0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xaae6b0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v6 as isize) as ::core::ffi::c_int
            + (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_Instr_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    CallSomeSamusCode(1 as uint16);
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(13 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 1 as uint16;
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x17 as uint8,
        field_1: 0x1d as uint8,
        field_2: 0xd6fc as VoidP,
    });
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn N00bTubeCracks_Init() {
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xaae2dd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xaae2dd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ChozoStatue_Init() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).properties = ((*v0).properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int)) as uint16;
    (*v0).spritemap_pointer = addr_kSpritemap_Nothing_AA as ::core::ffi::c_int as uint16;
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    (*v0).ai_preinstr = (0xaae7a6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*v0).parameter_1 = 0 as uint16;
    (*v0).palette_index = 0 as uint16;
    (*gEnemyData(0 as ::core::ffi::c_int)).layer = 0 as uint16;
    let mut parameter_2: uint16 = (*v0).parameter_2;
    (*v0).current_instruction = *(g_rom.offset(
        (((0xaae7a2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaae7a2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if parameter_2 != 0 {
        sub_AAE784();
    } else {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 160 as ::core::ffi::c_int) as isize) =
                kChozoStatue_Palettes[v3 as usize] as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 144 as ::core::ffi::c_int) as isize) =
                kChozoStatue_Palette[v3 as usize];
            i -= 2 as ::core::ffi::c_int;
        }
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x4a as uint8,
            field_1: 0x17 as uint8,
            field_2: 0xd6ee as VoidP,
        });
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x17 as uint8,
            field_1: 0x1d as uint8,
            field_2: 0xd6fc as VoidP,
        });
    };
}
#[no_mangle]
pub unsafe extern "C" fn sub_AAE784() {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) =
            kChozoStatue_Palettes3[v1 as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) =
            kChozoStatue_Palettes2[v1 as usize] as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0xc as uint8,
        field_1: 0x1d as uint8,
        field_2: 0xd6d6 as VoidP,
    });
}
#[no_mangle]
pub unsafe extern "C" fn ChozoStatue_Main() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*v0).ai_preinstr as ::core::ffi::c_int | 0xaa0000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Shaktool_PreInstr_0(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int);
        if (*E).shakt_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kShaktool_Ilist_E461 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).shakt_var_A = -(256 as ::core::ffi::c_int) as uint16;
            (*E).shakt_var_B = 256 as uint16;
        }
    }
}
