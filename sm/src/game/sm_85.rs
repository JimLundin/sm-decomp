use crate::game::sm_80::{
    HandleMusicQueue, QueueSfx1_Max6, ReadJoypadInputs, WaitForNMI_NoUpdate_Async,
};
use crate::game::sm_82::{CancelSoundEffects, HandleSoundEffects, QueueSamusMovementSfx};
use crate::rtl::{g_ram, g_rom, ReadReg, ReadRegWord, RomPtr, Unreachable, WriteReg, WriteRegWord};

pub use crate::types::{
    __int16_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, uint16, uint16_t, uint32,
    uint32_t, uint8, uint8_t, CoroutineRet, SnesRegs, VoidP,
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
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MsgBoxConfig {
    pub modify_box_func: VoidP,
    pub draw_initial_tilemap: VoidP,
    pub message_tilemap: VoidP,
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
    Consts_60, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram3800, Ram4000,
    Ram4000_Backups,
};
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
#[inline]
unsafe fn RomPtr_85(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x850000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
pub const fnWriteLargeMessageBoxTilemap: uint32 = 8749658 as uint32;
pub const fnWriteSmallMessageBoxTilemap: uint32 = 8749705 as uint32;
pub const fnDrawShootButtonAndSetupPpuForLargeMessageBox: uint32 = 8750021 as uint32;
pub const fnDrawRunButtonAndSetupPpuForLargeMessageBox: uint32 = 8750028 as uint32;
pub const fnSetupPpuForSmallMessageBox: uint32 = 8750134 as uint32;
pub const fnSetupPpuForLargeMessageBox: uint32 = 8750145 as uint32;
pub static mut message_box_das0l_value: uint16 = 0;
static mut kMsgBoxSpecialButtonTilemapOffs: [uint16; 27] = [
    0 as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x120 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kTileNumbersForButtonLetters: [uint16; 8] = [
    0x28e0 as ::core::ffi::c_int as uint16,
    0x3ce1 as ::core::ffi::c_int as uint16,
    0x2cf7 as ::core::ffi::c_int as uint16,
    0x38f8 as ::core::ffi::c_int as uint16,
    0x38d0 as ::core::ffi::c_int as uint16,
    0x38eb as ::core::ffi::c_int as uint16,
    0x38f1 as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
];
unsafe fn InitializePpuForMessageBoxes() {
    *((&raw mut g_ram as *mut uint8).offset(0x5f9 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb1 as uint8);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb as uint8);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0x1f as uint8);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0 as uint8);
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .msgbox
        .backup_of_enabled_hdma_channels =
        *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize);
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .misc
        .field_3E8[3 as ::core::ffi::c_int as usize] =
        *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize);
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) = 88 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6a as ::core::ffi::c_int as isize) = 23 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x70 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x73 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x20 as uint8);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x40 as uint8);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x80 as uint8);
    ReadReg(BG3HOFS as ::core::ffi::c_int as uint16);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8);
    ReadReg(BG3VOFS as ::core::ffi::c_int as uint16);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8);
    let mut i: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
    );
    ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16);
    WriteRegWord(DMAP1 as ::core::ffi::c_int as uint16, 0x3981 as uint16);
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
}
unsafe fn ClearMessageBoxBg3Tilemap() {
    let mut v0: int16 = 0;
    v0 = 1790 as int16;
    loop {
        *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3800 as ::core::ffi::c_int as isize)
            as *mut Ram3800))
            .cinematic_bg_tilemap as *mut uint16 as *mut uint8)
            .offset(v0 as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0xe as uint16;
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
        if !(v0 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
    );
    WriteRegWord(DMAP1 as ::core::ffi::c_int as uint16, 0x1801 as uint16);
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        ((&raw mut g_ram as *mut uint8).offset(0x3800 as ::core::ffi::c_int as isize)
            as *mut Ram3800 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as uint16,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
}
static mut kLargeMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
static mut kSmallMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
unsafe fn WriteMessageTilemap() -> uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize) as *mut uint16) =
        112 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize) as *mut uint16) =
        124 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 112 as ::core::ffi::c_int {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap[i as usize] = 0 as uint16;
        i += 1;
    }
    let mut r0: uint16 = (*(g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as isize,
        ))
    .message_tilemap as uint16;
    let mut r9: uint16 = ((*(g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .message_tilemap as ::core::ffi::c_int
        - r0 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = r9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    r9 = (r9 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
    message_box_das0l_value = r9;
    let mut v1: uint16 = 32 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] = *((RomPtr_85
            as unsafe fn(uint16_t) -> *const uint8)(
            r0 as uint16_t
        )
        .offset(v2 as isize)
            as *const uint8
            as *mut uint16);
        v1 = v1.wrapping_add(1);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    return v1;
}
unsafe fn WriteLargeMessageBoxTilemap() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kLargeMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap();
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap
            [(i_0 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kLargeMsgBoxTopBottomBorderTilemap
                [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i_0 = i_0.wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
unsafe fn WriteSmallMessageBoxTilemap() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kSmallMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap();
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap
            [(i_0 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kSmallMsgBoxTopBottomBorderTilemap
                [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i_0 = i_0.wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
unsafe fn CallMsgBoxDraw(mut ea: uint32) {
    match ea {
        8749658 => {
            WriteLargeMessageBoxTilemap();
            return;
        }
        8749705 => {
            WriteSmallMessageBoxTilemap();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
unsafe fn MsgBoxMakeHdmaTable() {
    *((&raw mut g_ram as *mut uint8).offset(0x5a8 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((31488 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5aa as ::core::ffi::c_int as isize) as *mut uint16) =
        99 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 31744 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize) as *mut uint16) =
        148 as uint16;
    let mut v0: uint16 = 123 as uint16;
    let mut v1: uint16 = 124 as uint16;
    let mut r20: uint16 = 30 as uint16;
    loop {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap[v0 as usize] =
            (*((&raw mut g_ram as *mut uint8).offset(0x5aa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x5a8 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh2 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5aa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh2 = (*fresh2).wrapping_sub(1);
        let ref mut fresh3 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5a8 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh3 = (*fresh3).wrapping_sub(1);
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap[v1 as usize] =
            (*((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x5a4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        v0 = v0.wrapping_sub(1);
        let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5a6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
        let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
            .offset(0x5a4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh5 = (*fresh5).wrapping_add(1);
        v1 = v1.wrapping_add(1);
        r20 = r20.wrapping_sub(1);
        if !(r20 != 0) {
            break;
        }
    }
    let mut v2: uint16 = (v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
    loop {
        *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 480 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
unsafe fn SetupMessageBoxBg3YscrollHdma() {
    *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .menu
        .palette_backup_in_menu as *mut uint16)
        .offset(64 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint32) =
        -(516947713 as ::core::ffi::c_int) as uint32;
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap[450 as ::core::ffi::c_int as usize] = 12542 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .msgbox
        .indirect_hdma[6 as ::core::ffi::c_int as usize] = 0 as uint8;
    WriteReg(DMAP6 as ::core::ffi::c_int as uint16, 0x42 as uint8);
    WriteReg(BBAD6 as ::core::ffi::c_int as uint16, 0x12 as uint8);
    WriteReg(A1T6L as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(DAS6L as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(A1T6H as ::core::ffi::c_int as uint16, 0x33 as uint8);
    WriteReg(DAS6H as ::core::ffi::c_int as uint16, 0x33 as uint8);
    WriteReg(A1B6 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(DAS60 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(A2A6L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A2A6H as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(NTRL6 as ::core::ffi::c_int as uint16, 0 as uint8);
    MsgBoxMakeHdmaTable();
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0x40 as uint8);
}
unsafe fn SetupPpuForActiveMessageBox(mut r52: uint16) {
    SetupMessageBoxBg3YscrollHdma();
    r52 = (r52 as ::core::ffi::c_int + 22528 as ::core::ffi::c_int) as uint16;
    WriteRegWord(VMADDL as ::core::ffi::c_int as uint16, r52);
    WriteRegWord(DMAP1 as ::core::ffi::c_int as uint16, 0x1801 as uint16);
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16);
    WriteRegWord(
        DAS1L as ::core::ffi::c_int as uint16,
        message_box_das0l_value,
    );
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
}
unsafe fn DrawSpecialButtonAndSetupPpuForLargeMessageBox(mut a: uint16) {
    let mut v1: uint16 = 0 as uint16;
    if a as ::core::ffi::c_int & kButton_A as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        v1 = 2 as uint16;
        if a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            v1 = 4 as uint16;
            if a as ::core::ffi::c_int & kButton_X as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                v1 = 6 as uint16;
                if a as ::core::ffi::c_int & kButton_Y as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v1 = 8 as uint16;
                    if a as ::core::ffi::c_int & kButton_Select as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        v1 = 10 as uint16;
                        if a as ::core::ffi::c_int & kButton_L as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            v1 = 12 as uint16;
                            if a as ::core::ffi::c_int & kButton_R as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                v1 = 14 as uint16;
                            }
                        }
                    }
                }
            }
        }
    }
    *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap as *mut uint16)
        .offset(256 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
        .offset(
            kMsgBoxSpecialButtonTilemapOffs[(*((&raw mut g_ram as *mut uint8)
                .offset(0x1c1f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int as isize,
        ) as *mut uint16) = kTileNumbersForButtonLetters
        [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    SetupPpuForActiveMessageBox(416 as uint16);
}
unsafe fn DrawShootButtonAndSetupPpuForLargeMessageBox() {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
unsafe fn DrawRunButtonAndSetupPpuForLargeMessageBox() {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
unsafe fn SetupPpuForSmallMessageBox() {
    SetupPpuForActiveMessageBox(448 as uint16);
}
unsafe fn SetupPpuForLargeMessageBox() {
    SetupPpuForActiveMessageBox(416 as uint16);
}
unsafe fn CallMsgBoxModify(mut ea: uint32) {
    match ea {
        8750021 => {
            DrawShootButtonAndSetupPpuForLargeMessageBox();
            return;
        }
        8750028 => {
            DrawRunButtonAndSetupPpuForLargeMessageBox();
            return;
        }
        8750134 => {
            SetupPpuForSmallMessageBox();
            return;
        }
        8750145 => {
            SetupPpuForLargeMessageBox();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
unsafe fn InitializeMessageBox() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x1c1f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    CallMsgBoxDraw(
        ((*(g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .draw_initial_tilemap as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
    );
    CallMsgBoxModify(
        ((*(g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .modify_box_func as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
    );
}
static mut kSaveConfirmationSelectionTilemap: [uint16; 96] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3cf8 as ::core::ffi::c_int as uint16,
    0x3ce4 as ::core::ffi::c_int as uint16,
    0x3cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2ced as ::core::ffi::c_int as uint16,
    0x2cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3cf8 as ::core::ffi::c_int as uint16,
    0x3ce4 as ::core::ffi::c_int as uint16,
    0x3cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2ced as ::core::ffi::c_int as uint16,
    0x2cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2cf8 as ::core::ffi::c_int as uint16,
    0x2ce4 as ::core::ffi::c_int as uint16,
    0x2cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3ced as ::core::ffi::c_int as uint16,
    0x3cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
unsafe fn ToggleSaveConfirmationSelection() {
    let ref mut fresh7 = *((&raw mut g_ram as *mut uint8)
        .offset(0x5f9 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = 64 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x5f9 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v0 = 128 as uint16;
    }
    let mut v1: uint16 = 128 as uint16;
    let mut r52: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    loop {
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kSaveConfirmationSelectionTilemap
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = v1.wrapping_add(1);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        r52 -= 1;
        if !(r52 != 0) {
            break;
        }
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_6059A0 as ::core::ffi::c_int as uint16,
    );
    WriteRegWord(DMAP1 as ::core::ffi::c_int as uint16, 0x1801 as uint16);
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x180 as uint16);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
}
unsafe fn RestorePpuForMessageBox() {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
    );
    WriteRegWord(DMAP1 as ::core::ffi::c_int as uint16, 0x1801 as uint16);
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) =
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels;
    WriteReg(
        HDMAEN as ::core::ffi::c_int as uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels,
    );
    *(&raw mut g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize) =
        (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .misc
            .field_3E8[3 as ::core::ffi::c_int as usize];
    *(&raw mut g_ram as *mut uint8).offset(0x6a as ::core::ffi::c_int as isize) =
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize);
    *(&raw mut g_ram as *mut uint8).offset(0x70 as ::core::ffi::c_int as isize) =
        *(&raw mut g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize);
    *(&raw mut g_ram as *mut uint8).offset(0x73 as ::core::ffi::c_int as isize) =
        *(&raw mut g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8);
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(25 as ::core::ffi::c_int as isize) as uint8,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(25 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(26 as ::core::ffi::c_int as isize) as uint8,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(26 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
    );
}
unsafe fn OpenMessageBox_Async() -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize);
    's_65: {
        let mut current_block_10: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                current_block_10 = 1734715445409302261;
            }
            1 => {
                current_block_10 = 1734715445409302261;
            }
            _ => {
                current_block_10 = 1856101646708284338;
            }
        }
        loop {
            match current_block_10 {
                1856101646708284338 => {
                    break 's_65;
                }
                _ => {
                    let mut _coret_: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    HandleMusicQueue();
                    HandleSoundEffects();
                    MsgBoxMakeHdmaTable();
                    if *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        == 0x1800 as ::core::ffi::c_int
                    {
                        current_block_10 = 1856101646708284338;
                        continue;
                    }
                    let ref mut fresh8 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x5a2 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh8 = (*fresh8 as ::core::ffi::c_int + 512 as ::core::ffi::c_int) as uint16;
                    current_block_10 = 1734715445409302261;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
unsafe fn HandleMessageBoxInteraction_Async() -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize);
    's_196: {
        let mut current_block_34: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                if *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == 23 as ::core::ffi::c_int
                    || *((&raw mut g_ram as *mut uint8)
                        .offset(0x1c1f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        == 28 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0x5f9 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    current_block_34 = 16534385235503378281;
                } else {
                    *((&raw mut g_ram as *mut uint8).offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16) = 10 as uint16;
                    if *((&raw mut g_ram as *mut uint8)
                        .offset(0x1c1f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 20 as ::core::ffi::c_int
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0x1c1f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            != 21 as ::core::ffi::c_int
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0x1c1f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            != 22 as ::core::ffi::c_int
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0x1c1f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            != 24 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x77e as ::core::ffi::c_int as isize)
                            as *mut uint16) = 360 as uint16;
                    }
                    current_block_34 = 11768874745533072813;
                }
            }
            1 => {
                current_block_34 = 16534385235503378281;
            }
            2 => {
                current_block_34 = 11768874745533072813;
            }
            3 => {
                current_block_34 = 13352786176918754511;
            }
            _ => {
                current_block_34 = 7746103178988627676;
            }
        }
        loop {
            match current_block_34 {
                7746103178988627676 => {
                    break 's_196;
                }
                11768874745533072813 => {
                    let mut _coret__0: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret__0 != 0 {
                        *_state_variable_ = 2 as uint8;
                        return _coret__0 as CoroutineRet;
                    }
                    HandleMusicQueue();
                    HandleSoundEffects();
                    let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh6 = (*fresh6).wrapping_sub(1);
                    if *fresh6 != 0 {
                        current_block_34 = 11768874745533072813;
                    } else {
                        current_block_34 = 13352786176918754511;
                    }
                }
                16534385235503378281 => {
                    let mut _coret_: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    HandleMusicQueue();
                    HandleSoundEffects();
                    ReadJoypadInputs();
                    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        == 0
                    {
                        current_block_34 = 16534385235503378281;
                        continue;
                    }
                    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & kButton_A as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        current_block_34 = 7746103178988627676;
                        continue;
                    }
                    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & kButton_B as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x5f9 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 2 as uint16;
                        current_block_34 = 7746103178988627676;
                    } else {
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            & (kButton_Select as ::core::ffi::c_int
                                | kButton_Left as ::core::ffi::c_int
                                | kButton_Right as ::core::ffi::c_int)
                            != 0 as ::core::ffi::c_int
                        {
                            ToggleSaveConfirmationSelection();
                            QueueSfx1_Max6(0x37 as uint16);
                        }
                        current_block_34 = 16534385235503378281;
                    }
                }
                _ => {
                    let mut _coret__1: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret__1 != 0 {
                        *_state_variable_ = 3 as uint8;
                        return _coret__1 as CoroutineRet;
                    }
                    ReadJoypadInputs();
                    if (if (*((&raw mut g_ram as *mut uint8)
                        .offset(0xc2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                        < 1 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                    } else {
                        *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                    }) == 0 as ::core::ffi::c_int
                    {
                        current_block_34 = 13352786176918754511;
                    } else {
                        current_block_34 = 7746103178988627676;
                    }
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
unsafe fn CloseMessageBox_Async() -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut g_ram as *mut uint8).offset(0x615 as ::core::ffi::c_int as isize);
    's_58: {
        let mut current_block_10: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 | 1 => {
                current_block_10 = 15518350547184569309;
            }
            _ => {
                current_block_10 = 17965632435239708295;
            }
        }
        loop {
            match current_block_10 {
                17965632435239708295 => {
                    break 's_58;
                }
                _ => {
                    let mut _coret_: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    HandleMusicQueue();
                    HandleSoundEffects();
                    MsgBoxMakeHdmaTable();
                    let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x5a2 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh1 = (*fresh1 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as uint16;
                    if *((&raw mut g_ram as *mut uint8).offset(0x5a2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block_10 = 15518350547184569309;
                    } else {
                        current_block_10 = 17965632435239708295;
                    }
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn DisplayMessageBox_Async(mut a: uint16) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize);
    's_254: {
        let mut current_block_60: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                    as *mut uint16) = a;
                CancelSoundEffects();
                InitializePpuForMessageBoxes();
                ClearMessageBoxBg3Tilemap();
                InitializeMessageBox();
                current_block_60 = 6253325665877293079;
            }
            2 => {
                current_block_60 = 6253325665877293079;
            }
            3 => {
                current_block_60 = 15094944477431624519;
            }
            4 => {
                current_block_60 = 4487664752189900950;
            }
            8 => {
                current_block_60 = 11722929202011421503;
            }
            5 => {
                current_block_60 = 4928684536100880175;
            }
            6 => {
                current_block_60 = 12462294883523643517;
            }
            7 => {
                current_block_60 = 17013389759922219937;
            }
            _ => {
                current_block_60 = 14945149239039849694;
            }
        }
        match current_block_60 {
            6253325665877293079 => {
                let mut _coret_: uint8 = OpenMessageBox_Async() as uint8;
                if _coret_ != 0 {
                    *_state_variable_ = 2 as uint8;
                    return _coret_ as CoroutineRet;
                }
                current_block_60 = 15094944477431624519;
            }
            _ => {}
        }
        match current_block_60 {
            15094944477431624519 => {
                let mut _coret__0: uint8 = HandleMessageBoxInteraction_Async() as uint8;
                if _coret__0 != 0 {
                    *_state_variable_ = 3 as uint8;
                    return _coret__0 as CoroutineRet;
                }
                current_block_60 = 4487664752189900950;
            }
            _ => {}
        }
        match current_block_60 {
            4487664752189900950 => {
                let mut _coret__1: uint8 = CloseMessageBox_Async() as uint8;
                if _coret__1 != 0 {
                    *_state_variable_ = 4 as uint8;
                    return _coret__1 as CoroutineRet;
                }
                if *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == 28 as ::core::ffi::c_int
                    && *((&raw mut g_ram as *mut uint8).offset(0x5f9 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 2 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                        as *mut uint16) = 24 as uint16;
                    ClearMessageBoxBg3Tilemap();
                    QueueSfx1_Max6(0x2e as uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16) = 160 as uint16;
                    current_block_60 = 18317007320854588510;
                } else {
                    current_block_60 = 6717214610478484138;
                }
            }
            _ => {}
        }
        loop {
            match current_block_60 {
                14945149239039849694 => {
                    break 's_254;
                }
                17013389759922219937 => {
                    let mut _coret__5: uint8 = CloseMessageBox_Async() as uint8;
                    if _coret__5 != 0 {
                        *_state_variable_ = 7 as uint8;
                        return _coret__5 as CoroutineRet;
                    }
                    *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
                        as *mut uint16) = 28 as uint16;
                    current_block_60 = 6717214610478484138;
                }
                11722929202011421503 => {
                    let mut _coret__2: uint8 = WaitForNMI_NoUpdate_Async() as uint8;
                    if _coret__2 != 0 {
                        *_state_variable_ = 8 as uint8;
                        return _coret__2 as CoroutineRet;
                    }
                    let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh0 = (*fresh0).wrapping_sub(1);
                    if *fresh0 != 0 {
                        current_block_60 = 18317007320854588510;
                        continue;
                    }
                    InitializeMessageBox();
                    current_block_60 = 4928684536100880175;
                }
                18317007320854588510 => {
                    HandleMusicQueue();
                    HandleSoundEffects();
                    current_block_60 = 11722929202011421503;
                }
                4928684536100880175 => {
                    let mut _coret__3: uint8 = OpenMessageBox_Async() as uint8;
                    if _coret__3 != 0 {
                        *_state_variable_ = 5 as uint8;
                        return _coret__3 as CoroutineRet;
                    }
                    current_block_60 = 12462294883523643517;
                }
                12462294883523643517 => {
                    let mut _coret__4: uint8 = HandleMessageBoxInteraction_Async() as uint8;
                    if _coret__4 != 0 {
                        *_state_variable_ = 6 as uint8;
                        return _coret__4 as CoroutineRet;
                    }
                    current_block_60 = 17013389759922219937;
                }
                _ => {
                    ClearMessageBoxBg3Tilemap();
                    RestorePpuForMessageBox();
                    QueueSamusMovementSfx();
                    if *((&raw mut g_ram as *mut uint8)
                        .offset(0x1c1f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        == 20 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x998 as ::core::ffi::c_int as isize)
                            as *mut uint16) = kGameState_12_Pausing as ::core::ffi::c_int as uint16;
                    }
                    current_block_60 = 14945149239039849694;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn DisplayMessageBox_Poll(mut a: uint16) -> ::core::ffi::c_int {
    if a as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x1c1f as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return *((&raw mut g_ram as *mut uint8).offset(0x5f9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xdc8 as ::core::ffi::c_int as isize) as *mut uint16) =
        a;
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn DisplayMessageBox(mut a: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0xdc8 as ::core::ffi::c_int as isize) as *mut uint16) =
        a;
}
