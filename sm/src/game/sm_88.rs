extern "C" {
    static mut g_ram: [uint8; 131072];
    static mut g_rom: *const uint8;
    fn mov24(dst: *mut LongPtr, src: uint32);
    fn Load24(src: *const LongPtr) -> uint32;
    fn Unreachable() -> bool;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn WriteReg(reg: uint16, value: uint8);
    fn WriteRegWord(reg: uint16, value: uint16);
    fn Mult8x8(a: uint8, b: uint8) -> uint16;
    static kSinCosTable8bit_Sext: [int16; 320];
    fn CheckEventHappened(a: uint16) -> uint16;
    fn CheckBossBitForCurArea(a: uint16) -> uint8;
    fn HandleMusicQueue();
    fn QueueMusic_Delayed8(a: uint16);
    fn QueueSfx1_Max15(a: uint16);
    fn QueueSfx1_Max6(a: uint16);
    fn QueueSfx2_Max6(a: uint16);
    fn SetEventHappened(a: uint16);
    fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs);
    fn SpawnEprojWithRoomGfx(j: uint16, a: uint16);
    fn SpawnAnimtiles(j: uint16);
    fn ConfigureTitleSequenceGradientHDMA();
    fn CallSomeSamusCode(a: uint16) -> uint16;
    fn Hdmaobj_CrystalFlash() -> uint8;
    fn CanXrayShowBlocks() -> bool;
    fn CalculateXrayHdmaTableInner(
        k: uint16,
        j: uint16,
        r18: uint16,
        r20: uint16,
        off_screen: bool,
        dest_addr: *mut uint16,
    );
    fn HdmaobjPreInstr_XraySetup(k: uint16);
    fn InitializeSuitPickupHdma();
    fn ResponsibleForXrayStandupGlitch();
    fn SamusFunc_F433();
    fn Samus_LoadSuitPalette();
    fn Samus_SetAnimationFrameIfPoseChanged();
    fn Xray_SetupStage1_FreezeTimeBackup(k: uint16);
    fn Xray_SetupStage2_ReadBg1_2ndScreen();
    fn Xray_SetupStage3_ReadBg1_1stScreen();
    fn Xray_SetupStage4();
    fn Xray_SetupStage5();
    fn Xray_SetupStage6();
    fn Xray_SetupStage7();
    fn Xray_SetupStage8_SetBackdropColor();
    fn MotherBrain_CalculateRainbowBeamHdma();
}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int8, int8_t, uint16,
    uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, Func_U8, Func_V, Func_Y_V, Func_Y_Y,
    LongPtr, SnesRegs, VoidP, VramWriteEntry,
};
pub const MIRR7: SnesRegs = 17279;
pub const UNUSED7: SnesRegs = 17275;
pub const NTRL7: SnesRegs = 17274;
pub const A2A7H: SnesRegs = 17273;
pub const A2A7L: SnesRegs = 17272;
pub const DAS70: SnesRegs = 17271;
pub const DAS7H: SnesRegs = 17270;
pub const DAS7L: SnesRegs = 17269;
pub const A1B7: SnesRegs = 17268;
pub const A1T7H: SnesRegs = 17267;
pub const A1T7L: SnesRegs = 17266;
pub const BBAD7: SnesRegs = 17265;
pub const DMAP7: SnesRegs = 17264;
pub const MIRR6: SnesRegs = 17263;
pub const UNUSED6: SnesRegs = 17259;
pub const NTRL6: SnesRegs = 17258;
pub const A2A6H: SnesRegs = 17257;
pub const A2A6L: SnesRegs = 17256;
pub const DAS60: SnesRegs = 17255;
pub const DAS6H: SnesRegs = 17254;
pub const DAS6L: SnesRegs = 17253;
pub const A1B6: SnesRegs = 17252;
pub const A1T6H: SnesRegs = 17251;
pub const A1T6L: SnesRegs = 17250;
pub const BBAD6: SnesRegs = 17249;
pub const DMAP6: SnesRegs = 17248;
pub const MIRR5: SnesRegs = 17247;
pub const UNUSED5: SnesRegs = 17243;
pub const NTRL5: SnesRegs = 17242;
pub const A2A5H: SnesRegs = 17241;
pub const A2A5L: SnesRegs = 17240;
pub const DAS50: SnesRegs = 17239;
pub const DAS5H: SnesRegs = 17238;
pub const DAS5L: SnesRegs = 17237;
pub const A1B5: SnesRegs = 17236;
pub const A1T5H: SnesRegs = 17235;
pub const A1T5L: SnesRegs = 17234;
pub const BBAD5: SnesRegs = 17233;
pub const DMAP5: SnesRegs = 17232;
pub const MIRR4: SnesRegs = 17231;
pub const UNUSED4: SnesRegs = 17227;
pub const NTRL4: SnesRegs = 17226;
pub const A2A4H: SnesRegs = 17225;
pub const A2A4L: SnesRegs = 17224;
pub const DAS40: SnesRegs = 17223;
pub const DAS4H: SnesRegs = 17222;
pub const DAS4L: SnesRegs = 17221;
pub const A1B4: SnesRegs = 17220;
pub const A1T4H: SnesRegs = 17219;
pub const A1T4L: SnesRegs = 17218;
pub const BBAD4: SnesRegs = 17217;
pub const DMAP4: SnesRegs = 17216;
pub const MIRR3: SnesRegs = 17215;
pub const UNUSED3: SnesRegs = 17211;
pub const NTRL3: SnesRegs = 17210;
pub const A2A3H: SnesRegs = 17209;
pub const A2A3L: SnesRegs = 17208;
pub const DAS30: SnesRegs = 17207;
pub const DAS3H: SnesRegs = 17206;
pub const DAS3L: SnesRegs = 17205;
pub const A1B3: SnesRegs = 17204;
pub const A1T3H: SnesRegs = 17203;
pub const A1T3L: SnesRegs = 17202;
pub const BBAD3: SnesRegs = 17201;
pub const DMAP3: SnesRegs = 17200;
pub const MIRR2: SnesRegs = 17199;
pub const UNUSED2: SnesRegs = 17195;
pub const NTRL2: SnesRegs = 17194;
pub const A2A2H: SnesRegs = 17193;
pub const A2A2L: SnesRegs = 17192;
pub const DAS20: SnesRegs = 17191;
pub const DAS2H: SnesRegs = 17190;
pub const DAS2L: SnesRegs = 17189;
pub const A1B2: SnesRegs = 17188;
pub const A1T2H: SnesRegs = 17187;
pub const A1T2L: SnesRegs = 17186;
pub const BBAD2: SnesRegs = 17185;
pub const DMAP2: SnesRegs = 17184;
pub const MIRR1: SnesRegs = 17183;
pub const UNUSED1: SnesRegs = 17179;
pub const NTRL1: SnesRegs = 17178;
pub const A2A1H: SnesRegs = 17177;
pub const A2A1L: SnesRegs = 17176;
pub const DAS10: SnesRegs = 17175;
pub const DAS1H: SnesRegs = 17174;
pub const DAS1L: SnesRegs = 17173;
pub const A1B1: SnesRegs = 17172;
pub const A1T1H: SnesRegs = 17171;
pub const A1T1L: SnesRegs = 17170;
pub const BBAD1: SnesRegs = 17169;
pub const DMAP1: SnesRegs = 17168;
pub const MIRR0: SnesRegs = 17167;
pub const UNUSED0: SnesRegs = 17163;
pub const NTRL0: SnesRegs = 17162;
pub const A2A0H: SnesRegs = 17161;
pub const A2A0L: SnesRegs = 17160;
pub const DAS00: SnesRegs = 17159;
pub const DAS0H: SnesRegs = 17158;
pub const DAS0L: SnesRegs = 17157;
pub const A1B0: SnesRegs = 17156;
pub const A1T0H: SnesRegs = 17155;
pub const A1T0L: SnesRegs = 17154;
pub const BBAD0: SnesRegs = 17153;
pub const DMAP0: SnesRegs = 17152;
pub const JOY4H: SnesRegs = 16927;
pub const JOY4L: SnesRegs = 16926;
pub const JOY3H: SnesRegs = 16925;
pub const JOY3L: SnesRegs = 16924;
pub const JOY2H: SnesRegs = 16923;
pub const JOY2L: SnesRegs = 16922;
pub const JOY1H: SnesRegs = 16921;
pub const JOY1L: SnesRegs = 16920;
pub const RDMPYH: SnesRegs = 16919;
pub const RDMPYL: SnesRegs = 16918;
pub const RDDIVH: SnesRegs = 16917;
pub const RDDIVL: SnesRegs = 16916;
pub const RDIO: SnesRegs = 16915;
pub const HVBJOY: SnesRegs = 16914;
pub const TIMEUP: SnesRegs = 16913;
pub const RDNMI: SnesRegs = 16912;
pub const MEMSEL: SnesRegs = 16909;
pub const HDMAEN: SnesRegs = 16908;
pub const MDMAEN: SnesRegs = 16907;
pub const VTIMEH: SnesRegs = 16906;
pub const VTIMEL: SnesRegs = 16905;
pub const HTIMEH: SnesRegs = 16904;
pub const HTIMEL: SnesRegs = 16903;
pub const WRDIVB: SnesRegs = 16902;
pub const WRDIVH: SnesRegs = 16901;
pub const WRDIVL: SnesRegs = 16900;
pub const WRMPYB: SnesRegs = 16899;
pub const WRMPYA: SnesRegs = 16898;
pub const WRIO: SnesRegs = 16897;
pub const NMITIMEN: SnesRegs = 16896;
pub const JOYB: SnesRegs = 16407;
pub const JOYA: SnesRegs = 16406;
pub const WMADDH: SnesRegs = 8579;
pub const WMADDM: SnesRegs = 8578;
pub const WMADDL: SnesRegs = 8577;
pub const WMDATA: SnesRegs = 8576;
pub const APUI03: SnesRegs = 8515;
pub const APUI02: SnesRegs = 8514;
pub const APUI01: SnesRegs = 8513;
pub const APUI00: SnesRegs = 8512;
pub const STAT78: SnesRegs = 8511;
pub const STAT77: SnesRegs = 8510;
pub const OPVCT: SnesRegs = 8509;
pub const OPHCT: SnesRegs = 8508;
pub const RDCGRAM: SnesRegs = 8507;
pub const RDVRAMH: SnesRegs = 8506;
pub const RDVRAML: SnesRegs = 8505;
pub const RDOAM: SnesRegs = 8504;
pub const SLHV: SnesRegs = 8503;
pub const MPYH: SnesRegs = 8502;
pub const MPYM: SnesRegs = 8501;
pub const MPYL: SnesRegs = 8500;
pub const SETINI: SnesRegs = 8499;
pub const COLDATA: SnesRegs = 8498;
pub const CGADSUB: SnesRegs = 8497;
pub const CGWSEL: SnesRegs = 8496;
pub const TSW: SnesRegs = 8495;
pub const TMW: SnesRegs = 8494;
pub const TS: SnesRegs = 8493;
pub const TM: SnesRegs = 8492;
pub const WOBJLOG: SnesRegs = 8491;
pub const WBGLOG: SnesRegs = 8490;
pub const WH3: SnesRegs = 8489;
pub const WH2: SnesRegs = 8488;
pub const WH1: SnesRegs = 8487;
pub const WH0: SnesRegs = 8486;
pub const WOBJSEL: SnesRegs = 8485;
pub const W34SEL: SnesRegs = 8484;
pub const W12SEL: SnesRegs = 8483;
pub const CGDATA: SnesRegs = 8482;
pub const CGADD: SnesRegs = 8481;
pub const M7Y: SnesRegs = 8480;
pub const M7X: SnesRegs = 8479;
pub const M7D: SnesRegs = 8478;
pub const M7C: SnesRegs = 8477;
pub const M7B: SnesRegs = 8476;
pub const M7A: SnesRegs = 8475;
pub const M7SEL: SnesRegs = 8474;
pub const VMDATAH: SnesRegs = 8473;
pub const VMDATAL: SnesRegs = 8472;
pub const VMADDH: SnesRegs = 8471;
pub const VMADDL: SnesRegs = 8470;
pub const VMAIN: SnesRegs = 8469;
pub const BG4VOFS: SnesRegs = 8468;
pub const BG4HOFS: SnesRegs = 8467;
pub const BG3VOFS: SnesRegs = 8466;
pub const BG3HOFS: SnesRegs = 8465;
pub const BG2VOFS: SnesRegs = 8464;
pub const BG2HOFS: SnesRegs = 8463;
pub const BG1VOFS: SnesRegs = 8462;
pub const BG1HOFS: SnesRegs = 8461;
pub const BG34NBA: SnesRegs = 8460;
pub const BG12NBA: SnesRegs = 8459;
pub const BG4SC: SnesRegs = 8458;
pub const BG3SC: SnesRegs = 8457;
pub const BG2SC: SnesRegs = 8456;
pub const BG1SC: SnesRegs = 8455;
pub const MOSAIC: SnesRegs = 8454;
pub const BGMODE: SnesRegs = 8453;
pub const OAMDATA: SnesRegs = 8452;
pub const OAMADDH: SnesRegs = 8451;
pub const OAMADDL: SnesRegs = 8450;
pub const OBSEL: SnesRegs = 8449;
pub const INIDISP: SnesRegs = 8448;
pub use crate::types::EnemyData;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct HdmaScrollEntry {
    pub top_pos: uint16,
    pub scroll_subspeed: uint16,
    pub scroll_speed: uint16,
    pub hdma_data_table_entry: VoidP,
}
pub use crate::types::{SamusPose, SpawnHdmaObject_Args};
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
pub use crate::types::{Consts_86, ExpandingSquareTransitionHdma, Ram4000, Ram4000_Backups};
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
pub use crate::types::Consts_87;
pub const addr_kAnimtiles_TourianStatue_Draygon: Consts_87 = 34142;
pub const addr_kAnimtiles_TourianStatue_Kraid: Consts_87 = 34136;
pub const addr_kAnimtiles_TourianStatue_Ridley: Consts_87 = 34130;
pub const addr_kAnimtiles_TourianStatue_Phantoon: Consts_87 = 34124;
pub const addr_kAnimtiles_Spores: Consts_87 = 33533;
pub const addr_kAnimtiles_Rain: Consts_87 = 33511;
pub const addr_kAnimtiles_Acid: Consts_87 = 33481;
pub const addr_kAnimtiles_Lava: Consts_87 = 33451;
pub const addr_kAnimtiles_WreckedShipTradmillLeft: Consts_87 = 33403;
pub const addr_kAnimtiles_WreckedShipTradmillRight: Consts_87 = 33397;
pub use crate::types::Consts_88;
pub const addr_word_88DF71: Consts_88 = 57201;
pub const addr_word_88DF6B: Consts_88 = 57195;
pub const addr_word_88DF65: Consts_88 = 57189;
pub const addr_word_88DF5F: Consts_88 = 57183;
pub const addr_off_88ADA6: Consts_88 = 44454;
pub const addr_off_88AD9C: Consts_88 = 44444;
pub const addr_word_88A8E8: Consts_88 = 43240;
pub const addr_byte_889F06: Consts_88 = 40710;
pub const addr_kPowerBombExplosionShapeDef0: Consts_88 = 37446;
pub const addr_loc_889180: Consts_88 = 37248;
pub const addr_locret_88858A: Consts_88 = 34186;
pub use crate::types::Consts_89;
pub const addr_kIndirectHdmaTable_PowerBombExplodeRight: Consts_89 = 41217;
pub const addr_kIndirectHdmaTable_PowerBombExplodeLeft: Consts_89 = 38912;
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
pub use crate::types::SpawnHardcodedPlmArgs;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe extern "C" fn __CFADD__uint8(mut x: uint8, mut y: uint8) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn RomPtr_88(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x880000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
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
unsafe extern "C" fn gVramWriteEntry(mut a: ::core::ffi::c_int) -> *mut VramWriteEntry {
    return (&raw mut g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
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
pub const fnnullsub_56: uint32 = 8491739 as uint32;
pub const fnDoorTransitionFunction_FadeOutScreen: ::core::ffi::c_int =
    0x82e2db as ::core::ffi::c_int;
pub const fnDoorTransition_FadeInScreenAndFinish: ::core::ffi::c_int =
    0x82e737 as ::core::ffi::c_int;
pub const fnLayerBlendingHandler: ::core::ffi::c_int = 0x888000 as ::core::ffi::c_int;
pub const fnPauseHook_Empty: ::core::ffi::c_int = 0x8883e1 as ::core::ffi::c_int;
pub const fnnullsub_293: ::core::ffi::c_int = 0x8884b8 as ::core::ffi::c_int;
pub const fnHdmaobjInstr_Delete: uint32 = 8947049 as uint32;
pub const fnHdmaobjInstr_SetPreInstr: uint32 = 8947056 as uint32;
pub const fnHdmaobjInstr_ClearPreInstr: uint32 = 8947076 as uint32;
pub const fnHdmaobjInstr_CallFarFunc: uint32 = 8947124 as uint32;
pub const fnHdmaobjInstr_Goto: uint32 = 8947180 as uint32;
pub const fnHdmaobjInstr_GotoRel: uint32 = 8947185 as uint32;
pub const fnHdmaobjInstr_DecrementAndGoto: uint32 = 8947207 as uint32;
pub const fnHdmaobjInstr_DecrementAndGotoRel: uint32 = 8947215 as uint32;
pub const fnHdmaobjInstr_SetTimer: uint32 = 8947222 as uint32;
pub const fnHdmaobjInstr_SetHdmaControl: uint32 = 8947234 as uint32;
pub const fnHdmaobjInstr_SetHdmaTarget: uint32 = 8947255 as uint32;
pub const fnHdmaobjInstr_SetHdmaTablePtr: uint32 = 8947276 as uint32;
pub const fnHdmaobjInstr_SetHdmaTableBank: uint32 = 8947285 as uint32;
pub const fnHdmaobjInstr_SetIndirectHdmaDataBank: uint32 = 8947306 as uint32;
pub const fnHdmaobjInstr_Sleep: uint32 = 8947330 as uint32;
pub const fnHdmaobjPreInstr_Xray: uint32 = 8947439 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc0_NoBeam: uint32 = 8947506 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc1_BeamWidening: uint32 = 8947540 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc2_FullBeam: uint32 = 8947627 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc3_DeactivateBeam: uint32 = 8948020 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc4_DeactivateBeam: uint32 = 8948154 as uint32;
pub const fnHdmaobjPreInstr_XrayFunc5_DeactivateBeam: uint32 = 8948232 as uint32;
pub const fnHdmaobj_PreExplodeWhite: uint32 = 8948500 as uint32;
pub const fnHdmaobj_PreExplodeYellow: uint32 = 8948530 as uint32;
pub const fnHdmaobj_ExplodeYellow: uint32 = 8948537 as uint32;
pub const fnHdmaobj_ExplodeWhite: uint32 = 8948551 as uint32;
pub const fnHdmaobj_CleanUpTryCrystalFlash: uint32 = 8948558 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_SetWindowConf: uint32 = 8948623 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow: uint32 = 8948632 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionYellow: uint32 = 8949225 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionWhite: uint32 = 8949426 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionWhite: uint32 = 8949983 as uint32;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionYellow: uint32 = 8950184 as uint32;
pub const fnCrystalFlashSetupPart1: uint32 = 8954596 as uint32;
pub const fnCrystalFlashSetupPart2: uint32 = 8954633 as uint32;
pub const fnCrystalFlashCleanup: uint32 = 8954647 as uint32;
pub const fnHdmaobjPreInstr_CrystalFlash_CustomLayerBlend: uint32 = 8954681 as uint32;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow: uint32 = 8954717 as uint32;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage1_Explosion: uint32 = 8955218 as uint32;
pub const fnHdmaobjPreInstr_FxType22_BG3Yscroll: uint32 = 8955459 as uint32;
pub const fnHdmaobjInstr_SetFlagB: uint32 = 8955500 as uint32;
pub const fnHdmaobjPreInstr_BG3Xscroll: uint32 = 8955507 as uint32;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll: uint32 = 8957362 as uint32;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll2: uint32 = 8957370 as uint32;
pub const fnHdmaobjPreInstr_SkyLandBG2XscrollInner: uint32 = 8957378 as uint32;
pub const fnHdmaobjPreInstr_FirefleaBG3XScroll: uint32 = 8958140 as uint32;
pub const fnFxRisingFunction_LavaAcid: ::core::ffi::c_int = 0x88b343 as ::core::ffi::c_int;
pub const fnFxRisingFunction_LavaAcid_WaitToRise: ::core::ffi::c_int =
    0x88b367 as ::core::ffi::c_int;
pub const fnFxRisingFunction_LavaAcid_Raising: ::core::ffi::c_int = 0x88b382 as ::core::ffi::c_int;
pub const fnHdmaobjInstr_B3A9: uint32 = 8958889 as uint32;
pub const fnHdmaobjPreInstr_LavaAcidBG3YScroll: uint32 = 8958896 as uint32;
pub const fnHdmaobjInstr_SetFlagB_Copy: uint32 = 8959182 as uint32;
pub const fnHdmaobjPreInstr_LavaAcidBG2YScroll: uint32 = 8959189 as uint32;
pub const fnFxRisingFunction_C428_WaterNormal: ::core::ffi::c_int = 0x88c428 as ::core::ffi::c_int;
pub const fnFxRisingFunction_WaterWaitToRise: ::core::ffi::c_int = 0x88c44c as ::core::ffi::c_int;
pub const fnFxRisingFunction_WaterRising: ::core::ffi::c_int = 0x88c458 as ::core::ffi::c_int;
pub const fnHdmaobjInstr_SetFlagB_Copy2: uint32 = 8963175 as uint32;
pub const fnHdmaobjPreInstr_WaterBG3XScroll: uint32 = 8963214 as uint32;
pub const fnHdmaobjInstr_SetFlagB_Copy3: uint32 = 8963458 as uint32;
pub const fnHdmaobjPreInstr_WaterBG2XScroll: uint32 = 8963465 as uint32;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func2: uint32 = 8963556 as uint32;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func1: uint32 = 8963638 as uint32;
pub const fnsub_88D916: uint32 = 8968470 as uint32;
pub const fnHdmaobjInstr_SetVideoMode1: uint32 = 8968521 as uint32;
pub const fnHdmaobjInstr_1938_RandomNumber: uint32 = 8968577 as uint32;
pub const fnHdmaobjPreInstr_RainBg3Scroll: uint32 = 8968609 as uint32;
pub const fnHdmaobjPreInstr_SporesBG3Xscroll: uint32 = 8968775 as uint32;
pub const fnnullsub_112: uint32 = 8969006 as uint32;
pub const fnHdmaobjPreInstr_FogBG3Scroll: uint32 = 8969014 as uint32;
pub const fnHdmaobjPreInstr_CheckLotsOfEventsHappened: uint32 = 8969175 as uint32;
pub const fnHdmaobjPreInstr_DC23: uint32 = 8969251 as uint32;
pub const fnHdmaobjPreInstr_DC69: uint32 = 8969321 as uint32;
pub const fnHdmaobjPreInstr_DCBA: uint32 = 8969402 as uint32;
pub const fnHdmaobjInstr_GotoIfEventHappened: uint32 = 8969419 as uint32;
pub const fnHdmaobjPreInstr_BombTorizoHazeColorMathBgColor: uint32 = 8969539 as uint32;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive: uint32 = 8969744 as uint32;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead: uint32 = 8969749 as uint32;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingIn: ::core::ffi::c_int =
    0x88de2d as ::core::ffi::c_int;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadedIn: ::core::ffi::c_int =
    0x88de74 as ::core::ffi::c_int;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingOut: ::core::ffi::c_int =
    0x88de96 as ::core::ffi::c_int;
pub const fnnullsub_113: uint32 = 8970129 as uint32;
pub const fnnullsub_114: uint32 = 8970130 as uint32;
pub const fnHdmaobjPreInstr_DF94: uint32 = 8970132 as uint32;
pub const fnHdmaobjPreInstr_VariaSuitPickup: uint32 = 8970278 as uint32;
pub const fnHdmaobjPreInstr_GravitySuitPickup: uint32 = 8970332 as uint32;
pub const fnHdmaobjPreInstr_E449: uint32 = 8971337 as uint32;
pub const fnHdmaobjInstr_E4BD: uint32 = 8971453 as uint32;
pub const fnHdmaobjPreInstr_E567: uint32 = 8971623 as uint32;
pub const fnnullsub_357: uint32 = 8972070 as uint32;
pub const fnInitializeRainbowBeam: uint32 = 8972135 as uint32;
pub const fnHdmaobjPreInstr_E7BC: uint32 = 8972220 as uint32;
pub const fnHdmaobjInstr_InitMorphBallEyeBeamHdma: uint32 = 8972567 as uint32;
pub const fnHdmaobjPreInstr_E9E6: uint32 = 8972774 as uint32;
pub const fnHdmaobjPreInstr_EA3C: uint32 = 8972860 as uint32;
pub const fnHdmaobjPreInstr_EACB: uint32 = 8973003 as uint32;
pub const fnHdmaobjInsr_ConfigTitleSequenceGradientHDMA: uint32 = 8973215 as uint32;
pub const fnHdmaobjPreInstr_Backdrop_TitleSequenceGradient: uint32 = 8973232 as uint32;
pub const fnHdmaobjPreInstr_ColorMathControlB_TitleGradient: uint32 = 8973266 as uint32;
pub const fnHdmaobjPreInstr_IntroCutsceneCrossfade: uint32 = 8973341 as uint32;
pub const fnHdmaobjInstr_EC9F_ClearVars: uint32 = 8973471 as uint32;
pub const fnHdmaobjPreInstr_ECB6: uint32 = 8973494 as uint32;
pub const fnCinematicFunctionOpening: ::core::ffi::c_int = 0x8b9b68 as ::core::ffi::c_int;
pub const fnXray_SetupStage1_FreezeTimeBackup: uint32 = 9554681 as uint32;
pub const fnXray_SetupStage2_ReadBg1_2ndScreen: uint32 = 9554716 as uint32;
pub const fnXray_SetupStage3_ReadBg1_1stScreen: uint32 = 9554775 as uint32;
pub const fnXray_SetupStage4: uint32 = 9554830 as uint32;
pub const fnXray_SetupStage5: uint32 = 9556179 as uint32;
pub const fnXray_SetupStage6: uint32 = 9556339 as uint32;
pub const fnXray_SetupStage7: uint32 = 9556384 as uint32;
pub const fnHdmaobjPreInstr_XraySetup: uint32 = 9556607 as uint32;
pub const fnXray_SetupStage8_SetBackdropColor: uint32 = 9556668 as uint32;
pub const fnInitializeSuitPickupHdma: uint32 = 9557650 as uint32;
pub const fnnullsub_309: uint32 = 10785852 as uint32;
#[no_mangle]
pub unsafe extern "C" fn nullsub_9(mut j: uint16) -> uint16 {
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeLayerBlending() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 51 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_4_PhantoonIntro(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_6(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 20 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 39 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_8(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 20 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 34 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_A(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 50 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_C(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(94 as ::core::ffi::c_int) as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn nullsub_10(mut j: uint16) -> uint16 {
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_10(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_14(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(77 as ::core::ffi::c_int) as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_16(mut j: uint16) -> uint16 {
    *(&raw mut j as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 6 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(79 as ::core::ffi::c_int) as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_1A(mut j: uint16) -> uint16 {
    *(&raw mut j as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 21 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 53 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_1C(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 21 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 85 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_18(mut j: uint16) -> uint16 {
    j = 2 as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0x30 as ::core::ffi::c_int
        == 0x30 as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 36 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 19 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 4 as uint8;
    }
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn nullsub_11(mut j: uint16) -> uint16 {
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_26(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 119 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_28(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(77 as ::core::ffi::c_int) as uint8;
    if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) = 37 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    }
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_2A(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(77 as ::core::ffi::c_int) as uint8;
    if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) = 38 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = 66 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    }
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_2C(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 0 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_2E(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(77 as ::core::ffi::c_int) as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_32(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 68 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(78 as ::core::ffi::c_int) as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_34(mut j: uint16) -> uint16 {
    j = 6 as uint16;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendFunc_24(mut j: uint16) -> uint16 {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 16 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 51 as uint8;
    return j;
}
#[no_mangle]
pub unsafe extern "C" fn HandleLayerBlendingXrayCanShowBlocks() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) =
        -(56 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 3 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 34 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        (*(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            | 0x73 as ::core::ffi::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn HandleLayerBlendingXrayCantShowBlocks() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 3 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 34 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        (*(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            | 0x61 as ::core::ffi::c_int) as uint8;
    if *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == addr_kRoom_cefb as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HandleLayerBlendingXrayFirefleaRoom() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 3 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(77 as ::core::ffi::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendPowerBombFunc_0() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 55 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendPowerBombFunc_4() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 55 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 6 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 6 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn LayerBlendPowerBombFunc_6() {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 49 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
}
static mut kLayerBlendFuncTable: [Option<Func_Y_Y>; 27] = unsafe {
    [
        Some(nullsub_9 as unsafe extern "C" fn(uint16) -> uint16),
        Some(nullsub_9 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_4_PhantoonIntro as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_6 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_8 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_A as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_C as unsafe extern "C" fn(uint16) -> uint16),
        Some(nullsub_10 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_10 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_10 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_14 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_16 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_18 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_1A as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_1C as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_18 as unsafe extern "C" fn(uint16) -> uint16),
        Some(nullsub_11 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_14 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_24 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_26 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_28 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_2A as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_2C as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_2E as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_18 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_32 as unsafe extern "C" fn(uint16) -> uint16),
        Some(LayerBlendFunc_34 as unsafe extern "C" fn(uint16) -> uint16),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn LayerBlendingHandler() {
    let mut v0: uint16 = 0 as uint16;
    let mut v1: uint8 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8;
    if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8
        != 0
    {
        InitializeLayerBlending();
        v0 = kLayerBlendFuncTable[(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(v0);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            HandleLayerBlendingXrayCanShowBlocks();
        } else if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x2000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            HandleLayerBlendingXrayCantShowBlocks();
        } else if *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            HandleLayerBlendingXrayFirefleaRoom();
        }
    } else {
        HandleLayerBlendingPowerBomb(v0);
    };
}
static mut kLayerBlendPowerBombFuncs: [Option<Func_V>; 4] = unsafe {
    [
        Some(LayerBlendPowerBombFunc_0 as unsafe extern "C" fn() -> ()),
        Some(LayerBlendPowerBombFunc_0 as unsafe extern "C" fn() -> ()),
        Some(LayerBlendPowerBombFunc_4 as unsafe extern "C" fn() -> ()),
        Some(LayerBlendPowerBombFunc_6 as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn HandleLayerBlendingPowerBomb(mut j: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == addr_kRoom_a66a as ::core::ffi::c_int
    {
        j = 6 as uint16;
    }
    kLayerBlendPowerBombFuncs[(j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn EnableHdmaObjects() {
    let ref mut fresh31 = *((&raw mut g_ram as *mut uint8)
        .offset(0x18b0 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh31 = (*fresh31 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DisableHdmaObjects() {
    let ref mut fresh30 = *((&raw mut g_ram as *mut uint8)
        .offset(0x18b0 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh30 = (*fresh30 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn WaitUntilEndOfVblankAndClearHdma() {
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0 as uint8);
    sub_8882AC();
}
#[no_mangle]
pub unsafe extern "C" fn sub_8882AC() {
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 0 as uint8;
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitializeSpecialEffectsForNewRoom() {
    *((&raw mut g_ram as *mut uint8).offset(0x607 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x609 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == addr_kRoom_9804 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_96ba as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_b32e as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_b457 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_dd58 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_dede as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x609 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut i: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    while i != 0x80 as ::core::ffi::c_int {
        WriteReg(
            (i + 17152 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17153 as ::core::ffi::c_int) as SnesRegs as uint16,
            0x13 as uint8,
        );
        WriteReg(
            (i + 17154 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17155 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17156 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17157 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17158 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17160 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        WriteReg(
            (i + 17161 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
        );
        i += 16 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x195c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1960 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcad8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcada as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcadc as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x941 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1e6f as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1e6d as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1988 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1964 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0x8000 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1980 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbb as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa9 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x601 as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut VoidP) =
        -(30720 as ::core::ffi::c_int) as VoidP;
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x604 as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut VoidP) =
        -(30720 as ::core::ffi::c_int) as VoidP;
    (*((&raw mut g_ram as *mut uint8).offset(0x601 as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr = (0x8883e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    (*((&raw mut g_ram as *mut uint8).offset(0x604 as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr = (0x8883e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    WriteReg(WMADDL as ::core::ffi::c_int as uint16, 0xf0 as uint8);
    WriteReg(WMADDM as ::core::ffi::c_int as uint16, 0xff as uint8);
    WriteReg(WMADDH as ::core::ffi::c_int as uint16, 1 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6f as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x72 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5e as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 73 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5a as ::core::ffi::c_int as isize) = 90 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 90 as uint8;
}
unsafe extern "C" fn SpawnHdmaObjectInner(
    mut k: uint16,
    mut p: *mut uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r24: uint16,
) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x8884b8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18fc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 136 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *p.offset(1 as ::core::ffi::c_int as isize);
    *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = swap16(r18);
    *((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (r24 as ::core::ffi::c_int | r20 as ::core::ffi::c_int) as uint16;
    WriteRegWord(
        (r20 as ::core::ffi::c_int + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *p,
    );
    return k;
}
#[no_mangle]
pub unsafe extern "C" fn SpawnHdmaObject(
    mut db: uint8,
    mut p: *const ::core::ffi::c_void,
) -> uint16 {
    let mut r18: uint16 = 1024 as uint16;
    let mut r20: uint16 = 32 as uint16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        if *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            == 0
        {
            return SpawnHdmaObjectInner(
                v3,
                p as *mut uint16,
                r18,
                r20,
                ((db as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
            );
        }
        let mut v4: bool = r18 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r18 = (r18 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if v4 {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
        r20 = (r20 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int == 12 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SpawnHdmaObjectToSlot0xA(
    mut db: uint8,
    mut p: *const ::core::ffi::c_void,
) {
    SpawnHdmaObjectInner(
        0xa as uint16,
        p as *mut uint16,
        0x8000 as uint16,
        0x70 as uint16,
        ((db as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaObjectHandler() {
    let mut v1: int8 = 0;
    HandleMusicQueue();
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x8000 as uint16;
        static mut unk_8884D5: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x28 as uint8,
            addr: 0x8ace as VoidP,
        };
        static mut unk_8884DD: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x29 as uint8,
            addr: 0x8b80 as VoidP,
        };
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_8884D5 as *const ::core::ffi::c_void,
        );
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_8884DD as *const ::core::ffi::c_void,
        );
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1982 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0x18b0 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 0 as uint8;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                as *mut uint16) = i as uint16;
            v1 = *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as isize) as int8;
            if v1 != 0 {
                let ref mut fresh58 =
                    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize);
                *fresh58 = (*fresh58 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint8;
                HdmaobjInstructionHandler(i as uint8);
                i = *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int;
            }
            i += 2 as ::core::ffi::c_int;
            if !(i != 12 as ::core::ffi::c_int) {
                break;
            }
        }
        LayerBlendingHandler();
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstructionHandler(mut k: uint8) {
    let mut kh: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    CallHdmaobjPreInstr(
        ((*((&raw mut g_ram as *mut uint8).offset(0x18fc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(kh as isize) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int) as uint32,
        k as uint16,
    );
    let ref mut fresh59 = *((&raw mut g_ram as *mut uint8)
        .offset(0x18e4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(kh as isize);
    let fresh60 = *fresh59;
    *fresh59 = (*fresh59).wrapping_sub(1);
    if fresh60 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut base: *const uint8 = RomBankBase(
            *((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize),
        );
        let mut p: *const uint8 = base.offset(
            *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int as isize,
        );
        while *(p as *mut uint16) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            p = CallHdmaobjInstr(
                (*(p as *mut uint16) as ::core::ffi::c_int | 0x880000 as ::core::ffi::c_int)
                    as uint32,
                k as uint16,
                p.offset(2 as ::core::ffi::c_int as isize),
            );
            if (p as uintptr_t) < 0x10000 as ::core::ffi::c_int as uintptr_t {
                if p.is_null() {
                    return;
                }
                p = base.offset(p as uintptr_t as isize);
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(kh as isize) = *(p as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(kh as isize) = *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(kh as isize) = p.offset(4 as ::core::ffi::c_int as isize).offset_from(base)
            as ::core::ffi::c_long as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_Delete(mut k: uint16, mut hdp: *const uint8) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return ::core::ptr::null::<uint8>();
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetPreInstr(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(hdp as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x18fc as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *hdp.offset(2 as ::core::ffi::c_int as isize);
    return hdp.offset(3 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_ClearPreInstr(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        addr_locret_88858A as ::core::ffi::c_int as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn CallHdmaobjInstrFunc(mut ea: uint32, mut k: uint16) {
    match ea {
        8948500 => {
            Hdmaobj_PreExplodeWhite();
            return;
        }
        8948530 => {
            Hdmaobj_PreExplodeYellow();
            return;
        }
        8948537 => {
            Hdmaobj_ExplodeYellow();
            return;
        }
        8948551 => {
            Hdmaobj_ExplodeWhite();
            return;
        }
        8948558 => {
            Hdmaobj_CleanUpTryCrystalFlash(k);
            return;
        }
        8954596 => {
            CrystalFlashSetupPart1();
            return;
        }
        8954633 => {
            CrystalFlashSetupPart2();
            return;
        }
        8954647 => {
            CrystalFlashCleanup(k);
            return;
        }
        8970129 => return,
        8970130 => return,
        8972070 => return,
        8972135 => {
            InitializeRainbowBeam();
            return;
        }
        9554681 => {
            Xray_SetupStage1_FreezeTimeBackup(k);
            return;
        }
        9554716 => {
            Xray_SetupStage2_ReadBg1_2ndScreen();
            return;
        }
        9554775 => {
            Xray_SetupStage3_ReadBg1_1stScreen();
            return;
        }
        9554830 => {
            Xray_SetupStage4();
            return;
        }
        9556179 => {
            Xray_SetupStage5();
            return;
        }
        9556339 => {
            Xray_SetupStage6();
            return;
        }
        9556384 => {
            Xray_SetupStage7();
            return;
        }
        9556668 => {
            Xray_SetupStage8_SetBackdropColor();
            return;
        }
        9557650 => {
            InitializeSuitPickupHdma();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_CallFarFunc(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    CallHdmaobjInstrFunc(Load24(hdp as *mut LongPtr), k);
    return hdp.offset(3 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_Goto(mut k: uint16, mut hdp: *const uint8) -> *const uint8 {
    return *(hdp as *mut uint16) as uintptr_t as *const uint8;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_GotoRel(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    return hdp.offset(*hdp as int8 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_DecrementAndGoto(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh63 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1908 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh64 = *fresh63;
    *fresh63 = (*fresh63).wrapping_sub(1);
    if fresh64 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return hdp.offset(2 as ::core::ffi::c_int as isize);
    } else {
        return *(hdp as *mut uint16) as uintptr_t as *const uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_DecrementAndGotoRel(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh61 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1908 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh62 = *fresh61;
    *fresh61 = (*fresh61).wrapping_sub(1);
    if fresh62 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return hdp.offset(1 as ::core::ffi::c_int as isize);
    } else {
        return hdp.offset(*hdp as int8 as ::core::ffi::c_int as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetTimer(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *hdp;
    return hdp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetHdmaControl(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
    );
    return hdp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetHdmaTarget(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + BBAD0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
    );
    return hdp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetHdmaTablePtr(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(hdp as *mut uint16);
    return hdp.offset(2 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetHdmaTableBank(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + A1B0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
    );
    return hdp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetIndirectHdmaDataBank(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + DAS00 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
    );
    return hdp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_Sleep(mut k: uint16, mut hdp: *const uint8) -> *const uint8 {
    let mut base: *const uint8 = RomBankBase(
        *((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize),
    );
    *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (hdp.offset_from(base) as ::core::ffi::c_long - 2 as ::core::ffi::c_long) as uint16;
    return ::core::ptr::null::<uint8>();
}
#[no_mangle]
pub unsafe extern "C" fn RaiseOrLowerFx() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    let mut r24_r22: uint32 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    if *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 = (r24_r22.wrapping_add(
            (*((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        ) >> 16 as ::core::ffi::c_int) as uint16;
        let ref mut fresh13 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1976 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh13 = (*fresh13 as uint32).wrapping_add(r24_r22) as uint16 as uint16;
        if v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v2 = -(1 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        if *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >= v2 as ::core::ffi::c_int
        {
            return 0 as uint8;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            return 1 as uint8;
        }
    } else {
        let mut v0: uint16 = (r24_r22.wrapping_add(
            (*((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        ) >> 16 as ::core::ffi::c_int) as uint16;
        let ref mut fresh14 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1976 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh14 = (*fresh14 as uint32).wrapping_add(r24_r22) as uint16 as uint16;
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v0 = 0 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
        if *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >= v0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        return (*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >= v0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc0_NoBeam(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CalculateXrayHdmaTable();
        let ref mut fresh137 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh137 = (*fresh137).wrapping_add(1);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc1_BeamWidening(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
                as *mut uint16,
            2048 as uint32,
        );
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let ref mut fresh136 = *((&raw mut g_ram as *mut uint8)
                .offset(0xa7a as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh136 = (*fresh136).wrapping_add(1);
        }
        CalculateXrayHdmaTable();
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc2_FullBeam(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        HandleMovingXrayUpDown();
        CalculateXrayHdmaTable();
    } else {
        let ref mut fresh135 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh135 = (*fresh135).wrapping_add(1);
    };
}
#[no_mangle]
pub unsafe extern "C" fn HandleMovingXrayUpDown() {
    if *((&raw mut g_ram as *mut uint8).offset(0x9aa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MoveXrayUp();
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9ac as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MoveXrayDown();
    }
}
#[no_mangle]
pub unsafe extern "C" fn MoveXrayUp() {
    let mut v0: bool = false;
    let mut v1: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            if ((*((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
                || {
                    v0 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int;
                    let ref mut fresh52 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh52 = (*fresh52).wrapping_sub(1);
                    ((*((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (!v0 as ::core::ffi::c_int
                            + *((&raw mut g_ram as *mut uint8)
                                .offset(0xa84 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                as ::core::ffi::c_int)) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                }
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                    .offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            }
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 256 as ::core::ffi::c_int
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
            || {
                v1 = (((*((&raw mut g_ram as *mut uint8)
                    .offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                    .wrapping_add(
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xa82 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32)
                            << 16 as ::core::ffi::c_int
                            | 1 as ::core::ffi::c_int as uint16 as uint32,
                    )
                    >> 16 as ::core::ffi::c_int) as int16;
                let ref mut fresh53 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh53 = (*fresh53).wrapping_add(1);
                v1 as ::core::ffi::c_int != 256 as ::core::ffi::c_int
                    && (v1 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
            }
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) = (256 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MoveXrayDown() {
    let mut v0: int16 = 0;
    let mut v1: bool = false;
    if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 128 as ::core::ffi::c_int
        {
            if (*((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
                || {
                    v0 = (((*((&raw mut g_ram as *mut uint8)
                        .offset(0xa84 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xa82 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32)
                        .wrapping_add(
                            (*((&raw mut g_ram as *mut uint8)
                                .offset(0xa82 as ::core::ffi::c_int as isize)
                                as *mut uint16) as uint32)
                                << 16 as ::core::ffi::c_int
                                | 1 as ::core::ffi::c_int as uint16 as uint32,
                        )
                        >> 16 as ::core::ffi::c_int) as int16;
                    let ref mut fresh50 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh50 = (*fresh50).wrapping_add(1);
                    v0 as ::core::ffi::c_int != 128 as ::core::ffi::c_int
                        && (v0 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                }
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) = (128 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
            }
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 128 as ::core::ffi::c_int
    {
        if ((*((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            || {
                v1 = *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
                let ref mut fresh51 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh51 = (*fresh51).wrapping_sub(1);
                *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - (!v1 as ::core::ffi::c_int
                        + *((&raw mut g_ram as *mut uint8)
                            .offset(0xa84 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                    != 128 as ::core::ffi::c_int
                    && ((*((&raw mut g_ram as *mut uint8)
                        .offset(0xa82 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (!v1 as ::core::ffi::c_int
                            + *((&raw mut g_ram as *mut uint8)
                                .offset(0xa84 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                as ::core::ffi::c_int)
                        - 128 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
            }
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xa84 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculateXrayHdmaTable() {
    let mut v0: int16 = 0;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        v0 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int) as int16;
    } else {
        v0 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + 3 as ::core::ffi::c_int) as int16;
    }
    let mut v1: uint16 = 0;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 5 as ::core::ffi::c_int
    {
        v1 = (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    } else {
        v1 = (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16;
    }
    let mut current_block_11: u64;
    if (v0 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 4 as ::core::ffi::c_int
        {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    } else {
        if ((v0 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            CalculateXrayHdmaTableInner(
                v0 as uint16,
                v1,
                *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                false_0 != 0,
                (&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                    as *mut uint16,
            );
            return;
        }
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 8 as ::core::ffi::c_int
        {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    }
    match current_block_11 {
        10599921512955367680 => {}
        _ => {
            CalculateXrayHdmaTableInner(
                v0 as uint16,
                v1,
                *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                true_0 != 0,
                (&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                    as *mut uint16,
            );
            return;
        }
    }
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc3_DeactivateBeam(mut k: uint16) {
    let mut v1: int16 = 0;
    let mut v4: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    mov24(
        (&raw mut g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize) as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut g_ram as *mut uint8).offset(0xa8a as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    v1 = 4096 as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 36 as ::core::ffi::c_int
    {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks() {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh133 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh133 = (*fresh133 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    let mut v2: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x18b2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) =
        *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            );
    let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    if ((*((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 240 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v4 = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v4).size = 2048 as uint16;
        (*v4).src.addr =
            (((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000 as *mut uint8)
                .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
                + 4096 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v4).src.bank as *mut uint16) = 126 as uint16;
        (*v4).vram_dst = ((*(&raw mut g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        let ref mut fresh134 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh134 = (*fresh134).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc4_DeactivateBeam(mut k: uint16) {
    let mut v1: int16 = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    v1 = 4096 as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 36 as ::core::ffi::c_int
    {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks() {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh131 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh131 = (*fresh131 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    if ((*((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 240 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v3 = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v3).size = 2048 as uint16;
        (*v3).src.addr =
            (((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000 as *mut uint8)
                .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
                + 6144 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
        (*v3).vram_dst = (((*(&raw mut g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 1024 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        let ref mut fresh132 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh132 = (*fresh132).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_XrayFunc5_DeactivateBeam(mut k: uint16) {
    let mut v1: int16 = 0;
    v1 = 4096 as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 36 as ::core::ffi::c_int
    {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks() {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh130 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh130 = (*fresh130 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa7c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa7e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa84 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa86 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        mov24(
            (&raw mut g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize)
                as *mut LongPtr,
            0x980001 as uint32,
        );
        *(((&raw mut g_ram as *mut uint8).offset(0xa8a as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        ResponsibleForXrayStandupGlitch();
        *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = 0 as uint16;
        QueueSfx1_Max6(0xa as uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            != 36 as ::core::ffi::c_int
        {
            *(&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
        }
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(i as isize) = 0xff as uint16;
            i -= 1;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
static mut kHdmaobjPreInstr_XrayFuncs: [Option<Func_Y_V>; 6] = unsafe {
    [
        Some(HdmaobjPreInstr_XrayFunc0_NoBeam as unsafe extern "C" fn(uint16) -> ()),
        Some(HdmaobjPreInstr_XrayFunc1_BeamWidening as unsafe extern "C" fn(uint16) -> ()),
        Some(HdmaobjPreInstr_XrayFunc2_FullBeam as unsafe extern "C" fn(uint16) -> ()),
        Some(HdmaobjPreInstr_XrayFunc3_DeactivateBeam as unsafe extern "C" fn(uint16) -> ()),
        Some(HdmaobjPreInstr_XrayFunc4_DeactivateBeam as unsafe extern "C" fn(uint16) -> ()),
        Some(HdmaobjPreInstr_XrayFunc5_DeactivateBeam as unsafe extern "C" fn(uint16) -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_Xray(mut k: uint16) {
    let mut v1: int16 = 0;
    v1 = 4096 as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 36 as ::core::ffi::c_int
    {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks() {
            v1 = 0x4000 as int16;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x27 as uint16;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x47 as uint16;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 0x87 as uint16;
        }
    }
    let ref mut fresh138 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh138 = (*fresh138 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
    kHdmaobjPreInstr_XrayFuncs[*((&raw mut g_ram as *mut uint8)
        .offset(0xa7a as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")(
        (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xa7a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SpawnPowerBombExplosion() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x4000 as uint16;
    } else {
        QueueSfx1_Max15(1 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x8000 as uint16;
        static mut unk_888ABA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x28 as uint8,
            addr: 0x8ace as VoidP,
        };
        static mut unk_888AC2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x29 as uint8,
            addr: 0x8b80 as VoidP,
        };
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_888ABA as *const ::core::ffi::c_void,
        );
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_888AC2 as *const ::core::ffi::c_void,
        );
    };
}
static mut kPowerBombPreExplosionRadiusSpeed: uint16 = 0x3000 as uint16;
static mut kPowerBombPreExplosionRadiusAccel: uint16 = 0x80 as uint16;
static mut g_word_888B96: uint16 = 3 as uint16;
static mut kPowerBombExplosionRadiusSpeed: uint16 = 0 as uint16;
static mut kPowerBombExplosionRadiusAccel: uint16 = 0x30 as uint16;
#[no_mangle]
pub unsafe extern "C" fn Hdmaobj_PreExplodeWhite() {
    *(&raw mut g_ram as *mut uint8).offset(0xc606 as ::core::ffi::c_int as isize) =
        -(1 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0xc607 as ::core::ffi::c_int as isize) = 0 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize) as *mut uint16) =
        1024 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        kPowerBombPreExplosionRadiusSpeed;
}
#[no_mangle]
pub unsafe extern "C" fn Hdmaobj_PreExplodeYellow() {
    *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16) =
        addr_byte_889F06 as ::core::ffi::c_int as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Hdmaobj_ExplodeYellow() {
    *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16) =
        1024 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        kPowerBombExplosionRadiusSpeed;
}
#[no_mangle]
pub unsafe extern "C" fn Hdmaobj_ExplodeWhite() {
    *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16) =
        addr_kPowerBombExplosionShapeDef0 as ::core::ffi::c_int as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Hdmaobj_CleanUpTryCrystalFlash(mut v0: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != *((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        || Hdmaobj_CrystalFlash() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    CallSomeSamusCode(0x1e as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_SetWindowConf(mut k: uint16) {
    let ref mut fresh129 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh129 = (*fresh129 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int =
            k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh125 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh125 = (*fresh125).wrapping_sub(1);
        if *fresh125 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            let ref mut fresh126 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1938 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as uint8 as ::core::ffi::c_int as isize);
            let fresh127 = *fresh126;
            *fresh126 = (*fresh126).wrapping_sub(1);
            if fresh127 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 1 as uint16;
                let ref mut fresh128 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x18cc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh128 = (*fresh128 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            } else {
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(0 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x20 as ::core::ffi::c_int) as uint8;
                }
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(1 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x40 as ::core::ffi::c_int) as uint8;
                }
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(2 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x80 as ::core::ffi::c_int) as uint8;
                }
                *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(k as uint8 as ::core::ffi::c_int as isize) = g_word_888B96 as uint8;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdma_LeftOfScreen(mut k: uint16, mut j: *const uint8) {
    loop {
        let mut w: uint8 = *j;
        let mut c: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        if __CFADD__uint8(w, c) {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as isize) = (w as ::core::ffi::c_int + c as ::core::ffi::c_int) as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as isize) = 0 as uint8;
        } else {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as isize) = 0 as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as isize) = 1 as uint8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k as ::core::ffi::c_int != 192 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdma_OnScreen(mut k: uint16, mut j: *const uint8) {
    loop {
        let mut w: uint8 = *j;
        if w == 0 {
            break;
        }
        let mut c: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        let mut right: uint8 = (if __CFADD__uint8(c, w) as ::core::ffi::c_int != 0 {
            255 as ::core::ffi::c_int
        } else {
            c as ::core::ffi::c_int + w as ::core::ffi::c_int
        }) as uint8;
        let mut left: uint8 = (if c as ::core::ffi::c_int >= w as ::core::ffi::c_int {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(k as isize) = left;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(k as isize) = right;
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k as ::core::ffi::c_int != 192 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdma_RightOfScreen(mut k: uint16, mut j: *const uint8) {
    loop {
        let mut w: uint8 = *j;
        if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int)
            < w as ::core::ffi::c_int
        {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as isize) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xce6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - w as ::core::ffi::c_int) as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as isize) = -(1 as ::core::ffi::c_int) as uint8;
        } else {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as isize) = -(1 as ::core::ffi::c_int) as uint8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as isize) = -(2 as ::core::ffi::c_int) as uint8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k as ::core::ffi::c_int != 192 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdmaObjectTablePtrs(mut k: uint16) {
    let mut v1: uint16 = 0;
    if (*((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >= 0x300 as ::core::ffi::c_int
        || {
            *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xce2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 = (*((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 as ::core::ffi::c_int >= 0x300 as ::core::ffi::c_int
        }
    {
        v1 = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int) - 256 as ::core::ffi::c_int)
            as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 0 as ::core::ffi::c_int)
                as isize,
        ) = (3 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        ) = (3 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int)
        as uint16;
}
static mut k_out: uint16 = 0;
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdmaScaled_LeftOfScreen(
    mut k: uint16,
    mut j: uint16,
    mut multval: uint8,
) -> uint16 {
    let mut v2: int8 = 0;
    let mut v6: uint8 = 0;
    let mut v7: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        v2 = *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as int8;
        let mut Reg: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut v4: bool = __CFADD__uint8(Reg, v2 as uint8);
        let mut v5: uint8 = (Reg as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint8;
        if v4 {
            v6 = v5;
            v7 = 0 as uint8;
        } else {
            v6 = 0 as uint8;
            v7 = -(1 as ::core::ffi::c_int) as uint8;
        }
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v7;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v6;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                (k as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8;
        }
        *(&raw mut j as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((v6 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v7 as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdmaScaled_OnScreen(
    mut k: uint16,
    mut j: uint16,
    mut multval: uint8,
) -> uint16 {
    let mut v2: int8 = 0;
    let mut v5: int8 = 0;
    let mut v6: uint8 = 0;
    let mut v9: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        v2 = *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as int8;
        let mut Reg: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut v4: bool = __CFADD__uint8(Reg, v2 as uint8);
        v5 = (Reg as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as int8;
        if v4 {
            v5 = -(1 as ::core::ffi::c_int) as int8;
        }
        v6 = v5 as uint8;
        let mut v7: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        let mut v8: uint8 = Reg;
        v4 = (v7 as ::core::ffi::c_int) < v8 as ::core::ffi::c_int;
        v9 = (v7 as ::core::ffi::c_int - v8 as ::core::ffi::c_int) as uint8;
        if v4 {
            v9 = 0 as uint8;
        }
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v9;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v6;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                (k as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8;
        }
        *(&raw mut j as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((v6 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v9 as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdmaScaled_RightOfScreen(
    mut k: uint16,
    mut j: uint16,
    mut multval: uint8,
) -> uint16 {
    let mut v5: int8 = 0;
    let mut v6: int8 = 0;
    let mut v7: int8 = 0;
    let mut v10: int8 = 0;
    let mut v8: uint8 = 0;
    let mut v9: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut v2: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        let mut Reg: uint8 = (Mult8x8(
            multval,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut v4: bool = (v2 as ::core::ffi::c_int) < Reg as ::core::ffi::c_int;
        v5 = (v2 as ::core::ffi::c_int - Reg as ::core::ffi::c_int) as int8;
        if v4 {
            v6 = v5;
            v7 = -(1 as ::core::ffi::c_int) as int8;
        } else {
            v6 = -(1 as ::core::ffi::c_int) as int8;
            v7 = 0 as int8;
        }
        v10 = v7;
        v8 = v6 as uint8;
        v9 = v10 as uint8;
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v8;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = v9;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                (k as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8;
        }
        *(&raw mut j as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((v9 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v8 as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(mut k: uint16) {
    let mut v2: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k);
    let mut kk: ::core::ffi::c_int = Mult8x8(
        ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8,
        *(g_rom.offset(
            (((0x88a286 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88a286 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize),
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as uint16,
                96 as uint16,
                ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint8,
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as uint16,
                96 as uint16,
                ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint8,
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as uint16,
            96 as uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
        );
    }
    let mut i: ::core::ffi::c_int = k_out as ::core::ffi::c_int;
    loop {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(i as isize) = v2 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(i as isize) = ((v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        i -= 1;
        if !(i & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v3: ::core::ffi::c_int = kk + 1 as ::core::ffi::c_int;
    while v3 != 0xc0 as ::core::ffi::c_int {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 0 as uint8;
        v3 += 1;
    }
    let mut t: ::core::ffi::c_int = 3 as ::core::ffi::c_int
        * ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int);
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t + 0 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    let ref mut fresh122 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh122 = (*fresh122 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x8600 as ::core::ffi::c_int
    {
        let ref mut fresh123 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh123 = (*fresh123 as ::core::ffi::c_int
            + kPowerBombExplosionRadiusAccel as ::core::ffi::c_int) as uint16;
    } else {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = 1 as uint16;
        let ref mut fresh124 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize);
        *fresh124 = (*fresh124 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k);
    let mut v1: *const uint8 = RomPtr_88(
        *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            CalculatePowerBombHdma_OnScreen(0 as uint16, v1);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as uint16, v1);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as uint16, v1);
    }
    let mut t: ::core::ffi::c_int = 3 as ::core::ffi::c_int
        * ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int);
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset(t as isize)
        as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    let ref mut fresh118 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh118 = (*fresh118 as ::core::ffi::c_int + 192 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == addr_byte_889F06 as ::core::ffi::c_int
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 1 as uint16;
        let ref mut fresh119 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh119 = (*fresh119 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 32 as uint16;
    }
    if (*((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        < 0x10000 as ::core::ffi::c_int
    {
        let ref mut fresh120 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh120 = (*fresh120 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh121 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh121 = (*fresh121 as ::core::ffi::c_int
            + kPowerBombExplosionRadiusAccel as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculatePowerBombHdmaTablePointers(mut v0: uint16) {
    let mut v1: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >= 0x300 as ::core::ffi::c_int
            || {
                *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
                    as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xce2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int) as uint16;
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int) as uint16;
                v1 as ::core::ffi::c_int >= 0x300 as ::core::ffi::c_int
            }
        {
            v1 = 0 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) = ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int)
            - 256 as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (3 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int)
            as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) = (3 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int)
            as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k);
    let mut v1: uint16 = 96 as uint16;
    let mut v2: uint16 = 0;
    let mut kk: ::core::ffi::c_int =
        Mult8x8(
            *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
            *(g_rom.offset(
                (((0x88a286 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a286 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        )
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as uint16,
                v1,
                *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as uint16,
                v1,
                *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as uint16,
            v1,
            *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
        );
    }
    let mut i: uint8 = k_out as uint8;
    loop {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(i as isize) = v2 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(i as isize) = *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        i = i.wrapping_sub(1);
        if !(i as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v3: uint8 = (kk + 1 as ::core::ffi::c_int) as uint8;
    while v3 as ::core::ffi::c_int != 0xc0 as ::core::ffi::c_int {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 0 as uint8;
        v3 = v3.wrapping_add(1);
    }
    let mut t: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xcec as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t * 3 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t * 3 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((t * 3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    let ref mut fresh115 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcec as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh115 = (*fresh115 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x9200 as ::core::ffi::c_int
    {
        let ref mut fresh116 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh116 = (*fresh116 as ::core::ffi::c_int
            - kPowerBombPreExplosionRadiusAccel as ::core::ffi::c_int)
            as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh117 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh117 = (*fresh117 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k);
    let mut v1: *const uint8 = RomPtr_88(
        *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            CalculatePowerBombHdma_OnScreen(0 as uint16, v1);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as uint16, v1);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as uint16, v1);
    }
    let mut t: ::core::ffi::c_int = (*((&raw mut g_ram as *mut uint8)
        .offset(0xcec as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((3 as ::core::ffi::c_int * t) as isize)
        as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((3 as ::core::ffi::c_int * t + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x889079 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x889079 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((3 as ::core::ffi::c_int * t + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    let ref mut fresh111 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh111 = (*fresh111 as ::core::ffi::c_int + 192 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcf2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0xa206 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh112 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh112 = (*fresh112 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        < 0x10000 as ::core::ffi::c_int
    {
        let ref mut fresh113 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh113 = (*fresh113 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh114 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh114 = (*fresh114 as ::core::ffi::c_int
            - kPowerBombPreExplosionRadiusAccel as ::core::ffi::c_int)
            as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SpawnCrystalFlashHdmaObjs() {
    *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x888000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    static mut unk_88A2B0: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x28 as uint8,
        addr: 0xa2bd as VoidP,
    };
    static mut unk_88A2B8: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x29 as uint8,
        addr: 0xa32a as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A2B0 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A2B8 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CrystalFlashSetupPart1() {
    *(&raw mut g_ram as *mut uint8).offset(0xc606 as ::core::ffi::c_int as isize) =
        -(1 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0xc607 as ::core::ffi::c_int as isize) = 0 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize) as *mut uint16) =
        1024 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        kPowerBombPreExplosionRadiusSpeed;
    QueueSfx1_Max6(1 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn CrystalFlashSetupPart2() {
    *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16) =
        1024 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        kPowerBombExplosionRadiusSpeed;
}
#[no_mangle]
pub unsafe extern "C" fn CrystalFlashCleanup(mut k: uint16) {
    let mut v0: uint16 = k;
    *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(mut k: uint16) {
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 8 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) = 51 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh109 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh109 = (*fresh109).wrapping_sub(1);
        if *fresh109 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            if (*(&raw mut g_ram as *mut uint8)
                .offset(0x74 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                | *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                | *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int)
                & 0x1f as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(0 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x20 as ::core::ffi::c_int) as uint8;
                }
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(1 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x40 as ::core::ffi::c_int) as uint8;
                }
                if *(&raw mut g_ram as *mut uint8)
                    .offset(0x74 as ::core::ffi::c_int as isize)
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    *(&raw mut g_ram as *mut uint8)
                        .offset(0x74 as ::core::ffi::c_int as isize)
                        .offset(2 as ::core::ffi::c_int as isize) =
                        ((*(&raw mut g_ram as *mut uint8)
                            .offset(0x74 as ::core::ffi::c_int as isize)
                            .offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x80 as ::core::ffi::c_int) as uint8;
                }
                *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) = g_word_888B96 as uint8;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 1 as uint16;
                let ref mut fresh110 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x18cc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh110 = (*fresh110 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CalculateCrystalFlashHdmaObjectTablePtrs(mut k: uint16) {
    let mut v1: uint16 = 0;
    if (*((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >= 0x300 as ::core::ffi::c_int
        || {
            *((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xce2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 = (*((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 as ::core::ffi::c_int >= 0x300 as ::core::ffi::c_int
        }
    {
        v1 = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int) - 256 as ::core::ffi::c_int)
            as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (3 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        ) = (3 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0xce8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(
    mut k: uint16,
    mut j: uint16,
) -> uint16 {
    let mut right: uint8 = 0;
    let mut left: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut w: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut c: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        left = (if w as ::core::ffi::c_int + c as ::core::ffi::c_int >= 256 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        right = (if w as ::core::ffi::c_int + c as ::core::ffi::c_int >= 256 as ::core::ffi::c_int {
            w as ::core::ffi::c_int + c as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = left;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = right;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(
    mut k: uint16,
    mut j: uint16,
) -> uint16 {
    let mut right: uint8 = 0;
    let mut left: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut w: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut c: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        right = (if (c as ::core::ffi::c_int + w as ::core::ffi::c_int) < 256 as ::core::ffi::c_int
        {
            c as ::core::ffi::c_int + w as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        left = (if c as ::core::ffi::c_int - w as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = left;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = right;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(
    mut k: uint16,
    mut j: uint16,
) -> uint16 {
    let mut left: uint8 = 0;
    let mut right: uint8 = 0;
    loop {
        let mut r20: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut w: uint8 = (Mult8x8(
            ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(g_rom.offset(
                (((0x88a206 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88a206 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut c: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8;
        right = (if (c as ::core::ffi::c_int) < w as ::core::ffi::c_int {
            255 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        left = (if (c as ::core::ffi::c_int) < w as ::core::ffi::c_int {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc406 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = left;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xc506 as ::core::ffi::c_int as isize)
                .offset(k as uint8 as isize) = right;
            if k as uint8 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !(j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    k_out = *(&raw mut k as *mut uint8).offset(0 as ::core::ffi::c_int as isize) as uint16;
    return ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    CalculateCrystalFlashHdmaObjectTablePtrs(k);
    let mut v2: uint16 = 0;
    let mut kk: ::core::ffi::c_int = Mult8x8(
        ((*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8,
        *(g_rom.offset(
            (((0x88a286 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88a286 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize),
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xce6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            v2 = CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(kk as uint16, 96 as uint16);
        } else {
            v2 =
                CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(kk as uint16, 96 as uint16);
        }
    } else {
        v2 = CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(kk as uint16, 96 as uint16);
    }
    let mut i: ::core::ffi::c_int = k_out as ::core::ffi::c_int;
    loop {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(i as uint8 as isize) = v2 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(i as uint8 as isize) = ((v2 as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        i -= 1;
        if !(i & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v3: uint8 = (kk + 1 as ::core::ffi::c_int) as uint8;
    while v3 as ::core::ffi::c_int != 0xc0 as ::core::ffi::c_int {
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc406 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0xc506 as ::core::ffi::c_int as isize)
            .offset(v3 as isize) = 0 as uint8;
        v3 = v3.wrapping_add(1);
    }
    let mut pos: ::core::ffi::c_int = (*((&raw mut g_ram as *mut uint8)
        .offset(0xcea as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int)
        * 3 as ::core::ffi::c_int;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((pos + 0 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((pos + 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
        (((0x888d85 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x888d85 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset((pos + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    let ref mut fresh106 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh106 = (*fresh106 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x2000 as ::core::ffi::c_int
    {
        let ref mut fresh107 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcf0 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh107 = (*fresh107 as ::core::ffi::c_int
            + kPowerBombExplosionRadiusAccel as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh108 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh108 = (*fresh108 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1908 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_22_ScrollingSky() {
    *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) = 1248 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) = 6 as uint16;
    static mut unk_88A62E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x2 as uint8,
        field_1: 0x12 as uint8,
        addr: 0xad63 as VoidP,
    };
    static mut unk_88A636: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xad4e as VoidP,
    };
    static mut unk_88A63E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0xd as uint8,
        addr: 0xad39 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A62E as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A636 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A63E as *const ::core::ffi::c_void,
    );
    HdmaobjPreInstr_FxType22_BG3Yscroll(0 as uint16);
}
unsafe extern "C" fn SetupSomeHdmaTablesBG3(
    mut r24: uint16,
    mut r0: uint16,
    mut r3: uint16,
    mut r6: uint16,
    mut r9: uint16,
) -> uint16 {
    let mut v1: int16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0x598 as ::core::ffi::c_int as isize) as *mut uint16) =
        32 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9c00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 31 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    let mut v0: uint16 = 3 as uint16;
    v1 = (*((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 32 as ::core::ffi::c_int) as int16;
    let mut r18: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 32 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = 0 as uint16;
    while ((v1 as ::core::ffi::c_int
        - *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r0 as uint16_t)
            .offset(v2 as isize) as *const uint8 as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || (v1 as ::core::ffi::c_int
            - *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r9 as uint16_t)
                .offset(v2 as isize) as *const uint8 as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
        if (v2 as ::core::ffi::c_int - r24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return v0;
        }
    }
    let mut v3: uint16 = 0;
    if r18 as ::core::ffi::c_int - 1248 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v3 = (r18 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
    } else {
        v3 = (r18 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
    }
    let mut R22_: uint16 = v3;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        (*((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r3 as uint16_t)
            .offset(v2 as isize) as *const uint8 as *mut uint16) as ::core::ffi::c_int
            - v3 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = (R22_
        as ::core::ffi::c_int
        + *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r6 as uint16_t)
            .offset(v2 as isize) as *const uint8 as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x598 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    loop {
        let mut r20: uint16 = *(&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as uint16;
        r18 = (r18 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
        let ref mut fresh49 = *((&raw mut g_ram as *mut uint8)
            .offset(0x598 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh49 = (*fresh49 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x598 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            break;
        }
        while ((r18 as ::core::ffi::c_int
            - *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r0 as uint16_t)
                .offset(v2 as isize) as *const uint8 as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            || (r18 as ::core::ffi::c_int
                - *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r9 as uint16_t)
                    .offset(v2 as isize) as *const uint8 as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            v2 = (v2 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
            if (v2 as ::core::ffi::c_int - r24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                return v0;
            }
        }
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) =
            *((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r3 as uint16_t)
                .offset(v2 as isize) as *const uint8 as *mut uint16);
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset((v0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) =
            (*((RomPtr_88 as unsafe extern "C" fn(uint16_t) -> *const uint8)(r6 as uint16_t)
                .offset(v2 as isize) as *const uint8 as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x598 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
    }
    return v0;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_FxType22_BG3Yscroll(mut k: uint16) {
    DamageSamusInTopRow();
    k = SetupSomeHdmaTablesBG3(
        78 as uint16,
        addr_word_88A8E8 as ::core::ffi::c_int as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16,
    );
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(k as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetFlagB(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    return hdp;
}
static mut g_word_88C46E: [int16; 16] = [
    0 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
];
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_BG3Xscroll(mut k: uint16) {
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *(&raw mut g_ram as *mut uint8).offset(0x5b5 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x596 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0x596 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                & 0x1e as ::core::ffi::c_int) as uint16;
        }
        let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x596 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v2: uint16 = 0 as uint16;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(0x9e80 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
                    (((0x88a938 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x88a938 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh104 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        let fresh105 = *fresh104;
        *fresh104 = (*fresh104).wrapping_sub(1);
        if fresh105 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 6 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = (*((&raw mut g_ram as *mut uint8)
                .offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint16;
        }
        let mut v5: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x9e80 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                (g_word_88C46E[(i >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            v5 = (v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    }
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9f00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_2 as *mut uint16) = 31 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 31 as uint16;
    let mut r18: uint16 = 31 as uint16;
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_0 as *mut uint16) = -(25088 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25088 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    } else {
        r18 = HdmaFunc_A786(0x105 as uint16, r18, 177 as uint16, 0 as uint16);
    }
    let mut v7: uint16 = 5 as uint16;
    r18 = HdmaFunc_A786(
        5 as uint16,
        r18,
        -(24960 as ::core::ffi::c_int) as uint16,
        128 as uint16,
    );
    v8 = (224 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16;
    while (v8 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        v8 = (v8 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 144 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset(v7 as isize) as *mut uint8 as *mut uint16) = 144 as uint16;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = -(24960
            as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset((v7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = -(24960 as ::core::ffi::c_int) as uint16;
        v7 = (v7 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    }
    v9 = v8;
    if v8 != 0 {
        v9 = (v8 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as int16;
    }
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = v9 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset(v7 as isize) as *mut uint8 as *mut uint16) = v9 as uint16;
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_1)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(24960 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset((v7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = -(24960 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_0)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset((v7 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaFunc_A786(
    mut k: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r22: uint16,
) -> uint16 {
    let mut v0: uint16 = k;
    let mut v2: int16 = 0;
    let mut v1: uint16 = (1216 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut r24: uint16 = (1216 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if (1216 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 1216 as ::core::ffi::c_int
    {
        if v1 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            r18 = (1216 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        } else {
            r18 = (1216 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int - 193 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v1 = 193 as uint16;
            }
            r18 = v1;
            r24 = v1;
            loop {
                v2 = (v1 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16;
                if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    break;
                }
                r24 = v2 as uint16;
                *((&raw mut (*((&raw mut g_ram as *mut uint8)
                    .offset(0x9e00 as ::core::ffi::c_int as isize)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0 as ::core::ffi::c_int as isize))
                .field_0)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
                    (r22 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
                *((&raw mut (*((&raw mut g_ram as *mut uint8)
                    .offset(0x9e00 as ::core::ffi::c_int as isize)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0 as ::core::ffi::c_int as isize))
                .field_1)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = r20;
                v1 = r24;
                v0 = (v0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
            }
        }
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (r22
            as ::core::ffi::c_int | r24 as ::core::ffi::c_int) as uint16;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = r20;
    }
    return r18;
}
static mut unk_88A7EF: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xad76 as VoidP,
};
static mut unk_88A80B: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xad89 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_20() {
    let ref mut fresh47 =
        *(&raw mut g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize);
    *fresh47 = (*fresh47 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint8;
    let ref mut fresh48 =
        *(&raw mut g_ram as *mut uint8).offset(0x91c as ::core::ffi::c_int as isize);
    *fresh48 = (*fresh48 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint8;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A7EF as *const ::core::ffi::c_void,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x59a as ::core::ffi::c_int as isize) as *mut uint16) =
        224 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x59c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn RoomSetupAsm_ScrollingSkyOcean() {
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 74 as uint8;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A80B as *const ::core::ffi::c_void,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x59a as ::core::ffi::c_int as isize) as *mut uint16) =
        224 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x59c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DamageSamusInTopRow() {
    if ((*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 17 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_SkyLandBG2XscrollInner(mut k: uint16) {
    let mut i: uint16 = 0;
    let mut r24: uint16 = 0 as uint16;
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 74 as uint8;
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v3: *mut uint16 = (&raw mut g_ram as *mut uint8).offset(
            (*(g_rom.offset(
                (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .hdma_data_table_entry as isize,
        ) as *mut uint8 as *mut uint16;
        AddToHiLo(
            v3.offset(1 as ::core::ffi::c_int as isize) as *mut uint16,
            v3.offset(0 as ::core::ffi::c_int as isize) as *mut uint16,
            ((*(g_rom.offset(
                (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .scroll_speed as uint32)
                << 16 as ::core::ffi::c_int
                | (*(g_rom.offset(
                    (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut HdmaScrollEntry)
                    .offset(v1 as isize))
                .scroll_subspeed as uint32,
        );
        v1 = v1.wrapping_add(1);
        if !(v1 as ::core::ffi::c_int * 8 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0x9f80 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(44 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9f80 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(45 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9f00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 31 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 1438 as uint16;
    let mut r18: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 32 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 224 as ::core::ffi::c_int) as uint16;
    let mut v5: uint16 = 0 as uint16;
    let mut v6: uint16 = 3 as uint16;
    loop {
        while (r18 as ::core::ffi::c_int
            - (*(g_rom.offset(
                (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut HdmaScrollEntry)
                .offset(v5 as isize))
            .top_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
            && ((r18 as ::core::ffi::c_int
                - (*(g_rom.offset(
                    (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut HdmaScrollEntry)
                    .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize))
                .top_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        {
            let mut v8: uint16 = ((*(g_rom.offset(
                (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut HdmaScrollEntry)
                .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize))
            .top_pos as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as uint16;
            r24 = v8;
            if v8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut g_ram as *mut uint8)
                    .offset(0x9f00 as ::core::ffi::c_int as isize)
                    .offset(v6 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
                *((&raw mut g_ram as *mut uint8)
                    .offset(0x9f00 as ::core::ffi::c_int as isize)
                    .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    as *mut uint8 as *mut uint16) =
                    ((*(g_rom.offset(
                        (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut HdmaScrollEntry)
                        .offset(v5 as isize))
                    .hdma_data_table_entry as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int) as uint16;
                v6 = (v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
                v8 = (r24 as ::core::ffi::c_int - 127 as ::core::ffi::c_int) as uint16;
            }
            *((&raw mut g_ram as *mut uint8)
                .offset(0x9f00 as ::core::ffi::c_int as isize)
                .offset(v6 as isize) as *mut uint8 as *mut uint16) = v8;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x9f00 as ::core::ffi::c_int as isize)
                .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                as *mut uint8 as *mut uint16) =
                ((*(g_rom.offset(
                    (((0x88aec1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x88aec1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut HdmaScrollEntry)
                    .offset(v5 as isize))
                .hdma_data_table_entry as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int) as uint16;
            r18 = (r18 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
            v6 = (v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
            if r18 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut g_ram as *mut uint8)
                    .offset(0x9f00 as ::core::ffi::c_int as isize)
                    .offset((v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
                    as *mut uint8 as *mut uint16) = 0 as uint16;
                return;
            }
        }
        v5 = v5.wrapping_add(1);
        if !(v5 as ::core::ffi::c_int * 8 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    i = (1535 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    loop {
        r24 = i;
        if i as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            break;
        }
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset(v6 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 181 as uint16;
        v6 = (v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
        i = (r24 as ::core::ffi::c_int - 127 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset(v6 as isize) as *mut uint8 as *mut uint16) = i;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 181 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9f00 as ::core::ffi::c_int as isize)
        .offset((v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_SkyLandBG2Xscroll(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_SkyLandBG2Xscroll2(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RoomMainAsm_ScrollingSky(mut src: *const uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9f00 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v1: *mut VramWriteEntry = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v1.offset(0 as ::core::ffi::c_int as isize)).size = 64 as uint16;
        (*v1.offset(1 as ::core::ffi::c_int as isize)).size = 64 as uint16;
        (*v1.offset(2 as ::core::ffi::c_int as isize)).size = 64 as uint16;
        (*v1.offset(3 as ::core::ffi::c_int as isize)).size = 64 as uint16;
        let mut tt: ::core::ffi::c_int = (*((&raw mut g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f8 as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int;
        if tt & 0x8000 as ::core::ffi::c_int != 0 {
            tt = 0 as ::core::ffi::c_int;
        }
        let mut v2: ::core::ffi::c_int =
            8 as ::core::ffi::c_int * tt as uint8 as ::core::ffi::c_int;
        let mut y1: ::core::ffi::c_int =
            (tt >> 8 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
        let mut y2: ::core::ffi::c_int = ((tt >> 8 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as uint8 as ::core::ffi::c_int;
        let mut v3: VoidP = (*src.offset(y1 as isize) as ::core::ffi::c_int + v2) as VoidP;
        (*v1.offset(0 as ::core::ffi::c_int as isize)).src.addr = v3;
        (*v1.offset(1 as ::core::ffi::c_int as isize)).src.addr =
            (v3 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as VoidP;
        let mut v5: VoidP = (*src.offset(y2 as isize) as ::core::ffi::c_int + v2) as VoidP;
        (*v1.offset(2 as ::core::ffi::c_int as isize)).src.addr = v5;
        (*v1.offset(3 as ::core::ffi::c_int as isize)).src.addr =
            (v5 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as VoidP;
        (*v1.offset(0 as ::core::ffi::c_int as isize)).src.bank = 0x8a as uint8;
        (*v1.offset(1 as ::core::ffi::c_int as isize)).src.bank = 0x8a as uint8;
        (*v1.offset(2 as ::core::ffi::c_int as isize)).src.bank = 0x8a as uint8;
        (*v1.offset(3 as ::core::ffi::c_int as isize)).src.bank = 0x8a as uint8;
        let mut t: ::core::ffi::c_int = (*(&raw mut g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int;
        let mut v6: uint16 = (t + 4 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x1f8 as ::core::ffi::c_int)) as uint16;
        (*v1.offset(0 as ::core::ffi::c_int as isize)).vram_dst = v6;
        (*v1.offset(1 as ::core::ffi::c_int as isize)).vram_dst =
            (v6 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (t + 4 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 240 as ::core::ffi::c_int
                & 0x1f8 as ::core::ffi::c_int)) as uint16;
        (*v1.offset(2 as ::core::ffi::c_int as isize)).vram_dst = v7;
        (*v1.offset(3 as ::core::ffi::c_int as isize)).vram_dst =
            (v7 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn RoomCode_ScrollingSkyLand() {
    RoomMainAsm_ScrollingSky(
        RomPtr_88(addr_off_88AD9C as ::core::ffi::c_int as uint16_t) as *const uint16
    );
}
#[no_mangle]
pub unsafe extern "C" fn RoomMainAsm_ScrollingSkyOcean() {
    RoomMainAsm_ScrollingSky(
        RomPtr_88(addr_off_88ADA6 as ::core::ffi::c_int as uint16_t) as *const uint16
    );
}
static mut unk_88B08C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x11 as uint8,
    addr: 0xb0ac as VoidP,
};
static mut kFirefleaFlashingShades: [uint16; 12] = [
    0 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x200 as ::core::ffi::c_int as uint16,
    0x300 as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x500 as ::core::ffi::c_int as uint16,
    0x600 as ::core::ffi::c_int as uint16,
    0x500 as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x300 as ::core::ffi::c_int as uint16,
    0x200 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
];
static mut kFirefleaDarknessShades: [uint16; 6] = [
    0 as ::core::ffi::c_int as uint16,
    0x600 as ::core::ffi::c_int as uint16,
    0xc00 as ::core::ffi::c_int as uint16,
    0x1200 as ::core::ffi::c_int as uint16,
    0x1800 as ::core::ffi::c_int as uint16,
    0x1900 as ::core::ffi::c_int as uint16,
];
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_24() {
    *((&raw mut g_ram as *mut uint8).offset(0x1778 as ::core::ffi::c_int as isize)
        as *mut uint16) = 6 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x177a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B08C as *const ::core::ffi::c_void,
    );
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x177c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x177e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1780 as ::core::ffi::c_int as isize)
        as *mut uint16) = 24 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1782 as ::core::ffi::c_int as isize)
        as *mut uint16) = kFirefleaFlashingShades[0 as ::core::ffi::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_FirefleaBG3XScroll(mut k: uint16) {
    let mut v2: int8 = 0;
    let mut v1: uint16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
        .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        | 0xc as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let ref mut fresh103 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1778 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh103 = (*fresh103).wrapping_sub(1);
        if *fresh103 == 0 {
            *((&raw mut g_ram as *mut uint8).offset(0x1778 as ::core::ffi::c_int as isize)
                as *mut uint16) = 6 as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x177e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0x177a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                if (*((&raw mut g_ram as *mut uint8).offset(0x177a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >= 0xc as ::core::ffi::c_int
                {
                    v1 = 0 as uint16;
                }
            } else {
                v1 = 6 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0x177a as ::core::ffi::c_int as isize)
                as *mut uint16) = v1;
        }
        v2 = ((kFirefleaDarknessShades[(*((&raw mut g_ram as *mut uint8)
            .offset(0x177e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            + kFirefleaFlashingShades[*((&raw mut g_ram as *mut uint8)
                .offset(0x177a as ::core::ffi::c_int as isize)
                as *mut uint16) as usize] as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) =
            (v2 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) =
            (v2 as ::core::ffi::c_int | 0x40 as ::core::ffi::c_int) as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) =
            (v2 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ExpandingContractingHdmaEffect() {
    let ref mut fresh32 = *((&raw mut g_ram as *mut uint8)
        .offset(0x59a as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh32 = (*fresh32).wrapping_add(1);
    if *fresh32 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x59a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5a0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            let ref mut fresh33 = *((&raw mut g_ram as *mut uint8)
                .offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh33 = (*fresh33 as ::core::ffi::c_int - 1024 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 0x2000 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0x2000 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x5a0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        } else {
            let ref mut fresh34 = *((&raw mut g_ram as *mut uint8)
                .offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh34 = (*fresh34 as ::core::ffi::c_int + 1024 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 0x8000 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let ref mut fresh35 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x5a0 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh35 = (*fresh35).wrapping_add(1);
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 128 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 128 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x5a8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 127 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x5aa as ::core::ffi::c_int as isize)
            as *mut uint16) = 127 as uint16;
    }
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x5a8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 32 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) =
            (*((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            (*((&raw mut g_ram as *mut uint8).offset(0x5aa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x5a8 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        r18 = (r18 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        if Unreachable() {
            let ref mut fresh36 = *((&raw mut g_ram as *mut uint8)
                .offset(0x5a6 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh36 = (*fresh36).wrapping_add(1);
            let ref mut fresh37 = *((&raw mut g_ram as *mut uint8)
                .offset(0x5aa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh37 = (*fresh37).wrapping_sub(1);
        }
        let ref mut fresh38 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5a4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh38 = (*fresh38).wrapping_add(1);
        let ref mut fresh39 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5a8 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh39 = (*fresh39).wrapping_sub(1);
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        r20 = r20.wrapping_sub(1);
        if !(r20 != 0) {
            break;
        }
    }
}
static mut g_word_88B256: [uint16; 17] = [
    0x46 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0x8000 as ::core::ffi::c_int as uint16,
];
#[no_mangle]
pub unsafe extern "C" fn HandleEarthquakeSoundEffect() {
    let mut v1: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x609 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && {
            let ref mut fresh43 = *((&raw mut g_ram as *mut uint8)
                .offset(0x609 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh43 = (*fresh43).wrapping_sub(1);
            *fresh43 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        }
    {
        let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x607 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v1 = g_word_88B256[(*((&raw mut g_ram as *mut uint8)
            .offset(0x607 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize] as int16;
        if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v0 = 0 as uint16;
            v1 = g_word_88B256[0 as ::core::ffi::c_int as usize] as int16;
        }
        QueueSfx2_Max6(v1 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x609 as ::core::ffi::c_int as isize)
            as *mut uint16) = (g_word_88B256[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            + (v0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            + (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 3 as ::core::ffi::c_int)) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x607 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
static mut unk_88B289: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xc3e1 as VoidP,
};
static mut unk_88B291: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x10 as uint8,
    addr: 0xc3f0 as VoidP,
};
static mut unk_88B2B1: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xc3e1 as VoidP,
};
static mut unk_88B2B9: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x10 as uint8,
    addr: 0xc3f0 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_2_Lava() {
    *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x88b343 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1978 as ::core::ffi::c_int as isize) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B289 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B291 as *const ::core::ffi::c_void,
    );
    SpawnBG3ScrollHdmaObject();
    SpawnAnimtiles(addr_kAnimtiles_Lava as ::core::ffi::c_int as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_4_Acid() {
    *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x88b343 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1978 as ::core::ffi::c_int as isize) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B2B1 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B2B9 as *const ::core::ffi::c_void,
    );
    SpawnBG3ScrollHdmaObject();
    SpawnAnimtiles(addr_kAnimtiles_Acid as ::core::ffi::c_int as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn FxHandleTide() {
    let mut v2: uint16 = 0;
    let mut v5: uint16 = 0;
    if (*(((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut int16) as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x1974 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int;
        let mut v1: uint16 = (8 as ::core::ffi::c_int
            * kSinCosTable8bit_Sext[v0 as usize] as ::core::ffi::c_int)
            as uint16;
        if kSinCosTable8bit_Sext[v0 as usize] as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh40 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1972 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh40 = (*fresh40).wrapping_sub(1);
        }
        *(((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v1;
        if kSinCosTable8bit_Sext[v0 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v2 = (*((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 288 as ::core::ffi::c_int) as uint16;
        } else {
            v2 = (*((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 192 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
    } else if *(((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut v3: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x1974 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int;
        let mut v4: uint16 = (32 as ::core::ffi::c_int
            * kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int)
            as uint16;
        if kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int & 0x400 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh41 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1972 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh41 = (*fresh41).wrapping_sub(1);
        }
        *(((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v4;
        if kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v5 = (*((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 224 as ::core::ffi::c_int) as uint16;
        } else {
            v5 = (*((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1974 as ::core::ffi::c_int as isize)
            as *mut uint16) = v5;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_LavaAcid() {
    if *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == *((&raw mut g_ram as *mut uint8)
                        .offset(0x1978 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
            {
                return;
            }
        } else if !(*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            && (*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
        {
            return;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88b367 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_LavaAcid_WaitToRise() {
    HandleEarthquakeSoundEffect();
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 21 as uint16;
    let ref mut fresh44 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh44 = (*fresh44 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    let ref mut fresh45 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1980 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh45 = (*fresh45).wrapping_sub(1);
    if *fresh45 == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88b382 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_LavaAcid_Raising() {
    HandleEarthquakeSoundEffect();
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 21 as uint16;
    let ref mut fresh42 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh42 = (*fresh42 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    if RaiseOrLowerFx() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88b343 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_B3A9(mut k: uint16, mut hdp: *const uint8) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 112 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn CallFxRisingFunc(mut ea: uint32) {
    match ea {
        8958787 => {
            FxRisingFunction_LavaAcid();
            return;
        }
        8958823 => {
            FxRisingFunction_LavaAcid_WaitToRise();
            return;
        }
        8958850 => {
            FxRisingFunction_LavaAcid_Raising();
            return;
        }
        8963112 => {
            FxRisingFunction_C428_WaterNormal();
            return;
        }
        8963148 => {
            FxRisingFunction_WaterWaitToRise();
            return;
        }
        8963160 => {
            FxRisingFunction_WaterRising();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
static mut kLavaSoundEffects: [uint8; 8] = [
    0x12 as ::core::ffi::c_int as uint8,
    0x13 as ::core::ffi::c_int as uint8,
    0x14 as ::core::ffi::c_int as uint8,
    0x12 as ::core::ffi::c_int as uint8,
    0x13 as ::core::ffi::c_int as uint8,
    0x14 as ::core::ffi::c_int as uint8,
    0x12 as ::core::ffi::c_int as uint8,
    0x13 as ::core::ffi::c_int as uint8,
];
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_LavaAcidBG3YScroll(mut k: uint16) {
    let mut current_block: u64;
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1984 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        CallFxRisingFunc(
            (*((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                | 0x880000 as ::core::ffi::c_int) as uint32,
        );
        FxHandleTide();
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0x1960 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0x1976 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                ),
        );
        *((&raw mut g_ram as *mut uint8).offset(0xcadc as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x9c00 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        v2 = (((*((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
            .wrapping_add(
                (*((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            )
            >> 16 as ::core::ffi::c_int) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v3 = (v2 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            if v3 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
                v4 = ((v3 as ::core::ffi::c_int ^ 0x1f as ::core::ffi::c_int)
                    & 0x1f as ::core::ffi::c_int
                    | 0x100 as ::core::ffi::c_int) as int16;
                current_block = 135057599068437321;
            } else if (v3 as uint16 as ::core::ffi::c_int) < 0x100 as ::core::ffi::c_int {
                v4 = !(v3 as ::core::ffi::c_int) as uint8 as int16;
                current_block = 135057599068437321;
            } else {
                current_block = 4068382217303356765;
            }
        } else {
            current_block = 4068382217303356765;
        }
        match current_block {
            4068382217303356765 => {
                v4 = 0 as int16;
            }
            _ => {}
        }
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = v4 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            let mut v5: ::core::ffi::c_int =
                *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int;
            let ref mut fresh101 = *((&raw mut g_ram as *mut uint8)
                .offset(0x192c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize);
            let fresh102 = *fresh101;
            *fresh101 = (*fresh101).wrapping_sub(1);
            if fresh102 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v5 as isize) = 112 as uint16;
                QueueSfx2_Max6(
                    kLavaSoundEffects[(*((&raw mut g_ram as *mut uint8)
                        .offset(0x5e5 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        as ::core::ffi::c_int
                        & 7 as ::core::ffi::c_int) as usize] as uint16,
                );
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) = swap16(
            *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v8 = (*((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as int16;
            if (v8 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v8 = 255 as int16;
                current_block = 7317190685143662286;
            } else if (v8 as uint16 as ::core::ffi::c_int) < 0x200 as ::core::ffi::c_int {
                current_block = 7317190685143662286;
            } else {
                current_block = 11042950489265723346;
            }
        } else {
            current_block = 11042950489265723346;
        }
        match current_block {
            11042950489265723346 => {
                v8 = 511 as int16;
            }
            _ => {}
        }
        *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = (3 as ::core::ffi::c_int
            * ((v8 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
                & 0x3ff as ::core::ffi::c_int)
            + 0xb62a as ::core::ffi::c_int) as uint16;
        return;
    } else {
        return;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetFlagB_Copy(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_LavaAcidBG2YScroll(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9c44 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            Handle_LavaAcidBG2YScroll_Func3(k);
        } else {
            Handle_LavaAcidBG2YScroll_Func2(k);
        }
    } else {
        Handle_LavaAcidBG2YScroll_Func1(k);
    }
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = (3 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 0xc0b1 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Handle_LavaAcidBG2YScroll_Func1(mut v0: uint16) {
    WriteReg(
        (*((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 17153 as ::core::ffi::c_int) as SnesRegs as uint16,
        0x10 as uint8,
    );
    let mut v1: uint8 = 30 as uint8;
    let mut v2: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int) as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9c46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint8;
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Handle_LavaAcidBG2YScroll_Func2(mut v0: uint16) {
    WriteReg(
        (*((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 17153 as ::core::ffi::c_int) as SnesRegs as uint16,
        0xf as uint8,
    );
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh54 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1920 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh55 = *fresh54;
    *fresh54 = (*fresh54).wrapping_sub(1);
    if fresh55 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 6 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize);
    let mut v4: uint8 = 30 as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9c46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*(g_rom.offset(
                (((0x88b589 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88b589 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                & 0x1ff as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Handle_LavaAcidBG2YScroll_Func3(mut v0: uint16) {
    WriteReg(
        (*((&raw mut g_ram as *mut uint8).offset(0x18c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + BBAD0 as ::core::ffi::c_int) as SnesRegs as uint16,
        0x10 as uint8,
    );
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh56 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1920 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh57 = *fresh56;
    *fresh56 = (*fresh56).wrapping_sub(1);
    if fresh57 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = (*(((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut v4: uint8 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 30 as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9c46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*(g_rom.offset(
                (((0x88b60a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88b60a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                & 0x1ff as ::core::ffi::c_int) as uint16;
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
static mut unk_88C40F: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x11 as uint8,
    addr: 0xd856 as VoidP,
};
static mut unk_88C41F: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xd847 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_6_Water() {
    *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x88c428 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1978 as ::core::ffi::c_int as isize) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88C40F as *const ::core::ffi::c_void,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88C41F as *const ::core::ffi::c_void,
        );
    }
    SpawnBG3ScrollHdmaObject();
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_C428_WaterNormal() {
    if *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if (*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == *((&raw mut g_ram as *mut uint8)
                        .offset(0x1978 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
            {
                return;
            }
        } else if !(*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            && (*((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
        {
            return;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88c44c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_WaterWaitToRise() {
    let ref mut fresh46 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1980 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh46 = (*fresh46).wrapping_sub(1);
    if *fresh46 == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88c458 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxRisingFunction_WaterRising() {
    if RaiseOrLowerFx() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x88c428 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetFlagB_Copy2(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn CallFxRisingFunction(mut ea: uint32) {
    match ea {
        8958787 => {
            FxRisingFunction_LavaAcid();
            return;
        }
        8958823 => {
            FxRisingFunction_LavaAcid_WaitToRise();
            return;
        }
        8958850 => {
            FxRisingFunction_LavaAcid_Raising();
            return;
        }
        8963112 => {
            FxRisingFunction_C428_WaterNormal();
            return;
        }
        8963148 => {
            FxRisingFunction_WaterWaitToRise();
            return;
        }
        8963160 => {
            FxRisingFunction_WaterRising();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_WaterBG3XScroll(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1984 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        return;
    }
    CallFxRisingFunction(
        (*((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x880000 as ::core::ffi::c_int) as uint32,
    );
    FxHandleTide();
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x195c as ::core::ffi::c_int as isize) as *mut uint16,
        ((*((&raw mut g_ram as *mut uint8).offset(0x1972 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x1970 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
            .wrapping_add(
                (*((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0x1976 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            ),
    );
    let mut v3: int16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else if v3 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = ((v3 as ::core::ffi::c_int ^ 0x1f as ::core::ffi::c_int)
            & 0x1f as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16;
    } else if (v3 as uint16 as ::core::ffi::c_int) < 0x100 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = !(v3 as ::core::ffi::c_int) as uint8 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r20: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *(((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int) as uint16;
    let ref mut fresh98 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1920 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v5 as isize);
    let fresh99 = *fresh98;
    *fresh98 = (*fresh98).wrapping_sub(1);
    if fresh99 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 10 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
            & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v7: uint8 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v5 as isize) as uint8;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset((v7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = (g_word_88C46E[(i >> 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int) as uint16;
        v7 = (v7 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        i -= 2 as ::core::ffi::c_int;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh100 = *((&raw mut g_ram as *mut uint8)
            .offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh100 = (*fresh100 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    }
    let mut v9: int16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as int16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v9 = 511 as int16;
    } else if v9 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        if v9 as uint16 as ::core::ffi::c_int >= 0x200 as ::core::ffi::c_int {
            v9 = 511 as int16;
        }
    } else {
        v9 = (v9 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int)
            as int16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (3 as ::core::ffi::c_int
            * ((v9 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
                & 0x3ff as ::core::ffi::c_int)
            + 0xc645 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetFlagB_Copy3(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_WaterBG2XScroll(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9c44 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k);
    } else if *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k);
    } else {
        HdmaobjPreInstr_WaterBG2XScroll_Func2(k);
    }
    let mut v1: int16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as int16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v1 = 511 as int16;
    } else if v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        if v1 as uint16 as ::core::ffi::c_int >= 0x200 as ::core::ffi::c_int {
            v1 = 511 as int16;
        }
    } else {
        v1 = (*((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as int16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x18d8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (3
        as ::core::ffi::c_int
        * ((v1 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) & 0x3ff as ::core::ffi::c_int)
        + 0xcf46 as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_WaterBG2XScroll_Func2(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh96 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1920 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh97 = *fresh96;
    *fresh96 = (*fresh96).wrapping_sub(1);
    if fresh97 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 6 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
            & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = (*(((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut v4: uint8 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 30 as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9c46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) = (g_word_88C46E[(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_WaterBG2XScroll_Func1(mut k: uint16) {
    let mut v1: uint8 = 30 as uint8;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16);
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9c46 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) = v2;
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint8;
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SpawnBG3ScrollHdmaObject() {
    static mut unk_88D869: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd8d0 as VoidP,
    };
    SpawnHdmaObjectToSlot0xA(
        0x88 as uint8,
        &raw const unk_88D869 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_28_CeresRidley() {
    static mut unk_88D8E2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x5 as uint8,
        addr: 0xd906 as VoidP,
    };
    static mut unk_88D8EA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x2c as uint8,
        addr: 0xd91d as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D8E2 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D8EA as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn sub_88D916() {
    *((&raw mut g_ram as *mut uint8).offset(0x7eb as ::core::ffi::c_int as isize) as *mut uint16) =
        9 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_CeresElevator() {
    static mut unk_88D92C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x5 as uint8,
        addr: 0xd939 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D92C as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_SetVideoMode1(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x7eb as ::core::ffi::c_int as isize) as *mut uint16) =
        9 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_A_Rain() {
    static mut unk_88D95C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd96c as VoidP,
    };
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D95C as *const ::core::ffi::c_void,
    );
    SpawnBG3ScrollHdmaObject();
    SpawnAnimtiles(addr_kAnimtiles_Rain as ::core::ffi::c_int as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_1938_RandomNumber(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
        (((0x88d992 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x88d992 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    )
        as *const uint8
        as *mut uint16)
        .offset(
            ((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int
                & 6 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_RainBg3Scroll(mut k: uint16) {
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1984 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut v1: ::core::ffi::c_int =
            k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1944 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh94 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh94 = (*fresh94 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1944 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xcadc as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1950 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh95 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh95 = (*fresh95 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1950 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_8_Spores() {
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    static mut unk_88DA1D: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xda2d as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DA1D as *const ::core::ffi::c_void,
    );
    SpawnBG3ScrollHdmaObject();
    SpawnAnimtiles(addr_kAnimtiles_Spores as ::core::ffi::c_int as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_SporesBG3Xscroll(mut k: uint16) {
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1984 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut v1: ::core::ffi::c_int =
            k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v3;
        *((&raw mut g_ram as *mut uint8).offset(0xcadc as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1938 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_C() {
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    static mut unk_88DB14: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xdb19 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DB14 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_FogBG3Scroll(mut k: uint16) {
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 92 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1984 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut v1: ::core::ffi::c_int =
            k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xcade as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh92 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh92 = (*fresh92 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcadc as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh93 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh93 = (*fresh93 as ::core::ffi::c_int + 80 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_26_TourianEntranceStatue() {
    static mut unk_88DBBA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd856 as VoidP,
    };
    static mut unk_88DBC2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x10 as uint8,
        addr: 0xdcfa as VoidP,
    };
    if CheckEventHappened(0xa as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x6 as uint8,
            field_1: 0xc as uint8,
            field_2: 0xb777 as VoidP,
        });
        *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 514 as uint16;
    }
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 74 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x196c as ::core::ffi::c_int as isize)
        as *mut uint16) = (0x88c428 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1978 as ::core::ffi::c_int as isize) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DBBA as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DBC2 as *const ::core::ffi::c_void,
    );
    SpawnBG3ScrollHdmaObject();
}
#[no_mangle]
pub unsafe extern "C" fn sub_88DBCB(mut k: uint16) {
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_CheckLotsOfEventsHappened(mut v0: uint16) {
    if CheckEventHappened(6 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        if CheckEventHappened(7 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            if CheckEventHappened(8 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
            {
                if CheckEventHappened(9 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                    != 0
                {
                    let ref mut fresh90 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x1e6f as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh90 =
                        (*fresh90 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
                    if *((&raw mut g_ram as *mut uint8)
                        .offset(0x1e6f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x192c as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(
                                (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = 300 as uint16;
                        v0 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x18b2 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
                            .offset(0x18b2 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x18e4 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize) = 1 as uint16;
                        let ref mut fresh91 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x18cc as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize);
                        *fresh91 =
                            (*fresh91 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
        }
    }
    sub_88DBCB(v0);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_DC23(mut k: uint16) {
    let mut v0: uint16 = k;
    HandleEarthquakeSoundEffect();
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 13 as uint16;
    let ref mut fresh87 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh87 = (*fresh87 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    let mut v1: ::core::ffi::c_int = v0 as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh88 = *((&raw mut g_ram as *mut uint8)
        .offset(0x192c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh88 = (*fresh88).wrapping_sub(1);
    if *fresh88 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as ::core::ffi::c_int as uint16,
            0 as uint16,
        );
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        let ref mut fresh89 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh89 = (*fresh89 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    sub_88DBCB(v0);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_DC69(mut k: uint16) {
    HandleEarthquakeSoundEffect();
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 13 as uint16;
    let ref mut fresh85 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh85 = (*fresh85 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            ((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            -(0x4000 as ::core::ffi::c_int) as uint32,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            == 0xff10 as ::core::ffi::c_int
        {
            SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
                field_0: 0x6 as uint8,
                field_1: 0xc as uint8,
                field_2: 0xb773 as VoidP,
            });
            SetEventHappened(0xa as uint16);
            *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            let ref mut fresh86 = *((&raw mut g_ram as *mut uint8)
                .offset(0x18cc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh86 = (*fresh86 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
        sub_88DBCB(k);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_DCBA(mut v0: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x1e6d as ::core::ffi::c_int as isize)
        as *mut uint16) = 0x8000 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 514 as uint16;
    sub_88DBCB(v0);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_GotoIfEventHappened(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    if CheckEventHappened(0xa as uint16) != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = -(240 as ::core::ffi::c_int) as uint16;
        *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0 as *mut uint16) = -(240 as ::core::ffi::c_int) as uint16;
        return *(hdp as *mut uint16) as uintptr_t as *const uint8;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 0 as uint16;
        *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        return hdp.offset(2 as ::core::ffi::c_int as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn SpawnBombTorizoHaze() {
    static mut unk_88DD36: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x2 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdd4a as VoidP,
    };
    static mut unk_88DD3E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdd62 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DD36 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DD3E as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 44 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FxTypeFunc_2C_Haze() {
    static mut unk_88DDD4: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xded3 as VoidP,
    };
    static mut unk_88DDDD: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdeeb as VoidP,
    };
    if CheckBossBitForCurArea(1 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88DDDD as *const ::core::ffi::c_void,
        );
    } else {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88DDD4 as *const ::core::ffi::c_void,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(mut k: uint16) {
    sub_88DE18(k, 0x80 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(mut k: uint16) {
    sub_88DE18(k, 0x20 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn sub_88DE18(mut k: uint16, mut a: uint16) {
    let mut v2: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = a;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x99c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x82e737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x88de2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(mut k: uint16) {
    let mut v1: int8 = 0;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 44 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        == 16 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x88de74 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: uint8 = k as uint8;
        let mut r20: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize);
        v1 = *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize) as int8;
        let mut i: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut r18: uint8 = v1 as uint8;
            *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as uint8 as ::core::ffi::c_int as isize) =
                (r20 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint8;
            v1 = (r18 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int8;
            if ((r18 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v1 = 0 as int8;
            }
            i -= 1;
        }
        let ref mut fresh84 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh84 = (*fresh84).wrapping_add(1);
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(mut k: uint16) {
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 44 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x99c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x82e2db as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18f0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x88de96 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(mut k: uint16) {
    let mut v1: int8 = 0;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 44 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint8 = k as uint8;
        let mut r20: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize);
        v1 = *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize) as int8;
        let mut i: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut r18: uint8 = v1 as uint8;
            *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as uint8 as ::core::ffi::c_int as isize) =
                (r20 as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint8;
            v1 = (r18 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int8;
            if ((r18 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v1 = 0 as int8;
            }
            i -= 1;
        }
        let ref mut fresh83 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh83 = (*fresh83).wrapping_sub(1);
    }
}
static mut unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xdf4f as VoidP,
};
static mut unk_88DF41: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xdf6b as VoidP,
};
static mut unk_88DF4A: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x2 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xdf77 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn sub_88DF34() {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF38 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn sub_88DF3D() {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF41 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn sub_88DF46() {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF4A as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_DF94(mut v0: uint16) {
    let mut v1: int16 = 0;
    if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .properties as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        && (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 320 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .y_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        && {
            v1 = ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                - 304 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        }
    {
        if v1 as ::core::ffi::c_int - 40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = addr_word_88DF6B as ::core::ffi::c_int as uint16;
        } else {
            let mut v4: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = 1 as uint16;
            if v1 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) = addr_word_88DF5F as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) = addr_word_88DF65 as ::core::ffi::c_int as uint16;
            }
        }
    } else {
        let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_word_88DF71 as ::core::ffi::c_int as uint16;
    };
}
static mut off_88E04E: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_1 as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_3 as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_6 as unsafe extern "C" fn() -> uint8),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_VariaSuitPickup(mut k: uint16) {
    if off_88E04E[*((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")() as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) = 18 as uint16;
    }
}
static mut off_88E084: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_1 as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe extern "C" fn() -> uint8),
        Some(GravitySuitPickup_3 as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe extern "C" fn() -> uint8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe extern "C" fn() -> uint8),
        Some(GravitySuitPickup_6 as unsafe extern "C" fn() -> uint8),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_GravitySuitPickup(mut k: uint16) {
    if off_88E084[*((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")() as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) =
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) = 18 as uint16;
    }
}
static mut g_word_88E0D5: uint16 = 0x7878 as uint16;
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_0_LightBeamAppears() -> uint8 {
    let mut v0: int16 = 0;
    let mut v3: int16 = 0;
    let ref mut fresh15 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh15 = (*fresh15 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    v0 = *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = g_word_88E0D5;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v0 -= 1;
        if !(v0 as ::core::ffi::c_int > 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v2: uint16 = 510 as uint16;
    v3 = *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) as int16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = g_word_88E0D5;
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        v3 -= 1;
        if !(v3 as ::core::ffi::c_int > 0 as ::core::ffi::c_int) {
            break;
        }
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh16 = (*fresh16).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = 30840 as uint16;
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_1() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint8;
    let ref mut fresh17 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *fresh17 = (*fresh17 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint8;
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8 as ::core::ffi::c_int
        - 97 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh18 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh18 = (*fresh18).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x846c as uint16;
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_2_LightBeamWidens() -> uint8 {
    let mut v1: int8 = 0;
    let mut v2: int16 = 0;
    let mut v4: int8 = 0;
    let mut v6: int8 = 0;
    let mut v8: int8 = 0;
    let mut v10: int8 = 0;
    AdvanceSuitPickupColorMathSubscreenToWhite();
    let mut v0: bool = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint8;
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8
        == 0
        || v0 as ::core::ffi::c_int != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = -(256 as ::core::ffi::c_int) as uint16;
    } else {
        v1 = (*((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as int8;
        if __CFADD__uint8(
            *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
            *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
        ) {
            v1 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = v1 as uint8;
    }
    v2 = 0 as int16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        v4 = (*((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *(g_rom.offset(
                (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v3 as isize) as ::core::ffi::c_int) as int8;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v4 = 0 as int8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v2 as uint16 as ::core::ffi::c_int as isize) = v4 as uint8;
        let mut v5: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        v6 = (*(g_rom.offset(
            (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(v3 as isize) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as int8;
        if __CFADD__uint8(
            *(g_rom.offset(
                (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v3 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
        ) {
            v6 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v5 as ::core::ffi::c_int as isize) = v6 as uint8;
        v2 = (v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
        v3 = v3.wrapping_add(1);
        if !((v2 as ::core::ffi::c_int) < 0x100 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v7: uint16 = (v3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    loop {
        v8 = (*((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *(g_rom.offset(
                (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v7 as isize) as ::core::ffi::c_int) as int8;
        if (v8 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v8 = 0 as int8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v2 as uint16 as ::core::ffi::c_int as isize) = v8 as uint8;
        let mut v9: uint16 = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        v10 = (*(g_rom.offset(
            (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(v7 as isize) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as int8;
        if __CFADD__uint8(
            *(g_rom.offset(
                (((0x88e3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x88e3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v7 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
        ) {
            v10 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v9 as ::core::ffi::c_int as isize) = v10 as uint8;
        v2 = (v9 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
        v7 = v7.wrapping_sub(1);
        if !((v2 as ::core::ffi::c_int) < 0x200 as ::core::ffi::c_int) {
            break;
        }
    }
    let ref mut fresh19 = *((&raw mut g_ram as *mut uint8)
        .offset(0xddc as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh19 = (*fresh19 as ::core::ffi::c_int + 96 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0xff00 as ::core::ffi::c_int
    {
        let ref mut fresh20 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh20 = (*fresh20).wrapping_add(1);
        let ref mut fresh21 = *((&raw mut g_ram as *mut uint8)
            .offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh21 = (*fresh21 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_4_LightBeamShrinks() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xdf3 as ::core::ffi::c_int as isize) != 0 {
        AdvanceSuitPickupColorMathToBlue();
    } else {
        AdvanceSuitPickupColorMathToOrange();
    }
    let ref mut fresh25 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh25 = (*fresh25 as ::core::ffi::c_int
        + ((*((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)) as uint16;
    let mut v0: int16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v0 -= 1;
        if !(v0 as ::core::ffi::c_int > 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v2: uint16 = 510 as uint16;
    let mut v3: int16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as int16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        v3 -= 1;
        if !(v3 as ::core::ffi::c_int > 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let ref mut fresh26 = *((&raw mut g_ram as *mut uint8)
        .offset(0xddc as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh26 = (*fresh26 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xddc as ::core::ffi::c_int as isize)
            as *mut uint16) = 256 as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh27 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh27 = (*fresh27).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1793 as ::core::ffi::c_int) as uint16;
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_5_LightBeamDissipates() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int) as uint8;
    let ref mut fresh28 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *fresh28 = (*fresh28 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(128 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xdee as ::core::ffi::c_int as isize)
        as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8 as ::core::ffi::c_int
        - 112 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh29 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_6() -> uint8 {
    QueueMusic_Delayed8(3 as uint16);
    return GravitySuitPickup_6();
}
#[no_mangle]
pub unsafe extern "C" fn GravitySuitPickup_6() -> uint8 {
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    mov24(
        (&raw mut g_ram as *mut uint8).offset(0xa88 as ::core::ffi::c_int as isize) as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut g_ram as *mut uint8).offset(0xa8a as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa8c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa8e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x18b2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh12 = *((&raw mut g_ram as *mut uint8)
        .offset(0x18cc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v0 as isize);
    *fresh12 = (*fresh12 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v0 as isize) = 1 as uint16;
    CallSomeSamusCode(0xb as uint16);
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn AdvanceSuitPickupColorMathSubscreenToWhite() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 63 as ::core::ffi::c_int
    {
        let ref mut fresh0 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8;
        if *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) =
                63 as uint8;
        }
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 95 as ::core::ffi::c_int
    {
        let ref mut fresh1 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
        *fresh1 = (*fresh1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8;
        if *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            - 96 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize) =
                95 as uint8;
        }
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 0x9f as ::core::ffi::c_int
    {
        let ref mut fresh2 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
        *fresh2 = (*fresh2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8;
        if *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize) =
                -(97 as ::core::ffi::c_int) as uint8;
        }
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn AdvanceSuitPickupColorMathToOrange() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 63 as ::core::ffi::c_int
    {
        let ref mut fresh6 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
        *fresh6 = (*fresh6).wrapping_sub(1);
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 77 as ::core::ffi::c_int
    {
        let ref mut fresh7 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
        *fresh7 = (*fresh7).wrapping_sub(1);
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 0x83 as ::core::ffi::c_int
    {
        let ref mut fresh8 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
        *fresh8 = (*fresh8).wrapping_sub(1);
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn VariaSuitPickup_3() -> uint8 {
    let ref mut fresh22 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh22 = (*fresh22 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
    let ref mut fresh23 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9a4 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh23 = (*fresh23 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16) =
        kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
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
    Samus_LoadSuitPalette();
    let ref mut fresh24 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh24 = (*fresh24).wrapping_add(1);
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn GravitySuitPickup_3() -> uint8 {
    let ref mut fresh9 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    let ref mut fresh10 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9a4 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh10 = (*fresh10 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16) =
        kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
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
    Samus_LoadSuitPalette();
    let ref mut fresh11 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh11 = (*fresh11).wrapping_add(1);
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn AdvanceSuitPickupColorMathToBlue() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 48 as ::core::ffi::c_int
    {
        let ref mut fresh3 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize);
        *fresh3 = (*fresh3).wrapping_sub(1);
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 73 as ::core::ffi::c_int
    {
        let ref mut fresh4 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf1 as ::core::ffi::c_int as isize);
        *fresh4 = (*fresh4).wrapping_sub(1);
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 0x90 as ::core::ffi::c_int
    {
        let ref mut fresh5 =
            *(&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize);
        *fresh5 = (*fresh5).wrapping_sub(1);
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_E449(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x1988 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) = 26 as uint16;
    } else if *(&raw mut (*((&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(3 as ::core::ffi::c_int as isize))
    .parameter_1 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize)
        != 0
    {
        if *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0xf78 as ::core::ffi::c_int as isize) as *mut EnemyData)
            .offset(3 as ::core::ffi::c_int as isize))
            .parameter_1 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xff as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8)
                .offset(0x1986 as ::core::ffi::c_int as isize) as *mut uint16) = 4
                as uint16;
            let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
                .offset(0x18b2 as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let ref mut fresh82 = *((&raw mut g_ram as *mut uint8)
                .offset(0x18cc as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v0 as isize);
            *fresh82 = (*fresh82 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)
                as uint16;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x18e4 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v0 as isize) = 1 as uint16;
        }
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    };
}
static mut unk_88E4A0: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xe4a8 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn sub_88E487(mut v0: uint16, mut r22: uint16) {
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(2 as ::core::ffi::c_int as isize))
    .parameter_1 = v0;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(3 as ::core::ffi::c_int as isize))
    .ai_var_D = 0 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(3 as ::core::ffi::c_int as isize))
    .ai_var_E = 0 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(3 as ::core::ffi::c_int as isize))
    .ai_preinstr = r22;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E4A0 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_E4BD(mut k: uint16, mut hdp: *const uint8) -> *const uint8 {
    let mut v2: ::core::ffi::c_uint = 0;
    let mut v3: ::core::ffi::c_uint = 0;
    let mut v4: ::core::ffi::c_uint = 0;
    let mut v5: ::core::ffi::c_uint = 0;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .parameter_1 = (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(2 as ::core::ffi::c_int as isize))
    .parameter_1;
    if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(2 as ::core::ffi::c_int as isize))
    .parameter_1 as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16) = 192 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16) = 192 as uint16;
        v4 = 12620032 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            *(&raw mut v4 as *mut uint16).offset(
                (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        *(((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9004 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(28288 as ::core::ffi::c_int) as uint16;
        v5 = 12620032 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut g_ram as *mut uint8).offset(0x9008 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            *(&raw mut v5 as *mut uint16).offset(
                (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        *(((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v5 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900a as ::core::ffi::c_int as isize)
            as *mut uint16) = addr_loc_889180 as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900c as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900f as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9012 as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
            as *mut uint16) = 160 as uint16;
        v2 = 10522880 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            *(&raw mut v2 as *mut uint16).offset(
                (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        *(((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v2 as uint16;
        v3 = 10522880 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut g_ram as *mut uint8).offset(0x9008 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            *(&raw mut v3 as *mut uint16).offset(
                (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        *(((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v3 as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0x900c as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            -(28416 as ::core::ffi::c_int) as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0x9012 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            -(28416 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9004 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(28352 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900a as ::core::ffi::c_int as isize)
            as *mut uint16) = -(28352 as ::core::ffi::c_int) as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0x900f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            -(28352 as ::core::ffi::c_int) as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) =
            -(28352 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9018 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = 0 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_E567(mut v0: uint16) {
    let mut v7: int16 = 0;
    let mut j: uint16 = 0;
    let mut v8: uint16 = 0;
    let mut r28: uint16 = 0;
    let mut r30: uint16 = 0;
    if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .parameter_1
        != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(1 as ::core::ffi::c_int as isize))
        .parameter_1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            r28 = 4 as uint16;
            r30 = 128 as uint16;
        } else {
            r28 = 8 as uint16;
            r30 = 64 as uint16;
        }
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh79 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        let fresh80 = *fresh79;
        *fresh79 = (*fresh79).wrapping_sub(1);
        if fresh80 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = (2 as ::core::ffi::c_int
                * (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(3 as ::core::ffi::c_int as isize))
                .ai_preinstr as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                & 0x1ff as ::core::ffi::c_int) as uint16;
        }
        let mut r20: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        let mut v4: uint16 = 0 as uint16;
        loop {
            let mut v11: uint16 = v4;
            let mut v5: uint16 = r20;
            let mut v6: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize];
            if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                let mut r18: uint16 = -(v7 as ::core::ffi::c_int) as uint16;
                let mut r22: uint16 = (Mult8x8(
                    -(v7 as int8 as ::core::ffi::c_int) as uint8,
                    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as uint8,
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22 = (r22 as ::core::ffi::c_int
                    + Mult8x8(
                        *(&raw mut r18 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xf78 as ::core::ffi::c_int as isize)
                            as *mut EnemyData)
                            .offset(3 as ::core::ffi::c_int as isize))
                        .ai_var_D as uint8,
                    ) as ::core::ffi::c_int) as uint16;
                let mut r24: uint16 = Mult8x8(
                    r18 as uint8,
                    *(&raw mut (*((&raw mut g_ram as *mut uint8)
                        .offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                );
                let mut mult: uint8 = Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut g_ram as *mut uint8)
                        .offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                ) as uint8;
                r22 = (r22 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
                r18 = ((r22 as ::core::ffi::c_int
                    + ((mult as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v11;
                *(((&raw mut g_ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(v11 as ::core::ffi::c_int as isize) as *mut uint16) =
                    (*((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - r18 as ::core::ffi::c_int) as uint16;
            } else {
                let mut r18_0: uint16 =
                    kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
                let mut r22_0: uint16 = (Mult8x8(
                    v7 as uint8,
                    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as uint8,
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22_0 = (r22_0 as ::core::ffi::c_int
                    + Mult8x8(
                        *(&raw mut r18_0 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xf78 as ::core::ffi::c_int as isize)
                            as *mut EnemyData)
                            .offset(3 as ::core::ffi::c_int as isize))
                        .ai_var_D as uint8,
                    ) as ::core::ffi::c_int) as uint16;
                let mut r24_0: uint16 = Mult8x8(
                    r18_0 as uint8,
                    *(&raw mut (*((&raw mut g_ram as *mut uint8)
                        .offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                );
                let mut mult_0: uint8 = Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut g_ram as *mut uint8)
                        .offset(0xf78 as ::core::ffi::c_int as isize)
                        as *mut EnemyData)
                        .offset(3 as ::core::ffi::c_int as isize))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                ) as uint8;
                r22_0 = (r22_0 as ::core::ffi::c_int + r24_0 as ::core::ffi::c_int) as uint16;
                r18_0 = ((r22_0 as ::core::ffi::c_int
                    + ((mult_0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v11;
                *(((&raw mut g_ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(v11 as ::core::ffi::c_int as isize) as *mut uint16) = (r18_0
                    as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
            }
            v4 = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v4 as ::core::ffi::c_int - r30 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(1 as ::core::ffi::c_int as isize))
        .parameter_1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *(((&raw mut g_ram as *mut uint8).offset(0x9180 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(i as isize) as *mut uint16) =
                    (*((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + *((&raw mut g_ram as *mut uint8)
                            .offset(0xb5 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                        - *(((&raw mut g_ram as *mut uint8)
                            .offset(0x9100 as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(i as isize) as *mut uint16)
                            as ::core::ffi::c_int) as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
        } else {
            j = 62 as uint16;
            while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                *(((&raw mut g_ram as *mut uint8).offset(0x9140 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                    (*((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + *((&raw mut g_ram as *mut uint8)
                            .offset(0xb5 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                        - *(((&raw mut g_ram as *mut uint8)
                            .offset(0x9100 as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(j as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16;
                j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            }
        }
    } else {
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh81 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh81 = (*fresh81 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    };
}
static mut unk_88E722: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xe727 as VoidP,
};
static mut unk_88E74C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x41 as uint8,
    field_1: 0x26 as uint8,
    addr: 0xe751 as VoidP,
};
#[no_mangle]
pub unsafe extern "C" fn MotherBrainRisingHdmaObject() -> uint16 {
    return SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E722 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SpawnMotherBrainRainbowBeamHdma() -> uint16 {
    return SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E74C as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn InitializeRainbowBeam() {
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 71 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x8f as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 36 as uint16;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 100 as uint8;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25344 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = -(32736 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    MotherBrain_CalculateRainbowBeamHdma();
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_E7BC(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 19 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
            as *mut uint16) = 36 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 27 as ::core::ffi::c_int
        {
            MotherBrain_CalculateRainbowBeamHdma();
            sub_88E7ED();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn sub_88E7ED() {
    let mut v0: uint16 = *(g_rom.offset(
        (((0x88e833 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x88e833 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let ref mut fresh75 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        *fresh75 = (*fresh75).wrapping_add(1);
        let ref mut fresh76 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        *fresh76 = (*fresh76).wrapping_add(1);
        let ref mut fresh77 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        *fresh77 = (*fresh77).wrapping_add(1);
        let ref mut fresh78 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        *fresh78 = (*fresh78).wrapping_add(1);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        v0 = *(g_rom.offset(
            (((0x88e833 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88e833 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
    }
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = (v0 as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int
        | 0x20 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) =
        (v0 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int
            | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) =
        ((v0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn SpawnMorphBallEyeBeamHdma() {
    static mut unk_88E8E4: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x41 as uint8,
        field_1: 0x26 as uint8,
        addr: 0xe8ec as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E8E4 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_InitMorphBallEyeBeamHdma(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_uint = 0;
    let mut v3: ::core::ffi::c_uint = 0;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = 48 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 48 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 80 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 80 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
        as *mut uint16) = 228 as uint16;
    v2 = 14979328 as ::core::ffi::c_int as ::core::ffi::c_uint;
    *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = *(&raw mut v2 as *mut uint16)
        .offset(
            (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *(((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9004 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(28216 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
        as *mut uint16) = 152 as uint16;
    v3 = 37520 as ::core::ffi::c_uint;
    *(((&raw mut g_ram as *mut uint8).offset(0x9008 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = *(&raw mut v3 as *mut uint16)
        .offset(
            (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *(((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v3 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .ai_var_C = 1 as uint16;
    let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9082 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn sub_88E987(mut v0: uint16) {
    let mut v3: uint16 = v0;
    let mut r18: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .ai_var_D;
    let mut r20: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x192c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v1: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(1 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        CalculateXrayHdmaTableInner(
            v2,
            v1,
            r18,
            r20,
            false_0 != 0,
            (&raw mut g_ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
                as *mut uint16,
        );
    } else {
        CalculateXrayHdmaTableInner(
            v2,
            v1,
            r18,
            r20,
            true_0 != 0,
            (&raw mut g_ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
                as *mut uint16,
        );
    }
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v3 as ::core::ffi::c_int as isize);
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v3 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v3 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_E9E6(mut k: uint16) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9082 as ::core::ffi::c_int as isize) as *mut uint16,
        0x4000 as uint32,
    );
    AddToHiLo(
        ((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as *mut uint16,
        ((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9082 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 4 as uint16;
        let mut v5: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh74 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 1 as uint16;
    }
    sub_88E987(k);
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_EA3C(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .ai_var_C
        != 0
    {
        sub_88E987(k);
        let mut v2: uint16 = (4 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x88ea8b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88ea8b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(
                (4 as ::core::ffi::c_int
                    * *((&raw mut g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16 as isize,
            );
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x88ea8b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88ea8b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x88ea8b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x88ea8b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9090 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh73 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh73 = (*fresh73 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_EACB(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x1986 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 64 as ::core::ffi::c_int
    {
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
        *(&raw mut g_ram as *mut uint8)
            .offset(0x74 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut g_ram as *mut uint8).offset(0x9100 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 255 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9004 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9008 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut v3: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh69 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh69 = (*fresh69 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 1 as uint16;
    } else {
        sub_88E987(k);
        let mut v4: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            ) as ::core::ffi::c_int
            != 32 as ::core::ffi::c_int
        {
            let ref mut fresh70 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0x18b2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int as isize,
                );
            *fresh70 = (*fresh70).wrapping_sub(1);
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 64 as ::core::ffi::c_int
        {
            let ref mut fresh71 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(v4 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            *fresh71 = (*fresh71).wrapping_sub(1);
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0x80 as ::core::ffi::c_int
        {
            let ref mut fresh72 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1920 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(v4 as ::core::ffi::c_int as isize);
            *fresh72 = (*fresh72).wrapping_sub(1);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn SpawnTitleScreenGradientObjs() {
    static mut unk_88EB6B: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0xeb85 as VoidP,
    };
    static mut unk_88EB63: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xeb73 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EB63 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EB6B as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInsr_ConfigTitleSequenceGradientHDMA(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    *(&raw mut g_ram as *mut uint8).offset(0x6f as ::core::ffi::c_int as isize) = 0 as uint8;
    ConfigureTitleSequenceGradientHDMA();
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_Backdrop_TitleSequenceGradient(mut k: uint16) {
    ConfigureTitleSequenceGradientHDMA();
    if *((&raw mut g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh68 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh68 = (*fresh68 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_ColorMathControlB_TitleGradient(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh67 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh67 = (*fresh67 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SpawnIntroCutsceneCrossfadeHdma() {
    static mut unk_88EBFB: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0xec03 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EBFB as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_IntroCutsceneCrossfade(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh66 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh66 = (*fresh66 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CinematicFunction_Intro_Func133() {
    let mut v0: ::core::ffi::c_uint = 0;
    let mut v1: ::core::ffi::c_uint = 0;
    *((&raw mut g_ram as *mut uint8).offset(0xd9c as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd9e as ::core::ffi::c_int as isize) as *mut uint16) =
        0x4000 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xda0 as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd82 as ::core::ffi::c_int as isize) as *mut uint16) =
        192 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd88 as ::core::ffi::c_int as isize) as *mut uint16) =
        192 as uint16;
    v0 = 12621824 as ::core::ffi::c_int as ::core::ffi::c_uint;
    *(((&raw mut g_ram as *mut uint8).offset(0xd84 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = *(&raw mut v0 as *mut uint16)
        .offset(
            (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *(((&raw mut g_ram as *mut uint8).offset(0xd82 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd86 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(26496 as ::core::ffi::c_int) as uint16;
    v1 = 12621824 as ::core::ffi::c_int as ::core::ffi::c_uint;
    *(((&raw mut g_ram as *mut uint8).offset(0xd8a as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = *(&raw mut v1 as *mut uint16)
        .offset(
            (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *(((&raw mut g_ram as *mut uint8).offset(0xd88 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = v1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd8c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(26496 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd8e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    static mut unk_88EC82: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xec8a as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EC82 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjInstr_EC9F_ClearVars(
    mut k: uint16,
    mut hdp: *const uint8,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1920 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x192c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1938 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    return hdp;
}
#[no_mangle]
pub unsafe extern "C" fn HdmaobjPreInstr_ECB6(mut k: uint16) {
    let mut v7: int16 = 0;
    let mut v8: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xd9c as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        let mut r28: uint16 = 4 as uint16;
        let mut n: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xda0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x1914 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
            & 0x1ff as ::core::ffi::c_int) as uint16;
        let mut r20: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1914 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        let mut v4: uint16 = 0 as uint16;
        loop {
            let mut v10: uint16 = v4;
            let mut v5: uint16 = r20;
            let mut v6: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize];
            if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                let mut r18: uint16 = -(v7 as ::core::ffi::c_int) as uint16;
                let mut r22: uint16 = (Mult8x8(
                    -(v7 as int8 as ::core::ffi::c_int) as uint8,
                    *(&raw mut g_ram as *mut uint8).offset(0xd9e as ::core::ffi::c_int as isize),
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22 = (r22 as ::core::ffi::c_int
                    + Mult8x8(
                        *(&raw mut r18 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                        *(&raw mut g_ram as *mut uint8)
                            .offset(0xd9e as ::core::ffi::c_int as isize),
                    ) as ::core::ffi::c_int) as uint16;
                let mut r24: uint16 = Mult8x8(
                    r18 as uint8,
                    *(&raw mut g_ram as *mut uint8).offset(0xd9f as ::core::ffi::c_int as isize),
                );
                let mut mult: uint8 = Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                    *(&raw mut g_ram as *mut uint8).offset(0xd9f as ::core::ffi::c_int as isize),
                ) as uint8;
                r22 = (r22 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
                r18 = ((r22 as ::core::ffi::c_int
                    + ((mult as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v10;
                *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (*((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - r18 as ::core::ffi::c_int) as uint16;
            } else {
                let mut r18_0: uint16 =
                    kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
                let mut r22_0: uint16 = (Mult8x8(
                    v7 as uint8,
                    *(&raw mut g_ram as *mut uint8).offset(0xd9e as ::core::ffi::c_int as isize),
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22_0 = (r22_0 as ::core::ffi::c_int
                    + Mult8x8(
                        *(&raw mut r18_0 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                        *(&raw mut g_ram as *mut uint8)
                            .offset(0xd9e as ::core::ffi::c_int as isize),
                    ) as ::core::ffi::c_int) as uint16;
                let mut r24_0: uint16 = Mult8x8(
                    r18_0 as uint8,
                    *(&raw mut g_ram as *mut uint8).offset(0xd9f as ::core::ffi::c_int as isize),
                );
                let mut mult_0: uint8 = Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                    *(&raw mut g_ram as *mut uint8).offset(0xd9f as ::core::ffi::c_int as isize),
                ) as uint8;
                r22_0 = (r22_0 as ::core::ffi::c_int + r24_0 as ::core::ffi::c_int) as uint16;
                r18_0 = ((r22_0 as ::core::ffi::c_int
                    + ((mult_0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v10;
                *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (r18_0 as ::core::ffi::c_int
                        + *((&raw mut g_ram as *mut uint8)
                            .offset(0xb9 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16;
            }
            v4 = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v4 as ::core::ffi::c_int - n) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x9800 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        let mut v2: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh65 = *((&raw mut g_ram as *mut uint8)
            .offset(0x18cc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        *fresh65 = (*fresh65 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18e4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn CallHdmaobjPreInstr(mut ea: uint32, mut k: uint16) {
    match ea {
        8491739 => return,
        8946872 => return,
        10785852 => return,
        9556607 => {
            HdmaobjPreInstr_XraySetup(k);
            return;
        }
        8947439 => {
            HdmaobjPreInstr_Xray(k);
            return;
        }
        8947506 => {
            HdmaobjPreInstr_XrayFunc0_NoBeam(k);
            return;
        }
        8947540 => {
            HdmaobjPreInstr_XrayFunc1_BeamWidening(k);
            return;
        }
        8947627 => {
            HdmaobjPreInstr_XrayFunc2_FullBeam(k);
            return;
        }
        8948020 => {
            HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k);
            return;
        }
        8948154 => {
            HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k);
            return;
        }
        8948232 => {
            HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k);
            return;
        }
        8948623 => {
            HdmaobjPreInstr_PowerBombExplode_SetWindowConf(k);
            return;
        }
        8948632 => {
            HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(k);
            return;
        }
        8949225 => {
            HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(k);
            return;
        }
        8949426 => {
            HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(k);
            return;
        }
        8949983 => {
            HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(k);
            return;
        }
        8950184 => {
            HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(k);
            return;
        }
        8954681 => {
            HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(k);
            return;
        }
        8954717 => {
            HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(k);
            return;
        }
        8955218 => {
            HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(k);
            return;
        }
        8955459 => {
            HdmaobjPreInstr_FxType22_BG3Yscroll(k);
            return;
        }
        8955507 => {
            HdmaobjPreInstr_BG3Xscroll(k);
            return;
        }
        8957362 => {
            HdmaobjPreInstr_SkyLandBG2Xscroll(k);
            return;
        }
        8957370 => {
            HdmaobjPreInstr_SkyLandBG2Xscroll2(k);
            return;
        }
        8957378 => {
            HdmaobjPreInstr_SkyLandBG2XscrollInner(k);
            return;
        }
        8958140 => {
            HdmaobjPreInstr_FirefleaBG3XScroll(k);
            return;
        }
        8958896 => {
            HdmaobjPreInstr_LavaAcidBG3YScroll(k);
            return;
        }
        8959189 => {
            HdmaobjPreInstr_LavaAcidBG2YScroll(k);
            return;
        }
        8963214 => {
            HdmaobjPreInstr_WaterBG3XScroll(k);
            return;
        }
        8963465 => {
            HdmaobjPreInstr_WaterBG2XScroll(k);
            return;
        }
        8963556 => {
            HdmaobjPreInstr_WaterBG2XScroll_Func2(k);
            return;
        }
        8963638 => {
            HdmaobjPreInstr_WaterBG2XScroll_Func1(k);
            return;
        }
        8968609 => {
            HdmaobjPreInstr_RainBg3Scroll(k);
            return;
        }
        8968775 => {
            HdmaobjPreInstr_SporesBG3Xscroll(k);
            return;
        }
        8969014 => {
            HdmaobjPreInstr_FogBG3Scroll(k);
            return;
        }
        8969175 => {
            HdmaobjPreInstr_CheckLotsOfEventsHappened(k);
            return;
        }
        8969251 => {
            HdmaobjPreInstr_DC23(k);
            return;
        }
        8969321 => {
            HdmaobjPreInstr_DC69(k);
            return;
        }
        8969402 => {
            HdmaobjPreInstr_DCBA(k);
            return;
        }
        8969539 => {
            HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k);
            return;
        }
        8969744 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(k);
            return;
        }
        8969749 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(k);
            return;
        }
        8969773 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k);
            return;
        }
        8969844 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k);
            return;
        }
        8969878 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k);
            return;
        }
        8970132 => {
            HdmaobjPreInstr_DF94(k);
            return;
        }
        8970278 => {
            HdmaobjPreInstr_VariaSuitPickup(k);
            return;
        }
        8970332 => {
            HdmaobjPreInstr_GravitySuitPickup(k);
            return;
        }
        8971337 => {
            HdmaobjPreInstr_E449(k);
            return;
        }
        8971623 => {
            HdmaobjPreInstr_E567(k);
            return;
        }
        8972220 => {
            HdmaobjPreInstr_E7BC(k);
            return;
        }
        8972774 => {
            HdmaobjPreInstr_E9E6(k);
            return;
        }
        8972860 => {
            HdmaobjPreInstr_EA3C(k);
            return;
        }
        8973003 => {
            HdmaobjPreInstr_EACB(k);
            return;
        }
        8973232 => {
            HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k);
            return;
        }
        8973266 => {
            HdmaobjPreInstr_ColorMathControlB_TitleGradient(k);
            return;
        }
        8973341 => {
            HdmaobjPreInstr_IntroCutsceneCrossfade(k);
            return;
        }
        8972070 => return,
        8973494 => {
            HdmaobjPreInstr_ECB6(k);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn CallHdmaobjInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: *const uint8,
) -> *const uint8 {
    match ea {
        8969006 => return j,
        8947049 => return HdmaobjInstr_Delete(k, j),
        8947056 => return HdmaobjInstr_SetPreInstr(k, j),
        8947076 => return HdmaobjInstr_ClearPreInstr(k, j),
        8947124 => return HdmaobjInstr_CallFarFunc(k, j),
        8947180 => return HdmaobjInstr_Goto(k, j),
        8947185 => return HdmaobjInstr_GotoRel(k, j),
        8947207 => return HdmaobjInstr_DecrementAndGoto(k, j),
        8947215 => return HdmaobjInstr_DecrementAndGotoRel(k, j),
        8947222 => return HdmaobjInstr_SetTimer(k, j),
        8947234 => return HdmaobjInstr_SetHdmaControl(k, j),
        8947255 => return HdmaobjInstr_SetHdmaTarget(k, j),
        8947276 => return HdmaobjInstr_SetHdmaTablePtr(k, j),
        8947285 => return HdmaobjInstr_SetHdmaTableBank(k, j),
        8947306 => return HdmaobjInstr_SetIndirectHdmaDataBank(k, j),
        8947330 => return HdmaobjInstr_Sleep(k, j),
        8955500 => return HdmaobjInstr_SetFlagB(k, j),
        8959182 => return HdmaobjInstr_SetFlagB_Copy(k, j),
        8963175 => return HdmaobjInstr_SetFlagB_Copy2(k, j),
        8963458 => return HdmaobjInstr_SetFlagB_Copy3(k, j),
        8968521 => return HdmaobjInstr_SetVideoMode1(k, j),
        8968577 => return HdmaobjInstr_1938_RandomNumber(k, j),
        8969419 => return HdmaobjInstr_GotoIfEventHappened(k, j),
        8971453 => return HdmaobjInstr_E4BD(k, j),
        8972567 => return HdmaobjInstr_InitMorphBallEyeBeamHdma(k, j),
        8973471 => return HdmaobjInstr_EC9F_ClearVars(k, j),
        8958889 => return HdmaobjInstr_B3A9(k, j),
        8973215 => return HdmaobjInsr_ConfigTitleSequenceGradientHDMA(k, j),
        8968470 => {
            sub_88D916();
            return j;
        }
        _ => {
            Unreachable();
            return ::core::ptr::null::<uint8>();
        }
    };
}
