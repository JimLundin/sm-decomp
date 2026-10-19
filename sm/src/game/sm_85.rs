use crate::game::sm_80::{
    HandleMusicQueue, QueueSfx1_Max6, ReadJoypadInputs, WaitForNMI_NoUpdate_Async,
};
use crate::game::sm_82::{CancelSoundEffects, HandleSoundEffects, QueueSamusMovementSfx};
use crate::game_state::Game;
use crate::rtl::{ReadReg, ReadRegWord, RomPtr, Unreachable, WriteReg, WriteRegWord};

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
unsafe fn RomPtr_85(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x850000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
pub const fnWriteLargeMessageBoxTilemap: uint32 = 8749658;
pub const fnWriteSmallMessageBoxTilemap: uint32 = 8749705;
pub const fnDrawShootButtonAndSetupPpuForLargeMessageBox: uint32 = 8750021;
pub const fnDrawRunButtonAndSetupPpuForLargeMessageBox: uint32 = 8750028;
pub const fnSetupPpuForSmallMessageBox: uint32 = 8750134;
pub const fnSetupPpuForLargeMessageBox: uint32 = 8750145;

static kMsgBoxSpecialButtonTilemapOffs: [uint16; 27] = [
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
static kTileNumbersForButtonLetters: [uint16; 8] = [
    0x28e0 as ::core::ffi::c_int as uint16,
    0x3ce1 as ::core::ffi::c_int as uint16,
    0x2cf7 as ::core::ffi::c_int as uint16,
    0x38f8 as ::core::ffi::c_int as uint16,
    0x38d0 as ::core::ffi::c_int as uint16,
    0x38eb as ::core::ffi::c_int as uint16,
    0x38f1 as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
];
unsafe fn InitializePpuForMessageBoxes(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16) = 0 as uint16;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb1 as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0x1f as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .msgbox
        .backup_of_enabled_hdma_channels = *(&raw mut game.g_ram as *mut uint8).offset(0x85);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .misc
        .field_3E8[3 as ::core::ffi::c_int as usize] =
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 88 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6a) = 23 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x70) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x73) = 0 as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x20 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    ReadReg(BG3HOFS as ::core::ffi::c_int as uint16, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    ReadReg(BG3VOFS as ::core::ffi::c_int as uint16, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    let mut i: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
        game,
    );
    ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16, game);
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x3981 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn ClearMessageBoxBg3Tilemap(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 1790 as int16;
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
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
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
static kLargeMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
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
static kSmallMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
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
unsafe fn WriteMessageTilemap(game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16) = 112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16) = 124 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16) = 0 as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 112 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[i as usize] = 0 as uint16;
        i += 1;
    }
    let mut r0: uint16 = (*(game.g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as isize,
        ))
    .message_tilemap as uint16;
    let mut r9: uint16 = ((*(game.g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f) as *mut uint16) as isize))
    .message_tilemap as ::core::ffi::c_int
        - r0 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = r9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    r9 = (r9 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
    game.message_box_das0l_value = r9;
    let mut v1: uint16 = 32 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            *((RomPtr_85 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r0 as uint16_t, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16);
        v1 = v1.wrapping_add(1);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    return v1;
}
unsafe fn WriteLargeMessageBoxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kLargeMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap(game);
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
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
unsafe fn WriteSmallMessageBoxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kSmallMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap(game);
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
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
unsafe fn CallMsgBoxDraw(mut ea: uint32, game: &mut Game) {
    match ea {
        8749658 => {
            WriteLargeMessageBoxTilemap(game);
            return;
        }
        8749705 => {
            WriteSmallMessageBoxTilemap(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
fn MsgBoxMakeHdmaTable(game: &mut Game) {
    let radius = u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]);
    let upper = 0x7b00u16.wrapping_sub(radius) >> 8;
    let lower = radius.wrapping_add(0x7c00) >> 8;

    // Each scanline and its edge move together, so their difference stays constant.
    for word in game.g_ram[0x30bc..0x30f8].chunks_exact_mut(2) {
        word.copy_from_slice(&99u16.wrapping_sub(upper).to_le_bytes());
    }
    for word in game.g_ram[0x30f8..0x3134].chunks_exact_mut(2) {
        word.copy_from_slice(&148u16.wrapping_sub(lower).to_le_bytes());
    }
    game.g_ram[0x3134..0x31e0].fill(0);

    // Preserve the scratch words left after all 30 scanlines.
    game.g_ram[0x5a4..0x5a6].copy_from_slice(&lower.wrapping_add(30).to_le_bytes());
    game.g_ram[0x5a6..0x5a8].copy_from_slice(&178u16.to_le_bytes());
    game.g_ram[0x5a8..0x5aa].copy_from_slice(&upper.wrapping_sub(30).to_le_bytes());
    game.g_ram[0x5aa..0x5ac].copy_from_slice(&69u16.to_le_bytes());
}
unsafe fn SetupMessageBoxBg3YscrollHdma(game: &mut Game) {
    game.g_ram[0x3380..0x3387].copy_from_slice(&[0xff, 0x00, 0x30, 0xe1, 0xfe, 0x30, 0x00]);
    WriteReg(DMAP6 as ::core::ffi::c_int as uint16, 0x42 as uint8, game);
    WriteReg(BBAD6 as ::core::ffi::c_int as uint16, 0x12 as uint8, game);
    WriteReg(A1T6L as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(DAS6L as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(A1T6H as ::core::ffi::c_int as uint16, 0x33 as uint8, game);
    WriteReg(DAS6H as ::core::ffi::c_int as uint16, 0x33 as uint8, game);
    WriteReg(A1B6 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS60 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(A2A6L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A6H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(NTRL6 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    MsgBoxMakeHdmaTable(game);
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
}
unsafe fn SetupPpuForActiveMessageBox(mut r52: uint16, game: &mut Game) {
    SetupMessageBoxBg3YscrollHdma(game);
    r52 = (r52 as ::core::ffi::c_int + 22528 as ::core::ffi::c_int) as uint16;
    WriteRegWord(VMADDL as ::core::ffi::c_int as uint16, r52, game);
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(
        DAS1L as ::core::ffi::c_int as uint16,
        game.message_box_das0l_value,
        game,
    );
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn DrawSpecialButtonAndSetupPpuForLargeMessageBox(mut a: uint16, game: &mut Game) {
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
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap as *mut uint16)
        .offset(256) as *mut uint16 as *mut uint8)
        .offset(
            kMsgBoxSpecialButtonTilemapOffs[(*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int as isize,
        ) as *mut uint16) = kTileNumbersForButtonLetters
        [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    SetupPpuForActiveMessageBox(416 as uint16, game);
}
unsafe fn DrawShootButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16),
        game,
    );
}
unsafe fn DrawRunButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16),
        game,
    );
}
unsafe fn SetupPpuForSmallMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(448 as uint16, game);
}
unsafe fn SetupPpuForLargeMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(416 as uint16, game);
}
unsafe fn CallMsgBoxModify(mut ea: uint32, game: &mut Game) {
    match ea {
        8750021 => {
            DrawShootButtonAndSetupPpuForLargeMessageBox(game);
            return;
        }
        8750028 => {
            DrawRunButtonAndSetupPpuForLargeMessageBox(game);
            return;
        }
        8750134 => {
            SetupPpuForSmallMessageBox(game);
            return;
        }
        8750145 => {
            SetupPpuForLargeMessageBox(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn InitializeMessageBox(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c1f)
        as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    CallMsgBoxDraw(
        ((*(game.g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .draw_initial_tilemap as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
        game,
    );
    CallMsgBoxModify(
        ((*(game.g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .modify_box_func as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
static kSaveConfirmationSelectionTilemap: [uint16; 96] = [
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
unsafe fn ToggleSaveConfirmationSelection(game: &mut Game) {
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = 64 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v0 = 128 as uint16;
    }
    let mut v1: uint16 = 128 as uint16;
    let mut r52: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
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
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x180 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn RestorePpuForMessageBox(game: &mut Game) {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels;
    WriteReg(
        HDMAEN as ::core::ffi::c_int as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels,
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .misc
            .field_3E8[3 as ::core::ffi::c_int as usize];
    *(&raw mut game.g_ram as *mut uint8).offset(0x6a) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x69);
    *(&raw mut game.g_ram as *mut uint8).offset(0x70) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e);
    *(&raw mut game.g_ram as *mut uint8).offset(0x73) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x71);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8, game);
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(25) as uint8,
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(25)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(26) as uint8,
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(26)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
        game,
    );
}
unsafe fn OpenMessageBox_Async(game: &mut Game) -> CoroutineRet {
    match game.g_ram[0x615] {
        0 => game.g_ram[0x5a2..0x5a4].fill(0),
        1 => {}
        _ => {
            game.g_ram[0x615] = 0;
            return 0;
        }
    }
    loop {
        let result = WaitForNMI_NoUpdate_Async(game);
        if result != 0 {
            game.g_ram[0x615] = 1;
            return result;
        }
        HandleMusicQueue(game);
        HandleSoundEffects(game);
        MsgBoxMakeHdmaTable(game);
        let radius = u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]);
        if radius == 0x1800 {
            break;
        }
        game.g_ram[0x5a2..0x5a4].copy_from_slice(&radius.wrapping_add(0x200).to_le_bytes());
    }
    game.g_ram[0x615] = 0;
    0
}
unsafe fn HandleMessageBoxInteraction_Async(game: &mut Game) -> CoroutineRet {
    enum Phase {
        Confirmation,
        Delay,
        Dismiss,
    }
    let mut phase = match game.g_ram[0x615] {
        0 => {
            let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]);
            if matches!(message, 23 | 28) {
                game.g_ram[0x5f9..0x5fb].fill(0);
                Phase::Confirmation
            } else {
                game.g_ram[0x77e..0x780].copy_from_slice(&10u16.to_le_bytes());
                if !matches!(message, 20 | 21 | 22 | 24) {
                    game.g_ram[0x77e..0x780].copy_from_slice(&360u16.to_le_bytes());
                }
                Phase::Delay
            }
        }
        1 => Phase::Confirmation,
        2 => Phase::Delay,
        3 => Phase::Dismiss,
        _ => {
            game.g_ram[0x615] = 0;
            return 0;
        }
    };
    loop {
        match phase {
            Phase::Confirmation => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 1;
                    return result;
                }
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                ReadJoypadInputs(game);
                let keys = u16::from_le_bytes([game.g_ram[0x8f], game.g_ram[0x90]]);
                if keys & kButton_A as u16 != 0 {
                    break;
                }
                if keys & kButton_B as u16 != 0 {
                    game.g_ram[0x5f9..0x5fb].copy_from_slice(&2u16.to_le_bytes());
                    break;
                }
                if keys & (kButton_Select | kButton_Left | kButton_Right) as u16 != 0 {
                    ToggleSaveConfirmationSelection(game);
                    QueueSfx1_Max6(0x37, game);
                }
            }
            Phase::Delay => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 2;
                    return result;
                }
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                let timer =
                    u16::from_le_bytes([game.g_ram[0x77e], game.g_ram[0x77f]]).wrapping_sub(1);
                game.g_ram[0x77e..0x780].copy_from_slice(&timer.to_le_bytes());
                if timer == 0 {
                    phase = Phase::Dismiss;
                }
            }
            Phase::Dismiss => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 3;
                    return result;
                }
                ReadJoypadInputs(game);
                let bug_fix_counter = u16::from_le_bytes([game.g_ram[0xc2], game.g_ram[0xc3]]);
                let keys = if bug_fix_counter == 0 {
                    u16::from_le_bytes([game.g_ram[0x8f], game.g_ram[0x90]])
                } else {
                    game.joypad1_lastkeys()
                };
                if keys != 0 {
                    break;
                }
            }
        }
    }
    game.g_ram[0x615] = 0;
    0
}
unsafe fn CloseMessageBox_Async(game: &mut Game) -> CoroutineRet {
    if matches!(game.g_ram[0x615], 0 | 1) {
        loop {
            let result = WaitForNMI_NoUpdate_Async(game);
            if result != 0 {
                game.g_ram[0x615] = 1;
                return result;
            }
            HandleMusicQueue(game);
            HandleSoundEffects(game);
            MsgBoxMakeHdmaTable(game);
            let radius =
                u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]).wrapping_sub(0x200);
            game.g_ram[0x5a2..0x5a4].copy_from_slice(&radius.to_le_bytes());
            if radius & 0x8000 != 0 {
                break;
            }
        }
    }
    game.g_ram[0x615] = 0;
    0
}
pub unsafe fn DisplayMessageBox_Async(a: uint16, game: &mut Game) -> CoroutineRet {
    enum Phase {
        Open,
        Interact,
        Close,
        SaveDelay,
        SaveWait,
        OpenSaved,
        InteractSaved,
        CloseSaved,
        Restore,
    }
    let mut phase = match game.g_ram[0x614] {
        0 => {
            game.g_ram[0x1c1f..0x1c21].copy_from_slice(&a.to_le_bytes());
            CancelSoundEffects(game);
            InitializePpuForMessageBoxes(game);
            ClearMessageBoxBg3Tilemap(game);
            InitializeMessageBox(game);
            Phase::Open
        }
        2 => Phase::Open,
        3 => Phase::Interact,
        4 => Phase::Close,
        8 => Phase::SaveWait,
        5 => Phase::OpenSaved,
        6 => Phase::InteractSaved,
        7 => Phase::CloseSaved,
        _ => {
            game.g_ram[0x614] = 0;
            return 0;
        }
    };
    loop {
        match phase {
            Phase::Open | Phase::OpenSaved => {
                let saved = matches!(phase, Phase::OpenSaved);
                let result = OpenMessageBox_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 5 } else { 2 };
                    return result;
                }
                phase = if saved {
                    Phase::InteractSaved
                } else {
                    Phase::Interact
                };
            }
            Phase::Interact | Phase::InteractSaved => {
                let saved = matches!(phase, Phase::InteractSaved);
                let result = HandleMessageBoxInteraction_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 6 } else { 3 };
                    return result;
                }
                phase = if saved {
                    Phase::CloseSaved
                } else {
                    Phase::Close
                };
            }
            Phase::Close | Phase::CloseSaved => {
                let saved = matches!(phase, Phase::CloseSaved);
                let result = CloseMessageBox_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 7 } else { 4 };
                    return result;
                }
                if saved {
                    game.g_ram[0x1c1f..0x1c21].copy_from_slice(&28u16.to_le_bytes());
                    phase = Phase::Restore;
                } else {
                    let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]);
                    if message == 28
                        && u16::from_le_bytes([game.g_ram[0x5f9], game.g_ram[0x5fa]]) != 2
                    {
                        game.g_ram[0x1c1f..0x1c21].copy_from_slice(&24u16.to_le_bytes());
                        ClearMessageBoxBg3Tilemap(game);
                        QueueSfx1_Max6(0x2e, game);
                        game.g_ram[0x77e..0x780].copy_from_slice(&160u16.to_le_bytes());
                        phase = Phase::SaveDelay;
                    } else {
                        phase = Phase::Restore;
                    }
                }
            }
            Phase::SaveDelay => {
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                phase = Phase::SaveWait;
            }
            Phase::SaveWait => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = 8;
                    return result;
                }
                let timer =
                    u16::from_le_bytes([game.g_ram[0x77e], game.g_ram[0x77f]]).wrapping_sub(1);
                game.g_ram[0x77e..0x780].copy_from_slice(&timer.to_le_bytes());
                if timer == 0 {
                    InitializeMessageBox(game);
                    phase = Phase::OpenSaved;
                } else {
                    phase = Phase::SaveDelay;
                }
            }
            Phase::Restore => {
                ClearMessageBoxBg3Tilemap(game);
                RestorePpuForMessageBox(game);
                QueueSamusMovementSfx(game);
                if u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) == 20 {
                    game.g_ram[0x998..0x99a]
                        .copy_from_slice(&(kGameState_12_Pausing as u16).to_le_bytes());
                }
                break;
            }
        }
    }
    game.g_ram[0x614] = 0;
    0
}
pub fn DisplayMessageBox_Poll(a: uint16, game: &mut Game) -> ::core::ffi::c_int {
    if a == u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) {
        game.g_ram[0x1c1f..0x1c21].fill(0);
        return u16::from_le_bytes([game.g_ram[0x5f9], game.g_ram[0x5fa]]) as ::core::ffi::c_int;
    }
    game.g_ram[0xdc8..0xdca].copy_from_slice(&a.to_le_bytes());
    -1
}
pub fn DisplayMessageBox(a: uint16, game: &mut Game) {
    game.g_ram[0xdc8..0xdca].copy_from_slice(&a.to_le_bytes());
}
