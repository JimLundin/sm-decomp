use crate::game::sm_80::{CheckBossBitForCurArea, Multiply16x16};
use crate::game::sm_84::{SpawnPLM, SpawnPLMWithCollision};
use crate::game::sm_90::KillProjectile;
use crate::game::sm_a0::CalculateAngleFromXY;
use crate::game_state::Game;
use crate::rtl::{Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};

pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, DoorDef, Func_V,
    OamEnt, SamusPose, VoidP,
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
pub use crate::types::Consts_94;
pub const addr_word_94B197: Consts_94 = 45463;
pub const addr_word_94B193: Consts_94 = 45459;
pub const addr_word_94B18F: Consts_94 = 45455;
pub const addr_word_94B18B: Consts_94 = 45451;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CollInfo {
    pub ci_r18_r20: int32,
    pub ci_r24: uint16,
    pub ci_r26: uint16,
    pub ci_r28: uint16,
    pub ci_r30: uint16,
    pub ci_r32: uint16,
    pub ci_r38: uint16,
    pub ci_r40: uint16,
}
pub type Func_CollInfo_U8 = unsafe fn(*mut CollInfo, &mut Game) -> uint8;
pub use crate::types::Pair_Bool_Amt;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PostGrappleCollInfo {
    pub pgci_r26: uint16,
    pub pgci_r28: uint16,
    pub pgci_r32: uint16,
}
pub type Func_PostGrappleCollInfo_U16 = unsafe fn(*mut PostGrappleCollInfo, &mut Game) -> uint16;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
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
unsafe fn RomPtr_8F(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_94(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x940000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gOamEnt(mut v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    return (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe fn get_DoorDef(mut a: uint16, game: &mut Game) -> *mut DoorDef {
    return RomPtr(
        (0x830000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut DoorDef;
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
pub const fnDoorTransitionFunction_HandleElevator: ::core::ffi::c_int =
    0x82e17d as ::core::ffi::c_int;
pub const fnGrappleInstr_Goto: uint32 = 9744628 as uint32;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5 as ::core::ffi::c_int;

static kAlignPos_Tab1: [uint8; 512] = [
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
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
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
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
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
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
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
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
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
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
static kAlignYPos_Tab0: [uint8; 512] = [
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
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
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
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
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
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
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    14 as ::core::ffi::c_int as uint8,
    11 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    16 as ::core::ffi::c_int as uint8,
    13 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
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
static kTab948E54: [uint8; 20] = [
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
];
static kBlockColl_Horiz_Slope_NonSquare_Tab: [uint16; 64] = [
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x1000 as ::core::ffi::c_int as uint16,
    0xb0 as ::core::ffi::c_int as uint16,
    0x1000 as ::core::ffi::c_int as uint16,
    0xb0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x1000 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x1000 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0x1000 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0x800 as ::core::ffi::c_int as uint16,
    0xd8 as ::core::ffi::c_int as uint16,
    0x800 as ::core::ffi::c_int as uint16,
    0xd8 as ::core::ffi::c_int as uint16,
    0x600 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0x600 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0x600 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0x4000 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0x4000 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0x6000 as ::core::ffi::c_int as uint16,
    0x50 as ::core::ffi::c_int as uint16,
    0x6000 as ::core::ffi::c_int as uint16,
    0x50 as ::core::ffi::c_int as uint16,
    0x6000 as ::core::ffi::c_int as uint16,
    0x50 as ::core::ffi::c_int as uint16,
];
unsafe fn PostGrappleColl_Horiz_Slope_NonSquare(
    pgci: *mut PostGrappleCollInfo,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    let negative = game.samus_collision_direction() & 1 != 0;
    let x = game.samus_x_pos();
    let block = game.cur_block_index();
    let width = game.room_width_in_blocks() as u8;
    let column = if width == 0 {
        block
    } else {
        block % u16::from(width)
    };
    if x >> 4 != column {
        return 0xffff;
    }
    // k selects the slope bytes even when the live collision index differs.
    let bts = game.g_ram[0x16402 + usize::from(k)];
    if (bts & 0x40 != 0) == negative {
        return PostGrappleColl_Horiz_Solid(pgci, game);
    }
    let y = game.samus_y_pos();
    let y = if bts & 0x80 != 0 { y ^ 15 } else { y };
    let index = usize::from(bts & 0x1f) * 16 + usize::from(y & 15);
    let boundary = i16::from(kAlignPos_Tab1[index] & 0x1f);
    let difference = boundary - ((*pgci).pgci_r32 & 15) as i16 - 1;
    if negative {
        if difference <= 0 {
            !(difference as u16)
        } else {
            0xffff
        }
    } else if difference >= 0 {
        difference as u16
    } else {
        0xffff
    }
}
unsafe fn PostGrappleColl_Vert_Slope_NonSquare(
    mut pgci: *mut PostGrappleCollInfo,
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        let mut mod_0: uint16 = SnesModulus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int
            != mod_0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
        let mut temp_collision_DD4: uint16 =
            ((*pgci).pgci_r32 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            return PostGrappleColl_Vert_Solid(pgci, game);
        }
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            ^ (if *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(k as isize) as ::core::ffi::c_int
                & 0x40 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                0xf as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })) as uint16;
        let mut v5: uint16 = (16 as ::core::ffi::c_int
            * (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(k as isize) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int)
            + (v4 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
            as uint16;
        let mut result: uint16 = ((kAlignYPos_Tab0[v5 as usize] as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            - temp_collision_DD4 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        return (if result as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            result as ::core::ffi::c_int
        } else {
            -(1 as ::core::ffi::c_int)
        }) as uint16;
    }
    let mut mod_1: uint16 = SnesModulus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        != mod_1 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int) as uint16;
    }
    let mut temp_collision_DD4_0: uint16 =
        ((*pgci).pgci_r32 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        return PostGrappleColl_Vert_Solid(pgci, game);
    }
    let mut v8: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        ^ (if *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x40 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            0xf as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as uint16;
    let mut v9: uint16 = (16 as ::core::ffi::c_int
        * (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
        + (v8 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
        as uint16;
    let mut v10: uint16 = ((kAlignYPos_Tab0[v9 as usize] as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        - temp_collision_DD4_0 as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as uint16;
    return (if v10 as int16 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
        !(v10 as ::core::ffi::c_int)
    } else {
        -(1 as ::core::ffi::c_int)
    }) as uint16;
}
unsafe fn PostGrappleColl_Horiz_Slope_Square(
    mut pgci: *mut PostGrappleCollInfo,
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    let mut current_block: u64;
    let mut temp_collision_DD4: uint16 = (4 as ::core::ffi::c_int
        * (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (temp_collision_DD4 as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*pgci).pgci_r32 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int)) as uint16;
    if (*pgci).pgci_r26 == 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if kTab948E54[v1 as usize] == 0 {
                return -(1 as ::core::ffi::c_int) as uint16;
            }
            current_block = 11148800648680365515;
        } else {
            current_block = 16129070931773799691;
        }
    } else if (*pgci).pgci_r26 as ::core::ffi::c_int != (*pgci).pgci_r28 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        current_block = 16129070931773799691;
    } else {
        current_block = 10886091980245723256;
    }
    match current_block {
        16129070931773799691 => {
            if kTab948E54[v1 as usize] != 0 {
                current_block = 11148800648680365515;
            } else {
                current_block = 10886091980245723256;
            }
        }
        _ => {}
    }
    match current_block {
        10886091980245723256 => {
            if kTab948E54[(v1 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize] == 0 {
                return -(1 as ::core::ffi::c_int) as uint16;
            }
        }
        _ => {}
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        return ((*pgci).pgci_r32 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint16;
    } else {
        return ((*pgci).pgci_r32 as ::core::ffi::c_int & 7 as ::core::ffi::c_int
            ^ 7 as ::core::ffi::c_int) as uint16;
    };
}
unsafe fn PostGrappleColl_Vertical_Slope_Square(
    mut pgci: *mut PostGrappleCollInfo,
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    let mut current_block: u64;
    let mut temp_collision_DD4: uint16 = (4 as ::core::ffi::c_int
        * (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (temp_collision_DD4 as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*pgci).pgci_r32 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int)) as uint16;
    if (*pgci).pgci_r26 == 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if kTab948E54[v1 as usize] == 0 {
                return -(1 as ::core::ffi::c_int) as uint16;
            }
            current_block = 856425524694780040;
        } else {
            current_block = 15830000096881139597;
        }
    } else if (*pgci).pgci_r26 as ::core::ffi::c_int != (*pgci).pgci_r28 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        current_block = 15830000096881139597;
    } else {
        current_block = 10886091980245723256;
    }
    match current_block {
        15830000096881139597 => {
            if kTab948E54[v1 as usize] != 0 {
                current_block = 856425524694780040;
            } else {
                current_block = 10886091980245723256;
            }
        }
        _ => {}
    }
    match current_block {
        10886091980245723256 => {
            if kTab948E54[(v1 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize] == 0 {
                return -(1 as ::core::ffi::c_int) as uint16;
            }
        }
        _ => {}
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        return ((*pgci).pgci_r32 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint16;
    } else {
        return ((*pgci).pgci_r32 as ::core::ffi::c_int & 7 as ::core::ffi::c_int
            ^ 7 as ::core::ffi::c_int) as uint16;
    };
}
unsafe fn PostGrappleColl_Horiz_Slope(pgci: *mut PostGrappleCollInfo, game: &mut Game) -> uint16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Horiz_Slope_Square(pgci, block, game)
    } else {
        PostGrappleColl_Horiz_Slope_NonSquare(pgci, block, game)
    }
}
unsafe fn PostGrappleColl_Horiz_Solid(
    mut pgci: *mut PostGrappleCollInfo,
    game: &mut Game,
) -> uint16 {
    return ((*pgci).pgci_r32 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
}
unsafe fn PostGrappleColl_Vert_Slope(pgci: *mut PostGrappleCollInfo, game: &mut Game) -> uint16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Vertical_Slope_Square(pgci, block, game)
    } else {
        PostGrappleColl_Vert_Slope_NonSquare(pgci, block, game)
    }
}
unsafe fn PostGrappleColl_Vert_Solid(
    mut pgci: *mut PostGrappleCollInfo,
    game: &mut Game,
) -> uint16 {
    return ((*pgci).pgci_r32 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
}
unsafe fn PostGrappleColl_Horiz(
    pgci: *mut PostGrappleCollInfo,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    game.set_cur_block_index(k >> 1);
    match game.current_level_block() >> 12 {
        1 => PostGrappleColl_Horiz_Slope(pgci, game),
        8..=15 => PostGrappleColl_Horiz_Solid(pgci, game),
        _ => 0xffff,
    }
}
unsafe fn PostGrappleColl_Vert(
    pgci: *mut PostGrappleCollInfo,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    game.set_cur_block_index(k >> 1);
    match game.current_level_block() >> 12 {
        1 => PostGrappleColl_Vert_Slope(pgci, game),
        8..=15 => PostGrappleColl_Vert_Solid(pgci, game),
        _ => 0xffff,
    }
}
unsafe fn PostGrappleCollisionDetect_Right(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe04 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut r28: uint16 = Samus_GetYposSpan(game);
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut R32: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafe as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as uint16;
    v0 = (R32 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut pgci: PostGrappleCollInfo = PostGrappleCollInfo {
        pgci_r26: r28,
        pgci_r28: r28,
        pgci_r32: R32,
    };
    loop {
        v2 = PostGrappleColl_Horiz(&raw mut pgci, v1, game) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut v3: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v3 as ::core::ffi::c_int
                >= *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xe04 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xe04 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v3;
            }
        }
        v1 = (v1 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        pgci.pgci_r26 = pgci.pgci_r26.wrapping_sub(1);
        if !(pgci.pgci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
unsafe fn PostGrappleCollisionDetect_Left(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe06 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut r28: uint16 = Samus_GetYposSpan(game);
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut R32: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    v0 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut pgci: PostGrappleCollInfo = PostGrappleCollInfo {
        pgci_r26: r28,
        pgci_r28: r28,
        pgci_r32: R32,
    };
    loop {
        v2 = PostGrappleColl_Horiz(&raw mut pgci, v1, game) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut v3: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v3 as ::core::ffi::c_int
                >= *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xe06 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xe06 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v3;
            }
        }
        v1 = (v1 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        pgci.pgci_r26 = pgci.pgci_r26.wrapping_sub(1);
        if !(pgci.pgci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
unsafe fn PostGrappleCollisionDetect_Down(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut r28: uint16 = Samus_GetXposSpan(game);
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    let mut R32: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb00 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as uint16;
    let mut prod: uint16 = Mult8x8(
        (R32 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    v0 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut pgci: PostGrappleCollInfo = PostGrappleCollInfo {
        pgci_r26: r28,
        pgci_r28: r28,
        pgci_r32: R32,
    };
    loop {
        v2 = PostGrappleColl_Vert(&raw mut pgci, v1, game) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut v3: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v3 as ::core::ffi::c_int
                >= *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xe08 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v3;
            }
        }
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        pgci.pgci_r26 = pgci.pgci_r26.wrapping_sub(1);
        if !(pgci.pgci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
unsafe fn PostGrappleCollisionDetect_Up(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe0a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut r28: uint16 = Samus_GetXposSpan(game);
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    let mut R32: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    v0 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut pgci: PostGrappleCollInfo = PostGrappleCollInfo {
        pgci_r26: r28,
        pgci_r28: r28,
        pgci_r32: R32,
    };
    loop {
        v2 = PostGrappleColl_Vert(&raw mut pgci, v1, game) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut v3: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v3 as ::core::ffi::c_int
                >= *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xe0a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xe0a as ::core::ffi::c_int as isize)
                    as *mut uint16) = v3;
            }
        }
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        pgci.pgci_r26 = pgci.pgci_r26.wrapping_sub(1);
        if !(pgci.pgci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn PostGrappleCollisionDetect_X(game: &mut Game) {
    PostGrappleCollisionDetect_Right(game);
    PostGrappleCollisionDetect_Left(game);
}
pub unsafe fn PostGrappleCollisionDetect_Y(game: &mut Game) {
    PostGrappleCollisionDetect_Down(game);
    PostGrappleCollisionDetect_Up(game);
}
unsafe fn BlockColl_Horiz_Slope_NonSquare(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_slope_bts();
    if bts & 0x80 != 0 {
        return 0;
    }
    let y_speed = u32::from_le_bytes([
        game.g_ram[0xb2c],
        game.g_ram[0xb2d],
        game.g_ram[0xb2e],
        game.g_ram[0xb2f],
    ]);
    if y_speed != 0 {
        return 0;
    }
    let displacement = (*ci).ci_r18_r20;
    // Keep the middle sixteen bits of the arithmetic right shift.
    let shifted = (displacement >> 8) as u16;
    let factor = u32::from(kBlockColl_Horiz_Slope_NonSquare_Tab[usize::from(bts & 0x1f) * 2 + 1]);
    (*ci).ci_r18_r20 = if displacement >= 0 {
        (u32::from(shifted) * factor) as i32
    } else {
        ((u32::from(shifted.wrapping_neg()) * factor) as i32).wrapping_neg()
    };
    0
}
unsafe fn BlockColl_Vert_Slope_NonSquare(ci: *mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    let direction = game.samus_collision_direction();
    let upward = direction & 1 != 0;
    let x_position = game.samus_x_pos();
    // The SNES division register receives only the low byte of room width.
    let width = game.room_width_in_blocks() as u8;
    let block = game.cur_block_index();
    let column = if width == 0 {
        block
    } else {
        block % u16::from(width)
    };
    if x_position >> 4 != column {
        return 0;
    }
    let bts = game.g_ram[0x16402 + usize::from(k)];
    if (bts & 0x80 != 0) == upward {
        return 0;
    }
    let x_offset = if bts & 0x40 != 0 {
        x_position ^ 15
    } else {
        x_position
    };
    let index = usize::from(bts & 0x1f) * 16 + usize::from(x_offset & 15);
    let height = kAlignYPos_Tab0[index] & 0x1f;
    let radius = game.samus_y_radius();
    let y_offset = if upward {
        radius.wrapping_add((*ci).ci_r24).wrapping_sub(1) & 15
    } else {
        ((*ci).ci_r24.wrapping_sub(radius) & 15) ^ 15
    };
    let adjustment = (height as i16) - (y_offset as i16) - 1;
    if adjustment > 0 {
        return 0;
    }
    let displacement = (((*ci).ci_r18_r20 >> 16) as i16)
        .wrapping_add(adjustment)
        .max(0);
    (*ci).ci_r18_r20 = i32::from(displacement) << 16;
    1
}
pub unsafe fn Samus_AlignYPosSlope(game: &mut Game) {
    let movement_flags = u16::from_le_bytes([game.g_ram[0xa46], game.g_ram[0xa47]]);
    if movement_flags & 2 == 0 {
        return;
    }
    let x = game.samus_x_pos();
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    CalculateBlockAt(x, radius.wrapping_add(y).wrapping_sub(1), 0, 0, game);
    if game.current_level_block() & 0xf000 == 0x1000 && game.current_block_bts() & 0x1f >= 5 {
        let radius = game.samus_y_radius();
        let y = game.samus_y_pos();
        let offset = radius.wrapping_add(y).wrapping_sub(1) & 15;
        let bts = game.current_block_bts();
        if bts & 0x80 == 0 {
            let x = game.samus_x_pos();
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(kAlignYPos_Tab0[index] & 0x1f) - offset as i16 - 1;
            if adjustment < 0 {
                game.g_ram[0xafa..0xafc]
                    .copy_from_slice(&y.wrapping_add(adjustment as u16).to_le_bytes());
                game.set_samus_pos_adjusted_by_slope_flag(1);
            }
        }
    }
    // Re-read the position after the bottom-edge alignment before looking up the top block.
    let x = game.samus_x_pos();
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    CalculateBlockAt(x, y.wrapping_sub(radius), 0, 0, game);
    if game.current_level_block() & 0xf000 == 0x1000 && game.current_block_bts() & 0x1f >= 5 {
        let y = game.samus_y_pos();
        let radius = game.samus_y_radius();
        let offset = (y.wrapping_sub(radius) & 15) ^ 15;
        let bts = game.current_block_bts();
        if bts & 0x80 != 0 {
            let x = game.samus_x_pos();
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(kAlignYPos_Tab0[index] & 0x1f) - offset as i16 - 1;
            if adjustment <= 0 {
                game.g_ram[0xafa..0xafc]
                    .copy_from_slice(&y.wrapping_sub(adjustment as u16).to_le_bytes());
                game.set_samus_pos_adjusted_by_slope_flag(1);
            }
        }
    }
}
fn square_slope_occupied(
    index: u16,
    first: u16,
    last: u16,
    position: u16,
    radius: u16,
    other: u16,
) -> bool {
    if first == 0 && radius.wrapping_add(position).wrapping_sub(1) & 8 == 0 {
        return kTab948E54[usize::from(index)] != 0;
    }
    if first != 0 && first == last && position.wrapping_sub(radius) & 8 != 0 {
        return kTab948E54[usize::from(index ^ other)] != 0;
    }
    kTab948E54[usize::from(index)] != 0 || kTab948E54[usize::from(index ^ other)] != 0
}
fn square_slope_delta(edge: u16, radius: u16, position: u16, negative: bool) -> i32 {
    // Truncate to signed sixteen bits before clamping at the eight-pixel boundary.
    let delta = if negative {
        radius
            .wrapping_add(edge | 7)
            .wrapping_add(1)
            .wrapping_sub(position) as i16
    } else {
        (edge & 0xfff8).wrapping_sub(radius).wrapping_sub(position) as i16
    };
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
unsafe fn BlockColl_Horiz_Slope_Square(
    ci: *mut CollInfo,
    slope: uint16,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    let orientation = u16::from(game.g_ram[0x16402 + usize::from(k)] >> 6);
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ (((*ci).ci_r32 & 8) >> 3));
    let perpendicular_position = game.samus_y_pos();
    let perpendicular_radius = game.samus_y_radius();
    if !square_slope_occupied(
        index,
        (*ci).ci_r26,
        (*ci).ci_r28,
        perpendicular_position,
        perpendicular_radius,
        2,
    ) {
        return 0;
    }
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_x_radius();
    let position = game.samus_x_pos();
    (*ci).ci_r18_r20 = square_slope_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_x_subpos(subposition);
    1
}
unsafe fn BlockColl_Vert_Slope_Square(
    ci: *mut CollInfo,
    slope: uint16,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    let orientation = u16::from(game.g_ram[0x16402 + usize::from(k)] >> 6);
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ (((*ci).ci_r32 & 8) >> 2));
    let perpendicular_position = game.samus_x_pos();
    let perpendicular_radius = game.samus_x_radius();
    if !square_slope_occupied(
        index,
        (*ci).ci_r26,
        (*ci).ci_r28,
        perpendicular_position,
        perpendicular_radius,
        1,
    ) {
        return 0;
    }
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_y_radius();
    let position = game.samus_y_pos();
    (*ci).ci_r18_r20 = square_slope_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_y_subpos(subposition);
    if !negative {
        game.set_samus_pos_adjusted_by_slope_flag(1);
    }
    1
}
fn apply_spike_block_damage(amount: u16, game: &mut Game) {
    let invincibility = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
    if invincibility != 0 {
        return;
    }
    game.g_ram[0x18a8..0x18aa].copy_from_slice(&60u16.to_le_bytes());
    game.g_ram[0x18aa..0x18ac].copy_from_slice(&10u16.to_le_bytes());
    let damage = u16::from_le_bytes([game.g_ram[0xa50], game.g_ram[0xa51]]);
    game.g_ram[0xa50..0xa52].copy_from_slice(&damage.wrapping_add(amount).to_le_bytes());
    // Spike blocks read only the facing byte, unlike the spike-air reaction.
    let knockback = u16::from((game.g_ram[0xa1e] ^ 12) & 8 != 0);
    game.g_ram[0xa54..0xa56].copy_from_slice(&knockback.to_le_bytes());
}
fn ClearCarry_4(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
// The boundary difference wraps to signed sixteen bits before the direction clamp.
fn solid_collision_delta(edge: u16, radius: u16, position: u16, negative: bool) -> i32 {
    let delta = if negative {
        radius
            .wrapping_add(edge | 15)
            .wrapping_add(1)
            .wrapping_sub(position) as i16
    } else {
        (edge & 0xfff0).wrapping_sub(radius).wrapping_sub(position) as i16
    };
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
// Keep the context raw: PLM callbacks can also access its active displacement.
unsafe fn BlockColl_Horiz_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_x_radius();
    let position = game.samus_x_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_x_subpos(subposition);
    1
}
unsafe fn BlockColl_Vert_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_y_radius();
    let position = game.samus_y_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_y_subpos(subposition);
    1
}
unsafe fn BlockColl_Horiz_Slope(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let slope = u16::from(game.current_block_bts() & 0x1f);
    if slope < 5 {
        return BlockColl_Horiz_Slope_Square(ci, slope, game.cur_block_index(), game);
    }
    game.set_current_slope_bts(u16::from(game.current_block_bts()));
    BlockColl_Horiz_Slope_NonSquare(ci, game)
}
unsafe fn BlockColl_Vert_Slope(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let slope = u16::from(game.current_block_bts() & 0x1f);
    if slope < 5 {
        return BlockColl_Vert_Slope_Square(ci, slope, game.cur_block_index(), game);
    }
    game.set_current_slope_bts(u16::from(game.current_block_bts()));
    BlockColl_Vert_Slope_NonSquare(ci, game.cur_block_index(), game)
}
fn ClearCarry_5(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
fn BlockColl_Vert_SpikeAir(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
unsafe fn react_to_spike_block(game: &mut Game) {
    let bts = game.current_block_bts();
    // Preserve the former sixteen-entry table's panic before solid collision.
    assert!(bts < 16, "spike-block BTS index out of bounds: {bts}");
    match bts {
        0 => {
            let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
            // Check the boss before reading the live invincibility timer.
            if area != 3 || CheckBossBitForCurArea(1, game) & 1 != 0 {
                apply_spike_block_damage(60, game);
            }
        }
        1 | 3 => apply_spike_block_damage(16, game),
        _ => {}
    }
}
unsafe fn BlockColl_Horiz_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    react_to_spike_block(game);
    BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
}
unsafe fn BlockColl_Vert_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    react_to_spike_block(game);
    BlockColl_Vert_SolidShootGrappleBlock(ci, game)
}
// High-bit BTS values select an area-specific table in the live ROM.
unsafe fn special_collision_plm(bts: u8, area_tables: u32, game: &mut Game) -> u16 {
    if bts & 0x80 != 0 {
        let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
        let table = game.read_rom_word(area_tables, usize::from(area) * 2);
        if table & 0x8000 == 0 {
            // Retain the legacy invalid-window diagnostic and g_fail write.
            RomPtr_94(table, game);
        }
        // The legacy mapper ignores bit 15 after diagnosing an invalid window.
        game.read_rom_word(0x948000 | u32::from(table), usize::from(bts & 0x7f) * 2)
    } else {
        game.read_rom_word(0x949139, usize::from(bts) * 2)
    }
}
unsafe fn BlockColl_Horiz_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492d9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    if bts & 0x80 != 0 {
        result & 1
    } else {
        0
    }
}
unsafe fn BlockColl_Vert_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockColl_Horiz_SpecialAir(ci, game)
}
unsafe fn BlockColl_Horiz_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492e9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        // Setup can adjust displacement or Samus RAM; collision reads remain live.
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Vert_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492e9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Horiz_BombableAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 == 0 {
        let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
        SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    }
    0
}
unsafe fn BlockColl_Vert_BombableAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockColl_Horiz_BombableAir(ci, game)
}
unsafe fn BlockColl_Horiz_BombBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        return BlockColl_Horiz_SolidShootGrappleBlock(ci, game);
    }
    let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        // Setup may adjust the displacement and Samus RAM before collision.
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Vert_BombBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        return BlockColl_Vert_SolidShootGrappleBlock(ci, game);
    }
    let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
pub unsafe fn BlockColl_Horiz_Door(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    game.g_ram[0x99c..0x99e].copy_from_slice(&0xe17du16.to_le_bytes());
    let bts = game.current_block_bts();
    let door_list = u16::from_le_bytes([game.g_ram[0x7b5], game.g_ram[0x7b6]]);
    let entry = door_list.wrapping_add(u16::from(bts & 0x7f) * 2);
    if entry & 0x8000 == 0 {
        RomPtr_8F(entry, game);
    }
    let door = game.read_rom_word(0x8f8000 | u32::from(entry), 0);
    if door & 0x8000 == 0 {
        get_DoorDef(door, game);
    }
    let destination = game.read_rom_word(0x838000 | u32::from(door), 0);
    if destination & 0x8000 == 0 {
        // Preserve the original pose comparison, even though its threshold is a state value.
        let pose = u16::from_le_bytes([game.g_ram[0xa1c], game.g_ram[0xa1d]]);
        if pose < 9 {
            game.g_ram[0xe16..0xe18].copy_from_slice(&1u16.to_le_bytes());
        }
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        game.g_ram[0x78d..0x78f].copy_from_slice(&door.to_le_bytes());
        game.g_ram[0x998..0x99a].copy_from_slice(&9u16.to_le_bytes());
        0
    }
}
pub unsafe fn BlockColl_Vert_Door(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    game.g_ram[0x99c..0x99e].copy_from_slice(&0xe17du16.to_le_bytes());
    let bts = game.current_block_bts();
    let door_list = u16::from_le_bytes([game.g_ram[0x7b5], game.g_ram[0x7b6]]);
    let entry = door_list.wrapping_add(u16::from(bts & 0x7f) * 2);
    if entry & 0x8000 == 0 {
        RomPtr_8F(entry, game);
    }
    let door = game.read_rom_word(0x8f8000 | u32::from(entry), 0);
    if door & 0x8000 == 0 {
        get_DoorDef(door, game);
    }
    let destination = game.read_rom_word(0x838000 | u32::from(door), 0);
    if destination & 0x8000 == 0 {
        // Preserve the original pose comparison, even though its threshold is a state value.
        let pose = u16::from_le_bytes([game.g_ram[0xa1c], game.g_ram[0xa1d]]);
        if pose < 9 {
            game.g_ram[0xe16..0xe18].copy_from_slice(&1u16.to_le_bytes());
        }
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        game.g_ram[0x78d..0x78f].copy_from_slice(&door.to_le_bytes());
        game.g_ram[0x998..0x99a].copy_from_slice(&9u16.to_le_bytes());
        0
    }
}
fn BlockReact_HorizExt(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let offset = i16::from(game.current_block_bts() as i8);
    if offset == 0 {
        return 0;
    }
    game.set_cur_block_index(game.cur_block_index().wrapping_add(offset as u16));
    0xff
}
fn BlockReact_VertExt(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let offset = i16::from(game.current_block_bts() as i8);
    if offset == 0 {
        return 0;
    }
    let block_offset = (offset as u16).wrapping_mul(game.room_width_in_blocks());
    game.set_cur_block_index(game.cur_block_index().wrapping_add(block_offset));
    0xff
}
fn samus_block_span(position: u16, radius: u16) -> u16 {
    let first_block_edge = position.wrapping_sub(radius) & 0xfff0;
    radius
        .wrapping_add(position)
        .wrapping_sub(1)
        .wrapping_sub(first_block_edge)
        >> 4
}
fn Samus_GetYposSpan(game: &mut Game) -> uint16 {
    let position = game.samus_y_pos();
    let radius = game.samus_y_radius();
    samus_block_span(position, radius)
}
fn Samus_GetXposSpan(game: &mut Game) -> uint16 {
    let position = game.samus_x_pos();
    let radius = game.samus_x_radius();
    samus_block_span(position, radius)
}
unsafe fn BlockColl_Horiz_CheckColl(ci: &mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    game.set_cur_block_index(k >> 1);
    // Keep the raw context alive across retries and nested PLM setup calls.
    let context = ci as *mut CollInfo;
    let result = loop {
        let result = match game.current_level_block() >> 12 {
            0 | 4 | 6 => ClearCarry_4(context, game),
            1 => BlockColl_Horiz_Slope(context, game),
            2 => ClearCarry_5(context, game),
            3 => BlockColl_Horiz_SpecialAir(context, game),
            5 => BlockReact_HorizExt(context, game),
            7 => BlockColl_Horiz_BombableAir(context, game),
            8 | 12 | 14 => BlockColl_Horiz_SolidShootGrappleBlock(context, game),
            9 => BlockColl_Horiz_Door(context, game),
            10 => BlockColl_Horiz_SpikeBlock(context, game),
            11 => BlockColl_Horiz_SpecialBlock(context, game),
            13 => BlockReact_VertExt(context, game),
            15 => BlockColl_Horiz_BombBlock(context, game),
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            break result;
        }
    };
    result
}
unsafe fn BlockColl_Vert_CheckColl(ci: &mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    game.set_cur_block_index(k >> 1);
    // Keep the same raw context valid across every extension-block retry.
    let context = ci as *mut CollInfo;
    let result = loop {
        let result = match game.current_level_block() >> 12 {
            0 | 4 | 6 => ClearCarry_4(context, game),
            1 => BlockColl_Vert_Slope(context, game),
            2 => BlockColl_Vert_SpikeAir(context, game),
            3 => BlockColl_Vert_SpecialAir(context, game),
            5 => BlockReact_HorizExt(context, game),
            7 => BlockColl_Vert_BombableAir(context, game),
            8 | 12 | 14 => BlockColl_Vert_SolidShootGrappleBlock(context, game),
            9 => BlockColl_Vert_Door(context, game),
            10 => BlockColl_Vert_SpikeBlock(context, game),
            11 => BlockColl_Vert_SpecialBlock(context, game),
            13 => BlockReact_VertExt(context, game),
            15 => BlockColl_Vert_BombBlock(context, game),
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            break result;
        }
    };
    result
}
unsafe fn BlockColl_Handle_Horiz(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    let span = Samus_GetYposSpan(game);
    let y = game.samus_y_pos();
    let y_radius = game.samus_y_radius();
    let row = (y.wrapping_sub(y_radius) >> 4) as u8;
    let width = game.room_width_in_blocks() as u8;
    let row_start = u16::from(row) * u16::from(width);
    let x = game.samus_x_pos();
    let fraction = game.samus_x_subpos();
    let target =
        (((u32::from(x) << 16) | u32::from(fraction)).wrapping_add(amt as u32) >> 16) as u16;
    let radius = game.samus_x_radius();
    let edge = if amt >= 0 {
        radius.wrapping_add(target).wrapping_sub(1)
    } else {
        target.wrapping_sub(radius)
    };
    let mut block_offset = row_start.wrapping_add(edge >> 4).wrapping_mul(2);
    let mut context = CollInfo {
        ci_r18_r20: amt,
        ci_r24: target,
        ci_r26: span,
        ci_r28: span,
        ci_r30: 0,
        ci_r32: edge,
        ci_r38: 0,
        ci_r40: 0,
    };
    while BlockColl_Horiz_CheckColl(&mut context, block_offset, game) & 1 == 0 {
        // Callbacks can change the room width and remaining row count.
        block_offset = block_offset.wrapping_add(game.room_width_in_blocks().wrapping_mul(2));
        context.ci_r26 = context.ci_r26.wrapping_sub(1);
        if context.ci_r26 & 0x8000 != 0 {
            return Pair_Bool_Amt {
                flag: false,
                amt: context.ci_r18_r20,
            };
        }
    }
    Pair_Bool_Amt {
        flag: true,
        amt: context.ci_r18_r20,
    }
}
unsafe fn BlockColl_Handle_Vert_LeftToRight(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    handle_vertical_collision(amt, false, game)
}
unsafe fn BlockColl_Handle_Vert_RightToLeft(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    handle_vertical_collision(amt, true, game)
}
unsafe fn handle_vertical_collision(
    amt: i32,
    right_to_left: bool,
    game: &mut Game,
) -> Pair_Bool_Amt {
    let span = Samus_GetXposSpan(game);
    let y = game.samus_y_pos();
    let fraction = game.samus_y_subpos();
    let target =
        (((u32::from(y) << 16) | u32::from(fraction)).wrapping_add(amt as u32) >> 16) as u16;
    let radius = game.samus_y_radius();
    let edge = if amt >= 0 {
        radius.wrapping_add(target).wrapping_sub(1)
    } else {
        target.wrapping_sub(radius)
    };
    let row = (edge >> 4) as u8;
    let width = game.room_width_in_blocks() as u8;
    let row_start = u16::from(row) * u16::from(width);
    let x = game.samus_x_pos();
    let radius = game.samus_x_radius();
    let column = if right_to_left {
        radius.wrapping_add(x).wrapping_sub(1) >> 4
    } else {
        x.wrapping_sub(radius) >> 4
    };
    game.set_cur_block_index(row_start.wrapping_add(column));
    let mut context = CollInfo {
        ci_r18_r20: amt,
        ci_r24: target,
        ci_r26: if right_to_left { 0 } else { span },
        ci_r28: span,
        ci_r30: 0,
        ci_r32: edge,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut block_offset = game.cur_block_index().wrapping_mul(2);
    while BlockColl_Vert_CheckColl(&mut context, block_offset, game) & 1 == 0 {
        if right_to_left {
            context.ci_r26 = context.ci_r26.wrapping_add(1);
            if context.ci_r28 < context.ci_r26 {
                return Pair_Bool_Amt {
                    flag: false,
                    amt: context.ci_r18_r20,
                };
            }
            block_offset = block_offset.wrapping_sub(2);
        } else {
            context.ci_r26 = context.ci_r26.wrapping_sub(1);
            if context.ci_r26 & 0x8000 != 0 {
                return Pair_Bool_Amt {
                    flag: false,
                    amt: context.ci_r18_r20,
                };
            }
            block_offset = block_offset.wrapping_add(2);
        }
    }
    Pair_Bool_Amt {
        flag: true,
        amt: context.ci_r18_r20,
    }
}
fn prepare_pose_collision(game: &mut Game) {
    let direction = game.samus_collision_direction();
    game.set_samus_collision_direction((direction | 15));
    game.set_flag_samus_in_quicksand(0);
}
fn finish_pose_collision(pair: Pair_Bool_Amt, game: &mut Game) -> i32 {
    game.set_samus_collision_flag(u16::from(pair.flag));
    pair.amt.wrapping_abs()
}
pub unsafe fn WallJumpBlockCollDetect(amt: int32, game: &mut Game) -> int32 {
    prepare_pose_collision(game);
    let pair = BlockColl_Handle_Horiz(amt, game);
    finish_pose_collision(pair, game)
}
unsafe fn CollDetectDueToPoseChange_SingleBlock(amt: int32, game: &mut Game) -> int32 {
    prepare_pose_collision(game);
    let frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
    let pair = if frame & 1 == 0 {
        BlockColl_Handle_Vert_LeftToRight(amt, game)
    } else {
        BlockColl_Handle_Vert_RightToLeft(amt, game)
    };
    finish_pose_collision(pair, game)
}
pub unsafe fn Samus_CollDetectChangedPose(amt: int32, game: &mut Game) -> int32 {
    let integer = (amt >> 16) as i16;
    if integer.wrapping_abs() as u16 & 0xfff8 == 0 {
        return CollDetectDueToPoseChange_SingleBlock(amt, game);
    }
    let probe = i32::from(((integer as u16 & 0xfff0) | 8) as i16) << 16;
    let adjusted = CollDetectDueToPoseChange_SingleBlock(probe, game);
    let collided = game.samus_collision_flag();
    if collided != 0 {
        return adjusted;
    }
    CollDetectDueToPoseChange_SingleBlock(amt, game)
}
fn add_samus_position(position: &mut [u8], amount: i32) {
    // RAM stores the integer word first, then the fractional word.
    let integer = u16::from_le_bytes([position[0], position[1]]);
    let fraction = u16::from_le_bytes([position[2], position[3]]);
    let result = ((u32::from(integer) << 16) | u32::from(fraction)).wrapping_add(amount as u32);
    position[2..4].copy_from_slice(&(result as u16).to_le_bytes());
    position[0..2].copy_from_slice(&((result >> 16) as u16).to_le_bytes());
}
pub unsafe fn Samus_MoveRight_NoSolidColl(mut amt: int32, game: &mut Game) -> int32 {
    let collided = if amt != 0 {
        game.set_flag_samus_in_quicksand(0);
        let pair = BlockColl_Handle_Horiz(amt, game);
        amt = pair.amt;
        pair.flag
    } else {
        false
    };
    game.set_samus_collision_flag(u16::from(collided));
    add_samus_position(&mut game.g_ram[0xaf6..0xafa], amt);
    amt
}
pub unsafe fn Samus_MoveDown_NoSolidColl(mut amt: int32, game: &mut Game) -> int32 {
    if amt != 0 {
        game.set_samus_pos_adjusted_by_slope_flag(0);
        game.set_flag_samus_in_quicksand(0);
        let frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
        let pair = if frame & 1 == 0 {
            BlockColl_Handle_Vert_LeftToRight(amt, game)
        } else {
            BlockColl_Handle_Vert_RightToLeft(amt, game)
        };
        amt = pair.amt;
        if pair.flag {
            game.set_samus_collision_flag(1);
            add_samus_position(&mut game.g_ram[0xafa..0xafe], amt);
            return amt;
        }
    }
    let slope_collision = game.flag_samus_in_quicksand();
    game.set_samus_collision_flag(u16::from(slope_collision != 0));
    add_samus_position(&mut game.g_ram[0xafa..0xafe], amt);
    amt
}
fn BlockInsideReact_Slope(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
fn BlockInsideReact_ShootableAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockInsideReact_SpecialAir_Default(game);
    0
}
fn BlockInsideReact_SpikeAir_BTS2(game: &mut Game) {
    let contact_damage = u16::from_le_bytes([game.g_ram[0xa6e], game.g_ram[0xa6f]]);
    if contact_damage == 0 {
        let invincibility = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
        if invincibility == 0 {
            game.g_ram[0x18a8..0x18aa].copy_from_slice(&60u16.to_le_bytes());
            game.g_ram[0x18aa..0x18ac].copy_from_slice(&10u16.to_le_bytes());
            let damage = u16::from_le_bytes([game.g_ram[0xa50], game.g_ram[0xa51]]);
            game.g_ram[0xa50..0xa52].copy_from_slice(&damage.wrapping_add(16).to_le_bytes());
            let facing = u16::from_le_bytes([game.g_ram[0xa1e], game.g_ram[0xa1f]]);
            let knockback = u16::from((facing ^ 12) & 8 != 0);
            game.g_ram[0xa54..0xa56].copy_from_slice(&knockback.to_le_bytes());
        }
    }
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpikeAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    // Keep the former sixteen-entry table's rejection of invalid BTS values.
    assert!(bts < 16, "spike-air BTS index out of bounds: {bts}");
    if bts == 2 {
        BlockInsideReact_SpikeAir_BTS2(game);
    }
    0
}
fn BlockInsideReact_Special_(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockInsideReact_SpecialAir_Default(game);
    0
}
fn BlockInsideReact_SpecialAir_Default(game: &mut Game) {
    let speed_table = (addr_kSamusSpeedTable_Normal_X as u16).wrapping_add(12);
    game.g_ram[0xa6c..0xa6e].copy_from_slice(&speed_table.to_le_bytes());
}
unsafe fn special_air_movement_enabled(game: &mut Game) -> bool {
    let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
    if area == 3 && CheckBossBitForCurArea(1, game) & 1 == 0 {
        return false;
    }
    let speed = u16::from_le_bytes([game.g_ram[0xb2e], game.g_ram[0xb2f]]);
    speed == 0
}
fn set_special_air_displacement(displacement: u16, game: &mut Game) {
    game.g_ram[0xb56..0xb58].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb58..0xb5a].copy_from_slice(&displacement.to_le_bytes());
}
unsafe fn BlockInsideReact_SpecialAir_8(game: &mut Game) {
    if special_air_movement_enabled(game) {
        set_special_air_displacement(2, game);
    }
    BlockInsideReact_SpecialAir_Default(game);
}
unsafe fn BlockInsideReact_SpecialAir_9(game: &mut Game) {
    if special_air_movement_enabled(game) {
        set_special_air_displacement(0xfffe, game);
    }
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpecialAir_10(game: &mut Game) {
    set_special_air_displacement(2, game);
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpecialAir_11(game: &mut Game) {
    set_special_air_displacement(0xfffe, game);
    BlockInsideReact_SpecialAir_Default(game);
}
unsafe fn BlockInsideReact_SpecialAir_70(game: &mut Game) {
    let sample = game.inside_block_reaction_samus_point();
    if sample == 1 {
        SpawnPLM(addr_kPlmHeader_B6FF as u16, game);
    }
}
const SPECIAL_AIR_PLM_TABLES: [u16; 8] = [
    0x9a06, 0x9a26, 0x9a46, 0x9a66, 0x9a86, 0x9aa6, 0x9ac6, 0x9ae6,
];
unsafe fn BlockInsideReact_SpecialAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
        let table = SPECIAL_AIR_PLM_TABLES[usize::from(area)];
        let plm = game.read_rom_word(0x940000 | u32::from(table), usize::from(bts & 0x7f) * 2);
        SpawnPLM(plm, game);
    } else {
        // Preserve the former eighty-entry dispatch table's invalid-BTS panic.
        assert!(bts < 80, "special-air BTS index out of bounds: {bts}");
        match bts {
            8 => BlockInsideReact_SpecialAir_8(game),
            9 => BlockInsideReact_SpecialAir_9(game),
            10 => BlockInsideReact_SpecialAir_10(game),
            11 => BlockInsideReact_SpecialAir_11(game),
            70 => BlockInsideReact_SpecialAir_70(game),
            _ => BlockInsideReact_SpecialAir_Default(game),
        }
    }
    0
}
unsafe fn dispatch_block_inside(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    match game.current_level_block() >> 12 {
        0 | 4 | 6 | 7 => BlockInsideReact_ShootableAir(ci, game),
        1 => BlockInsideReact_Slope(ci, game),
        2 => BlockInsideReact_SpikeAir(ci, game),
        3 => BlockInsideReact_SpecialAir(ci, game),
        5 => BlockReact_HorizExt(ci, game),
        13 => BlockReact_VertExt(ci, game),
        _ => BlockInsideReact_Special_(ci, game),
    }
}
unsafe fn react_inside_sample(x: u16, y: u16, game: &mut Game) {
    CalculateBlockAt(x, y, 0, 0, game);
    let mut context = CollInfo {
        ci_r18_r20: 0,
        ci_r24: 0,
        ci_r26: x,
        ci_r28: y,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    // Keep the same context alive across retries and callback-adjusted block indexes.
    while dispatch_block_inside(&raw mut context, game) & 0x80 != 0 {}
}
pub unsafe fn BlockInsideDetection(game: &mut Game) {
    game.g_ram[0xb4c..0xb4e].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb56..0xb58].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb58..0xb5a].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&0u16.to_le_bytes());
    game.set_inside_block_reaction_samus_point(0);
    let mut r26 = game.samus_x_pos();
    let radius = game.samus_y_radius();
    let y = game.samus_y_pos();
    let samus_bottom_boundary_position = radius.wrapping_add(y).wrapping_sub(1);
    let mut r28 = samus_bottom_boundary_position;
    react_inside_sample(r26, r28, game);
    game.set_inside_block_reaction_samus_point(1);
    let y = game.samus_y_pos();
    if (samus_bottom_boundary_position ^ y) & 0xfff0 != 0 {
        r26 = game.samus_x_pos();
        r28 = y;
        react_inside_sample(r26, r28, game);
    }
    game.set_inside_block_reaction_samus_point(2);
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    let top = y.wrapping_sub(radius);
    let changed_row_bits = (samus_bottom_boundary_position ^ top) & 0xfff0;
    // Preserve the original XOR test; this is not a simple center/top comparison.
    if changed_row_bits != 0 && (y ^ changed_row_bits) & 0xfff0 != 0 {
        r26 = game.samus_x_pos();
        r28 = top;
        react_inside_sample(r26, r28, game);
    }
}
fn CalculateBlockAt(x: uint16, y: uint16, x_offset: uint16, y_offset: uint16, game: &mut Game) {
    let x = x.wrapping_add(x_offset);
    let y = y.wrapping_add(y_offset);
    // The original signed tests accept only wrapped coordinates in 0..4096.
    let block = if x < 4096 && y < 4096 {
        let width = u16::from(game.room_width_in_blocks() as u8);
        (x >> 4) + (y >> 4) * width
    } else {
        0xffff
    };
    game.set_cur_block_index(block);
}
static kBlockFunc_9C73_Tab0: [uint8; 36] = [
    3 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    40 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    40 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    80 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    20 as ::core::ffi::c_int as uint8,
    80 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    100 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    100 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    100 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    100 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
];
static kBlockFunc_9C73_Tab1: [int8; 18] = [
    0 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    10 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    50 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    50 as ::core::ffi::c_int as int8,
    3 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    -(1 as ::core::ffi::c_int) as int8,
    10 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    -(1 as ::core::ffi::c_int) as int8,
    0 as ::core::ffi::c_int as int8,
    -(1 as ::core::ffi::c_int) as int8,
    0 as ::core::ffi::c_int as int8,
    -(1 as ::core::ffi::c_int) as int8,
];
unsafe fn BlockFunc_9C73(mut v0: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return kBlockFunc_9C73_Tab1[(2 as ::core::ffi::c_int
            * (*(((&raw mut game.g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as usize] as uint16;
    } else {
        let mut r18: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int;
        return kBlockFunc_9C73_Tab0
            [(3 as ::core::ffi::c_int * r18 + 2 as ::core::ffi::c_int) as usize]
            as uint16;
    };
}
pub unsafe fn BombOrPowerBomb_Func1(mut v0: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut temp_collision_DD2: uint16 = BlockFunc_9C73(v0, game);
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut r26: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        if ((*(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v3 = *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as int16;
            if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                let mut R28: uint16 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                if ((*(&raw mut v3 as *mut uint8).offset(
                    (::core::mem::size_of::<int16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x7ab as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    CalculateBlockAt(r26, R28, 0 as uint16, 0 as uint16, game);
                    let mut ci: CollInfo = CollInfo {
                        ci_r18_r20: 0,
                        ci_r24: 0 as uint16,
                        ci_r26: r26,
                        ci_r28: R28,
                        ci_r30: 0,
                        ci_r32: 0,
                        ci_r38: 0,
                        ci_r40: 0,
                    };
                    if temp_collision_DD2 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                        BlockColl_BombExplosion(&raw mut ci, v0, game);
                    } else {
                        BlockFunc_A06A(v0, game);
                    }
                }
            }
        }
    }
}
unsafe fn BlockColl_GetBlockIndexAbove(mut k: uint16, game: &mut Game) -> uint16 {
    return (k as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_GetBlockIndexRight(mut k: uint16, game: &mut Game) -> uint16 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + k as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_GetBlockIndexLeft(mut k: uint16) -> uint16 {
    return (k as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_GetBlockIndexBelow(mut k: uint16, game: &mut Game) -> uint16 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + k as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_BombExplosion(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16;
        if v2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (v2 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0xffff as ::core::ffi::c_int
            {
                let mut v3: uint16 = (2 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
                BlockBombedReact(
                    ci,
                    (2 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xdc4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16,
                    game,
                );
                let mut idx: uint16 = BlockColl_GetBlockIndexAbove(v3, game);
                BlockBombedReact(ci, idx, game);
                let mut idx2: uint16 = BlockColl_GetBlockIndexRight(idx, game);
                BlockBombedReact(ci, idx2, game);
                idx = BlockColl_GetBlockIndexLeft(idx2);
                BlockBombedReact(ci, idx, game);
                idx = BlockColl_GetBlockIndexBelow(idx, game);
                BlockBombedReact(ci, idx, game);
            }
        }
    }
}
unsafe fn ClearCarry_9(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
unsafe fn SetCarry_0(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn sub_949D5D(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5 as ::core::ffi::c_int
    {
        return 1 as uint8;
    } else {
        return BlockShotReactVert_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockBombedReact_Special(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut kBlockBombedReact_Region_Plm: *mut uint16 =
            RomPtr_94(0x9e44 as uint16_t, game) as *mut uint16;
        let mut v2: *const uint16 = RomPtr_94(
            *kBlockBombedReact_Region_Plm.offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as uint16_t,
            game,
        ) as *mut uint16;
        SpawnPLM(
            *v2.offset((v0 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as isize),
            game,
        );
    } else {
        let mut kBlockBombedReact_Special_Plm: *mut uint16 =
            RomPtr_94(0x9da4 as uint16_t, game) as *mut uint16;
        SpawnPLM(*kBlockBombedReact_Special_Plm.offset(v0 as isize), game);
    }
    return 1 as uint8;
}
unsafe fn BlockReact_ShootableAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut t: uint16 =
            *(RomPtr_94(0x9ea6 as uint16_t, game) as *mut uint16).offset(v0 as isize);
        if t as ::core::ffi::c_int == 0xb974 as ::core::ffi::c_int {
            (*ci).ci_r38 = 0 as uint16;
            (*ci).ci_r40 = 0xffff as uint16;
        }
        SpawnPLM(t, game);
    }
    return 0 as uint8;
}
unsafe fn BlockReact_Shootable(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm: *mut uint16 =
            RomPtr_94(0x9fc6 as uint16_t, game) as *mut uint16;
        let mut v2: *mut uint16 = RomPtr_94(
            *kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm.offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as uint16_t,
            game,
        ) as *mut uint16;
        SpawnPLM(
            *v2.offset((v0 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as isize),
            game,
        );
    } else {
        let mut t: uint16 =
            *(RomPtr_94(0x9ea6 as uint16_t, game) as *mut uint16).offset(v0 as isize);
        if t as ::core::ffi::c_int == 0xb974 as ::core::ffi::c_int {
            (*ci).ci_r38 = 0 as uint16;
            (*ci).ci_r40 = 0xffff as uint16;
        }
        SpawnPLM(t, game);
    }
    return 1 as uint8;
}
unsafe fn BlockReact_BombableAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut kBlockShotBombGrappleReact_BombableBlockPlm: *mut uint16 =
        RomPtr_94(0xa012 as uint16_t, game) as *mut uint16;
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    return 0 as uint8;
}
unsafe fn BlockReact_BombableBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut kBlockShotBombGrappleReact_BombableBlockPlm: *mut uint16 =
        RomPtr_94(0xa012 as uint16_t, game) as *mut uint16;
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    return 1 as uint8;
}
static kBlockBombedReact: [Option<Func_CollInfo_U8>; 16] = unsafe {
    [
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockBombedReact_Special as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
    ]
};
unsafe fn BlockBombedReact(mut ci: *mut CollInfo, mut v0: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    while kBlockBombedReact[((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x10002 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int)
        >> 12 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(ci, game) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {}
}
unsafe fn BlockFunc_A06A(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16;
    if v2 != 0 {
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16;
        let mut r20: uint16 = ((3 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as int16;
        if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v3 = 0 as int16;
        }
        let mut r22: uint16 =
            (v3 as uint16 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
        let mut v4: uint16 = ((r18 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int
            >= *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
        }
        let mut r24: uint16 = v4;
        let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xce4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int) as uint16;
        if (v5 as int16 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v5 = 0 as uint16;
        }
        let mut r26: uint16 = (v5 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
        let mut v6: uint16 = ((r20 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        if v6 as ::core::ffi::c_int
            >= *((&raw mut game.g_ram as *mut uint8).offset(0x7a7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
        }
        let mut R28: uint16 = v6;
        let mut a: uint16 = (r26 as ::core::ffi::c_int
            | (r22 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint16;
        BlockFunc_A0F4(a, r22, r24, r26, R28, game);
        BlockFunc_A11A(a, r24, r26, R28, game);
        BlockFunc_A0F4(
            (R28 as ::core::ffi::c_int | (r22 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16,
            r22,
            r24,
            r26,
            R28,
            game,
        );
        BlockFunc_A11A(
            (r26 as ::core::ffi::c_int | (r24 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16,
            r24,
            r26,
            R28,
            game,
        );
    };
}
unsafe fn BlockFunc_A0F4(
    mut a: uint16,
    mut r22: uint16,
    mut r24: uint16,
    mut r26: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut RegWord: uint16 = Mult8x8(
        a as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * (RegWord as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int)) as uint16;
    let mut v4: int16 = (r24 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: r24,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        BlockBombedReact(&raw mut ci, v3, game);
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v4 -= 1;
        if !(v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn BlockFunc_A11A(
    mut a: uint16,
    mut r24: uint16,
    mut r26: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut RegWord: uint16 = Mult8x8(
        a as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * (RegWord as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int)) as uint16;
    let mut v4: int16 = (r28 as ::core::ffi::c_int - r26 as ::core::ffi::c_int) as int16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: r24,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        BlockBombedReact(&raw mut ci, v3, game);
        v3 = (v3 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        v4 -= 1;
        if !(v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
unsafe fn BlockShotReactHoriz_Slope(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5 as ::core::ffi::c_int
    {
        return BlockShotReactVert_Slope_Square(
            ci,
            (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    } else {
        return BlockShotReactVert_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockShotReactVert_Slope(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5 as ::core::ffi::c_int
    {
        return BlockShotReactHoriz_Slope_Square(
            ci,
            (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    } else {
        return BlockShotReactHoriz_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockShotReactHoriz(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockShotReactHoriz: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockShotReactHoriz_Slope as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: uint8 = 0;
    if k as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0x7b9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        v1 = kBlockShotReactHoriz[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(ci, game);
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    if v1 != 0 {
        (*ci).ci_r40 = (*ci).ci_r40.wrapping_sub(1);
    }
    return v1;
}
unsafe fn BlockShotReactVert(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockShotReactVert: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockShotReactVert_Slope as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: uint8 = 0;
    if k as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0x7b9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        v1 = kBlockShotReactVert[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(ci, game);
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    if v1 != 0 {
        (*ci).ci_r40 = (*ci).ci_r40.wrapping_sub(1);
    }
    return v1;
}
unsafe fn BlockGetSomePos1(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r38: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    return ((*((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r38 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockGetSomePos2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r38: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    return ((*((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r38 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BlockCollNoWaveBeamHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos1(k, game);
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if amt < 0 as int32 {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || (*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactHoriz(&raw mut ci, v8, game);
        v8 = (v8 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if !(ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if ci.ci_r40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollNoWaveBeamVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if amt < 0 as int32 {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v9: uint16 = (2 as ::core::ffi::c_int
        * (Mult8x8(
            (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + v8 as ::core::ffi::c_int)) as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || (*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactVert(&raw mut ci, v9, game);
        v9 = (v9 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if !(ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if ci.ci_r40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollWaveBeamHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos1(k, game);
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if amt < 0 as int32 {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    let mut v9: uint8 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    if v9 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
        && ((v9 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        && ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        loop {
            BlockShotReactHoriz(&raw mut ci, v8, game);
            v8 = (v8 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int) as uint16;
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if !(ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
    return 0 as uint8;
}
pub unsafe fn BlockCollWaveBeamVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if amt < 0 as int32 {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut prod: uint16 = Mult8x8(
        (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v8: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v9: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v8 as ::core::ffi::c_int))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        return 0 as uint8;
    }
    let mut v10: uint8 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    if v10 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
        && ((v10 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        && ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        loop {
            BlockShotReactVert(&raw mut ci, v9, game);
            v9 = (v9 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if !(ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
    return 0 as uint8;
}
pub unsafe fn BlockCollMissileHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb8c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r28: uint16 = r24;
    let mut v7: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int))) as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as uint16,
        ci_r28: r28,
        ci_r30: 1 as uint16,
        ci_r32: 0,
        ci_r38: 0 as uint16,
        ci_r40: 0,
    };
    if (*(&raw mut r24 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        || BlockShotReactHoriz(&raw mut ci, v7, game) == 0
    {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollMissileVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r28: uint16 = r24;
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v7: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v7 as ::core::ffi::c_int))
        as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as uint16,
        ci_r28: r28,
        ci_r30: 1 as uint16,
        ci_r32: 0,
        ci_r38: 0 as uint16,
        ci_r40: 0,
    };
    if (*(&raw mut r24 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        || BlockShotReactVert(&raw mut ci, v8, game) == 0
    {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
unsafe fn BlockShotReact_Slope_NonSquare(
    mut ci: *mut CollInfo,
    mut j: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        ^ (if *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x40 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            0xf as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as uint16;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        ^ (if *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            0xf as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as uint16;
    let mut v5: uint16 = (kAlignYPos_Tab0[(16 as ::core::ffi::c_int
        * (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
        + (v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
        as usize] as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    if (v5 as ::core::ffi::c_int - v4 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        <= 0 as ::core::ffi::c_int
    {
        (*ci).ci_r38 = 0 as uint16;
        (*ci).ci_r40 = 0 as uint16;
        return 1 as uint8;
    } else {
        return 0 as uint8;
    };
}
unsafe fn BlockShotReactVert_Slope_NonSquare(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xdc4 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    let mut div: uint16 = SnesDivide(
        *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == div as ::core::ffi::c_int
    {
        return BlockShotReact_Slope_NonSquare(ci, v1, v0, game);
    } else {
        return 0 as uint8;
    };
}
unsafe fn BlockShotReactHoriz_Slope_NonSquare(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xdc4 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    let mut mod_0: uint16 = SnesModulus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == mod_0 as ::core::ffi::c_int
    {
        return BlockShotReact_Slope_NonSquare(ci, v1, v0, game);
    } else {
        return 0 as uint8;
    };
}
pub unsafe fn BlockCollSpreadBomb(mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockCollSpreadBomb: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(sub_949D5D as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r26: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r28: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    CalculateBlockAt(r26, r28, 0 as uint16, 0 as uint16, game);
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: 0 as uint16,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0 as uint16,
        ci_r32: 0 as uint16,
        ci_r38: 0,
        ci_r40: 0,
    };
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        BlockColl_BombExplosion(&raw mut ci, k, game);
        return 0 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockCollSpreadBomb[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
unsafe fn BlockShotReactVert_Slope_Square(
    mut ci: *mut CollInfo,
    mut a: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut temp_collision_DD4: uint16 =
        (4 as ::core::ffi::c_int * a as ::core::ffi::c_int) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*ci).ci_r28 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int)) as uint16;
    let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                || kTab948E54[v2 as usize] == 0
            {
                let mut v4: uint16 = (v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
                if *(((&raw mut game.g_ram as *mut uint8)
                    .offset(0xbc8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + *(((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb78 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                    || kTab948E54[v4 as usize] == 0
                {
                    return 0 as uint8;
                }
            }
            return 1 as uint8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 as ::core::ffi::c_int == (*ci).ci_r26 as ::core::ffi::c_int
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xbc8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v3 as isize) as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                return (kTab948E54[(v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    as uint8;
            }
        } else if *(((&raw mut game.g_ram as *mut uint8)
            .offset(0xbc8 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v3 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as *mut uint16 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            return (kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int as uint8;
        }
        if kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 1 as uint8;
        }
        return (kTab948E54[(v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
    }
    return (kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int as uint8;
}
unsafe fn BlockShotReactHoriz_Slope_Square(
    mut ci: *mut CollInfo,
    mut a: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut temp_collision_DD4: uint16 =
        (4 as ::core::ffi::c_int * a as ::core::ffi::c_int) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*ci).ci_r28 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int)) as uint16;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                || kTab948E54[v2 as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                let mut v4: uint16 = (v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
                if *(((&raw mut game.g_ram as *mut uint8)
                    .offset(0xbb4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + *(((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb64 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                    || kTab948E54[v4 as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                {
                    return 0 as uint8;
                }
            }
            return 1 as uint8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 as ::core::ffi::c_int == (*ci).ci_r26 as ::core::ffi::c_int
                && *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xdde as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xbb4 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xdde as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                return (kTab948E54[(v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    as uint8;
            }
        } else if *(((&raw mut game.g_ram as *mut uint8)
            .offset(0xbb4 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as *mut uint16 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            return (kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int as uint8;
        }
        if kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 1 as uint8;
        }
        return (kTab948E54[(v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    }
    return (kTab948E54[v2 as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int as uint8;
}
unsafe fn ClearCarryZero(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
unsafe fn SetCarryClearOvf(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn BlockReactGrapple_GrappleBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    static kBlockReactGrapple_GrappleBlockPlm: [uint16; 4] = [
        0xd0d8 as ::core::ffi::c_int as uint16,
        0xd0dc as ::core::ffi::c_int as uint16,
        0xd0e0 as ::core::ffi::c_int as uint16,
        0xd0d8 as ::core::ffi::c_int as uint16,
    ];
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xcf4 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh0 = (*fresh0 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    } else {
        return SpawnPLM(
            kBlockReactGrapple_GrappleBlockPlm[*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as usize],
            game,
        );
    };
}
unsafe fn BlockReactGrapple_SpikeBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    static kBlockReactGrapple_SpikeBlockPlm: [uint16; 16] = [
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e8 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
        0xd0e4 as ::core::ffi::c_int as uint16,
    ];
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    } else {
        return SpawnPLM(
            kBlockReactGrapple_SpikeBlockPlm[*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as usize],
            game,
        );
    };
}
pub unsafe fn BlockCollGrappleBeam(game: &mut Game) -> uint8 {
    let mut grapple_vel_x: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd22 as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as int32)
        << 6 as ::core::ffi::c_int;
    let mut grapple_vel_y: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd24 as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as int32)
        << 6 as ::core::ffi::c_int;
    let mut result: uint8 = 0 as uint8;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd10 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd0e as ::core::ffi::c_int as isize)
                as *mut uint16,
            grapple_vel_x as uint32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd14 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd12 as ::core::ffi::c_int as isize)
                as *mut uint16,
            grapple_vel_y as uint32,
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd06 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xd10 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xd0e as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xaf8 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(
                    ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd02 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int) as uint32,
                ),
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd0a as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xd14 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xd12 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xafc as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(
                    ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd04 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int) as uint32,
                ),
        );
        result = BlockReactGrapple(game);
        if result as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && result as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
            return result;
        }
        i += 1;
    }
    return result;
}
pub unsafe fn BlockReactGrapple(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    static kBlockReactGrapple: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReactGrapple_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReactGrapple_GrappleBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockReactGrapple[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
pub static kSinCosTable8bit_Sext: [int16; 320] = [
    -(256 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
    25 as ::core::ffi::c_int as int16,
    31 as ::core::ffi::c_int as int16,
    37 as ::core::ffi::c_int as int16,
    43 as ::core::ffi::c_int as int16,
    49 as ::core::ffi::c_int as int16,
    56 as ::core::ffi::c_int as int16,
    62 as ::core::ffi::c_int as int16,
    68 as ::core::ffi::c_int as int16,
    74 as ::core::ffi::c_int as int16,
    80 as ::core::ffi::c_int as int16,
    86 as ::core::ffi::c_int as int16,
    92 as ::core::ffi::c_int as int16,
    97 as ::core::ffi::c_int as int16,
    103 as ::core::ffi::c_int as int16,
    109 as ::core::ffi::c_int as int16,
    115 as ::core::ffi::c_int as int16,
    120 as ::core::ffi::c_int as int16,
    126 as ::core::ffi::c_int as int16,
    131 as ::core::ffi::c_int as int16,
    136 as ::core::ffi::c_int as int16,
    142 as ::core::ffi::c_int as int16,
    147 as ::core::ffi::c_int as int16,
    152 as ::core::ffi::c_int as int16,
    157 as ::core::ffi::c_int as int16,
    162 as ::core::ffi::c_int as int16,
    167 as ::core::ffi::c_int as int16,
    171 as ::core::ffi::c_int as int16,
    176 as ::core::ffi::c_int as int16,
    181 as ::core::ffi::c_int as int16,
    185 as ::core::ffi::c_int as int16,
    189 as ::core::ffi::c_int as int16,
    193 as ::core::ffi::c_int as int16,
    197 as ::core::ffi::c_int as int16,
    201 as ::core::ffi::c_int as int16,
    205 as ::core::ffi::c_int as int16,
    209 as ::core::ffi::c_int as int16,
    212 as ::core::ffi::c_int as int16,
    216 as ::core::ffi::c_int as int16,
    219 as ::core::ffi::c_int as int16,
    222 as ::core::ffi::c_int as int16,
    225 as ::core::ffi::c_int as int16,
    228 as ::core::ffi::c_int as int16,
    231 as ::core::ffi::c_int as int16,
    234 as ::core::ffi::c_int as int16,
    236 as ::core::ffi::c_int as int16,
    238 as ::core::ffi::c_int as int16,
    241 as ::core::ffi::c_int as int16,
    243 as ::core::ffi::c_int as int16,
    244 as ::core::ffi::c_int as int16,
    246 as ::core::ffi::c_int as int16,
    248 as ::core::ffi::c_int as int16,
    249 as ::core::ffi::c_int as int16,
    251 as ::core::ffi::c_int as int16,
    252 as ::core::ffi::c_int as int16,
    253 as ::core::ffi::c_int as int16,
    254 as ::core::ffi::c_int as int16,
    254 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    255 as ::core::ffi::c_int as int16,
    254 as ::core::ffi::c_int as int16,
    254 as ::core::ffi::c_int as int16,
    253 as ::core::ffi::c_int as int16,
    252 as ::core::ffi::c_int as int16,
    251 as ::core::ffi::c_int as int16,
    249 as ::core::ffi::c_int as int16,
    248 as ::core::ffi::c_int as int16,
    246 as ::core::ffi::c_int as int16,
    244 as ::core::ffi::c_int as int16,
    243 as ::core::ffi::c_int as int16,
    241 as ::core::ffi::c_int as int16,
    238 as ::core::ffi::c_int as int16,
    236 as ::core::ffi::c_int as int16,
    234 as ::core::ffi::c_int as int16,
    231 as ::core::ffi::c_int as int16,
    228 as ::core::ffi::c_int as int16,
    225 as ::core::ffi::c_int as int16,
    222 as ::core::ffi::c_int as int16,
    219 as ::core::ffi::c_int as int16,
    216 as ::core::ffi::c_int as int16,
    212 as ::core::ffi::c_int as int16,
    209 as ::core::ffi::c_int as int16,
    205 as ::core::ffi::c_int as int16,
    201 as ::core::ffi::c_int as int16,
    197 as ::core::ffi::c_int as int16,
    193 as ::core::ffi::c_int as int16,
    189 as ::core::ffi::c_int as int16,
    185 as ::core::ffi::c_int as int16,
    181 as ::core::ffi::c_int as int16,
    176 as ::core::ffi::c_int as int16,
    171 as ::core::ffi::c_int as int16,
    167 as ::core::ffi::c_int as int16,
    162 as ::core::ffi::c_int as int16,
    157 as ::core::ffi::c_int as int16,
    152 as ::core::ffi::c_int as int16,
    147 as ::core::ffi::c_int as int16,
    142 as ::core::ffi::c_int as int16,
    136 as ::core::ffi::c_int as int16,
    131 as ::core::ffi::c_int as int16,
    126 as ::core::ffi::c_int as int16,
    120 as ::core::ffi::c_int as int16,
    115 as ::core::ffi::c_int as int16,
    109 as ::core::ffi::c_int as int16,
    103 as ::core::ffi::c_int as int16,
    97 as ::core::ffi::c_int as int16,
    92 as ::core::ffi::c_int as int16,
    86 as ::core::ffi::c_int as int16,
    80 as ::core::ffi::c_int as int16,
    74 as ::core::ffi::c_int as int16,
    68 as ::core::ffi::c_int as int16,
    62 as ::core::ffi::c_int as int16,
    56 as ::core::ffi::c_int as int16,
    49 as ::core::ffi::c_int as int16,
    43 as ::core::ffi::c_int as int16,
    37 as ::core::ffi::c_int as int16,
    31 as ::core::ffi::c_int as int16,
    25 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(256 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
];
unsafe fn BlockFunc_A957(mut tmpD82: uint16, mut tmpD84: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut v7: int16 = 0;
    let mut v0: uint16 = tmpD82;
    let mut v2: uint16 = 0;
    let mut v3: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = tmpD82 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 7 as ::core::ffi::c_int) as uint16;
        } else {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        if kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 7 as ::core::ffi::c_int) as uint16;
        } else {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
            as *mut uint16) = v3;
    }
    let mut v4: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v5 = kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize];
    let mut v6: uint16 = 0;
    let mut v8: uint16 = 0;
    if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if v5 as ::core::ffi::c_int == -(256 as ::core::ffi::c_int) {
            v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - tmpD84 as ::core::ffi::c_int) as uint16;
        } else {
            let mut prod: uint16 = Mult8x8(
                tmpD84 as uint8,
                -(v5 as int8 as ::core::ffi::c_int) as uint8,
            );
            v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (prod as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
                as uint16;
        }
    } else if v5 as ::core::ffi::c_int == 256 as ::core::ffi::c_int {
        v6 = (tmpD84 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    } else {
        let mut prod_0: uint16 = Mult8x8(
            tmpD84 as uint8,
            kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint8,
        );
        v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (prod_0 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd90 as ::core::ffi::c_int as isize)
        as *mut uint16) = v6;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd94 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v6 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8 as uint16;
    v7 = kSinCosTable8bit_Sext[v4 as usize];
    if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if v7 as ::core::ffi::c_int == -(256 as ::core::ffi::c_int) {
            v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - tmpD84 as ::core::ffi::c_int) as uint16;
        } else {
            let mut prod_1: uint16 = Mult8x8(tmpD84 as uint8, -(v7 as ::core::ffi::c_int) as uint8);
            v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (prod_1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
                as uint16;
        }
    } else if v7 as ::core::ffi::c_int == 256 as ::core::ffi::c_int {
        v8 = (tmpD84 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    } else {
        let mut prod_2: uint16 =
            Mult8x8(tmpD84 as uint8, kSinCosTable8bit_Sext[v4 as usize] as uint8);
        v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (prod_2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd92 as ::core::ffi::c_int as isize)
        as *mut uint16) = v8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd96 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v8 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8 as uint16;
}
unsafe fn ClearCarry_10(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
unsafe fn SetCarry_3(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn BlockReact_AA64(game: &mut Game) -> uint8 {
    static kBlockReactLut_AB90_0: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_AA64_SpikeAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_AA64_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd94 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut RegWord: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xd96 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (RegWord as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockReactLut_AB90_0[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
static g_word_94AAF7: [uint16; 16] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
unsafe fn BlockReact_AA64_SpikeAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 0 as uint8;
    }
    let mut v0: int16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as int16;
    if v0 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v: uint32 = ((g_word_94AAF7[v0 as usize] as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xa4e as ::core::ffi::c_int as isize)
                    as *mut uint16,
                v,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 60 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
        }
    }
    return 0 as uint8;
}
static g_word_94AB70: [uint16; 16] = [
    60 as ::core::ffi::c_int as uint16,
    16 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
unsafe fn BlockReact_AA64_SpikeBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return 1 as uint8;
    }
    let mut v0: int16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as int16;
    if v0 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v: uint32 = ((g_word_94AB70[v0 as usize] as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xa4e as ::core::ffi::c_int as isize)
                    as *mut uint16,
                v,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 60 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
        }
    }
    return 1 as uint8;
}
static kBlockReactLut_AB90: [Option<Func_CollInfo_U8>; 16] = unsafe {
    [
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_AA64_SpikeAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_AA64_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
    ]
};
unsafe fn BlockFunc_ABB0(game: &mut Game) -> uint8 {
    let mut rv: uint8 = 0;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd94 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut RegWord: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xd96 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (RegWord as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        rv = kBlockReactLut_AB90[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
unsafe fn BlockFunc_ABE6(mut tmpD82: uint16, game: &mut Game) -> uint8 {
    let mut v0: uint8 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd98 as ::core::ffi::c_int as isize)
        as *mut uint16) = 6 as uint16;
    let mut grapple_beam_tmpD84: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xcfe as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int) as uint16;
    loop {
        BlockFunc_A957(tmpD82, grapple_beam_tmpD84, game);
        v0 = (BlockFunc_ABB0(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8;
        if v0 != 0 {
            break;
        }
        grapple_beam_tmpD84 =
            (grapple_beam_tmpD84 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd98 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
        if *fresh10 == 0 {
            return 0 as uint8;
        }
    }
    return v0;
}
pub unsafe fn BlockFunc_AC11(game: &mut Game) {
    let mut grapple_beam_tmpD82: uint16 = (2 as ::core::ffi::c_int
        * *(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint16;
    let mut grapple_beam_tmpD84: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xcfe as ::core::ffi::c_int as isize)
        as *mut uint16);
    BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd16 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd90 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd18 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd92 as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn BlockFunc_AC31(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        return 0 as uint8;
    }
    let mut end: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd00 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (end as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            end = 8 as uint16;
        }
        let mut increment: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        let mut grapple_beam_tmpD82: uint16 = (2 as ::core::ffi::c_int
            * *(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16);
        while v1 as ::core::ffi::c_int != end as ::core::ffi::c_int {
            let mut grapple_beam_tmpD84: uint16 =
                (increment + v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
            if BlockReact_AA64(game) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                    as *mut uint16) = v1;
                return 1 as uint8;
            }
            v1 = v1.wrapping_sub(1);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16) = end;
        return 0 as uint8;
    } else {
        if end as ::core::ffi::c_int >= 63 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            end = 63 as uint16;
        }
        let mut increment_0: ::core::ffi::c_int = 56 as ::core::ffi::c_int;
        let mut grapple_beam_tmpD82_0: uint16 = (2 as ::core::ffi::c_int
            * *(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint16;
        let mut v1_0: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16);
        while v1_0 as ::core::ffi::c_int != end as ::core::ffi::c_int {
            let mut grapple_beam_tmpD84_0: uint16 =
                (increment_0 + v1_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            BlockFunc_A957(grapple_beam_tmpD82_0, grapple_beam_tmpD84_0, game);
            if BlockReact_AA64(game) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                    as *mut uint16) = v1_0;
                return 1 as uint8;
            }
            v1_0 = v1_0.wrapping_add(1);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16) = end;
        return 0 as uint8;
    };
}
pub unsafe fn HandleMovementAndCollForSamusGrapple(game: &mut Game) -> uint8 {
    let mut v2: uint16 = 0;
    let mut v5: uint16 = 0;
    let mut current_block: u64;
    let mut prod16: uint16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v6: uint16 = 0;
    let mut grapple_beam_tmpD86: uint16 = 0;
    let mut grapple_beam_tmpD88: uint16 = 0;
    let mut tmpD9C: uint16 = 0;
    let mut v0: uint16 = 256 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = 160 as uint16;
    }
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd2e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        prod16 = (Multiply16x16(v1, v0) >> 8 as ::core::ffi::c_int) as uint16;
        if prod16 == 0 {
            return 0 as uint8;
        }
        tmpD9C = prod16;
        grapple_beam_tmpD88 = (2 as ::core::ffi::c_int
            * ((*((&raw mut game.g_ram as *mut uint8).offset(0xcfa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + prod16 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        v2 = 0;
        v2 = (2 as ::core::ffi::c_int
            * *(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int != grapple_beam_tmpD88 as ::core::ffi::c_int {
            loop {
                grapple_beam_tmpD86 = v2;
                let mut grapple_beam_tmpD82: uint16 =
                    (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                        & 0x1ff as ::core::ffi::c_int) as uint16;
                if BlockFunc_ABE6(grapple_beam_tmpD82, game) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0
                {
                    current_block = 7976072742316086414;
                    break;
                }
                v2 = (grapple_beam_tmpD86 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                if v2 as ::core::ffi::c_int == grapple_beam_tmpD88 as ::core::ffi::c_int {
                    current_block = 16784897620381762466;
                    break;
                }
            }
            match current_block {
                16784897620381762466 => {}
                _ => {
                    *(&raw mut v4 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                        0 as uint8;
                    *(&raw mut v4 as *mut uint8).offset(
                        (::core::mem::size_of::<int16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) = (grapple_beam_tmpD86 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        as uint8;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xcfa as ::core::ffi::c_int as isize)
                        as *mut uint16) =
                        (v4 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xcfc as ::core::ffi::c_int as isize)
                        as *mut uint16) =
                        (v4 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint16;
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd98 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 6 as ::core::ffi::c_int
                        && *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd98 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            != 5 as ::core::ffi::c_int
                        || *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xcfe as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            != 8 as ::core::ffi::c_int
                    {
                        current_block = 15145243721762997867;
                    } else {
                        current_block = 4207414038393540328;
                    }
                }
            }
        } else {
            current_block = 16784897620381762466;
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xcfa as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh4 = (*fresh4 as ::core::ffi::c_int + tmpD9C as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfc as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xcfa as ::core::ffi::c_int as isize)
                    as *mut uint16);
                let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xd36 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh5 =
                    (*fresh5 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
                let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xd30 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh6 = (*fresh6).wrapping_sub(1);
                if *fresh6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd30 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xd2e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v3 = (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd2e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 6 as ::core::ffi::c_int) as uint16;
                    if (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd2e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 6 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                    {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                } else {
                    v3 = (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd2e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + 6 as ::core::ffi::c_int) as uint16;
                    if ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd2e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + 6 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                }
                match current_block {
                    18377268871191777778 => {
                        v3 = 0 as uint16;
                    }
                    _ => {}
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
                    as *mut uint16) = v3;
                return 0 as uint8;
            }
        }
    } else {
        prod16 = (Multiply16x16(-(v1 as ::core::ffi::c_int) as uint16, v0)
            >> 8 as ::core::ffi::c_int) as uint16;
        if prod16 == 0 {
            return 0 as uint8;
        }
        tmpD9C = -(prod16 as ::core::ffi::c_int) as uint16;
        grapple_beam_tmpD88 = (2 as ::core::ffi::c_int
            * ((*((&raw mut game.g_ram as *mut uint8).offset(0xcfa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - prod16 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        v5 = 0;
        v5 = (2 as ::core::ffi::c_int
            * *(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint16;
        if v5 as ::core::ffi::c_int == grapple_beam_tmpD88 as ::core::ffi::c_int {
            current_block = 5091811861639934416;
        } else {
            current_block = 11777552016271000781;
        }
        loop {
            match current_block {
                11777552016271000781 => {
                    grapple_beam_tmpD86 = v5;
                    let mut grapple_beam_tmpD82_0: uint16 =
                        (v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 0x1ff as ::core::ffi::c_int) as uint16;
                    if BlockFunc_ABE6(grapple_beam_tmpD82_0, game) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        != 0
                    {
                        *(&raw mut v7 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                            0 as uint8;
                        *(&raw mut v7 as *mut uint8).offset(
                            (::core::mem::size_of::<int16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ) = (grapple_beam_tmpD86 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            as uint8;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xcfa as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (v7 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xcfc as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (v7 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint16;
                        if (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd98 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            == 6 as ::core::ffi::c_int
                            || *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xd98 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                as ::core::ffi::c_int
                                == 5 as ::core::ffi::c_int)
                            && *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xcfe as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                == 8 as ::core::ffi::c_int
                        {
                            current_block = 4207414038393540328;
                            break;
                        } else {
                            current_block = 15145243721762997867;
                            break;
                        }
                    } else {
                        v5 = (grapple_beam_tmpD86 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 0x1ff as ::core::ffi::c_int) as uint16;
                        if v5 as ::core::ffi::c_int == grapple_beam_tmpD88 as ::core::ffi::c_int {
                            current_block = 5091811861639934416;
                        } else {
                            current_block = 11777552016271000781;
                        }
                    }
                }
                _ => {
                    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xcfa as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh7 =
                        (*fresh7 as ::core::ffi::c_int + tmpD9C as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xcfc as ::core::ffi::c_int as isize)
                        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xcfa as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd36 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh8 =
                        (*fresh8 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
                    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd30 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh9 = (*fresh9).wrapping_sub(1);
                    if *fresh9 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd30 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                    }
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xd2e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block = 12997042908615822766;
                        break;
                    } else {
                        current_block = 10399321362245223758;
                        break;
                    }
                }
            }
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                match current_block {
                    10399321362245223758 => {
                        v6 = (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd2e as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            + 6 as ::core::ffi::c_int) as uint16;
                        if ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd2e as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            + 6 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int)
                            < 0 as ::core::ffi::c_int
                        {
                            current_block = 9815132409580070441;
                        } else {
                            current_block = 13619784596304402172;
                        }
                    }
                    _ => {
                        v6 = (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd2e as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - 6 as ::core::ffi::c_int) as uint16;
                        if (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd2e as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - 6 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                        {
                            current_block = 9815132409580070441;
                        } else {
                            current_block = 13619784596304402172;
                        }
                    }
                }
                match current_block {
                    13619784596304402172 => {
                        v6 = 0 as uint16;
                    }
                    _ => {}
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
                    as *mut uint16) = v6;
                return 0 as uint8;
            }
        }
    }
    match current_block {
        15145243721762997867 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd30 as ::core::ffi::c_int as isize)
                as *mut uint16) = 16 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
                as *mut uint16) = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0xd26 as ::core::ffi::c_int as isize)
                as *mut uint16) as int16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
                as *mut uint16) = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0xd2e as ::core::ffi::c_int as isize)
                as *mut uint16) as int16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as uint16;
            return 1 as uint8;
        }
        _ => {
            let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xd36 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh3 = (*fresh3 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            return 1 as uint8;
        }
    };
}
pub unsafe fn BlockFunc_AEE3(game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        ^ *((&raw mut game.g_ram as *mut uint8).offset(0xcfa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd38 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    } else {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd38 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh22 = (*fresh22).wrapping_add(1);
        if *fresh22 as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd26 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn GrappleFunc_AF87(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd62 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_94B197 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) =
            addr_word_94B193 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 14 as ::core::ffi::c_int) as isize) =
            addr_word_94B18F as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 13 as ::core::ffi::c_int) as isize) =
            addr_word_94B18B as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd40 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd3e as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd3c as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        i -= 8 as ::core::ffi::c_int;
    }
}
unsafe fn GrappleInstr_Goto(mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_94(j as uint16_t, game) as *mut uint16);
}
pub unsafe fn CallGrappleInstr(mut ea: uint32, mut j: uint16, game: &mut Game) -> uint16 {
    match ea {
        9744628 => return GrappleInstr_Goto(j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn HandleGrappleBeamGfx(game: &mut Game) {
    let mut v1: ::core::ffi::c_int = CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd1a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int;
    let mut r28_r26: uint32 = ((kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int)
        << 11 as ::core::ffi::c_int) as uint32;
    let mut r32_r30: uint32 = ((kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int)
        << 11 as ::core::ffi::c_int) as uint32;
    let mut chr: uint16 = ((*(&raw mut game.g_ram as *mut uint8)
        .offset(0xcfb as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int) as uint16;
    chr = (chr as ::core::ffi::c_int
        | 2 as ::core::ffi::c_int
            * ((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                ^ chr as ::core::ffi::c_int)
                & 0x40 as ::core::ffi::c_int
                ^ 0x40 as ::core::ffi::c_int)) as uint16;
    chr = ((chr as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    let mut r20_r18: uint32 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd1a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32;
    let mut r24_r22: uint32 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut n: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xcfe as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        / 8 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        - 1 as ::core::ffi::c_int;
    let mut q: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        let mut i: ::core::ffi::c_int = 0;
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd42 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(q as isize);
        let fresh18 = *fresh17;
        *fresh17 = (*fresh17).wrapping_sub(1);
        if fresh18 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            let mut v9: *const uint16 = ::core::ptr::null::<uint16>();
            i = *((&raw mut game.g_ram as *mut uint8).offset(0xd62 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(q as isize) as ::core::ffi::c_int;
            loop {
                v9 = RomPtr_94(i as uint16_t, game) as *const uint16;
                if *v9 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    break;
                }
                i = CallGrappleInstr(
                    (*v9 as ::core::ffi::c_int | 0x940000 as ::core::ffi::c_int) as uint32,
                    (i + 2 as ::core::ffi::c_int) as uint16,
                    game,
                ) as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xd42 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(q as isize) = *v9;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd62 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(q as isize) = (i + 4 as ::core::ffi::c_int) as uint16;
        }
        if (r24_r22 | r20_r18) & 0xff000000 as uint32 != 0 as uint32 {
            break;
        }
        let mut v12: uint16 = *(RomPtr_94(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xd62 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(q as isize) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16);
        DrawGrappleOams(
            (r20_r18 >> 16 as ::core::ffi::c_int) as uint16,
            (r24_r22 >> 16 as ::core::ffi::c_int) as uint16,
            (chr as ::core::ffi::c_int | v12 as ::core::ffi::c_int) as uint16,
            game,
        );
        r20_r18 = r20_r18.wrapping_add(r28_r26);
        r24_r22 = r24_r22.wrapping_add(r32_r30);
        q -= 1;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_B2_FaceR_Grapple_Air as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_B3_FaceL_Grapple_Air as ::core::ffi::c_int
    {
        DrawGrappleOams3(game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DrawGrappleOams3(game);
    }
}
unsafe fn DrawGrappleOams(
    mut x_r20: uint16,
    mut y_r24: uint16,
    mut chr_r38: uint16,
    game: &mut Game,
) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut OamEnt = gOamEnt(
        *((&raw mut game.g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).xcoord = x_r20 as uint8;
    (*v2).ycoord = y_r24 as uint8;
    *(&raw mut (*v2).charnum as *mut uint16) = chr_r38;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v1 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
unsafe fn DrawGrappleOams3(game: &mut Game) {
    let mut idx: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut OamEnt = gOamEnt(idx as ::core::ffi::c_int, game);
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd08 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
    (*v2).xcoord = x as uint8;
    (*v2).ycoord = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd0c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint8;
    *(&raw mut (*v2).charnum as *mut uint16) = 14880 as uint16;
    let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x570 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((idx as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as isize);
    *fresh19 = (*fresh19 as ::core::ffi::c_int
        | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int)
            << 2 as ::core::ffi::c_int
                * (idx as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) = (idx as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
