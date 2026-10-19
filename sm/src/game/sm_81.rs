use crate::game::sm_80::{
    HandleFadeIn, HandleFadeOut, HasQueuedMusic, LoadFromLoadStation, LoadMirrorOfExploredMapTiles,
    QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx3_Max6, ScreenOff, ScreenOn, SetupDmaTransfer,
};
use crate::game::sm_82::{
    AdvanceGradualColorChangeOfPalette, DisplayMapElevatorDestinations,
    DrawBorderAroundDataClearMode, DrawBorderAroundDataCopyMode, DrawBorderAroundSamusData,
    DrawFileCopyArrow, DrawFileSelectMapIcons, DrawMapScrollArrowAndCheckToScroll,
    DrawMenuSelectionMissile, DrawRoomSelectMap, DrawRoomSelectMapAreaLabel,
    HandleGameOverBabyMetroid, MapScrolling, SetupMapScrollingForFileSelectMap,
};
use crate::game::sm_88::{
    DisableHdmaObjects, EnableHdmaObjects, SpawnHdmaObject, WaitUntilEndOfVblankAndClearHdma,
};
use crate::game_state::Game;
use crate::rtl::{RomPtr, RtlWriteSram, Unreachable, WriteReg};
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, Func_V,
    LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry,
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
pub use crate::types::{EnemyData, GameState, RoomDefHeader, SpawnHdmaObject_Args};
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
pub use crate::types::{ExpandingSquareTransitionHdma, MapIconDataPointers, StartDmaCopy};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExpandingSquareVels {
    pub left_subvel: uint16,
    pub left_vel: uint16,
    pub right_subvel: uint16,
    pub right_vel: uint16,
    pub top_subvel: uint16,
    pub top_vel: uint16,
    pub bottom_subvel: uint16,
    pub bottom_vel: uint16,
}
pub use crate::types::{
    Consts_81, MapScrollArrowData, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram4000,
    Ram4000_Backups,
};
pub const addr_kAreaSelectBackgroundTilemaps: Consts_81 = 48922;
pub const addr_kAreaSelectForegroundTilemap: Consts_81 = 46874;
pub const addr_kMenuTilemap_DataCleared: Consts_81 = 46810;
pub const addr_kMenuTilemap_ClearSamusA: Consts_81 = 46746;
pub const addr_kMenuTilemap_ClearWhichData: Consts_81 = 46682;
pub const addr_kMenuTilemap_CopyCompleted: Consts_81 = 46650;
pub const addr_kMenuTilemap_No: Consts_81 = 46634;
pub const addr_kMenuTilemap_Yes: Consts_81 = 46618;
pub const addr_kMenuTilemap_IsThisOk: Consts_81 = 46594;
pub const addr_kMenuTilemap_CopySamusToSamus: Consts_81 = 46536;
pub const addr_kMenuTilemap_CopySamusToWhere: Consts_81 = 46486;
pub const addr_kMenuTilemap_CopyWhichData: Consts_81 = 46452;
pub const addr_kMenuTilemap_DataClearMode: Consts_81 = 46388;
pub const addr_kMenuTilemap_DataCopyMode: Consts_81 = 46328;
pub const addr_kMenuTilemap_Exit: Consts_81 = 46318;
pub const addr_kMenuTilemap_DataClear: Consts_81 = 46296;
pub const addr_kMenuTilemap_DataCopy: Consts_81 = 46276;
pub const addr_kMenuTilemap_NoData: Consts_81 = 46252;
pub const addr_word_81B4A8: Consts_81 = 46248;
pub const addr_kMenuTilemap_TIME: Consts_81 = 46240;
pub const addr_kMenuTilemap_Energy: Consts_81 = 46230;
pub const addr_kMenuTilemap_SamusC: Consts_81 = 46198;
pub const addr_kMenuTilemap_SamusB: Consts_81 = 46166;
pub const addr_kMenuTilemap_SamusA: Consts_81 = 46134;
pub const addr_kMenuTilemap_SamusData: Consts_81 = 46090;
pub const addr_kDownMapScrollArrowData: Consts_81 = 44880;
pub const addr_kUpMapScrollArrowData: Consts_81 = 44870;
pub const addr_kRightMapScrollArrowData: Consts_81 = 44860;
pub const addr_kLeftMapScrollArrowData: Consts_81 = 44850;
pub const addr_kMenuTilemap_NoGoToTitle: Consts_81 = 37792;
pub const addr_kMenuTilemap_YesReturnToGame: Consts_81 = 37708;
pub const addr_kMenuTilemap_TryAgain: Consts_81 = 37684;
pub const addr_kMenuTilemap_FindMetroidLarva: Consts_81 = 37636;
pub const addr_kMenuTilemap_GameOver: Consts_81 = 37596;
pub use crate::types::Consts_82;
pub const addr_stru_82F4D6: Consts_82 = 62678;
pub const addr_stru_82F4D0: Consts_82 = 62672;
pub const addr_stru_82F4CA: Consts_82 = 62666;
pub const addr_stru_82F4C4: Consts_82 = 62660;
pub const addr_stru_82F4B8: Consts_82 = 62648;
pub const addr_off_82F4B6: Consts_82 = 62646;
pub const addr_kMapIconDataPointers: Consts_82 = 51147;
pub const addr_kDummySamusWireframeTilemap: Consts_82 = 50745;
pub const addr_kEquipmentTilemapOffs_Boots: Consts_82 = 49282;
pub const addr_kEquipmentTilemapOffs_Suits: Consts_82 = 49270;
pub const addr_kEquipmentTilemapOffs_Weapons: Consts_82 = 49260;
pub const addr_kEquipmentTilemapOffs_Tanks: Consts_82 = 49256;
pub const addr_kEquipmentScreenTilemap_Blank: Consts_82 = 49178;
pub const addr_kEquipmentScreenTilemap_AUTO: Consts_82 = 48938;
pub const addr_kEquipmentScreenTilemap_MANUAL: Consts_82 = 48930;
pub const addr_kgameOverBabyMetridInstructionList: Consts_82 = 48167;
pub const addr_stru_82B9BE: Consts_82 = 47550;
pub const addr_stru_82B9B4: Consts_82 = 47540;
pub const addr_stru_82B9AA: Consts_82 = 47530;
pub const addr_stru_82B9A0: Consts_82 = 47520;
pub const addr_kPauseMenuMapData: Consts_82 = 38679;
pub const true_0: ::core::ffi::c_int = 1;
#[inline]
unsafe fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe fn RomPtr_81(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x810000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_82(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x820000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_8C(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x8c0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_92(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x920000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_93(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x930000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gOamEnt(mut v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    return (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_RoomDefHeader(mut a: uint16, game: &mut Game) -> *mut RoomDefHeader {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut RoomDefHeader;
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
static kFileSelectMap_AreaIndexes: [uint16; 6] = [
    0 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    5 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
];
pub unsafe fn SoftReset(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 0xffff as uint16;
}
pub unsafe fn SaveToSram(mut a: uint16, game: &mut Game) {
    let mut v7: uint16 = 0;
    let mut v11: uint16 = 0;
    let mut r20: uint16 = 0 as uint16;
    let mut r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
        as ::core::ffi::c_int
        * 256 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v4 as isize) as *mut uint8 as *mut uint16);
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    PackMapToSave(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd916) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd918) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16);
    let mut v5: uint16 = *(game.g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v6: *mut uint16 = (&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16;
    loop {
        let fresh94 = v6;
        v6 = v6.offset(1);
        v7 = *fresh94;
        *(game.g_sram.offset(
            (2 as ::core::ffi::c_int * (v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
                as isize,
        ) as *mut uint8 as *mut uint16) = v7;
        r20 = (r20 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint16;
        v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(v6 != (&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16) {
            break;
        }
    }
    let mut v8: uint16 = r18;
    let mut v9: uint16 = r20;
    let mut v10: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v10) as isize) as *mut uint8 as *mut uint16) =
        r20;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9;
    v11 = !(v9 as ::core::ffi::c_int) as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    RtlWriteSram(game);
}
pub unsafe fn LoadFromSram(mut a: uint16, game: &mut Game) -> uint8 {
    let mut r20: uint16 = 0 as uint16;
    let mut r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = *(game.g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((a as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize);
    let mut v2: *mut uint16 = (&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16;
    loop {
        let mut v3: uint16 = *(game.g_sram.offset(0) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let fresh1 = v2;
        v2 = v2.offset(1);
        *fresh1 = v3;
        r20 = (r20 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(v2 != (&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16) {
            break;
        }
    }
    let mut v4: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if r20 as ::core::ffi::c_int
        == *(game.g_sram.offset(0) as *mut uint16).offset(v4 as isize) as ::core::ffi::c_int
        && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
            == *(game.g_sram.offset(0x8) as *mut uint16).offset(v4 as isize) as ::core::ffi::c_int
        || r20 as ::core::ffi::c_int
            == *(game.g_sram.offset(0x1ff0) as *mut uint16).offset(v4 as isize)
                as ::core::ffi::c_int
            && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
                == *(game.g_sram.offset(0x1ff8) as *mut uint16).offset(v4 as isize)
                    as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        UnpackMapFromSave(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd916) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd918) as *mut uint16);
        return 0 as uint8;
    } else {
        r20 = 0 as uint16;
        let mut v7: uint16 = *(game.g_rom.offset(
            (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v8: uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as uint16;
        let mut v9: uint16 = 0 as uint16;
        loop {
            *(game.g_sram.offset(0) as *mut uint16)
                .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v9;
            v9 = (v9 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
            r20 = v9;
            v7 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v8 = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(v8 as ::core::ffi::c_long
                != ((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16 as *mut uint8)
                    .offset_from(&raw mut game.g_ram as *mut uint8)
                    as ::core::ffi::c_long)
            {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn UnpackMapFromSave(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 1792 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 6 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = *(game.g_rom.offset(
            (((0x818131 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818131 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(game.g_rom.offset(
                (((0x8182d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8182d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(i_0 as isize) as uint16_t,
            game,
        );
        let mut v3: ::core::ffi::c_int = *(game.g_rom.offset(
            (((0x818138 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818138 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let mut unpacked: *mut uint8 =
            (&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16 as *mut uint8;
        loop {
            let fresh2 = r0;
            r0 = r0.offset(1);
            let mut v4: uint16 =
                (i_0 * 256 as ::core::ffi::c_int + *fresh2 as ::core::ffi::c_int) as uint16;
            let fresh3 = v3;
            v3 = v3 + 1;
            *unpacked.offset(v4 as isize) = *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd91c)
                .offset(fresh3 as isize);
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        i_0 += 1;
    }
}
pub unsafe fn PackMapToSave(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = *(game.g_rom.offset(
            (((0x818131 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818131 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(i as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(game.g_rom.offset(
                (((0x8182d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8182d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(i as isize) as uint16_t,
            game,
        );
        let mut v1: ::core::ffi::c_int = *(game.g_rom.offset(
            (((0x818138 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818138 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i as isize) as ::core::ffi::c_int;
        let mut unpacked: *mut uint8 =
            (&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16 as *mut uint8;
        loop {
            let fresh95 = r0;
            r0 = r0.offset(1);
            let mut v3: ::core::ffi::c_int =
                i * 256 as ::core::ffi::c_int + *fresh95 as ::core::ffi::c_int;
            let fresh96 = v1;
            v1 = v1 + 1;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd91c)
                .offset(fresh96 as isize) = *unpacked.offset(v3 as isize);
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        i += 1;
    }
}
pub unsafe fn DrawSpritemap(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r22: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t, game);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int && idx < 0x200 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int =
            y_r18 as uint8 as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int;
        if if *pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            (y >= 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else if y & 0x100 as ::core::ffi::c_int != 0 {
            (y as uint8 as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((y as uint8 as ::core::ffi::c_int) < 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } != 0
        {
            x = 0x180 as ::core::ffi::c_int;
            y = 0xe0 as ::core::ffi::c_int;
        }
        (*oam).xcoord = x as uint8;
        (*oam).ycoord = y as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r22 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh9 = (*fresh9 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        idx += 4 as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawSpritemapOffScreen(
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r22: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtr_8C(j as uint16_t, game);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int && idx < 0x200 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int =
            y_r18 as uint8 as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int;
        if if *pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            (y < 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else if y & 0x100 as ::core::ffi::c_int != 0 {
            ((y as uint8 as ::core::ffi::c_int) < 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            (y as uint8 as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } != 0
        {
            x = 0x180 as ::core::ffi::c_int;
            y = 0xe0 as ::core::ffi::c_int;
        }
        (*oam).xcoord = x as uint8;
        (*oam).ycoord = y as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r22 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh10 = (*fresh10 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        idx += 4 as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawMenuSpritemap(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    mut chr_r3: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtr_82(
        *(game.g_rom.offset(
            (((0x82c569 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x82c569 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(a as isize) as uint16_t,
        game,
    );
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: uint16 =
            (k as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh4 = (*fresh4 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord = (j as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r3 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawSamusSpritemap(
    mut a: uint16,
    mut x_pos: uint16,
    mut y_pos: uint16,
    game: &mut Game,
) {
    if *(game.g_rom.offset(
        (((0x92808d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x92808d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut pp: *const uint8 = RomPtr_92(
        *(game.g_rom.offset(
            (((0x92808d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x92808d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(a as isize) as uint16_t,
        game,
    );
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut x: uint16 =
            (x_pos as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut v9: *mut OamEnt = gOamEnt(idx, game);
        (*v9).xcoord = x as uint8;
        let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh8 = (*fresh8 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v9).ycoord = (y_pos as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int) as uint8;
        *(&raw mut (*v9).charnum as *mut uint16) = *(pp.offset(3) as *mut uint16);
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawBeamGrappleSpritemap(
    mut a: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    game: &mut Game,
) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *(game.g_rom.offset(
                (((0x93a1a1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x93a1a1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(a as isize) as uint16_t,
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawProjectileSpritemap(
    mut k: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    game: &mut Game,
) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *((&raw mut game.g_ram as *mut uint8).offset(0xcb8) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as uint16_t,
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawGrappleOrProjectileSpritemap(
    mut pp: *const uint8,
    mut x_r20: uint16,
    mut y_r18: uint16,
    game: &mut Game,
) {
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut v4: *mut OamEnt = gOamEnt(idx, game);
        (*v4).xcoord = x as uint8;
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh5 = (*fresh5 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v4).ycoord = (y_r18 as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int) as uint8;
        *(&raw mut (*v4).charnum as *mut uint16) = *(pp.offset(3) as *mut uint16);
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawSpritemapWithBaseTile(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
    game: &mut Game,
) {
    if j as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return;
    }
    let mut pp: *mut uint8 = RomPtrWithBank(db, j as uint16_t, game) as *mut uint8;
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: uint16 = (r20_x as ::core::ffi::c_int
            + *(pp.offset(0) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord =
            (r18_y as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawSpritemapWithBaseTile2(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t, game);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: uint16 =
            (r20_x as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh12 = (*fresh12 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawSpritemapWithBaseTileOffscreen(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t, game);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: ::core::ffi::c_int =
            r20_x as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        (*oam).xcoord = x as uint8;
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh13 = (*fresh13 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawEprojSpritemapWithBaseTile(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r26: uint16,
    mut chr_r28: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t, game);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh6 = (*fresh6 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn DrawEprojSpritemapWithBaseTileOffscreen(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r26: uint16,
    mut chr_r28: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t, game);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh7 = (*fresh7 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn GameOverMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        ScreenOff(game);
        QueueSfx3_Max6(1 as uint16, game);
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        let mut v0: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v0 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize] =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v0 as isize);
            v0 = v0.wrapping_add(1);
            if !(((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 640 as ::core::ffi::c_int) as usize] =
                *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x51)
                    .offset((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as isize)
                    as *mut uint16);
            v1 = v1.wrapping_add(1);
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 54 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        MapVramForMenu(game);
        LoadInitialMenuTiles(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
        LoadMenuPalettes(game);
        let ref mut fresh89 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh89 = (*fresh89).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_24_FadeIn(game: &mut Game) {
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh90 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh90 = (*fresh90).wrapping_add(1);
    }
}
pub unsafe fn MapVramForMenu(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 81 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 88 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 92 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
}
pub unsafe fn LoadInitialMenuTiles(game: &mut Game) {
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    static unk_818DF1: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x8e8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x8e8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x5600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818DF1 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x30 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    static unk_818E11: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0xb68000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0xb68000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E11 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    static unk_818E31: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0xb6c000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0xb6c000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E31 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    static unk_818E51: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x8ed600 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x8ed600 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E51 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
pub unsafe fn LoadMenuPalettes(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            *(game.g_rom.offset(
                (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_3_Main(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let ref mut fresh91 = *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16);
        *fresh91 = (*fresh91 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_B as ::core::ffi::c_int
            | kButton_Start as ::core::ffi::c_int
            | kButton_A as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) != 0 {
            let ref mut fresh92 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh92 = (*fresh92).wrapping_add(1);
        } else {
            SaveToSram(
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
                game,
            );
            SoftReset(game);
        }
        return;
    }
    v0 = 30720 as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) != 0 {
        v0 = -(30720 as ::core::ffi::c_int) as int16;
    }
    v1 = (v0 as ::core::ffi::c_int | 0x28 as ::core::ffi::c_int) as int16;
    let mut v3: *mut OamEnt = gOamEnt(
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *(&raw mut (*v3).xcoord as *mut uint16) = v1 as uint16;
    *(&raw mut (*v3).charnum as *mut uint16) = 182 as uint16;
    let ref mut fresh93 = *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16);
    *fresh93 = (*fresh93 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RestorePalettesAndIoAfterDebugGameover(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v0 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v0 + 384 as ::core::ffi::c_int) as usize];
        v0 += 1;
        if !(((v0 * 2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x51)
            .offset((v1 * 2 as ::core::ffi::c_int) as isize) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v1 + 640 as ::core::ffi::c_int) as usize];
        v1 += 1;
        if !(((v1 * 2 as ::core::ffi::c_int - 54 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_5_Continue(game: &mut Game) {
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    static unk_819093: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x9ab200 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x9ab200 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_819093 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) = 0 as uint16;
    RestorePalettesAndIoAfterDebugGameover(game);
}
static kGameOverMenuFuncs: [Option<Func_V>; 8] = unsafe {
    [
        Some(GameOverMenu_0_FadeOutConfigGfx as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_1_Init as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_2_PlayMusic as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_3_FadeIn as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_4_Main as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_5_FadeOutToGameMap as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_6_LoadGameMapView as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_7_FadeOutToSoftReset as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn GameOverMenu(game: &mut Game) {
    kGameOverMenuFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn GameOverMenu_3_FadeIn(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh86 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh86 = (*fresh86).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_5_FadeOutToGameMap(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh83 = (*fresh83).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_7_FadeOutToSoftReset(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleGameOverBabyMetroid(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        SoftReset(game);
    }
}
pub unsafe fn GameOverMenu_6_LoadGameMapView(game: &mut Game) {
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
        kGameState_5_FileSelectMap as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
}
pub unsafe fn GameOverMenu_4_Main(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh84 = *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16);
        *fresh84 = (*fresh84 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .instruction_timer = 180 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 7 as uint16;
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
                as ::core::ffi::c_int
                == kGameState_31_SetUpNewGame as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16);
            } else {
                let ref mut fresh85 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh85 = (*fresh85).wrapping_add(1);
            }
            LoadFromSram(
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
                game,
            );
        }
        return;
    }
    let mut v0: uint16 = 160 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) != 0 {
        v0 = 192 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 40 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
}
pub unsafe fn GameOverMenu_1_Init(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff03 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8 as *mut uint16)
            .offset(v0 as isize) = 15 as uint16;
        v0 += 1;
        if !(((v0 * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v1: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int;
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 + 7 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x156 as uint16,
        addr_kMenuTilemap_GameOver as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x38a as uint16,
        addr_kMenuTilemap_FindMetroidLarva as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x414 as uint16,
        addr_kMenuTilemap_TryAgain as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x4ce as uint16,
        addr_kMenuTilemap_YesReturnToGame as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x5ce as uint16,
        addr_kMenuTilemap_NoGoToTitle as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .current_instruction = 0 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer = 0 as uint16;
    HandleGameOverBabyMetroid(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x6f) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x70) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    EnableHdmaObjects(game);
    static unk_819254: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0x927d as VoidP,
    };
    static unk_81925C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0x928d as VoidP,
    };
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_819254 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_81925C as *const ::core::ffi::c_void,
        game,
    );
    ScreenOn(game);
    let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh88 = (*fresh88).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 40 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = 160 as uint16;
}
pub unsafe fn GameOverMenu_2_PlayMusic(game: &mut Game) {
    if HasQueuedMusic(game) == 0 {
        let ref mut fresh87 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh87 = (*fresh87).wrapping_add(1);
        QueueMusic_Delayed8(4 as uint16, game);
    }
}
static kFileSelectMenuFuncs: [Option<Func_V>; 34] = unsafe {
    [
        Some(FileSelectMenu_0_FadeOutConfigGfx as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_1_LoadFileSelectMenuBG2 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_2_InitMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_3_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_4_Main as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_6_FileCopyInit as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_8 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_9_InitializeSelectDest as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_10_FileCopySelectDest as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_11_InitializeConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_12_FileCopyConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_13_FileCopyDoIt as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_14_CopyCompleted as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_16 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_18 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_20_FileClearInit as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_22_FileClearSelectSlot as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_23_FileClearInitConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_24_FileClearConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_25_FileClearDoClear as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_26_ClearCompleted as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_16 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_30 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_31_TurnSamusHelmet as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_32_FadeOutToOptions as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_33_FadeOutToTitle as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn FileSelectMenu(game: &mut Game) {
    kFileSelectMenuFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn FileSelectMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    ScreenOff(game);
    QueueSfx3_Max6(1 as uint16, game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh82 = (*fresh82).wrapping_add(1);
    MapVramForMenu(game);
    LoadInitialMenuTiles(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
    LoadFileSelectPalettes(game);
}
pub unsafe fn LoadFileSelectPalettes(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut v1: ::core::ffi::c_int = v0 >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            *(game.g_rom.offset(
                (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        v0 += 4 as ::core::ffi::c_int;
        if !(((v0 - 512 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn FileSelectMenu_32_FadeOutToOptions(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_2_GameOptionsMenu as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
        let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(v0 as isize) as *mut uint16) = 0 as uint16;
            v0 += 2 as ::core::ffi::c_int;
            if !(((v0 - 48 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe fn FileSelectMenu_33_FadeOutToTitle(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        SoftReset(game);
    }
}
pub unsafe fn FileSelectMenu_5_FadeOutFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    FileSelectMenu_15_FadeOutToMain(game);
}
pub unsafe fn FileSelectMenu_15_FadeOutToMain(game: &mut Game) {
    HandleFadeOut(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        ScreenOff(game);
        let ref mut fresh48 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh48 = (*fresh48).wrapping_add(1);
    }
}
static kMenuSelectionMissileXY: [uint16; 12] = [
    0x30 as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x58 as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xa3 as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xbb as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xd3 as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
pub unsafe fn FileSelectMenu_17_FadeInToMain(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    FileSelectMenu_7_FadeInFromMain(game);
}
pub unsafe fn FileSelectMenu_7_FadeInFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh46 = (*fresh46).wrapping_add(1);
    }
}
pub unsafe fn FileSelectMenu_6_FileCopyInit(game: &mut Game) {
    let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh76 = (*fresh76).wrapping_add(1);
    FileSelectMenu_Func1(game);
}
pub unsafe fn FileSelectMenu_Func1(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x150 as uint16,
        addr_kMenuTilemap_CopyWhichData as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    SetInitialFileCopyMenuSelection(game);
    SetFileCopyMenuSelectionMissilePosition(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 0 as uint16;
}
pub unsafe fn sub_819591(game: &mut Game) {
    SetInitialFileCopyMenuSelection(game);
}
pub unsafe fn SetInitialFileCopyMenuSelection(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int;
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(v0 & 1 as ::core::ffi::c_int != 0) {
        v0 >>= 1 as ::core::ffi::c_int;
        v1 += 1;
        if !(((v1 - 3 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = v1 as uint16;
}
pub unsafe fn ClearMenuTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn LoadMenuExitTilemap(game: &mut Game) {
    LoadMenuTilemap(
        0x688 as uint16,
        addr_kMenuTilemap_Exit as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DrawFileCopySaveFileInfo(game: &mut Game) {
    LoadFromSram_(0 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn DrawFileCopySaveSlotAInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16,
        0x218 as uint16,
        game,
    );
    DrawFileSelectionTime(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16,
        0x272 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x234 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x208 as uint16,
        addr_kMenuTilemap_SamusA as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DrawFileCopySaveSlotBInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
            as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_int) as uint16,
        0x318 as uint16,
        game,
    );
    DrawFileSelectionTime(
        (!(*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
            as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_int) as uint16,
        0x372 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x334 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x308 as uint16,
        addr_kMenuTilemap_SamusB as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DrawFileCopySaveSlotCInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
            as ::core::ffi::c_int)
            & 4 as ::core::ffi::c_int) as uint16,
        0x418 as uint16,
        game,
    );
    DrawFileSelectionTime(
        (!(*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
            as ::core::ffi::c_int)
            & 4 as ::core::ffi::c_int) as uint16,
        0x472 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x434 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x408 as uint16,
        addr_kMenuTilemap_SamusC as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn QueueTransferOfMenuTilemapToVramBG1(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v1: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
static kBitShl: [uint8; 3] = [
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
];
pub unsafe fn FileSelectMenu_8(game: &mut Game) {
    let mut v0: uint8 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
            as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        {
            let ref mut fresh73 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh73 = (*fresh73 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh74 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh74 = (*fresh74).wrapping_add(1);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh75 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh75 = (*fresh75 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        let mut current_block_18: u64;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v0 = v0.wrapping_sub(1);
                if !(v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 1109700713171191020;
                    break;
                }
                if kBitShl[v0 as usize] as ::core::ffi::c_int
                    & *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
                        as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 1398313740379482408;
                    break;
                }
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v0 = v0.wrapping_add(1);
                if !(((v0 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int8
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 1109700713171191020;
                    break;
                }
                if v0 as ::core::ffi::c_int == 3 as ::core::ffi::c_int
                    || kBitShl[v0 as usize] as ::core::ffi::c_int
                        & *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
                            as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 1398313740379482408;
                    break;
                }
            }
        } else {
            current_block_18 = 1109700713171191020;
        }
        match current_block_18 {
            1109700713171191020 => {}
            _ => {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                    as *mut uint16 as *mut uint8)
                    .offset(0) = v0;
                QueueSfx1_Max6(0x37 as uint16, game);
                SetFileCopyMenuSelectionMissilePosition(game);
                return;
            }
        }
        SetFileCopyMenuSelectionMissilePosition(game);
    };
}
pub unsafe fn SetFileCopyMenuSelectionMissilePosition(game: &mut Game) {
    static kFileCopyMissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        211 as ::core::ffi::c_int as uint16,
    ];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kFileCopyMissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(15) as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 22 as uint16;
}
pub unsafe fn FileSelectMenu_9_InitializeSelectDest(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawFileCopySelectDestinationSaveFileInfo(game);
    let ref mut fresh72 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh72 = (*fresh72).wrapping_add(1);
    let mut v0: uint16 = 0 as uint16;
    while !(v0 as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int)
    {
        v0 = v0.wrapping_add(1);
        if !(v0 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = v0;
    SetFileCopyMenuSelectionMissilePosition(game);
}
pub unsafe fn DrawFileCopySelectDestinationSaveFileInfo(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x148 as uint16,
        addr_kMenuTilemap_CopySamusToWhere as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[944 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    LoadMenuExitTilemap(game);
    LoadFromSram_(0 as uint16, game);
    let mut v0: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 0 {
        v0 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v0;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    let mut v1: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
        as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        v1 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v1;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    let mut v2: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        v2 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v2;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_10_FileCopySelectDest(game: &mut Game) {
    let mut current_block: u64;
    static kFileCopySelectDest_MissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        212 as ::core::ffi::c_int as uint16,
    ];
    let mut v0: uint16 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
            as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh69 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh69 = (*fresh69).wrapping_add(1);
            current_block = 16545170798630791882;
        } else {
            let ref mut fresh70 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh70 = (*fresh70 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
            current_block = 16203760046146113240;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x37 as uint16, game);
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            loop {
                v0 = v0.wrapping_sub(1);
                if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                        .offset(16) as ::core::ffi::c_int
                {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x37 as uint16, game);
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            loop {
                v0 = v0.wrapping_add(1);
                if !(v0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                        .offset(16) as ::core::ffi::c_int
                {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else {
            current_block = 16545170798630791882;
        }
        match current_block {
            16545170798630791882 => {}
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                    v0;
                current_block = 16545170798630791882;
            }
        }
    } else {
        let ref mut fresh71 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh71 = (*fresh71 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16);
        QueueSfx1_Max6(0x37 as uint16, game);
        FileSelectMenu_Func1(game);
        current_block = 16203760046146113240;
    }
    match current_block {
        16203760046146113240 => {
            return;
        }
        _ => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
                kFileCopySelectDest_MissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997)
                    as *mut uint16)
                    .offset(15) as usize];
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
                22 as uint16;
            return;
        }
    };
}
pub unsafe fn FileSelectMenu_11_InitializeConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x144 as uint16,
        addr_kMenuTilemap_CopySamusToSamus as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[942 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[955 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    DrawFileCopyClearConfirmation(game);
    let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh68 = (*fresh68).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 8 as uint16;
}
pub unsafe fn DrawFileCopyClearConfirmation(game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x514 as uint16,
        addr_kMenuTilemap_IsThisOk as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x59c as uint16,
        addr_kMenuTilemap_Yes as ::core::ffi::c_int as uint16,
        game,
    );
    let mut v0: ::core::ffi::c_int = 832 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v0 as usize)) = 15 as uint16;
        v0 += 1;
        if !(((v0 * 2 as ::core::ffi::c_int - 1728 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x65c as uint16,
        addr_kMenuTilemap_No as ::core::ffi::c_int as uint16,
        game,
    );
    DrawFileCopyConfirmationSaveFileInfo(game);
}
pub unsafe fn DrawFileCopyConfirmationSaveFileInfo(game: &mut Game) {
    LoadFromSram_(0 as uint16, game);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
            != 0
    {
        v0 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v0 as uint16;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
        as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        v1 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v1 as uint16;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
    {
        v2 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v2 as uint16;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_12_FileCopyConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh64 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
        *fresh64 = (*fresh64 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh65 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh65 = (*fresh65 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17);
            QueueSfx1_Max6(0x37 as uint16, game);
            return;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x38 as uint16, game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0
            {
                let ref mut fresh66 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh66 = (*fresh66 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
                FileSelectMenu_Func1(game);
            } else {
                let ref mut fresh67 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh67 = (*fresh67).wrapping_add(1);
            }
            return;
        }
    }
    let mut v0: uint16 = 184 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0 {
        v0 = 208 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 94 as uint16;
}
pub unsafe fn HandleFileCopyArrowPalette(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) != 0 {
        let ref mut fresh63 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh63 = (*fresh63).wrapping_sub(1);
        if *fresh63 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 4 as uint16;
            let mut v0: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(145);
            let mut v1: uint16 = 0 as uint16;
            loop {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 145 as ::core::ffi::c_int) as isize,
                ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 146 as ::core::ffi::c_int) as isize,
                );
                v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                if !(((v1 as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
                {
                    break;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(151) = v0;
        }
    }
}
pub unsafe fn FileSelectMenu_13_FileCopyDoIt(game: &mut Game) {
    static kBitShl_16bit: [uint16; 3] = [
        1 as ::core::ffi::c_int as uint16,
        2 as ::core::ffi::c_int as uint16,
        4 as ::core::ffi::c_int as uint16,
    ];
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    let mut src_addr: uint16 = *(game.g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as isize,
        );
    let mut dst_addr: uint16 = *(game.g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
                as isize,
        );
    memcpy(
        game.g_sram.offset(dst_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        game.g_sram.offset(src_addr as isize) as *mut uint8 as *const ::core::ffi::c_void,
        1628 as size_t,
    );
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(16) as ::core::ffi::c_int;
    let mut v10: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v9: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v8: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(17) as ::core::ffi::c_int;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16);
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v4) as isize) as *mut uint8 as *mut uint16) =
        v8 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v10 as uint16;
    let ref mut fresh61 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh61 = (*fresh61).wrapping_add(1);
    let mut v5: ::core::ffi::c_int = 640 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v5 as usize)) = 15 as uint16;
        v5 += 1;
        if !(((v5 * 2 as ::core::ffi::c_int - 1856 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16);
    *fresh62 = (*fresh62 as ::core::ffi::c_int
        | kBitShl_16bit[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(17) as usize] as ::core::ffi::c_int) as uint16;
    let mut v6: ::core::ffi::c_int = ((4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
        + 9 as ::core::ffi::c_int)
        << 6 as ::core::ffi::c_int)
        + 24 as ::core::ffi::c_int;
    let mut v7: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(736) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        v6 += 2 as ::core::ffi::c_int;
        v7 += 2 as ::core::ffi::c_int;
        if !(((v7 - 22 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x510 as uint16,
        addr_kMenuTilemap_CopyCompleted as ::core::ffi::c_int as uint16,
        game,
    );
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_14_CopyCompleted(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh60 = (*fresh60).wrapping_add(1);
        let mut slot: ::core::ffi::c_int =
            *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int;
        if slot & 0x8000 as ::core::ffi::c_int != 0
            || *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
                - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            || slot & *(game.g_sram.offset(0x1fee) as *mut uint16) as ::core::ffi::c_int != 0
        {
            slot = 0 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = slot as uint16;
    }
}
pub unsafe fn FileSelectMenu_18(game: &mut Game) {
    let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh59 = (*fresh59 as ::core::ffi::c_int - 14 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn FileSelectMenu_20_FileClearInit(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh58 = (*fresh58).wrapping_add(1);
    InitFileSelectMenuFileClear(game);
}
pub unsafe fn InitFileSelectMenuFileClear(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x50 as uint16,
        addr_kMenuTilemap_DataClearMode as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearWhichData as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    SetInitialFileCopyMenuSelection(game);
    SetFileClearMenuMissilePos(game);
}
pub unsafe fn FileSelectMenu_22_FileClearSelectSlot(game: &mut Game) {
    static kBitShl_: [uint8; 3] = [
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        4 as ::core::ffi::c_int as uint8,
    ];
    let mut v0: uint16 = 0;
    let mut v1: uint8 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    *(&raw mut v0 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
            as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh56 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh56 = (*fresh56).wrapping_add(1);
            return;
        }
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int)
    {
        let mut current_block_18: u64;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v1 = v1.wrapping_sub(1);
                if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 11307063007268554308;
                    break;
                }
                if kBitShl_[v1 as usize] as ::core::ffi::c_int
                    & *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
                        as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 11325801268588962582;
                    break;
                }
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v1 = v1.wrapping_add(1);
                if !(((v1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int8
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 11307063007268554308;
                    break;
                }
                if v1 as ::core::ffi::c_int == 3 as ::core::ffi::c_int
                    || kBitShl_[v1 as usize] as ::core::ffi::c_int
                        & *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
                            as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 11325801268588962582;
                    break;
                }
            }
        } else {
            current_block_18 = 11307063007268554308;
        }
        match current_block_18 {
            11307063007268554308 => {}
            _ => {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                    as *mut uint16 as *mut uint8)
                    .offset(0) = v1;
                *(&raw mut v0 as *mut uint8).offset(0) = 55 as uint8;
                QueueSfx1_Max6(v0, game);
                SetFileClearMenuMissilePos(game);
                return;
            }
        }
        SetFileClearMenuMissilePos(game);
        return;
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh57 = (*fresh57 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn SetFileClearMenuMissilePos(game: &mut Game) {
    static kFileClear_MissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        211 as ::core::ffi::c_int as uint16,
    ];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kFileClear_MissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(15) as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 22 as uint16;
}
pub unsafe fn FileSelectMenu_23_FileClearInitConfirm(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearSamusA as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[949 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 3 as uint16;
    DrawFileCopyClearConfirmation(game);
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
}
pub unsafe fn FileSelectMenu_24_FileClearConfirm(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: uint16 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh52 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
        *fresh52 = (*fresh52 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            current_block = 18062911466027671358;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
        {
            current_block = 504450154722759016;
        } else {
            QueueSfx1_Max6(0x38 as uint16, game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0
            {
                current_block = 18062911466027671358;
            } else {
                let ref mut fresh54 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh54 = (*fresh54).wrapping_add(1);
                return;
            }
        }
        match current_block {
            504450154722759016 => {}
            _ => {
                let ref mut fresh53 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh53 = (*fresh53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16);
                QueueSfx1_Max6(0x37 as uint16, game);
                InitFileSelectMenuFileClear(game);
                return;
            }
        }
    }
    v0 = 184 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0 {
        v0 = 208 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 94 as uint16;
}
pub unsafe fn FileSelectMenu_25_FileClearDoClear(game: &mut Game) {
    static kBitShl_Not: [uint16; 3] = [
        0xfffe as ::core::ffi::c_int as uint16,
        0xfffd as ::core::ffi::c_int as uint16,
        0xfffb as ::core::ffi::c_int as uint16,
    ];
    DrawBorderAroundDataClearMode(game);
    let mut sram_addr: ::core::ffi::c_int = *(game.g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as isize,
        ) as ::core::ffi::c_int;
    memset(
        game.g_sram.offset(sram_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        1628 as size_t,
    );
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(16) as ::core::ffi::c_int;
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v2) as isize) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh50 = (*fresh50).wrapping_add(1);
    NewSaveFile(game);
    LoadFromSram(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16);
    LoadMirrorOfExploredMapTiles(game);
    let mut v3: uint16 = 640 as uint16;
    loop {
        // File-clear rows share the full menu tilemap at $3600.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v3 as usize)) = 15 as uint16;
        v3 = v3.wrapping_add(1);
        if !(((v3 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1856 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16);
    *fresh51 = (*fresh51 as ::core::ffi::c_int
        & kBitShl_Not[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(16) as usize] as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x500 as uint16,
        addr_kMenuTilemap_DataCleared as ::core::ffi::c_int as uint16,
        game,
    );
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_26_ClearCompleted(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        let mut current_block_7: u64;
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh49 = (*fresh49).wrapping_add(1);
        if !(LoadFromSram(0 as uint16, game) == 0) {
            if LoadFromSram(1 as uint16, game) != 0 {
                if LoadFromSram(2 as uint16, game) != 0 {
                    current_block_7 = 17037298857681653719;
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) =
                        2 as uint16;
                    current_block_7 = 1394248824506584008;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = 1 as uint16;
                current_block_7 = 1394248824506584008;
            }
            match current_block_7 {
                17037298857681653719 => {}
                _ => {
                    return;
                }
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = 0 as uint16;
        return;
    }
}
pub unsafe fn FileSelectMenu_30(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh45 = (*fresh45 as ::core::ffi::c_int - 26 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn FileSelectMenu_31_TurnSamusHelmet(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(2)
            as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(4)
            as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) == 0
    {
        let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh44 = (*fresh44).wrapping_add(1);
    }
}
pub unsafe fn DrawFileSelectSamusHelmets(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
}
pub unsafe fn DrawFileSelectSlotSamusHelmet(mut k: uint16, game: &mut Game) {
    let mut v0: uint16 = k;
    let mut v1: int16 = 0;
    v1 = *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) as int16;
    if v1 != 0 {
        let mut v2: uint16 = (v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v2;
        if v2 == 0 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 8 as uint16;
            let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1997)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - 7 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
                v4 = 7 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v3 as isize) = v4;
        }
    }
    let mut v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v6: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v5 as isize)
            as ::core::ffi::c_int) as uint16;
    static kDrawFileSlotHelmet_Spritemaps: [uint16; 9] = [
        0x2c as ::core::ffi::c_int as uint16,
        0x2d as ::core::ffi::c_int as uint16,
        0x2e as ::core::ffi::c_int as uint16,
        0x2f as ::core::ffi::c_int as uint16,
        0x30 as ::core::ffi::c_int as uint16,
        0x31 as ::core::ffi::c_int as uint16,
        0x32 as ::core::ffi::c_int as uint16,
        0x33 as ::core::ffi::c_int as uint16,
        0x33 as ::core::ffi::c_int as uint16,
    ];
    DrawMenuSpritemap(
        kDrawFileSlotHelmet_Spritemaps
            [(v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v5 + 5 as ::core::ffi::c_int) as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v5 + 10 as ::core::ffi::c_int) as isize),
        3584 as uint16,
        game,
    );
}
static kFileSelectMapFuncs: [Option<Func_V>; 23] = unsafe {
    [
        Some(FileSelectMap_0 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_1 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_3_LoadAreaSelectBackgroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_5_ExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_6_AreaSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_7_PrepExpandSquareTransToRoomMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_8_ExpandSquareTransToRoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_9_InitRoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_10_RoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_11 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_11 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_13 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_14 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_15_ClearTileMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_16_LoadPalettes as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_18 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_20_SetupExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_21_MoveExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_22 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn FileSelectMap(game: &mut Game) {
    kFileSelectMapFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn FileSelectMap_22(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_2_GameOptionsMenu as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn FileSelectMenu_1_LoadFileSelectMenuBG2(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    let menu_tilemap = (&raw mut game.g_ram as *mut uint8)
        .add(0x3600)
        .cast::<uint16>();
    let mut i: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *menu_tilemap.add(i as usize) = *(game.g_rom.offset(
            (((0x8edc00 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8edc00 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i as isize);
        i -= 1;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v2 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh81 = (*fresh81).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
}
pub unsafe fn FileSelectMenu_2_InitMain(game: &mut Game) {
    let mut v0: uint16 = 0;
    if *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || {
            v0 = *(game.g_sram.offset(0x1fec) as *mut uint16);
            *(game.g_sram.offset(0x1fee) as *mut uint16) as ::core::ffi::c_int
                & *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        }
    {
        v0 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = v0;
    FileSelectMenu_16(game);
}
pub unsafe fn FileSelectMenu_16(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x56 as uint16,
        addr_kMenuTilemap_SamusData as ::core::ffi::c_int as uint16,
        game,
    );
    LoadMenuTilemap(
        0x146 as uint16,
        addr_kMenuTilemap_SamusA as ::core::ffi::c_int as uint16,
        game,
    );
    let mut c: uint8 = LoadFromSram_(0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x15c as uint16,
        game,
    );
    DrawFileSelectionTime(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x1b4 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x176 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x286 as uint16,
        addr_kMenuTilemap_SamusB as ::core::ffi::c_int as uint16,
        game,
    );
    c = LoadFromSram_(1 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x29c as uint16,
        game,
    );
    DrawFileSelectionTime(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x2f4 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x2b6 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x3c6 as uint16,
        addr_kMenuTilemap_SamusC as ::core::ffi::c_int as uint16,
        game,
    );
    c = LoadFromSram_(2 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x3dc as uint16,
        game,
    );
    DrawFileSelectionTime(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x434 as uint16,
        game,
    );
    LoadMenuTilemap(
        0x3f6 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) =
        (swap16(
            !(*((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) != 0 {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 0 as uint16;
        LoadMenuTilemap(
            0x508 as uint16,
            addr_kMenuTilemap_DataCopy as ::core::ffi::c_int as uint16,
            game,
        );
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 0 as uint16;
        LoadMenuTilemap(
            0x5c8 as uint16,
            addr_kMenuTilemap_DataClear as ::core::ffi::c_int as uint16,
            game,
        );
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x688 as uint16,
        addr_kMenuTilemap_Exit as ::core::ffi::c_int as uint16,
        game,
    );
    QueueTransferOfMenuTilemapToVramBG1(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(1) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(2) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(3) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(4) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(6) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(11) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(7) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(8) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(9) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(12) = 47 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(13) = 87 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(14) = 127 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    ScreenOn(game);
    let ref mut fresh47 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh47 = (*fresh47).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 0 as uint16;
}
pub unsafe fn LoadFromSram_(mut a: uint16, game: &mut Game) -> uint8 {
    return LoadFromSram(a, game);
}
pub unsafe fn FileSelectMenu_3_FadeInToMain(game: &mut Game) {
    DrawFileSelectSamusHelmets(game);
    let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh80 = (*fresh80).wrapping_add(1);
    }
}
pub unsafe fn DrawFileSelectionHealth(mut a: uint16, mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    if a != 0 {
        FileSelectClearRestOfMenuTilemapRow(k, game);
        LoadMenuTilemap(
            (k as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
            addr_kMenuTilemap_NoData as ::core::ffi::c_int as uint16,
            game,
        );
    } else {
        LoadMenuTilemap(
            k,
            addr_kMenuTilemap_Energy as ::core::ffi::c_int as uint16,
            game,
        );
        v2 = (k as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as int16;
        let mut n: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2)
            as *mut uint16) as ::core::ffi::c_int
            / 100 as ::core::ffi::c_int;
        let mut q: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2)
            as *mut uint16) as ::core::ffi::c_int
            % 100 as ::core::ffi::c_int;
        let mut mh: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9c4)
            as *mut uint16) as ::core::ffi::c_int
            / 100 as ::core::ffi::c_int;
        let mut m: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
        let mut v3: uint16 = (v2 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        loop {
            mh -= 1;
            if !(mh >= 0 as ::core::ffi::c_int) {
                break;
            }
            v4 = 153 as int16;
            if n != 0 {
                n -= 1;
                v4 = 152 as int16;
            }
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16)
                .offset(768) as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .palette_index as ::core::ffi::c_int
                    | v4 as ::core::ffi::c_int) as uint16;
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            m -= 1;
            if m == 0 {
                v3 = (v3 as ::core::ffi::c_int - 78 as ::core::ffi::c_int) as uint16;
                m = 8 as ::core::ffi::c_int;
            }
        }
        n = q / 10 as ::core::ffi::c_int;
        q = q % 10 as ::core::ffi::c_int;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(802) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | q + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(801) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | n + 8288 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn DrawFileSelectionTime(mut a: uint16, mut k: uint16, game: &mut Game) {
    if a == 0 {
        let mut r26: uint16 = k;
        let mut div_val: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9e0)
            as *mut uint16) as ::core::ffi::c_int
            / 10 as ::core::ffi::c_int;
        let mut mod_val: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9e0)
            as *mut uint16) as ::core::ffi::c_int
            % 10 as ::core::ffi::c_int;
        let mut v2: uint16 = r26;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(769) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | mod_val + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | div_val + 8288 as ::core::ffi::c_int) as uint16;
        LoadMenuTilemap(
            (r26 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16,
            addr_word_81B4A8 as ::core::ffi::c_int as uint16,
            game,
        );
        let mut div_min: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9de)
            as *mut uint16) as ::core::ffi::c_int
            / 10 as ::core::ffi::c_int;
        let mut mod_min: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9de)
            as *mut uint16) as ::core::ffi::c_int
            % 10 as ::core::ffi::c_int;
        let mut v3: uint16 = r26;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(772) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | mod_min + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(771) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | div_min + 8288 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn FileSelectMenu_4_Main(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: int16 = 0;
    let mut v1: uint16 = 0;
    let mut v2: uint16 = 0;
    let mut v3: ::core::ffi::c_int = 0;
    DrawBorderAroundSamusData(game);
    DrawMenuSelectionMissile(game);
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) != 0 {
                v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint16;
                if ((*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v1 = 5 as uint16;
                }
            } else {
                v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint16;
                if (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                        as ::core::ffi::c_int
                        - 5 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v1 = 2 as uint16;
                    }
                } else {
                    v1 = 5 as uint16;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = v1;
            current_block = 14136749492126903395;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
                as ::core::ffi::c_int
                & kButton_B as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x37 as uint16, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 33 as uint16;
                QueueSfx1_Max6(0x37 as uint16, game);
            }
            current_block = 8908889263641380247;
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16) != 0 {
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 5 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v2 = 0 as uint16;
                }
            } else {
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                        as ::core::ffi::c_int
                        - 5 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        v2 = 5 as uint16;
                    } else {
                        v2 = 0 as uint16;
                    }
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) = v2;
            current_block = 14136749492126903395;
        }
        match current_block {
            8908889263641380247 => {}
            _ => {
                QueueSfx1_Max6(0x37 as uint16, game);
            }
        }
    } else {
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) as int16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            QueueSfx1_Max6(0x2a as uint16, game);
            let ref mut fresh77 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh77 = (*fresh77 as ::core::ffi::c_int + 27 as ::core::ffi::c_int) as uint16;
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(
                    (2 as ::core::ffi::c_int
                        * (*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                            as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int)) as uint16
                        as ::core::ffi::c_int as isize,
                ) as *mut uint16) = 1 as uint16;
            *(game.g_sram.offset(0x1fec) as *mut uint8 as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16);
            *(game.g_sram.offset(0x1fee) as *mut uint8 as *mut uint16) =
                !(*((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            RtlWriteSram(game);
            if LoadFromSram(
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
                game,
            ) != 0
            {
                NewSaveFile(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x789) as *mut uint16) = 0 as uint16;
            } else {
                LoadMirrorOfExploredMapTiles(game);
            }
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x37 as uint16, game);
                let ref mut fresh78 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh78 = (*fresh78).wrapping_add(1);
                *((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xff0c as ::core::ffi::c_int
                        | 3 as ::core::ffi::c_int) as uint16;
            } else {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16)
                    as ::core::ffi::c_int
                    == 4 as ::core::ffi::c_int
                {
                    QueueSfx1_Max6(0x37 as uint16, game);
                    let ref mut fresh79 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                    *fresh79 =
                        (*fresh79 as ::core::ffi::c_int + 15 as ::core::ffi::c_int) as uint16;
                    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xff0c as ::core::ffi::c_int
                        | 3 as ::core::ffi::c_int) as int16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16) =
                        v0 as uint16;
                }
                if v0 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) =
                        33 as uint16;
                }
            }
            return;
        }
    }
    v3 = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v3 + 1 as ::core::ffi::c_int) as usize];
}
pub unsafe fn FileSelectMap_0(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    ClearMenuTilemap(game);
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
    LoadFileSelectPalettes(game);
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(14) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(30) = 0 as uint16;
    let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh43 = (*fresh43).wrapping_add(1);
}
pub unsafe fn FileSelectMap_1(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    if AdvanceGradualColorChangeOfPalette(0 as uint16, 0x40 as uint16, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
            as ::core::ffi::c_int
            != kFileSelectMap_AreaIndexes
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !((v0 as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            v0 = 0 as uint16;
            break;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) =
            (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        v1 = 0 as int16;
        loop {
            v2 = v1;
            if v1 as ::core::ffi::c_int
                == *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int
            {
                LoadActiveAreaMapForegroundColors(v1 as uint16, game);
            } else {
                LoadInactiveAreaMapForegroundColors(v1 as uint16, game);
            }
            v1 += 1;
            if !(((v2 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh42 = (*fresh42).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x787) as *mut uint16) = 0 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
    }
}
pub unsafe fn sub_81A3D1(mut k: uint16, game: &mut Game) {
    LoadActiveAreaMapForegroundColors(k, game);
}
static kActiveAreaMapForegroundColors: [uint16; 6] = [
    0 as ::core::ffi::c_int as uint16,
    0xa as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    0x16 as ::core::ffi::c_int as uint16,
    0x24 as ::core::ffi::c_int as uint16,
    0x2a as ::core::ffi::c_int as uint16,
];
static kInactiveAreaMapForegroundColors: [uint16; 6] = [
    0x30 as ::core::ffi::c_int as uint16,
    0x3a as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    0x54 as ::core::ffi::c_int as uint16,
    0x5a as ::core::ffi::c_int as uint16,
];
pub unsafe fn LoadActiveAreaMapForegroundColors(mut v0: uint16, game: &mut Game) {
    LoadAreaMapForegroundColors(kActiveAreaMapForegroundColors[v0 as usize], game);
}
pub unsafe fn LoadInactiveAreaMapForegroundColors(mut v0: uint16, game: &mut Game) {
    LoadAreaMapForegroundColors(kInactiveAreaMapForegroundColors[v0 as usize], game);
}
pub unsafe fn LoadAreaMapForegroundColors(mut j: uint16, game: &mut Game) {
    loop {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(
            (((0x81a4e6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81a4e6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            == 0xffff as ::core::ffi::c_int
        {
            break;
        }
        let mut v4: uint16 = j;
        let mut v2: uint16 = *(game.g_rom.offset(
            (((0x81a4e6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81a4e6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        let mut v3: uint16 = *(game.g_rom.offset(
            (((0x81a4e6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81a4e6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        let mut n: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0x81a40e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x81a40e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        j = (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn FileSelectMap_2_LoadAreaSelectForegroundTilemap(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr = addr_kAreaSelectForegroundTilemap as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 129 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x51) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x52) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | 0xf as ::core::ffi::c_int) as uint16;
    let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh18 = (*fresh18).wrapping_add(1);
}
pub unsafe fn FileSelectMap_18(game: &mut Game) {
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh17 = (*fresh17).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16),
        game,
    );
}
pub unsafe fn FileSelectMap_3_LoadAreaSelectBackgroundTilemap(game: &mut Game) {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh41 = (*fresh41).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16),
        game,
    );
}
pub unsafe fn LoadAreaSelectBackgroundTilemap(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr = (8 as ::core::ffi::c_int * 256 as ::core::ffi::c_int * j as ::core::ffi::c_int
        + addr_kAreaSelectBackgroundTilemaps as ::core::ffi::c_int) as VoidP;
    (*v2).src.bank = 0x81 as uint8;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x5a) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn FileSelectMap_4_SetupExpandingSquareTransition(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    WriteReg(TS as ::core::ffi::c_int as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    WriteReg(TSW as ::core::ffi::c_int as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 37 as uint8;
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0x25 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    SetupInitialExpandingSquareHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8, game);
    ConfigureWindow1ForExpandingSquare(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16 as *mut uint8).offset(0) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn ConfigureWindow1ForExpandingSquare(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    WriteReg(TM as ::core::ffi::c_int as uint16, 0x13 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    WriteReg(TMW as ::core::ffi::c_int as uint16, 0x13 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 35 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x23 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 3 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 35 as uint8;
    WriteReg(WOBJSEL as ::core::ffi::c_int as uint16, 0x23 as uint8, game);
}
pub unsafe fn SetupInitialExpandingSquareHDMA(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e22) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16 as *mut uint8).offset(0) =
        127 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16 as *mut uint8).offset(0) =
        -(127 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1 = 34 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_1 = 50 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_1 = 50 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP2 as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(BBAD2 as ::core::ffi::c_int as uint16, 0x26 as uint8, game);
    WriteReg(A1T2L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A1T2H as ::core::ffi::c_int as uint16, 0x9e as uint8, game);
    WriteReg(A1B2 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS20 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS2L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(DAS2H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A2L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A2H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(NTRL2 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_1 = 54 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_1 = 54 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP3 as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(BBAD3 as ::core::ffi::c_int as uint16, 0x27 as uint8, game);
    WriteReg(A1T3L as ::core::ffi::c_int as uint16, 0x10 as uint8, game);
    WriteReg(A1T3H as ::core::ffi::c_int as uint16, 0x9e as uint8, game);
    WriteReg(A1B3 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS30 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS3L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(DAS3H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A3L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A3H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(NTRL3 as ::core::ffi::c_int as uint16, 0 as uint8, game);
}
static kExpandingSquareTransitionSpeed: uint8 = 4 as uint8;
pub unsafe fn FileSelectMap_5_ExpandingSquare(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    DrawAreaSelectMapLabels(game);
    let mut v0: uint8 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as ::core::ffi::c_int
        - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
    if (((*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as ::core::ffi::c_int
        - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as int8
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16 as *mut uint8)
            .offset(0) = (*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16)
            as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint8;
        let ref mut fresh36 = *(&raw mut game.g_ram as *mut uint8).offset(0x69);
        *fresh36 = (*fresh36 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 24 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16) = 1 as uint16;
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [v1 as usize] = *(game.g_rom.offset(
                (((0xb6e000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xb6e000 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
            v1 = v1.wrapping_add(1);
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1600 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        loop {
            let fresh37 = v1;
            v1 = v1.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [fresh37 as usize] = 10241 as uint16;
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .bg2_room_select_map_tilemap as *mut uint16)
                .offset(170),
            game,
        );
        let mut v2: uint16 = 320 as uint16;
        let mut v3: uint16 = 959 as uint16;
        loop {
            let fresh38 = v3;
            v3 = v3.wrapping_sub(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [fresh38 as usize] = *(game.g_rom.offset(
                (((0x81b14b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x81b14b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            if !(v2 != 0) {
                break;
            }
        }
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v5 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v5).size = 2048 as uint16;
        (*v5).src.addr = ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as VoidP;
        (*v5).src.bank = 126 as uint8;
        (*v5).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v4 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh39 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0;
        *fresh39 = (*fresh39 as ::core::ffi::c_int
            - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3))
        .field_0 = v0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 = v0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3))
        .field_0 = v0;
        let ref mut fresh40 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        *fresh40 = (*fresh40 as ::core::ffi::c_int
            + kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16 as *mut uint8)
            .offset(0) = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16)
            as ::core::ffi::c_int
            - kExpandingSquareTransitionSpeed as ::core::ffi::c_int)
            as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16 as *mut uint8)
            .offset(0) = (kExpandingSquareTransitionSpeed as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16)
                as ::core::ffi::c_int) as uint8;
    };
}
pub unsafe fn FileSelectMap_6_AreaSelectMap(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: int16 = 0;
    v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Left as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) as int16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) != 0 {
            let mut R28: uint16 = DecAndWraparoundTo5(
                *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16),
            );
            if CheckIfFileSelectMapAreaCanBeSelected(R28, game) as ::core::ffi::c_int != 0
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28, game) as ::core::ffi::c_int != 0
                }
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28, game) as ::core::ffi::c_int != 0
                }
            {
                QueueSfx1_Max6(0x37 as uint16, game);
                SwitchActiveFileSelectMapArea(R28, game);
                DrawAreaSelectMapLabels(game);
                return;
            }
            current_block = 4115184416633590978;
        } else {
            current_block = 9480232856434185415;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Select as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
    {
        current_block = 9480232856434185415;
    } else {
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) as int16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) == 0 {
            current_block = 9480232856434185415;
        } else {
            let mut n: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
            let mut R28_0: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16);
            loop {
                R28_0 = WraparoundFrom6to0(R28_0);
                if CheckIfFileSelectMapAreaCanBeSelected(R28_0, game) != 0 {
                    current_block = 11194104282611034094;
                    break;
                }
                n -= 1;
                if n == 0 {
                    current_block = 4115184416633590978;
                    break;
                }
            }
            match current_block {
                4115184416633590978 => {}
                _ => {
                    SwitchActiveFileSelectMapArea(R28_0, game);
                    QueueSfx1_Max6(0x37 as uint16, game);
                    DrawAreaSelectMapLabels(game);
                    return;
                }
            }
        }
    }
    match current_block {
        9480232856434185415 => {
            if (v0 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 22 as uint16;
                DrawAreaSelectMapLabels(game);
                return;
            }
            if v0 as ::core::ffi::c_int
                & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x38 as uint16, game);
                SelectFileSelectMapArea(game);
                return;
            }
        }
        _ => {}
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn DecAndWraparoundTo5(mut a: uint16) -> uint16 {
    let mut result: uint16 = (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    if result as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        return 5 as uint16;
    }
    return result;
}
pub unsafe fn WraparoundFrom6to0(mut a: uint16) -> uint16 {
    let mut result: uint16 = (a as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if result as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        return 0 as uint16;
    }
    return result;
}
pub unsafe fn SelectFileSelectMapArea(game: &mut Game) {
    let mut v2: int8 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let ref mut fresh35 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh35 = (*fresh35).wrapping_add(1);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd918) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd916) as *mut uint16);
        DrawAreaSelectMapLabels(game);
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
        kFileSelectMap_AreaIndexes
            [*((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) as usize];
    let mut q: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
        (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int) as isize,
    ) as *mut uint8 as *mut uint16);
    let mut v0: uint16 = 0 as uint16;
    let mut r0: *const uint16 = RomPtr_82(
        *(RomPtr_82(
            (addr_kMapIconDataPointers as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16),
        game,
    ) as *const uint16;
    let mut r20: uint16 = 16 as uint16;
    's_43: loop {
        v2 = (q as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
        q = (q as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        if !(v2 == 0) {
            v3 = *r0.offset((2 as ::core::ffi::c_int * v0 as ::core::ffi::c_int) as isize) as int16;
            if !(v3 as ::core::ffi::c_int == -(2 as ::core::ffi::c_int)) {
                if v3 as ::core::ffi::c_int != -(1 as ::core::ffi::c_int) {
                    break;
                }
                v0 = -(1 as ::core::ffi::c_int) as uint16;
            }
        }
        v0 = v0.wrapping_add(1);
        r20 = r20.wrapping_sub(1);
        if !(r20 == 0) {
            continue;
        }
        loop {
            v4 = *r0.offset((2 as ::core::ffi::c_int * v0 as ::core::ffi::c_int) as isize) as int16;
            if v4 as ::core::ffi::c_int != -(2 as ::core::ffi::c_int) {
                if v4 as ::core::ffi::c_int != -(1 as ::core::ffi::c_int) {
                    break 's_43;
                }
                v0 = -(1 as ::core::ffi::c_int) as uint16;
            }
            v0 = v0.wrapping_add(1);
            r20 = r20.wrapping_sub(1);
            if r20 == 0 {
                Unreachable(game);
            }
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = v0;
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn CheckIfFileSelectMapAreaCanBeSelected(mut a: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int =
        2 as ::core::ffi::c_int * kFileSelectMap_AreaIndexes[a as usize] as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8)
        .offset(v1 as isize) as *mut uint8 as *mut uint16)
        != 0
    {
        return true_0 as uint16;
    }
    let mut t: uint16 = (*((&raw mut (*(game.g_rom.offset(
        (((0x82c7cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x82c7cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MapIconDataPointers)
        .offset(4))
    .crateria as *mut uint8)
        .offset(v1 as isize) as *mut uint16) as ::core::ffi::c_int
        + 64 as ::core::ffi::c_int) as uint16;
    return (t as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int) as ::core::ffi::c_int
        as uint16;
}
pub unsafe fn SwitchActiveFileSelectMapArea(mut R28: uint16, game: &mut Game) {
    LoadInactiveAreaMapForegroundColors(
        kFileSelectMap_AreaIndexes
            [*((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) as usize],
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) = R28;
    LoadActiveAreaMapForegroundColors(kFileSelectMap_AreaIndexes[R28 as usize], game);
    LoadAreaSelectBackgroundTilemap(
        kFileSelectMap_AreaIndexes
            [*((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) as usize],
        game,
    );
}
static kAreaSelectMapLabelPositions: [uint16; 12] = [
    0x5b as ::core::ffi::c_int as uint16,
    0x32 as ::core::ffi::c_int as uint16,
    0x2a as ::core::ffi::c_int as uint16,
    0x7f as ::core::ffi::c_int as uint16,
    0x5e as ::core::ffi::c_int as uint16,
    0xb5 as ::core::ffi::c_int as uint16,
    0xce as ::core::ffi::c_int as uint16,
    0x50 as ::core::ffi::c_int as uint16,
    0xce as ::core::ffi::c_int as uint16,
    0x9f as ::core::ffi::c_int as uint16,
    0x87 as ::core::ffi::c_int as uint16,
    0x8b as ::core::ffi::c_int as uint16,
];
pub unsafe fn DrawAreaSelectMapLabels(game: &mut Game) {
    let mut r3: uint16 = 0 as uint16;
    DrawMenuSpritemap(
        *(game.g_rom.offset(
            (((0x82c749 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x82c749 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0),
        0x80 as uint16,
        0x10 as uint16,
        r3,
        game,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        r3 = (if i
            == *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16)
                as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            512 as ::core::ffi::c_int
        }) as uint16;
        let mut v1: uint16 = (2 as ::core::ffi::c_int
            * kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int)
            as uint16;
        let mut r36: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset(v1 as isize) as *mut uint8 as *mut uint16);
        let mut v2: *const uint16 = RomPtr_82(
            *((&raw mut (*(game.g_rom.offset(
                (((0x82c7cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x82c7cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut MapIconDataPointers)
                .offset(4))
            .crateria as *mut uint8)
                .offset(v1 as ::core::ffi::c_int as isize) as *mut VoidP),
            game,
        ) as *const uint16;
        let mut R30: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        while *v2 as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
            let mut v4: ::core::ffi::c_int = r36 as ::core::ffi::c_int & 1 as ::core::ffi::c_int;
            r36 = (r36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            if !(v4 != 0 && *v2 as ::core::ffi::c_int != 0xfffe as ::core::ffi::c_int) {
                v2 = v2.offset(2);
                R30 -= 1;
                if !(R30 == 0) {
                    continue;
                }
                if !(*((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16)
                    as ::core::ffi::c_int
                    != 0
                    && *v2 as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int)
                {
                    break;
                }
            }
            let mut j: ::core::ffi::c_int = 4 as ::core::ffi::c_int
                * kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            DrawMenuSpritemap(
                (*(game.g_rom.offset(
                    (((0x82c749 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x82c749 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(0) as ::core::ffi::c_int
                    + kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16,
                kAreaSelectMapLabelPositions[j as usize],
                kAreaSelectMapLabelPositions[(j + 1 as ::core::ffi::c_int) as usize],
                r3,
                game,
            );
            break;
        }
        i += 1;
    }
}
static kRoomSelectMapExpandingSquareTimers: [uint16; 6] = [
    0x33 as ::core::ffi::c_int as uint16,
    0x35 as ::core::ffi::c_int as uint16,
    0x2d as ::core::ffi::c_int as uint16,
    0x33 as ::core::ffi::c_int as uint16,
    0x33 as ::core::ffi::c_int as uint16,
    0x22 as ::core::ffi::c_int as uint16,
];
pub unsafe fn FileSelectMap_7_PrepExpandSquareTransToRoomMap(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    SetupInitialExpandingSquareHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 50 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x32 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 5 as uint8;
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 5 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 34 as uint8;
    WriteReg(WOBJSEL as ::core::ffi::c_int as uint16, 0x22 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 48 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    DrawRoomSelectMapAreaLabel(
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .bg2_room_select_map_tilemap as *mut uint16)
            .offset(170),
        game,
    );
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v1: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 512 as uint16;
    (*v1).src.addr =
        ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    (*v1).src.bank = 126 as uint8;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16) =
        kRoomSelectMapExpandingSquareTimers
            [*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as usize];
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16) =
        kAreaSelectMapLabelPositions[v2 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16) =
        kAreaSelectMapLabelPositions[(v2 + 1 as ::core::ffi::c_int) as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16) = 0 as uint16;
    let mut vels: *const ExpandingSquareVels = (game.g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut ExpandingSquareVels;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e40) as *mut uint16) = (*vels).left_subvel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e42) as *mut uint16) = (*vels).left_vel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e44) as *mut uint16) = (*vels).right_subvel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e46) as *mut uint16) = (*vels).right_vel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e48) as *mut uint16) = (*vels).top_subvel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4a) as *mut uint16) = (*vels).top_vel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4c) as *mut uint16) = (*vels).bottom_subvel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4e) as *mut uint16) = (*vels).bottom_vel;
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8, game);
    let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh34 = (*fresh34).wrapping_add(1);
    QueueSfx1_Max6(0x3b as uint16, game);
}
pub unsafe fn SetupRoomSelectMapExpandingSquareTransHDMA(game: &mut Game) {
    let mut k: uint16 = 0 as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16);
    AddExpandingSqTransLeftIndirHDMA(v0, k, 0x9e22 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v0, k, 0x9e20 as uint16, game);
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16)
            as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) as uint8
        as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16) as uint8
            as ::core::ffi::c_int
    {
        v2 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v2, k, 0x9e32 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v2, k, 0x9e36 as uint16, game);
    let mut v4: uint16 = (-(32 as ::core::ffi::c_int)
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16)
            as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) as uint8
        as ::core::ffi::c_int
        == 0xe0 as ::core::ffi::c_int
    {
        v4 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v4, k, 0x9e22 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v4, k, 0x9e20 as uint16, game);
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn AddExpandingSqTransLeftIndirHDMA(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) {
    if a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) =
            (a as ::core::ffi::c_int - 127 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    } else {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    };
}
pub unsafe fn AddExpandingSqTransRightIndirHDMA(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) =
            (a as ::core::ffi::c_int - 127 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        return (k as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    } else {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        return (k as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap(game: &mut Game) {
    if HandleRoomSelectMapExpandingSquareTrans(game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh33 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh33 = (*fresh33).wrapping_add(1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn HandleRoomSelectMapExpandingSquareTrans(game: &mut Game) -> uint16 {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e42) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e40) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16) = 1 as uint16;
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e46) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e44) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16) as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16) = 255 as uint16;
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e4a) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e48) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16) = 1 as uint16;
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e4e) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e4c) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) as ::core::ffi::c_int
        - 224 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) = 224 as uint16;
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_sub(1);
    return *fresh0;
}
pub unsafe fn FileSelectMap_9_InitRoomSelectMap(game: &mut Game) {
    LoadMirrorOfExploredMapTiles(game);
    DrawRoomSelectMap(game);
    LoadFromLoadStation(game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let mut RoomDefHeader: *mut RoomDefHeader = get_RoomDefHeader(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).area_index_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a1) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).x_coordinate_on_map;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a3) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).y_coordinate_on_map;
    SetupMapScrollingForFileSelectMap(game);
    let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16);
    *fresh30 = (*fresh30 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 24 as uint16;
    let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8).offset(0x69) as *mut uint16);
    *fresh31 = (*fresh31 as ::core::ffi::c_int & !(4 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh32 = (*fresh32).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x776) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x778) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x77a) as *mut uint16) = 0 as uint16;
}
pub unsafe fn FileSelectMap_10_RoomSelectMap(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int8 = 0;
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    DrawFileSelectMapIcons(game);
    HandleFileSelectMapScrollArrows(game);
    MapScrolling(game);
    DisplayMapElevatorDestinations(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16) as ::core::ffi::c_int
            & kButton_Select as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let mut current_block_32: u64;
        QueueSfx1_Max6(0x38 as uint16, game);
        let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as isize,
        ) as *mut uint8 as *mut uint16);
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) as int16;
        loop {
            r24 = (r24 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            v1 -= 1;
            if !(v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
                break;
            }
        }
        let mut r0: *const uint16 = RomPtr_82(
            *(RomPtr_82(
                (addr_kMapIconDataPointers as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                            as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16),
            game,
        ) as *const uint16;
        v3 = (4 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        let mut r18: uint16 =
            *r0.offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut r20: uint16 = *r0.offset(
            ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block_32 = 4150833010744877889;
        } else {
            current_block_32 = 12800627514080957624;
        }
        loop {
            match current_block_32 {
                12800627514080957624 => {
                    let ref mut fresh26 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16);
                    *fresh26 = (*fresh26).wrapping_add(1);
                    if *fresh26 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        current_block_32 = 8236137900636309791;
                    } else {
                        current_block_32 = 5601891728916014340;
                    }
                }
                _ => {
                    let ref mut fresh27 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16);
                    *fresh27 = (*fresh27).wrapping_add(1);
                    current_block_32 = 8236137900636309791;
                }
            }
            match current_block_32 {
                8236137900636309791 => {
                    v5 = *r0.offset(
                        (2 as ::core::ffi::c_int
                            * *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                                as ::core::ffi::c_int) as isize,
                    ) as int16;
                    if v5 as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
                            0 as uint16;
                        r24 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
                            (2 as ::core::ffi::c_int
                                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f)
                                    as *mut uint16)
                                    as ::core::ffi::c_int) as isize,
                        ) as *mut uint8 as *mut uint16);
                    } else if v5 as ::core::ffi::c_int != -(2 as ::core::ffi::c_int) {
                        break;
                    } else {
                        current_block_32 = 4150833010744877889;
                        continue;
                    }
                }
                _ => {}
            }
            v4 = (r24 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
            r24 = (r24 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            if v4 == 0
                || *r0.offset(
                    (2 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                            as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                    >= 0xfffe as ::core::ffi::c_int
            {
                current_block_32 = 12800627514080957624;
            } else {
                break;
            }
        }
        let mut v6: uint16 = (4 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        let mut w7: uint16 = *r0.offset(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                    as ::core::ffi::c_int) as isize,
        );
        if w7 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || w7 as ::core::ffi::c_int
                - 256 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)
                    as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)
                as ::core::ffi::c_int
                + *r0.offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as int16;
            if v8 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if (v8 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    v8 = *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16)
                        as int16;
                }
            } else {
                v8 = 0 as int16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = v8 as uint16;
        }
        let mut v9: uint16 = (v6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let mut v10: uint16 =
            *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        if v10 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || v10 as ::core::ffi::c_int
                - 161 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)
                    as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            let mut v11: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb3)
                as *mut uint16) as ::core::ffi::c_int
                + *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int) as uint16;
            if (v11 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v11 = *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16);
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = v11;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh28 = (*fresh28 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
            as ::core::ffi::c_int
            != kFileSelectMap_AreaIndexes
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if (v0 as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) = 0 as uint16;
                return;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16) =
            (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x3c as uint16, game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
        QueueSfx1_Max6(0x38 as uint16, game);
    }
}
pub unsafe fn sub_81AEC8(game: &mut Game) {
    HandleFileSelectMapScrollArrows(game);
}
pub unsafe fn HandleFileSelectMapScrollArrows(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16) as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kLeftMapScrollArrowData as ::core::ffi::c_int as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5ae) as *mut uint16) as ::core::ffi::c_int
        - 232 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kRightMapScrollArrowData as ::core::ffi::c_int as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kUpMapScrollArrowData as ::core::ffi::c_int as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b2) as *mut uint16) as ::core::ffi::c_int
        - 145 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) as ::core::ffi::c_int
            == (*(game.g_rom.offset(
                (((0x81af50 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x81af50 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut MapScrollArrowData))
                .map_scroll_dir as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16) = 0 as uint16;
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kDownMapScrollArrowData as ::core::ffi::c_int as uint16,
            game,
        );
    };
}
pub unsafe fn FileSelectMap_11(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh25 = (*fresh25).wrapping_add(1);
}
pub unsafe fn FileSelectMap_13(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh24 = (*fresh24).wrapping_add(1);
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos =
            32 as uint16;
    }
}
pub unsafe fn FileSelectMap_14(game: &mut Game) {
    let fresh22 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(0))
    .x_pos;
    fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
    if fresh22.read_unaligned() == 0 {
        ScreenOff(game);
        let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh23 = (*fresh23).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn FileSelectMap_15_ClearTileMap(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 18 as uint8;
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh21 = (*fresh21).wrapping_add(1);
}
pub unsafe fn FileSelectMap_16_LoadPalettes(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    LoadMenuPalettes(game);
    v0 = 0 as int16;
    loop {
        v1 = v0;
        let fresh19 = v0;
        v0 = v0 + 1;
        LoadInactiveAreaMapForegroundColors(fresh19 as uint16, game);
        if !(((v1 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    LoadActiveAreaMapForegroundColors(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16),
        game,
    );
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16),
        game,
    );
    let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh20 = (*fresh20).wrapping_add(1);
}
pub unsafe fn FileSelectMap_20_SetupExpandingSquare(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    QueueSfx1_Max6(0x3c as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16) =
        (kRoomSelectMapExpandingSquareTimers
            [*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as usize]
            as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e40) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .left_subvel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e42) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .left_vel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e44) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .right_subvel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e46) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .right_vel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e48) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .top_subvel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4a) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .top_vel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4c) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .bottom_subvel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4e) as *mut uint16) =
        *((&raw mut (*(game.g_rom.offset(
            (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExpandingSquareVels)
            .offset(0))
        .bottom_vel as *mut uint8)
            .offset(
                (16 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16) = 248 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16) = 216 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16) = 0 as uint16;
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh16 = (*fresh16).wrapping_add(1);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 50 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x32 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 34 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
}
pub unsafe fn FileSelectMap_21_MoveExpandingSquare(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16,
        -(((*((&raw mut game.g_ram as *mut uint8).offset(0x9e42) as *mut uint16)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e40) as *mut uint16)
                as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16,
        -(((*((&raw mut game.g_ram as *mut uint8).offset(0x9e46) as *mut uint16)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e44) as *mut uint16)
                as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16,
        -(((*((&raw mut game.g_ram as *mut uint8).offset(0x9e4a) as *mut uint16)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e48) as *mut uint16)
                as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16,
        -(((*((&raw mut game.g_ram as *mut uint8).offset(0x9e4e) as *mut uint16)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x9e4c) as *mut uint16)
                as ::core::ffi::c_int) as int32) as uint32,
    );
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    DrawAreaSelectMapLabels(game);
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16);
    *fresh14 = (*fresh14).wrapping_sub(1);
    if *fresh14 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    }
}
pub unsafe fn NewSaveFile(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16) = 99 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) = 99 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9cc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9aa) as *mut uint16) =
        kButton_Up as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ac) as *mut uint16) =
        kButton_Down as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ae) as *mut uint16) =
        kButton_Left as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b0) as *mut uint16) =
        kButton_Right as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16) =
        kButton_A as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) =
        kButton_B as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) =
        kButton_X as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b8) as *mut uint16) =
        kButton_Y as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ba) as *mut uint16) =
        kButton_Select as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9be) as *mut uint16) =
        kButton_R as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9bc) as *mut uint16) =
        kButton_L as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9da) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9dc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9de) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ea) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e6) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e8) as *mut uint16) = 1 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8f0) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd908)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset((v0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 0 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    loop {
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v3: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v3 as ::core::ffi::c_int - 1792 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn FileSelectClearRestOfMenuTilemapRow(mut v0: uint16, game: &mut Game) {
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int
        - ((v0 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int);
    loop {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
pub unsafe fn LoadMenuTilemap(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut k_bak: ::core::ffi::c_int = k as ::core::ffi::c_int;
    loop {
        loop {
            v2 = *(RomPtr_81(j as uint16_t, game) as *mut uint16) as int16;
            if v2 as ::core::ffi::c_int != -(2 as ::core::ffi::c_int) {
                break;
            }
            j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            k_bak += 64 as ::core::ffi::c_int;
            k = k_bak as uint16;
        }
        if v2 as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
            break;
        }
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | v2 as ::core::ffi::c_int) as uint16;
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
