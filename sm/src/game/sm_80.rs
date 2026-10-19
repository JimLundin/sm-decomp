use crate::game_state::Game;

use crate::game::sm_81::{DrawSpritemap, LoadFromSram, SoftReset};
use crate::game::sm_82::{
    LoadCRETilesTilesetTilesAndPalette, LoadColorsForSpritesBeamsAndEnemies,
    LoadDestinationRoomThings, LoadLevelDataAndOtherThings, LoadLibraryBackground, LoadRoomMusic,
    NullFunc, ResetSoundQueues, UpdateMusicTrackIndex,
};
use crate::game::sm_84::{ClearPLMs, LoadRoomPlmGfx, SpawnHardcodedPlm};
use crate::game::sm_86::ClearEprojs;
use crate::game::sm_87::ClearAnimtiles;
use crate::game::sm_88::{InitializeSpecialEffectsForNewRoom, WaitUntilEndOfVblankAndClearHdma};
use crate::game::sm_89::LoadFXHeader;
use crate::game::sm_8b::InitializeIoDisplayLogo_Async;
use crate::game::sm_8d::ClearPaletteFXObjects;
use crate::game::sm_90::{InitializeMiniMapBroken, UpdateBeamTilesAndPalette};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::LoadEnemies;
use crate::rtl::{
    Load24, Mult8x8, ReadRegWord, RomPtr, RtlApuUpload, RtlApuWrite, RtlWriteSram, SnesDivide,
    SnesModulus, Unreachable, WriteReg, WriteRegWord,
};
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t,
    CoroutineRet, Func_U8, Func_V, LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry,
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
pub struct LoadStationList {
    pub room_ptr_: VoidP,
    pub door_ptr: VoidP,
    pub door_bts: uint16,
    pub screen_x_pos: uint16,
    pub screen_y_pos: uint16,
    pub samus_y_offset: uint16,
    pub samus_x_offset: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusTileAnimationTileDefs {
    pub src: LongPtr,
    pub part1_size: uint16,
    pub part2_size: uint16,
}
pub use crate::types::{GameState, Mode7VramWriteQueue, RoomDefHeader};
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
    Ram4000, Ram4000_Backups, StartDmaCopy, TileTable, TileTables, VramReadQueueEnt,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7CgvmWriteQueue {
    pub tag: uint8,
    pub src_addr: LongPtr,
    pub count: uint16,
    pub vram_addr: uint16,
    pub vmain: uint8,
}
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
pub use crate::types::Consts_80;
pub const addr_word_80A060: Consts_80 = 41056;
pub const addrl_kDigitTilesetsWeapon: Consts_80 = 8429011;
pub const addrl_kDigitTilesetsHealth: Consts_80 = 8428991;
pub const addr_kHudTilemaps_Missiles: Consts_80 = 39331;
pub const addr_kHudTilemaps_AutoReserve: Consts_80 = 39307;
pub use crate::types::SpawnHardcodedPlmArgs;
#[inline]
unsafe fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe fn RomPtr_80(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x800000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
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
unsafe fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
        game,
    );
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
pub const fnDoorTransition_FadeInScreenAndFinish: ::core::ffi::c_int =
    0x82e737 as ::core::ffi::c_int;
pub unsafe fn APU_UploadBank(mut addr: uint32, game: &mut Game) {
    if !game.g_use_my_apu_code {
        crate::native_audio::upload(addr, game);
        return;
    }
    RtlApuUpload(RomPtr(addr as uint32_t, game), game);
}
pub unsafe fn NextRandom(game: &mut Game) -> uint16 {
    let mut RegWord: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            * 5 as ::core::ffi::c_int) as uint16;
    let mut Reg: uint8 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        * 5 as ::core::ffi::c_int) as uint8;
    let mut carry: ::core::ffi::c_int = *(&raw mut RegWord as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + Reg as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int;
    *(&raw mut RegWord as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    let mut result: uint16 = ((carry >> 8 as ::core::ffi::c_int)
        + RegWord as ::core::ffi::c_int
        + 17 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) = result;
    return result;
}
pub unsafe fn ReleaseButtonsFilter(mut v0: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5dd as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
    let mut v1: bool = !(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x5d9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5d9 as ::core::ffi::c_int as isize)
        as *mut uint16) = (!(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int)
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if !v1 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5db as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5dd as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x5df as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        let ref mut fresh87 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5db as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh87 = (*fresh87).wrapping_sub(1);
        if *fresh87 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x5df as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x5db as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5e3 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x5df as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x5df as ::core::ffi::c_int as isize)
                as *mut uint16) = (!(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x8f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                & *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e1 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5df as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x5e3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            ^ *((&raw mut game.g_ram as *mut uint8).offset(0x5df as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)) as uint16;
}
pub fn PrepareBitAccess(a: uint16, game: &mut Game) -> uint16 {
    let bitmask = 1u16 << (a & 7);
    game.g_ram[0x5e7..0x5e9].copy_from_slice(&bitmask.to_le_bytes());
    a >> 3
}
pub unsafe fn SetBossBitForCurArea(mut a: uint16, game: &mut Game) {
    let ref mut fresh91 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    *fresh91 = (*fresh91 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint8;
}
pub unsafe fn ClearBossBitForCurArea(mut a: uint16, game: &mut Game) {
    let ref mut fresh19 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    *fresh19 = (*fresh19 as ::core::ffi::c_int & !(a as ::core::ffi::c_int)) as uint8;
}
pub unsafe fn CheckBossBitForCurArea(mut a: uint16, game: &mut Game) -> uint8 {
    return (a as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd828 as ::core::ffi::c_int as isize)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn SetEventHappened(mut a: uint16, game: &mut Game) {
    let mut v1: uint16 = PrepareBitAccess(a, game);
    let ref mut fresh94 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd820 as ::core::ffi::c_int as isize)
        .offset(v1 as isize);
    *fresh94 = (*fresh94 as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint8;
}
pub unsafe fn ClearEventHappened(mut v0: uint16, game: &mut Game) {
    let mut v1: uint16 = PrepareBitAccess(v0, game);
    let ref mut fresh20 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd820 as ::core::ffi::c_int as isize)
        .offset(v1 as isize);
    *fresh20 = (*fresh20 as ::core::ffi::c_int
        & !(*((&raw mut game.g_ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)) as uint8;
}
pub unsafe fn CheckEventHappened(mut a: uint16, game: &mut Game) -> uint16 {
    let mut idx: uint16 = PrepareBitAccess(a, game);
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd820 as ::core::ffi::c_int as isize)
            .offset(idx as isize) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn CopySuperMetroidString(game: &mut Game) {
    memcpy(
        game.g_sram.offset(0x1fe0 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_void,
        b"supermetroid\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        12 as size_t,
    );
    RtlWriteSram(game);
}
pub unsafe fn VerifySRAM(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f59 as ::core::ffi::c_int as isize)
        as *mut uint16) = 3 as uint16;
    if LoadFromSram(0 as uint16, game) as ::core::ffi::c_int != 0
        && LoadFromSram(1 as uint16, game) as ::core::ffi::c_int != 0
        && LoadFromSram(2 as uint16, game) as ::core::ffi::c_int != 0
    {
        memcpy(
            game.g_sram.offset(0x1fe0 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut ::core::ffi::c_void,
            b"madadameyohn\0" as *const u8 as *const ::core::ffi::c_char
                as *const ::core::ffi::c_void,
            12 as size_t,
        );
        RtlWriteSram(game);
    } else if memcmp(
        game.g_sram.offset(0x1fe0 as ::core::ffi::c_int as isize) as *mut uint8
            as *const ::core::ffi::c_void,
        b"supermetroid\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        12 as size_t,
    ) == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f59 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
}
pub unsafe fn Multiply16x16(mut a: uint16, mut j: uint16) -> uint32 {
    let mut result: uint32 = (a as uint32).wrapping_mul(j as uint32);
    return result;
}
pub unsafe fn WaitForNMI_Async(game: &mut Game) -> CoroutineRet {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) =
            0 as uint8;
        return 0 as CoroutineRet;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b4 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) = 1 as uint8;
    return 1 as CoroutineRet;
}
pub unsafe fn WaitForNMI_NoUpdate_Async(game: &mut Game) -> CoroutineRet {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) =
            0 as uint8;
        return 0 as CoroutineRet;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x782 as ::core::ffi::c_int as isize) = 1 as uint8;
    return 1 as CoroutineRet;
}
pub unsafe fn EnableNMI(game: &mut Game) {
    let mut v0: uint8 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    WriteReg(
        NMITIMEN as ::core::ffi::c_int as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int) as uint8,
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize) = v0;
}
pub unsafe fn DisableNMI(game: &mut Game) {
    let mut v0: uint8 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int) as uint8;
    WriteReg(
        NMITIMEN as ::core::ffi::c_int as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int) as uint8,
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize) = v0;
}
pub unsafe fn ScreenOff(game: &mut Game) {
    let ref mut fresh89 =
        *(&raw mut game.g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize);
    *fresh89 = (*fresh89 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn ScreenOn(game: &mut Game) {
    let ref mut fresh90 =
        *(&raw mut game.g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize);
    *fresh90 = (*fresh90 as ::core::ffi::c_int & !(0x80 as ::core::ffi::c_int)) as uint8;
}
pub unsafe fn memset7E(mut k: *mut uint16, mut a: uint16, mut j: uint16) {
    loop {
        let fresh99 = k;
        k = k.offset(1);
        *fresh99 = a;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !(j != 0) {
            break;
        }
    }
}
pub unsafe fn Vector_RESET_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut game.g_ram as *mut uint8).offset(0x611 as ::core::ffi::c_int as isize);
    let mut bak: uint16 = 0;
    let mut current_block_63: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        1 => {
            WriteReg(MEMSEL as ::core::ffi::c_int as uint16, 1 as uint8, game);
            *(&raw mut game.g_ram as *mut uint8).offset(0x86 as ::core::ffi::c_int as isize) =
                1 as uint8;
            bak = *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            memset(
                &raw mut game.g_ram as *mut uint8 as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                8192 as size_t,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) = bak;
            current_block_63 = 17803482508355472374;
        }
        2 => {
            current_block_63 = 17803482508355472374;
        }
        3 => {
            current_block_63 = 3640593987805443782;
        }
        _ => {
            current_block_63 = 5330834795799507926;
        }
    }
    match current_block_63 {
        17803482508355472374 => {
            let mut _coret_: uint8 = InitializeIoDisplayLogo_Async(game) as uint8;
            if _coret_ != 0 {
                *_state_variable_ = 2 as uint8;
                return _coret_ as CoroutineRet;
            }
            current_block_63 = 3640593987805443782;
        }
        _ => {}
    }
    match current_block_63 {
        3640593987805443782 => {
            APU_UploadBank(0xcf8000 as uint32, game);
            WriteReg(INIDISP as ::core::ffi::c_int as uint16, 0x8f as uint8, game);
            bak = *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            memset(
                &raw mut game.g_ram as *mut uint8 as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                0x10000 as ::core::ffi::c_int as size_t,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xc2 as ::core::ffi::c_int as isize)
                as *mut uint16) = bak;
            WriteReg(NMITIMEN as ::core::ffi::c_int as uint16, 0 as uint8, game);
            *(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize) =
                0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize) =
                0x8f as uint8;
            InitializeCpuIoRegs(game);
            InitializePpuIoRegs(game);
            WriteLotsOf0x1c2f(game);
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x643 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x643 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x643 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x649 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x649 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x649 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x64d as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x64d as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x64d as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x650 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x650 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x650 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x656 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x666 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0x676 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x53 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ClearOamExt(game);
            ClearUnusedOam(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x71d as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x71f as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x721 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            EnableNMI(game);
            RtlApuWrite(APUI00 as ::core::ffi::c_int as uint32, 0 as uint8, game);
            RtlApuWrite(APUI01 as ::core::ffi::c_int as uint32, 0 as uint8, game);
            RtlApuWrite(APUI02 as ::core::ffi::c_int as uint32, 0 as uint8, game);
            RtlApuWrite(APUI03 as ::core::ffi::c_int as uint32, 0 as uint8, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) = 97 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x63f as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(4 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(5 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(6 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(7 as ::core::ffi::c_int as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            VerifySRAM(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x686 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        _ => {}
    }
    *_state_variable_ = 2 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn InvalidInterrupt_Crash(game: &mut Game) {
    printf(b"InvalidInterrupt_Crash\n\0" as *const u8 as *const ::core::ffi::c_char);
    Unreachable(game);
    loop {}
}
pub unsafe fn LoadMirrorOfExploredMapTiles(game: &mut Game) {
    let mut v1: uint16 = swap16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7f7 as ::core::ffi::c_int as isize)
            .offset(v2 as isize) as *mut uint8 as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd52 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
        as *mut uint16) = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd908 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as uint16;
}
pub unsafe fn SaveExploredMapTilesToSaved(game: &mut Game) {
    let mut v1: uint16 = swap16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7 as ::core::ffi::c_int as isize)
                .offset(v2 as isize) as *mut uint8 as *mut uint16);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd908 as ::core::ffi::c_int as isize)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as *mut uint8 as *mut uint16);
        *fresh88 = (*fresh88 as ::core::ffi::c_int | 0xff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn InitializeCpuIoRegs(game: &mut Game) {
    WriteReg(NMITIMEN as ::core::ffi::c_int as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize) = 1 as uint8;
    WriteReg(WRIO as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(WRMPYA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(WRMPYB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(WRDIVL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(WRDIVH as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(WRDIVB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(HTIMEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(HTIMEH as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VTIMEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VTIMEH as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(MEMSEL as ::core::ffi::c_int as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x86 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn InitializePpuIoRegs(game: &mut Game) {
    WriteReg(INIDISP as ::core::ffi::c_int as uint16, 0x8f as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize) =
        0x8f as uint8;
    WriteReg(OBSEL as ::core::ffi::c_int as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52 as ::core::ffi::c_int as isize) = 3 as uint8;
    WriteReg(OAMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(OAMADDH as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = 0x80 as uint8;
    WriteReg(OAMDATA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(OAMDATA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BGMODE as ::core::ffi::c_int as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55 as ::core::ffi::c_int as isize) = 9 as uint8;
    WriteReg(MOSAIC as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(BG1SC as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize) = 64 as uint8;
    WriteReg(BG2SC as ::core::ffi::c_int as uint16, 0x44 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 68 as uint8;
    WriteReg(BG3SC as ::core::ffi::c_int as uint16, 0x48 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a as ::core::ffi::c_int as isize) = 72 as uint8;
    WriteReg(BG4SC as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(BG12NBA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(BG34NBA as ::core::ffi::c_int as uint16, 5 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e as ::core::ffi::c_int as isize) = 5 as uint8;
    WriteReg(BG1HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG1HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG1VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG1VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG2HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG2HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG2VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG2VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG4HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG4HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG4VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG4VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7SEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(M7A as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7B as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7C as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7D as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7X as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(M7Y as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WOBJSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WH0 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x63 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WH1 as ::core::ffi::c_int as uint16, 0xf8 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x64 as ::core::ffi::c_int as isize) =
        -(8 as ::core::ffi::c_int) as uint8;
    WriteReg(WH2 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x65 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WH3 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x66 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WBGLOG as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x67 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(WOBJLOG as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x68 as ::core::ffi::c_int as isize) = 0 as uint8;
    WriteReg(TM as ::core::ffi::c_int as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    WriteReg(TMW as ::core::ffi::c_int as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 17 as uint8;
    WriteReg(TS as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(TSW as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0xa1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71 as ::core::ffi::c_int as isize) =
        -(95 as ::core::ffi::c_int) as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0xe0 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0xe0 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x20 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 32 as uint8;
    WriteReg(SETINI as ::core::ffi::c_int as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn WriteLotsOf0x1c2f(game: &mut Game) {
    sub_8088EB(0x1c2f as uint16, game);
    sub_8088FE(0x1c2f as uint16, game);
    sub_808911(0x1c2f as uint16, game);
}
pub unsafe fn sub_8088EB(mut a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut uint16,
        a,
        0x800 as uint16,
    );
}
pub unsafe fn sub_8088FE(mut a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut uint16,
        a,
        0x800 as uint16,
    );
}
pub unsafe fn sub_808911(mut a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x6000 as ::core::ffi::c_int as isize)
            as *mut uint16,
        a,
        0x800 as uint16,
    );
}
fn tick_screen_fade(game: &mut Game) -> bool {
    let screen_fade_counter = u16::from_le_bytes([game.g_ram[0x725], game.g_ram[0x726]]);
    // Test the sign after the word decrement, then reload before changing brightness.
    let remaining = screen_fade_counter.wrapping_sub(1);
    let step_due = (remaining as i16) < 0;
    let next_counter = if step_due {
        let screen_fade_delay = u16::from_le_bytes([game.g_ram[0x723], game.g_ram[0x724]]);
        screen_fade_delay
    } else {
        remaining
    };
    game.g_ram[0x725..0x727].copy_from_slice(&next_counter.to_le_bytes());
    step_due
}

pub fn HandleFadeOut(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    match *reg_inidisp & 0x0f {
        0 => {}
        1 => *reg_inidisp = 0x80,
        brightness => *reg_inidisp = brightness - 1,
    }
}

pub fn HandleFadeIn(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    let brightness = reg_inidisp.wrapping_add(1) & 0x0f;
    if brightness != 0 {
        *reg_inidisp = brightness;
    }
}
pub fn ClearUnusedOam(game: &mut Game) {
    let oam_next_ptr = u16::from_le_bytes([game.g_ram[0x590], game.g_ram[0x591]]);
    let first_unused_sprite = usize::from(oam_next_ptr >> 2);
    for sprite in game.g_ram[0x370..0x570]
        .chunks_exact_mut(4)
        .skip(first_unused_sprite)
    {
        // Each four-byte OAM entry stores its Y coordinate in byte 1.
        sprite[1] = 0xf0;
    }
    game.g_ram[0x590..0x592].fill(0);
}

pub fn ClearOamExt(game: &mut Game) {
    game.g_ram[0x570..0x590].fill(0);
}
pub unsafe fn QueueMode7Transfers(mut db: uint8, mut k: uint16, game: &mut Game) {
    let mut p: *const uint8 = RomPtrWithBank(db, k as uint16_t, game);
    let mut dst: *mut uint8 = ((&raw mut game.g_ram as *mut uint8)
        .offset(0x2d0 as ::core::ffi::c_int as isize)
        as *mut Mode7VramWriteQueue as *mut uint8)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x334 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        );
    loop {
        let mut f: ::core::ffi::c_int = *(p as *mut uint8) as ::core::ffi::c_int;
        if f & 0x80 as ::core::ffi::c_int != 0 {
            *dst.offset(9 as ::core::ffi::c_int as isize) = 0 as uint8;
            memcpy(
                dst as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                9 as size_t,
            );
            p = p.offset(9 as ::core::ffi::c_int as isize);
            dst = dst.offset(9 as ::core::ffi::c_int as isize);
        } else {
            if !(f & 0x40 as ::core::ffi::c_int != 0) {
                break;
            }
            *dst.offset(7 as ::core::ffi::c_int as isize) = 0 as uint8;
            memcpy(
                dst as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                7 as size_t,
            );
            p = p.offset(7 as ::core::ffi::c_int as isize);
            dst = dst.offset(7 as ::core::ffi::c_int as isize);
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x334 as ::core::ffi::c_int as isize)
        as *mut uint16) = dst.offset_from(
        (&raw mut game.g_ram as *mut uint8).offset(0x2d0 as ::core::ffi::c_int as isize)
            as *mut Mode7VramWriteQueue as *mut uint8,
    ) as ::core::ffi::c_long as uint16;
}
pub unsafe fn NMI_ProcessMode7Queue(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x334 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        NMI_ProcessMode7QueueInner(
            (&raw mut game.g_ram as *mut uint8).offset(0x2d0 as ::core::ffi::c_int as isize)
                as *mut uint8,
            game,
        );
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x2d0 as ::core::ffi::c_int as isize)
            as *mut Mode7VramWriteQueue)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x334 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn NMI_ProcessMode7QueueInner(mut p: *const uint8, game: &mut Game) {
    loop {
        let mut v2: uint8 = 0;
        loop {
            v2 = *p;
            if v2 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            WriteReg(
                DMAP1 as ::core::ffi::c_int as uint16,
                (v2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint8,
                game,
            );
            let mut v5: *mut Mode7CgvmWriteQueue = p as *mut Mode7CgvmWriteQueue;
            WriteRegWord(
                A1T1L as ::core::ffi::c_int as uint16,
                (*v5).src_addr.addr as uint16,
                game,
            );
            WriteReg(
                A1B1 as ::core::ffi::c_int as uint16,
                (*v5).src_addr.bank,
                game,
            );
            WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, (*v5).count, game);
            if v2 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 {
                WriteReg(BBAD1 as ::core::ffi::c_int as uint16, 0x19 as uint8, game);
            } else {
                WriteReg(BBAD1 as ::core::ffi::c_int as uint16, 0x18 as uint8, game);
            }
            WriteRegWord(
                VMADDL as ::core::ffi::c_int as uint16,
                (*v5).vram_addr,
                game,
            );
            WriteReg(VMAIN as ::core::ffi::c_int as uint16, (*v5).vmain, game);
            WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
            p = p.offset(::core::mem::size_of::<Mode7CgvmWriteQueue>() as usize as isize);
        }
        if v2 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int == 0 {
            break;
        }
        WriteReg(
            DMAP1 as ::core::ffi::c_int as uint16,
            (v2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint8,
            game,
        );
        WriteRegWord(
            A1T1L as ::core::ffi::c_int as uint16,
            *(p.offset(1 as ::core::ffi::c_int as isize) as *mut uint16),
            game,
        );
        WriteReg(
            A1B1 as ::core::ffi::c_int as uint16,
            *p.offset(3 as ::core::ffi::c_int as isize),
            game,
        );
        WriteRegWord(
            DAS1L as ::core::ffi::c_int as uint16,
            *(p as *mut uint16).offset(2 as ::core::ffi::c_int as isize),
            game,
        );
        WriteReg(BBAD1 as ::core::ffi::c_int as uint16, 0x22 as uint8, game);
        WriteReg(
            CGADD as ::core::ffi::c_int as uint16,
            *p.offset(6 as ::core::ffi::c_int as isize),
            game,
        );
        WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
        p = p.offset(7 as ::core::ffi::c_int as isize);
    }
}
pub unsafe fn NMI_ProcessVramWriteQueue(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        (*gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .size = 0 as uint16;
        WriteRegWord(
            DMAP1 as ::core::ffi::c_int as uint16,
            0x1801 as uint16,
            game,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let mut e: *mut VramWriteEntry = gVramWriteEntry(i, game);
            if (*e).size == 0 {
                break;
            }
            WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, (*e).size, game);
            WriteRegWord(
                A1T1L as ::core::ffi::c_int as uint16,
                (*e).src.addr as uint16,
                game,
            );
            WriteReg(A1B1 as ::core::ffi::c_int as uint16, (*e).src.bank, game);
            WriteRegWord(
                VMAIN as ::core::ffi::c_int as uint16,
                (if (*e).vram_dst as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                    0x81 as ::core::ffi::c_int
                } else {
                    0x80 as ::core::ffi::c_int
                }) as uint16,
                game,
            );
            WriteRegWord(VMADDL as ::core::ffi::c_int as uint16, (*e).vram_dst, game);
            WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
            i += 7 as ::core::ffi::c_int;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    Nmi_ProcessHorizScrollingDma(game);
    Nmi_ProcessVertScrollingDma(game);
}
#[derive(Clone, Copy)]
enum ScrollingDmaLayout {
    Column,
    Row,
}

impl ScrollingDmaLayout {
    fn second_half_destination(self, address: u16) -> u16 {
        match self {
            Self::Column => address.wrapping_add(1),
            Self::Row => address | 0x20,
        }
    }
}

unsafe fn transfer_scrolling_tilemap(
    parameters: usize,
    initial_sources: [u16; 2],
    layout: ScrollingDmaLayout,
    game: &mut Game,
) {
    let enable = parameters + 12;
    if game.g_ram[enable] == 0 {
        return;
    }
    // Only the low enable byte is consumed, before reading transfer parameters.
    game.g_ram[enable] = 0;
    // Sizes, destinations and wrapped sources are three pairs of RAM words.
    let read_parameter = |game: &Game, offset: usize| {
        u16::from_le_bytes([
            game.g_ram[parameters + offset],
            game.g_ram[parameters + offset + 1],
        ])
    };

    let destination = read_parameter(game, 4);
    WriteRegWord(VMADDL as u16, destination, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, initial_sources[0], game);
    WriteReg(A1B1 as u16, 0x7e, game);
    let size = read_parameter(game, 0);
    // A zero unwrapped size still triggers a full 65,536-byte DMA.
    WriteRegWord(DAS1L as u16, size, game);
    WriteReg(MDMAEN as u16, 2, game);
    WriteRegWord(
        VMADDL as u16,
        layout.second_half_destination(destination),
        game,
    );
    WriteRegWord(DAS1L as u16, size, game);
    WriteRegWord(A1T1L as u16, initial_sources[1], game);
    WriteReg(MDMAEN as u16, 2, game);

    // Prepare the wrapped source even when there is no wrapped transfer.
    WriteRegWord(A1T1L as u16, read_parameter(game, 8), game);
    let wrapped_size = read_parameter(game, 2);
    if wrapped_size != 0 {
        WriteRegWord(DAS1L as u16, wrapped_size, game);
        let wrapped_destination = read_parameter(game, 6);
        WriteRegWord(VMADDL as u16, wrapped_destination, game);
        WriteReg(MDMAEN as u16, 2, game);
        WriteRegWord(
            VMADDL as u16,
            layout.second_half_destination(wrapped_destination),
            game,
        );
        WriteRegWord(DAS1L as u16, wrapped_size, game);
        WriteRegWord(A1T1L as u16, read_parameter(game, 10), game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

pub unsafe fn Nmi_ProcessHorizScrollingDma(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x81, game);
    let layout = ScrollingDmaLayout::Column;
    transfer_scrolling_tilemap(0x956, [0xc8c8, 0xc908], layout, game);
    transfer_scrolling_tilemap(0x972, [0xc9d0, 0xca10], layout, game);
}

pub unsafe fn Nmi_ProcessVertScrollingDma(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x80, game);
    let layout = ScrollingDmaLayout::Row;
    transfer_scrolling_tilemap(0x964, [0xc948, 0xc98c], layout, game);
    transfer_scrolling_tilemap(0x980, [0xca50, 0xca94], layout, game);
}
pub unsafe fn NMI_ProcessVramReadQueue(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8
        != 0
    {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x340 as ::core::ffi::c_int as isize)
            as *mut VramReadQueueEnt)
            .offset(0 as ::core::ffi::c_int as isize))
        .vram_target as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int as isize,
            ) = 0 as uint8;
        let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
        while (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
            as *mut VramReadQueueEnt)
            .offset(v0 as isize))
        .vram_target
            != 0
        {
            WriteRegWord(
                VMADDL as ::core::ffi::c_int as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .vram_target,
             game);
            ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16, game);
            WriteRegWord(
                DMAP1 as ::core::ffi::c_int as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .dma_parameters,
             game);
            WriteRegWord(
                A1T1L as ::core::ffi::c_int as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .src
                .addr as uint16,
                game,
            );
            WriteRegWord(
                A1T1H as ::core::ffi::c_int as uint16,
                *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x340 as ::core::ffi::c_int as isize)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .src
                .addr as *mut uint8)
                    .offset(1 as ::core::ffi::c_int as isize) as *mut VoidP),
                game,
            );
            WriteRegWord(
                DAS1L as ::core::ffi::c_int as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340 as ::core::ffi::c_int as isize)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .size,
             game);
            WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
            WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
            WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
            v0 += 1;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x360 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    }
}
pub unsafe fn HasQueuedMusic(game: &mut Game) -> uint8 {
    let mut v0: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 -= 2 as ::core::ffi::c_int;
        if v0 < 0 as ::core::ffi::c_int {
            return 0 as uint8;
        }
    }
    return 1 as uint8;
}
pub unsafe fn HandleMusicQueue(game: &mut Game) {
    let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x63f as ::core::ffi::c_int as isize) as *mut uint16);
    let fresh75 = *fresh74;
    *fresh74 = (*fresh74).wrapping_sub(1);
    let mut v0: bool = fresh75 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x63f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if !v0 {
            return;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x63d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x7f3 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x63d as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as uint16;
            *(&raw mut game.g_ram as *mut uint8).offset(0x64c as ::core::ffi::c_int as isize) =
                -(1 as ::core::ffi::c_int) as uint8;
            APU_UploadBank(
                Load24(
                    (game.g_rom.offset(
                        (((0x8fe7e1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x8fe7e1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut LongPtr as *mut uint8)
                        .offset(
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x63d as ::core::ffi::c_int as isize)
                                as *mut uint16) as uint8
                                as ::core::ffi::c_int as isize,
                        ) as *mut LongPtr,
                ),
                game,
            );
            *(&raw mut game.g_ram as *mut uint8).offset(0x64c as ::core::ffi::c_int as isize) =
                0 as uint8;
            let mut v5: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x63b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x619 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x63b as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (v5 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x686 as ::core::ffi::c_int as isize)
                as *mut uint16) = 8 as uint16;
            return;
        }
        let mut v1: uint8 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x63d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int) as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7f5 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x63d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int) as uint16;
        RtlApuWrite(
            APUI00 as ::core::ffi::c_int as uint32,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x63d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x7f as ::core::ffi::c_int) as uint8,
            game,
        );
        *(&raw mut game.g_ram as *mut uint8).offset(0x64c as ::core::ffi::c_int as isize) = v1;
        *((&raw mut game.g_ram as *mut uint8).offset(0x686 as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x63b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x63b as ::core::ffi::c_int as isize)
            as *mut uint16) = (v2 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x63b as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x63f as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x63b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x63d as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x619 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x63f as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x629 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
    };
}
pub unsafe fn QueueMusic_Delayed8(mut a: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < kGameState_40_TransitionToDemo as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
            & 0xe as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x63b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) as int8;
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = a;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
            & 0xe as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn QueueMusic_DelayedY(mut a: uint16, mut j: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x28 as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = a;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int;
        if (j as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
            v4 = 8 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x629 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = v4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x639 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v2 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn QueueSfx1_Max15(mut a: uint16, game: &mut Game) {
    QueueSfx1_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 15 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx1_Max9(mut a: uint16, game: &mut Game) {
    QueueSfx1_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 9 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx1_Max3(mut a: uint16, game: &mut Game) {
    QueueSfx1_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx1_Max1(mut a: uint16, game: &mut Game) {
    QueueSfx1_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx1_Max6(mut a: uint16, game: &mut Game) {
    QueueSfx1_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 6 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx1_Internal(mut a: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x653 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = a as uint8;
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x646 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - *(&raw mut game.g_ram as *mut uint8)
            .offset(0x643 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        < a as uint8 as ::core::ffi::c_int
    {
        let mut v2: uint8 = ((a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < 0x28 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            let mut v1: uint8 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize);
            let mut v3: uint8 = (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8;
            if v3 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v3 = 0 as uint8;
            }
            if v3 as ::core::ffi::c_int
                == *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x643 as ::core::ffi::c_int as isize)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
            {
                if (v2 as ::core::ffi::c_int)
                    < *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x656 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) as ::core::ffi::c_int
                {
                    *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x656 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) = v2;
                }
            } else {
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x656 as ::core::ffi::c_int as isize)
                    .offset(
                        *(&raw mut game.g_ram as *mut uint8)
                            .offset(0x646 as ::core::ffi::c_int as isize)
                            .offset(0 as ::core::ffi::c_int as isize)
                            as isize,
                    ) = v2;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x646 as ::core::ffi::c_int as isize)
                    .offset(0 as ::core::ffi::c_int as isize) = v3;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x656 as ::core::ffi::c_int as isize)
                    .offset(v3 as isize) = 0 as uint8;
            }
        }
    }
}
pub unsafe fn QueueSfx2_Max15(mut a: uint16, game: &mut Game) {
    QueueSfx2_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 15 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx2_Max9(mut a: uint16, game: &mut Game) {
    QueueSfx2_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 9 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx2_Max3(mut a: uint16, game: &mut Game) {
    QueueSfx2_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx2_Max1(mut a: uint16, game: &mut Game) {
    QueueSfx2_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx2_Max6(mut a: uint16, game: &mut Game) {
    QueueSfx2_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 6 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx2_Internal(mut a: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x653 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = a as uint8;
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x646 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - *(&raw mut game.g_ram as *mut uint8)
            .offset(0x643 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        < a as uint8 as ::core::ffi::c_int
    {
        let mut v2: uint8 = ((a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < 0x28 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            let mut v1: uint8 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            let mut v3: uint8 = (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8;
            if v3 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v3 = 0 as uint8;
            }
            if v3 as ::core::ffi::c_int
                == *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x643 as ::core::ffi::c_int as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
            {
                if (v2 as ::core::ffi::c_int)
                    < *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x666 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) as ::core::ffi::c_int
                {
                    *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x666 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) = v2;
                }
            } else {
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x666 as ::core::ffi::c_int as isize)
                    .offset(
                        *(&raw mut game.g_ram as *mut uint8)
                            .offset(0x646 as ::core::ffi::c_int as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as isize,
                    ) = v2;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x646 as ::core::ffi::c_int as isize)
                    .offset(1 as ::core::ffi::c_int as isize) = v3;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x666 as ::core::ffi::c_int as isize)
                    .offset(v3 as isize) = 0 as uint8;
            }
        }
    }
}
pub unsafe fn QueueSfx3_Max15(mut a: uint16, game: &mut Game) {
    QueueSfx3_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 15 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx3_Max9(mut a: uint16, game: &mut Game) {
    QueueSfx3_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 9 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx3_Max3(mut a: uint16, game: &mut Game) {
    QueueSfx3_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx3_Max1(mut a: uint16, game: &mut Game) {
    QueueSfx3_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx3_Max6(mut a: uint16, game: &mut Game) {
    QueueSfx3_Internal(
        ((a as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | 6 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn QueueSfx3_Internal(mut a: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x653 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = a as uint8;
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x646 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - *(&raw mut game.g_ram as *mut uint8)
            .offset(0x643 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        < a as uint8 as ::core::ffi::c_int
    {
        let mut v2: uint8 = ((a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < 0x28 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x592 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            let mut v1: uint8 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize);
            let mut v3: uint8 = (*(&raw mut game.g_ram as *mut uint8)
                .offset(0x646 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8;
            if v3 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
                v3 = 0 as uint8;
            }
            if v3 as ::core::ffi::c_int
                == *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x643 as ::core::ffi::c_int as isize)
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
            {
                if (v2 as ::core::ffi::c_int)
                    < *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x676 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) as ::core::ffi::c_int
                {
                    *(&raw mut game.g_ram as *mut uint8)
                        .offset(0x676 as ::core::ffi::c_int as isize)
                        .offset(v1 as isize) = v2;
                }
            } else {
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x676 as ::core::ffi::c_int as isize)
                    .offset(
                        *(&raw mut game.g_ram as *mut uint8)
                            .offset(0x646 as ::core::ffi::c_int as isize)
                            .offset(2 as ::core::ffi::c_int as isize)
                            as isize,
                    ) = v2;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x646 as ::core::ffi::c_int as isize)
                    .offset(2 as ::core::ffi::c_int as isize) = v3;
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x676 as ::core::ffi::c_int as isize)
                    .offset(v3 as isize) = 0 as uint8;
            }
        }
    }
}
pub unsafe fn SetupDmaTransfer(mut p: *const ::core::ffi::c_void, game: &mut Game) {
    let mut s: *const StartDmaCopy = p as *const StartDmaCopy;
    let mut v2: ::core::ffi::c_int = (*s).chan as ::core::ffi::c_int * 16 as ::core::ffi::c_int;
    WriteRegWord(
        (v2 + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *(&raw const (*s).dmap as *mut uint16),
        game,
    );
    WriteRegWord(
        (v2 + A1T0L as ::core::ffi::c_int) as SnesRegs as uint16,
        *(&raw const (*s).a1.addr as *mut uint16),
        game,
    );
    WriteReg(
        (v2 + A1B0 as ::core::ffi::c_int) as SnesRegs as uint16,
        (*s).a1.bank,
        game,
    );
    WriteRegWord(
        (v2 + DAS0L as ::core::ffi::c_int) as SnesRegs as uint16,
        (*s).das as uint16,
        game,
    );
}
pub unsafe fn NmiUpdateIoRegisters(game: &mut Game) {
    WriteReg(NMITIMEN as u16, game.g_ram[0x84], game);
    WriteReg(INIDISP as u16, game.g_ram[0x51], game);
    WriteReg(OBSEL as u16, game.g_ram[0x52], game);
    WriteReg(BGMODE as u16, game.g_ram[0x55], game);
    WriteReg(MOSAIC as u16, game.g_ram[0x57], game);
    WriteReg(BG1SC as u16, game.g_ram[0x58], game);
    WriteReg(BG2SC as u16, game.g_ram[0x59], game);
    WriteReg(BG3SC as u16, game.g_ram[0x5a], game);
    WriteReg(BG4SC as u16, game.g_ram[0x5c], game);
    WriteReg(BG12NBA as u16, game.g_ram[0x5d], game);
    WriteReg(BG34NBA as u16, game.g_ram[0x5e], game);
    WriteReg(M7SEL as u16, game.g_ram[0x5f], game);
    WriteReg(W12SEL as u16, game.g_ram[0x60], game);
    WriteReg(W34SEL as u16, game.g_ram[0x61], game);
    WriteReg(WOBJSEL as u16, game.g_ram[0x62], game);
    WriteReg(WH0 as u16, game.g_ram[0x63], game);
    WriteReg(WH1 as u16, game.g_ram[0x64], game);
    WriteReg(WH2 as u16, game.g_ram[0x65], game);
    WriteReg(WH3 as u16, game.g_ram[0x66], game);
    WriteReg(WBGLOG as u16, game.g_ram[0x67], game);
    WriteReg(WOBJLOG as u16, game.g_ram[0x68], game);
    game.g_ram[0x6a] = game.g_ram[0x69];
    WriteReg(TM as u16, game.g_ram[0x69], game);
    WriteReg(TMW as u16, game.g_ram[0x6c], game);
    WriteReg(TS as u16, game.g_ram[0x6b], game);
    WriteReg(TSW as u16, game.g_ram[0x6d], game);
    WriteReg(CGWSEL as u16, game.g_ram[0x6f], game);
    WriteReg(CGADSUB as u16, game.g_ram[0x72], game);
    game.g_ram[0x70] = game.g_ram[0x6e];
    game.g_ram[0x73] = game.g_ram[0x71];
    WriteReg(COLDATA as u16, game.g_ram[0x74], game);
    WriteReg(COLDATA as u16, game.g_ram[0x75], game);
    WriteReg(COLDATA as u16, game.g_ram[0x76], game);
    WriteReg(SETINI as u16, game.g_ram[0x77], game);

    // Scroll and Mode 7 words latch low then high bytes through the same address.
    WriteReg(BG1HOFS as u16, game.g_ram[0xb1], game);
    WriteReg(BG1HOFS as u16, game.g_ram[0xb2], game);
    WriteReg(BG1VOFS as u16, game.g_ram[0xb3], game);
    WriteReg(BG1VOFS as u16, game.g_ram[0xb4], game);
    WriteReg(BG2HOFS as u16, game.g_ram[0xb5], game);
    WriteReg(BG2HOFS as u16, game.g_ram[0xb6], game);
    WriteReg(BG2VOFS as u16, game.g_ram[0xb7], game);
    WriteReg(BG2VOFS as u16, game.g_ram[0xb8], game);
    WriteReg(BG3HOFS as u16, game.g_ram[0xb9], game);
    WriteReg(BG3HOFS as u16, game.g_ram[0xba], game);
    WriteReg(BG3VOFS as u16, game.g_ram[0xbb], game);
    WriteReg(BG3VOFS as u16, game.g_ram[0xbc], game);
    WriteReg(BG4HOFS as u16, game.g_ram[0xbd], game);
    WriteReg(BG4HOFS as u16, game.g_ram[0xbe], game);
    WriteReg(BG4VOFS as u16, game.g_ram[0xbf], game);
    WriteReg(BG4VOFS as u16, game.g_ram[0xc0], game);
    game.g_ram[0x7ec] = game.g_ram[0x56];
    if game.g_ram[0x55] & 7 == 7 || game.g_ram[0x56] & 7 == 7 {
        WriteReg(M7A as u16, game.g_ram[0x78], game);
        WriteReg(M7A as u16, game.g_ram[0x79], game);
        WriteReg(M7B as u16, game.g_ram[0x7a], game);
        WriteReg(M7B as u16, game.g_ram[0x7b], game);
        WriteReg(M7C as u16, game.g_ram[0x7c], game);
        WriteReg(M7C as u16, game.g_ram[0x7d], game);
        WriteReg(M7D as u16, game.g_ram[0x7e], game);
        WriteReg(M7D as u16, game.g_ram[0x7f], game);
        WriteReg(M7X as u16, game.g_ram[0x80], game);
        WriteReg(M7X as u16, game.g_ram[0x81], game);
        WriteReg(M7Y as u16, game.g_ram[0x82], game);
        WriteReg(M7Y as u16, game.g_ram[0x83], game);
    }
}
pub unsafe fn NmiUpdatePalettesAndOam(game: &mut Game) {
    // Channel 0 uploads the OAM entries and their high table from RAM.
    WriteRegWord(DMAP0 as u16, 0x0400, game);
    WriteRegWord(A1T0L as u16, 0x0370, game);
    WriteReg(A1B0 as u16, 0, game);
    WriteRegWord(DAS0L as u16, 0x0220, game);
    WriteRegWord(OAMADDL as u16, 0, game);

    // Channel 1 uploads palette_buffer to CGRAM.
    WriteRegWord(DMAP1 as u16, 0x2200, game);
    WriteRegWord(A1T1L as u16, 0xc000, game);
    WriteReg(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, 0x0200, game);
    WriteReg(CGADD as u16, 0, game);
    WriteReg(MDMAEN as u16, 3, game);
}
unsafe fn transfer_samus_tile_half(descriptor_addr: u16, vram_addr: u16, game: &mut Game) {
    // Keep the diagnostic and failure flag for low-address ROM mirrors.
    let _ = RomPtr_92(descriptor_addr, game);
    let descriptor = 0x928000 | u32::from(descriptor_addr & 0x7fff);
    // Packed fields: source address at 0, bank at 2, and byte counts at 3 and 5.
    WriteRegWord(VMADDL as u16, vram_addr, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, game.read_rom_word(descriptor, 0), game);
    WriteReg(A1B1 as u16, game.read_rom_byte(descriptor, 2), game);
    // A zero first count still runs a 65,536-byte DMA.
    WriteRegWord(DAS1L as u16, game.read_rom_word(descriptor, 3), game);
    WriteReg(MDMAEN as u16, 2, game);

    // Prepare the second destination/source even when its count is zero.
    WriteRegWord(VMADDL as u16, vram_addr.wrapping_add(0x100), game);
    let second_source = game
        .read_rom_word(descriptor, 0)
        .wrapping_add(game.read_rom_word(descriptor, 3));
    WriteRegWord(A1T1L as u16, second_source, game);
    if game.read_rom_word(descriptor, 5) != 0 {
        WriteRegWord(DAS1L as u16, game.read_rom_word(descriptor, 5), game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

pub unsafe fn NmiTransferSamusToVram(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x80, game);
    if game.g_ram[0x71d] != 0 {
        let top_half_src = u16::from_le_bytes([game.g_ram[0x71f], game.g_ram[0x720]]);
        transfer_samus_tile_half(top_half_src, 0x6000, game);
    }
    if game.g_ram[0x71e] != 0 {
        let bottom_half_src = u16::from_le_bytes([game.g_ram[0x721], game.g_ram[0x722]]);
        transfer_samus_tile_half(bottom_half_src, 0x6080, game);
    }
}
pub unsafe fn NmiProcessAnimtilesVramTransfers(game: &mut Game) {
    // Only bit 15 of animtiles_enable_flag enables transfers.
    if game.g_ram[0x1ef2] & 0x80 == 0 {
        return;
    }
    for slot in (0..6).rev() {
        let offset = slot * 2;
        let id = u16::from_le_bytes([game.g_ram[0x1ef5 + offset], game.g_ram[0x1ef6 + offset]]);
        if id == 0 {
            continue;
        }
        let source_index = 0x1f25 + offset;
        let source = u16::from_le_bytes([game.g_ram[source_index], game.g_ram[source_index + 1]]);
        if source == 0 {
            continue;
        }

        WriteRegWord(A1T0L as u16, source, game);
        WriteReg(A1B0 as u16, 0x87, game);
        WriteRegWord(DMAP0 as u16, 0x1801, game);
        let size = u16::from_le_bytes([game.g_ram[0x1f31 + offset], game.g_ram[0x1f32 + offset]]);
        WriteRegWord(DAS0L as u16, size, game);
        let vram_addr =
            u16::from_le_bytes([game.g_ram[0x1f3d + offset], game.g_ram[0x1f3e + offset]]);
        WriteRegWord(VMADDL as u16, vram_addr, game);
        WriteReg(VMAIN as u16, 0x80, game);
        WriteReg(MDMAEN as u16, 1, game);
        // Consume the request only after its transfer completes.
        game.g_ram[source_index..source_index + 2].fill(0);
    }
}
pub unsafe fn ReadJoypadInputs(game: &mut Game) {
    let mut RegWord: uint16 = ReadRegWord(JOY1L as ::core::ffi::c_int as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
        as *mut uint16) = RegWord;
    *((&raw mut game.g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) = (RegWord as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x97 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            ^ RegWord as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x93 as ::core::ffi::c_int as isize)
        as *mut uint16) = (RegWord as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x97 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            ^ RegWord as ::core::ffi::c_int)) as uint16;
    if RegWord as ::core::ffi::c_int != 0
        && RegWord as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x97 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh80 = (*fresh80).wrapping_sub(1);
        if *fresh80 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x93 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa3 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x89 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa3 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x87 as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x97 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let mut v1: uint16 = ReadRegWord(JOY2L as ::core::ffi::c_int as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
        *((&raw mut game.g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v1 as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x99 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                ^ v1 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x95 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v1 as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x99 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                ^ v1 as ::core::ffi::c_int)) as uint16;
        if v1 as ::core::ffi::c_int != 0
            && v1 as ::core::ffi::c_int
                == *((&raw mut game.g_ram as *mut uint8).offset(0x99 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
        {
            let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa5 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh81 = (*fresh81).wrapping_sub(1);
            if *fresh81 == 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x95 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x8d as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xa5 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x89 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa5 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x87 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x99 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8d as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x617 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kButton_Select as ::core::ffi::c_int
                    | kButton_Start as ::core::ffi::c_int
                    | kButton_L as ::core::ffi::c_int
                    | kButton_R as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x5c5 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x5c7 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x5cf as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x4000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x8b as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & (kButton_Select as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int)
                        == kButton_Select as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int
                    {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x5c5 as ::core::ffi::c_int as isize)
                            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8b as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                    }
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x8b as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & (kButton_Select as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
                        == kButton_Select as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int
                    {
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x5c7 as ::core::ffi::c_int as isize)
                            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8b as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                    }
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x5c7 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x80 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x84 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh82 =
                            (*fresh82 as ::core::ffi::c_int ^ 0x30 as ::core::ffi::c_int) as uint16;
                    }
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x5c7 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        let mut v2: bool = !(*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x5cf as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            as ::core::ffi::c_int)
                            & 0x8000 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int;
                        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x5cf as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh83 = (*fresh83 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
                            as uint16;
                        if v2 {
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5c9 as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9c6 as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5cb as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ca as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5cd as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ce as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9c6 as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ca as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ce as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                        } else {
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9c6 as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5c9 as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ca as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5cb as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x9ce as ::core::ffi::c_int as isize)
                                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x5cd as ::core::ffi::c_int as isize)
                                as *mut uint16);
                        }
                    }
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x5c7 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x40 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        let ref mut fresh84 = *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x5cf as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh84 = (*fresh84 as ::core::ffi::c_int ^ 0x2000 as ::core::ffi::c_int)
                            as uint16;
                    }
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x5c5 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x5c7 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                let ref mut fresh85 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x8d as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh85 = (*fresh85 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int) as uint16;
                let ref mut fresh86 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x91 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh86 = (*fresh86 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int) as uint16;
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            SoftReset(game);
        }
    }
}
pub unsafe fn Vector_NMI(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x5b4 as ::core::ffi::c_int as isize) != 0 {
        NmiUpdatePalettesAndOam(game);
        NmiTransferSamusToVram(game);
        NmiProcessAnimtilesVramTransfers(game);
        NmiUpdateIoRegisters(game);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i != 6 as ::core::ffi::c_int {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(i as isize)
                != 0
            {
                WriteRegWord(
                    (*(((&raw mut game.g_ram as *mut uint8)
                        .offset(0x18c0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(i as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        + 17154 as ::core::ffi::c_int) as SnesRegs as uint16,
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x18d8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(i as isize),
                    game,
                );
            }
            i += 1;
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0x55 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            || *(&raw mut game.g_ram as *mut uint8).offset(0x56 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 7 as ::core::ffi::c_int
        {
            NMI_ProcessMode7Queue(game);
        }
        NMI_ProcessVramWriteQueue(game);
        NMI_ProcessVramReadQueue(game);
        WriteReg(
            HDMAEN as ::core::ffi::c_int as uint16,
            *(&raw mut game.g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize),
            game,
        );
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b4 as ::core::ffi::c_int as isize) =
            0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5ba as ::core::ffi::c_int as isize) =
            0 as uint8;
        let ref mut fresh95 =
            *(&raw mut game.g_ram as *mut uint8).offset(0x5b5 as ::core::ffi::c_int as isize);
        *fresh95 = (*fresh95).wrapping_add(1);
        let ref mut fresh96 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh96 = (*fresh96).wrapping_add(1);
    } else {
        let ref mut fresh97 =
            *(&raw mut game.g_ram as *mut uint8).offset(0x5ba as ::core::ffi::c_int as isize);
        *fresh97 = (*fresh97).wrapping_add(1);
        if *fresh97 as ::core::ffi::c_int
            >= *(&raw mut game.g_ram as *mut uint8).offset(0x5bb as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x5bb as ::core::ffi::c_int as isize) =
                *(&raw mut game.g_ram as *mut uint8).offset(0x5ba as ::core::ffi::c_int as isize);
        }
    }
    let ref mut fresh98 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x5b8 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh98 = (*fresh98).wrapping_add(1);
}
pub unsafe fn CopyToVramNow(
    mut vram_dst: uint16,
    mut src: uint32,
    mut size: uint16,
    game: &mut Game,
) {
    WriteReg(INIDISP as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteRegWord(VMADDL as ::core::ffi::c_int as uint16, vram_dst, game);
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(A1T1L as ::core::ffi::c_int as uint16, src as uint16, game);
    WriteReg(
        A1B1 as ::core::ffi::c_int as uint16,
        (src >> 16 as ::core::ffi::c_int) as uint8,
        game,
    );
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, size, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteReg(INIDISP as ::core::ffi::c_int as uint16, 0xf as uint8, game);
}
pub unsafe fn IrqHandler_SetResult(mut a: uint16, mut y: uint16, mut x: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xab as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    WriteRegWord(VTIMEL as ::core::ffi::c_int as uint16, y, game);
    WriteRegWord(HTIMEL as ::core::ffi::c_int as uint16, x, game);
}
pub unsafe fn IrqHandler_0_Nothing(game: &mut Game) {
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(a, 0 as uint16, 0 as uint16, game);
}
pub unsafe fn IrqHandler_2_DisableIRQ(game: &mut Game) {
    let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh76 = (*fresh76 as ::core::ffi::c_int & !(0x30 as ::core::ffi::c_int)) as uint16;
    IrqHandler_SetResult(0 as uint16, 0 as uint16, 0 as uint16, game);
}
pub unsafe fn IrqHandler_4_Main_BeginHudDraw(game: &mut Game) {
    WriteReg(BG3SC as ::core::ffi::c_int as uint16, 0x5a as uint8, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(TM as ::core::ffi::c_int as uint16, 4 as uint8, game);
    IrqHandler_SetResult(6 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_6_Main_EndHudDraw(game: &mut Game) {
    WriteReg(
        CGWSEL as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x70 as ::core::ffi::c_int as isize),
        game,
    );
    WriteReg(
        CGADSUB as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x73 as ::core::ffi::c_int as isize),
        game,
    );
    WriteReg(
        BG3SC as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize),
        game,
    );
    WriteReg(
        TM as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x6a as ::core::ffi::c_int as isize),
        game,
    );
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(
        (if a as ::core::ffi::c_int != 0 {
            a as ::core::ffi::c_int
        } else {
            4 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_8_StartOfDoor_BeginHud(game: &mut Game) {
    WriteReg(BG3SC as ::core::ffi::c_int as uint16, 0x5a as uint8, game);
    WriteReg(TM as ::core::ffi::c_int as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    IrqHandler_SetResult(10 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_10_StartOfDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7b1 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x7b3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as ::core::ffi::c_int as uint16, v0, game);
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(
        (if a as ::core::ffi::c_int != 0 {
            a as ::core::ffi::c_int
        } else {
            8 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_12_Draygon_BeginHud(game: &mut Game) {
    WriteReg(TM as ::core::ffi::c_int as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    IrqHandler_SetResult(14 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_14_Draygon_EndHud(game: &mut Game) {
    WriteReg(
        BG3SC as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b as ::core::ffi::c_int as isize),
        game,
    );
    WriteReg(
        CGWSEL as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x70 as ::core::ffi::c_int as isize),
        game,
    );
    WriteReg(
        CGADSUB as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x73 as ::core::ffi::c_int as isize),
        game,
    );
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(
        (if a as ::core::ffi::c_int != 0 {
            a as ::core::ffi::c_int
        } else {
            12 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_16_VerticalDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as ::core::ffi::c_int as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    IrqHandler_SetResult(18 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_18_VerticalDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7b1 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x7b3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as ::core::ffi::c_int as uint16, v0, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x931 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(20 as uint16, 216 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_20_VerticalDoor_EndDraw(game: &mut Game) {
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(
        (if a as ::core::ffi::c_int != 0 {
            a as ::core::ffi::c_int
        } else {
            16 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_22_HorizDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as ::core::ffi::c_int as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    IrqHandler_SetResult(24 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_24_HorizDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7b1 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x7b3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as ::core::ffi::c_int as uint16, v0, game);
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0 as uint8, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x931 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(26 as uint16, 160 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_26_HorizDoor_EndDraw(game: &mut Game) {
    let mut a: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xa7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    IrqHandler_SetResult(
        (if a as ::core::ffi::c_int != 0 {
            a as ::core::ffi::c_int
        } else {
            22 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn EnableIrqInterrupts(game: &mut Game) {
    WriteRegWord(VTIMEL as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(HTIMEL as ::core::ffi::c_int as uint16, 152 as uint16, game);
    let ref mut fresh64 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh64 = (*fresh64 as ::core::ffi::c_int | 0x30 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EnableIrqInterruptsNow(game: &mut Game) {
    WriteRegWord(VTIMEL as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(HTIMEL as ::core::ffi::c_int as uint16, 152 as uint16, game);
    let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh65 = (*fresh65 as ::core::ffi::c_int | 0x30 as ::core::ffi::c_int) as uint16;
    WriteReg(
        NMITIMEN as ::core::ffi::c_int as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x84 as ::core::ffi::c_int as isize),
        game,
    );
}
pub unsafe fn DisableIrqInterrupts(game: &mut Game) {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x84 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh41 = (*fresh41 as ::core::ffi::c_int & !(0x30 as ::core::ffi::c_int)) as uint16;
}
static kIrqHandlers: [Func_V; 14] = [
    IrqHandler_0_Nothing,
    IrqHandler_2_DisableIRQ,
    IrqHandler_4_Main_BeginHudDraw,
    IrqHandler_6_Main_EndHudDraw,
    IrqHandler_8_StartOfDoor_BeginHud,
    IrqHandler_10_StartOfDoor_EndHud,
    IrqHandler_12_Draygon_BeginHud,
    IrqHandler_14_Draygon_EndHud,
    IrqHandler_16_VerticalDoor_BeginHud,
    IrqHandler_18_VerticalDoor_EndHud,
    IrqHandler_20_VerticalDoor_EndDraw,
    IrqHandler_22_HorizDoor_BeginHud,
    IrqHandler_24_HorizDoor_EndHud,
    IrqHandler_26_HorizDoor_EndDraw,
];
pub unsafe fn Vector_IRQ(game: &mut Game) {
    kIrqHandlers[(*((&raw mut game.g_ram as *mut uint8).offset(0xab as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize](game);
}
static kHudTilemaps_Row1to3: [uint16; 96] = [
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0x2c23 as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0x2c1e as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2822 as ::core::ffi::c_int as uint16,
    0x2822 as ::core::ffi::c_int as uint16,
    0x2823 as ::core::ffi::c_int as uint16,
    0x2813 as ::core::ffi::c_int as uint16,
    0x2c14 as ::core::ffi::c_int as uint16,
    0x2c1e as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0b as ::core::ffi::c_int as uint16,
    0x2c0c as ::core::ffi::c_int as uint16,
    0x2c0d as ::core::ffi::c_int as uint16,
    0x2c32 as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c09 as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c0f as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0x2c12 as ::core::ffi::c_int as uint16,
    0xa824 as ::core::ffi::c_int as uint16,
    0x2815 as ::core::ffi::c_int as uint16,
    0x2c16 as ::core::ffi::c_int as uint16,
    0x2c1e as ::core::ffi::c_int as uint16,
];
static kHudTilemaps_Missiles: [uint16; 22] = [
    0x344b as ::core::ffi::c_int as uint16,
    0x3449 as ::core::ffi::c_int as uint16,
    0x744b as ::core::ffi::c_int as uint16,
    0x344c as ::core::ffi::c_int as uint16,
    0x344a as ::core::ffi::c_int as uint16,
    0x744c as ::core::ffi::c_int as uint16,
    0x3434 as ::core::ffi::c_int as uint16,
    0x7434 as ::core::ffi::c_int as uint16,
    0x3435 as ::core::ffi::c_int as uint16,
    0x7435 as ::core::ffi::c_int as uint16,
    0x3436 as ::core::ffi::c_int as uint16,
    0x7436 as ::core::ffi::c_int as uint16,
    0x3437 as ::core::ffi::c_int as uint16,
    0x7437 as ::core::ffi::c_int as uint16,
    0x3438 as ::core::ffi::c_int as uint16,
    0x7438 as ::core::ffi::c_int as uint16,
    0x3439 as ::core::ffi::c_int as uint16,
    0x7439 as ::core::ffi::c_int as uint16,
    0x343a as ::core::ffi::c_int as uint16,
    0x743a as ::core::ffi::c_int as uint16,
    0x343b as ::core::ffi::c_int as uint16,
    0x743b as ::core::ffi::c_int as uint16,
];
pub unsafe fn AddMissilesToHudTilemap(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(10 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[0 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(11 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[1 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(12 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[2 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(42 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[3 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(43 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[4 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(44 as ::core::ffi::c_int as isize) =
            kHudTilemaps_Missiles[5 as ::core::ffi::c_int as usize];
    }
}
pub unsafe fn AddSuperMissilesToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x1c as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 12 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddPowerBombsToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x22 as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 20 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddGrappleToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x28 as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 28 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddXrayToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x2e as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 36 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddToTilemapInner(mut k: uint16, mut j: *const uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = *j.offset(0 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) =
            *j.offset(1 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 + 32 as ::core::ffi::c_int) as isize) =
            *j.offset(2 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 + 33 as ::core::ffi::c_int) as isize) =
            *j.offset(3 as ::core::ffi::c_int as isize);
    }
}
pub unsafe fn InitializeHud(game: &mut Game) {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605800 as ::core::ffi::c_int as uint16,
        game,
    );
    WriteRegWord(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint16, game);
    static unk_809A8F: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x80988b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x80988b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x40 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_809A8F as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 192 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            kHudTilemaps_Row1to3[(i >> 1 as ::core::ffi::c_int) as usize];
        i += 2 as ::core::ffi::c_int;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddXrayToHudTilemap(game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddGrappleToHudTilemap(game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        AddMissilesToHudTilemap(game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        AddSuperMissilesToHudTilemap(game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        AddPowerBombsToHudTilemap(game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa06 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa08 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa0a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa0c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa0e as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    InitializeMiniMapBroken();
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x94 as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x9c as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
                as *mut uint16),
            0xa2 as uint16,
            game,
        );
    }
    ToggleHudItemHighlight(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x1000 as uint16,
        game,
    );
    ToggleHudItemHighlight(
        *((&raw mut game.g_ram as *mut uint8).offset(0xa0e as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x1400 as uint16,
        game,
    );
    HandleHudTilemap(game);
}
static kEnergyTankIconTilemapOffsets: [uint16; 14] = [
    0x42 as ::core::ffi::c_int as uint16,
    0x44 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    0x48 as ::core::ffi::c_int as uint16,
    0x4a as ::core::ffi::c_int as uint16,
    0x4c as ::core::ffi::c_int as uint16,
    0x4e as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
    6 as ::core::ffi::c_int as uint16,
    8 as ::core::ffi::c_int as uint16,
    0xa as ::core::ffi::c_int as uint16,
    0xc as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
pub unsafe fn HandleHudTilemap(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        let mut v1: *const uint16 = RomPtr_80(
            addr_kHudTilemaps_AutoReserve as ::core::ffi::c_int as uint16_t,
            game,
        ) as *const uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            v1 = v1.offset(6 as ::core::ffi::c_int as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(8 as ::core::ffi::c_int as isize) =
            *v1.offset(0 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(9 as ::core::ffi::c_int as isize) =
            *v1.offset(1 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(40 as ::core::ffi::c_int as isize) =
            *v1.offset(2 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(41 as ::core::ffi::c_int as isize) =
            *v1.offset(3 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(72 as ::core::ffi::c_int as isize) =
            *v1.offset(4 as ::core::ffi::c_int as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(73 as ::core::ffi::c_int as isize) =
            *v1.offset(5 as ::core::ffi::c_int as isize);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0xa06 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa06 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut r20: uint16 = SnesDivide(
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16),
            100 as uint8,
        );
        let mut r18: uint16 = SnesModulus(
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16),
            100 as uint8,
        );
        let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = SnesDivide(
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            100 as uint8,
        ) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let mut v3: uint16 = 13360 as uint16;
            if r20 != 0 {
                r20 = r20.wrapping_sub(1);
                v3 = 10289 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    (kEnergyTankIconTilemapOffsets[(v2 >> 1 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) = v3;
            v2 += 2 as ::core::ffi::c_int;
            if !(((v2 - 28 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        DrawTwoHudDigits(
            addrl_kDigitTilesetsHealth as ::core::ffi::c_int as uint32,
            r18,
            0x8c as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0xa08 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa08 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x94 as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0xa0a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa0a as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5cf as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1f40 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            DrawThreeHudDigits(
                addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
                *((&raw mut game.g_ram as *mut uint8).offset(0xa0a as ::core::ffi::c_int as isize)
                    as *mut uint16),
                0x9c as uint16,
                game,
            );
        } else {
            DrawTwoHudDigits(
                addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
                *((&raw mut game.g_ram as *mut uint8).offset(0xa0a as ::core::ffi::c_int as isize)
                    as *mut uint16),
                0x9c as uint16,
                game,
            );
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0xa0c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa0c as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16);
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon as ::core::ffi::c_int as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
                as *mut uint16),
            0xa2 as uint16,
            game,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0xa0e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        ToggleHudItemHighlight(
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x1000 as uint16,
            game,
        );
        ToggleHudItemHighlight(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa0e as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x1400 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xa0e as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9d2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
            && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                != 20 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 0xc4f0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        {
            QueueSfx1_Max6(0x39 as uint16, game);
        }
    }
    let mut v4: uint16 = 5120 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0x5b5 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0x10 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v4 = 4096 as uint16;
    }
    ToggleHudItemHighlight(
        *((&raw mut game.g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
            as *mut uint16),
        v4,
        game,
    );
    let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 192 as uint16;
    v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size =
        ((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
    v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh71 = v5;
    v5 = v5.wrapping_add(1);
    (*gVramWriteEntry(fresh71 as ::core::ffi::c_int, game)).size = 126 as uint16;
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size =
        addr_unk_605820 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
static kHudItemTilemapOffsets: [uint16; 5] = [
    0x14 as ::core::ffi::c_int as uint16,
    0x1c as ::core::ffi::c_int as uint16,
    0x22 as ::core::ffi::c_int as uint16,
    0x28 as ::core::ffi::c_int as uint16,
    0x2e as ::core::ffi::c_int as uint16,
];
pub unsafe fn ToggleHudItemHighlight(mut a: uint16, mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
        as *mut uint16) = k;
    v2 = (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v3: ::core::ffi::c_int =
            kHudItemTilemapOffsets[v2 as usize] as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            != 11279 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc608 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v3 as isize) as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            != 11279 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc608 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v3 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 32 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            != 11279 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 32 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc608 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v3 + 32 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 33 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            != 11279 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 33 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc608 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v3 + 33 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if 2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int == 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != 11279 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v3 + 2 as ::core::ffi::c_int) as isize) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xc608 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset((v3 + 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xe3ff as ::core::ffi::c_int) as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 34 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != 11279 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v3 + 34 as ::core::ffi::c_int) as isize) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x77c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xc608 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset((v3 + 34 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xe3ff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn DrawThreeHudDigits(mut addr: uint32, mut a: uint16, mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = *(RomPtr(
        (addr as uint32_t).wrapping_add(
            (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int / 100 as ::core::ffi::c_int))
                as uint32_t,
        ),
        game,
    )
        as *mut uint16);
    DrawTwoHudDigits(
        addr,
        (a as ::core::ffi::c_int % 100 as ::core::ffi::c_int) as uint16,
        (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DrawTwoHudDigits(mut addr: uint32, mut a: uint16, mut k: uint16, game: &mut Game) {
    let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *(RomPtr(
        (addr as uint32_t).wrapping_add(
            (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int / 10 as ::core::ffi::c_int))
                as uint32_t,
        ),
        game,
    ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize) = *(RomPtr(
        (addr as uint32_t).wrapping_add(
            (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int % 10 as ::core::ffi::c_int))
                as uint32_t,
        ),
        game,
    ) as *mut uint16);
}
static kTimerProcessFuncs: [Option<Func_U8>; 7] = unsafe {
    [
        Some(ProcessTimer_Empty as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_CeresStart as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MotherBrainStart as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_InitialDelay as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MovementDelayed as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MovingIntoPlace as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_Decrement as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn ProcessTimer(game: &mut Game) -> uint8 {
    return kTimerProcessFuncs[*((&raw mut game.g_ram as *mut uint8)
        .offset(0x943 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn ProcessTimer_CeresStart(game: &mut Game) -> uint8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(32765 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn ProcessTimer_Empty(game: &mut Game) -> uint8 {
    return 0 as uint8;
}
pub unsafe fn ProcessTimer_MotherBrainStart(game: &mut Game) -> uint8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x300 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(32765 as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn ProcessTimer_InitialDelay(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        >= 0x10 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8;
    }
    return 0 as uint8;
}
pub unsafe fn ProcessTimer_MovementDelayed(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        >= 0x60 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8;
    }
    return ProcessTimer_Decrement(game);
}
pub unsafe fn ProcessTimer_MovingIntoPlace(game: &mut Game) -> uint8 {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x948 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 224 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 224 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >= 0xdc00 as ::core::ffi::c_int
    {
        v0 = 1 as ::core::ffi::c_int;
        v1 = -(9216 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x94a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 193 as ::core::ffi::c_int) as uint16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x94a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 193 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        < 0x3000 as ::core::ffi::c_int
    {
        v0 += 1;
        v2 = 12288 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x94a as ::core::ffi::c_int as isize)
        as *mut uint16) = v2;
    if v0 == 2 as ::core::ffi::c_int {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x943 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh18 = (*fresh18).wrapping_add(1);
    }
    return ProcessTimer_Decrement(game);
}
pub unsafe fn SetTimerMinutes(mut a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
}
pub unsafe fn ClearTimerRam(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0x8000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x94a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0x8000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn DecrementDecimal(mut number: *mut uint8, mut value: uint8) -> bool {
    let mut result: ::core::ffi::c_int = (*number as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        + (!(value as ::core::ffi::c_int) & 0xf as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int;
    if result < 0x10 as ::core::ffi::c_int {
        result = result - 0x6 as ::core::ffi::c_int
            & (if (result - 0x6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                0xf as ::core::ffi::c_int
            } else {
                0x1f as ::core::ffi::c_int
            });
    }
    result = (*number as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
        + (!(value as ::core::ffi::c_int) & 0xf0 as ::core::ffi::c_int)
        + result;
    if result < 0x100 as ::core::ffi::c_int {
        result -= 0x60 as ::core::ffi::c_int;
    }
    *number = result as uint8;
    return result < 0x100 as ::core::ffi::c_int;
}
pub unsafe fn ProcessTimer_Decrement(game: &mut Game) -> uint8 {
    static kTimerCentisecondDecrements: [uint8; 128] = [
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
    ];
    if DecrementDecimal(
        (&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize),
        kTimerCentisecondDecrements[(*((&raw mut game.g_ram as *mut uint8)
            .offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int) as usize],
    ) {
        if DecrementDecimal(
            (&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize),
            1 as uint8,
        ) {
            if DecrementDecimal(
                (&raw mut game.g_ram as *mut uint8).offset(0x947 as ::core::ffi::c_int as isize),
                1 as uint8,
            ) {
                *(&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize) =
                    0 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize) =
                    0 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x947 as ::core::ffi::c_int as isize) =
                    0 as uint8;
            } else {
                *(&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize) =
                    0x59 as uint8;
            }
        }
    }
    return (*(&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x947 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn DrawTimer(game: &mut Game) {
    DrawTimerSpritemap(
        0 as uint16,
        addr_word_80A060 as ::core::ffi::c_int as uint16,
        game,
    );
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x947 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0xffe4 as uint16,
        game,
    );
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x946 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0xfffc as uint16,
        game,
    );
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x945 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x14 as uint16,
        game,
    );
}
pub unsafe fn DrawTwoTimerDigits(mut a: uint16, mut k: uint16, game: &mut Game) {
    DrawTimerSpritemap(
        k,
        *(game.g_rom.offset(
            (((0x809fd4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x809fd4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int) >> 4 as ::core::ffi::c_int)
                    as isize,
            ),
        game,
    );
    DrawTimerSpritemap(
        (k as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16,
        *(game.g_rom.offset(
            (((0x809fd4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x809fd4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as isize),
        game,
    );
}
pub unsafe fn DrawTimerSpritemap(mut a: uint16, mut j: uint16, game: &mut Game) {
    DrawSpritemap(
        0x80 as uint8,
        j,
        (a as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x948 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x94a as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
        2560 as uint16,
        game,
    );
}
pub unsafe fn StartGameplay_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut game.g_ram as *mut uint8).offset(0x613 as ::core::ffi::c_int as isize);
    let mut current_block_54: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        0 => {
            WriteRegWord(MDMAEN as ::core::ffi::c_int as uint16, 0 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7f3 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7f5 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ResetSoundQueues(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x5f5 as ::core::ffi::c_int as isize)
                as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
            DisableNMI(game);
            DisableIrqInterrupts(game);
            LoadDestinationRoomThings(game);
            current_block_54 = 9431711618385729511;
        }
        1 => {
            current_block_54 = 9431711618385729511;
        }
        2 => {
            current_block_54 = 4121473168792104488;
        }
        3 => {
            current_block_54 = 3421737773041484705;
        }
        _ => {
            current_block_54 = 7659304154607701039;
        }
    }
    match current_block_54 {
        9431711618385729511 => {
            let mut _coret_: uint8 = Play20FramesOfMusic_Async(game) as uint8;
            if _coret_ != 0 {
                *_state_variable_ = 1 as uint8;
                return _coret_ as CoroutineRet;
            }
            ClearAnimtiles(game);
            WaitUntilEndOfVblankAndClearHdma(game);
            InitializeSpecialEffectsForNewRoom(game);
            ClearPLMs(game);
            ClearEprojs(game);
            ClearPaletteFXObjects(game);
            UpdateBeamTilesAndPalette(game);
            LoadColorsForSpritesBeamsAndEnemies(game);
            LoadEnemies(game);
            LoadRoomMusic(game);
            current_block_54 = 4121473168792104488;
        }
        _ => {}
    }
    match current_block_54 {
        4121473168792104488 => {
            let mut _coret__0: uint8 = Play20FramesOfMusic_Async(game) as uint8;
            if _coret__0 != 0 {
                *_state_variable_ = 2 as uint8;
                return _coret__0 as CoroutineRet;
            }
            UpdateMusicTrackIndex(game);
            NullFunc();
            ClearBG2Tilemap(game);
            LoadLevelDataAndOtherThings(game);
            LoadFXHeader(game);
            LoadLibraryBackground(game);
            CalculateLayer2Xpos(game);
            CalculateLayer2Ypos(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x921 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x917 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x923 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x919 as ::core::ffi::c_int as isize)
                as *mut uint16);
            CalculateBgScrolls(game);
            DisplayViewablePartOfRoom(game);
            EnableNMI(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
                as *mut uint16) = (if *((&raw mut game.g_ram as *mut uint8)
                .offset(0xa9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                4 as ::core::ffi::c_int
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa9 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            }) as uint16;
            EnableIrqInterrupts(game);
            current_block_54 = 3421737773041484705;
        }
        _ => {}
    }
    match current_block_54 {
        3421737773041484705 => {
            let mut _coret__1: uint8 = Play20FramesOfMusic_Async(game) as uint8;
            if _coret__1 != 0 {
                *_state_variable_ = 3 as uint8;
                return _coret__1 as CoroutineRet;
            }
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0x8 as uint8,
                    field_1: 0x8 as uint8,
                    field_2: 0xb7eb as VoidP,
                },
                game,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x99c as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x82e737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn Play20FramesOfMusic_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 =
        (&raw mut game.g_ram as *mut uint8).offset(0x614 as ::core::ffi::c_int as isize);
    's_57: {
        let mut current_block_9: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                EnableNMI(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x77e as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                current_block_9 = 8258075665625361029;
            }
            1 => {
                current_block_9 = 4436247046726756244;
            }
            _ => {
                current_block_9 = 13536709405535804910;
            }
        }
        loop {
            match current_block_9 {
                13536709405535804910 => {
                    break 's_57;
                }
                8258075665625361029 => {
                    if *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 20 as ::core::ffi::c_int
                    {
                        HandleMusicQueue(game);
                        current_block_9 = 4436247046726756244;
                    } else {
                        DisableNMI(game);
                        current_block_9 = 13536709405535804910;
                    }
                }
                _ => {
                    let mut _coret_: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x77e as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh79 = (*fresh79).wrapping_add(1);
                    current_block_9 = 8258075665625361029;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn ResumeGameplay(game: &mut Game) {
    WriteRegWord(MDMAEN as ::core::ffi::c_int as uint16, 0 as uint16, game);
    DisableNMI(game);
    DisableIrqInterrupts(game);
    LoadCRETilesTilesetTilesAndPalette(game);
    LoadLibraryBackground(game);
    DisplayViewablePartOfRoom(game);
    LoadRoomPlmGfx(game);
    EnableNMI(game);
    EnableIrqInterrupts(game);
}
pub unsafe fn DisplayViewablePartOfRoom(game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut v2: ::core::ffi::c_int = 0;
    let mut v3: ::core::ffi::c_int = 0;
    v1 = (*(&raw mut game.g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        - *(&raw mut game.g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x98e as ::core::ffi::c_int as isize)
        as *mut uint16) = (v1 & 0xf800 as ::core::ffi::c_int) as uint16;
    CalculateBgScrollAndLayerPositionBlocks(game);
    v2 = 0 as ::core::ffi::c_int;
    loop {
        v3 = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f9 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x907 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x909 as ::core::ffi::c_int as isize)
            as *mut uint16);
        UploadLevelDataColumn(game);
        if *(&raw mut game.g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fb as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fd as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x90b as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x90d as ::core::ffi::c_int as isize)
                as *mut uint16);
            UploadBackgroundDataColumn(game);
        }
        NMI_ProcessVramWriteQueue(game);
        let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh42 = (*fresh42).wrapping_add(1);
        let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x907 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh43 = (*fresh43).wrapping_add(1);
        let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8fb as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh44 = (*fresh44).wrapping_add(1);
        let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x90b as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh45 = (*fresh45).wrapping_add(1);
        v2 += 1;
        if !(v3 != 16 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn QueueClearingOfFxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 3838 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000))
            .xray_tilemaps as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 6222 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 3840 as uint16;
    (*v2).src.addr =
        ((&raw mut game.g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = addr_unk_605880 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ClearBG2Tilemap(game: &mut Game) {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        0x4800 as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1808 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        0xa29a as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x800 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        0x4800 as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1908 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        0xa29a as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x800 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
pub unsafe fn ClearFXTilemap(game: &mut Game) {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        0x5880 as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1808 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        0xa2f7 as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x780 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        0x5880 as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1908 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        0xa2f8 as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x780 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
pub unsafe fn CalculateLayer2Xpos(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        let mut t: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x91b as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfe as ::core::ffi::c_int;
        let mut var933: uint8 =
            (t * *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
            as *mut uint16) = (t * *((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + var933 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    }
    return 1 as uint8;
}
pub unsafe fn CalculateLayer2Ypos(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c as ::core::ffi::c_int as isize) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        let mut t: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x91c as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfe as ::core::ffi::c_int;
        let mut var933: uint8 = (t * *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) = (t * *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + var933 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    }
    return 1 as uint8;
}
pub unsafe fn CalculateBgScrolls(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x91d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x91f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x921 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x923 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn UpdateScrollVarsUpdateMap(game: &mut Game) {
    CalculateBgScrolls(game);
    UpdateBgGraphicsWhenScrolling(game);
}
pub unsafe fn CalculateLayer2PosAndScrollsWhenScrolling(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x91d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x91f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        if CalculateLayer2Xpos(game) == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x921 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
        if CalculateLayer2Ypos(game) == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x923 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
        UpdateBgGraphicsWhenScrolling(game);
    }
}
pub unsafe fn UpdateBgGraphicsWhenScrolling(game: &mut Game) {
    CalculateBgScrollAndLayerPositionBlocks(game);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut v1: bool =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x8ff as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f7 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x8ff as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x8ff as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if !v1 {
            v0 = 16 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v0) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x907 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v0) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f9 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x909 as ::core::ffi::c_int as isize)
            as *mut uint16);
        UploadLevelDataColumn(game);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut v3: bool = ((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x8fb as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x903 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8fb as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x903 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x903 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fb as ::core::ffi::c_int as isize)
                as *mut uint16);
            if !v3 {
                v2 = 16 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fb as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + v2) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x90b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + v2) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fd as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x90d as ::core::ffi::c_int as isize)
                as *mut uint16);
            UploadBackgroundDataColumn(game);
        }
    }
    let mut v4: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut v5: bool =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x8f9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x901 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f9 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x901 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x901 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f9 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if !v5 {
            v4 = 15 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v4) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x909 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v4) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x8f7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x907 as ::core::ffi::c_int as isize)
            as *mut uint16);
        UpdateLevelDataRow(game);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        let mut v6: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut v7: bool = ((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x8fd as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x905 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8fd as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x905 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x905 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fd as ::core::ffi::c_int as isize)
                as *mut uint16);
            if !v7 {
                v6 = 15 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fd as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + v6) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x90d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + v6) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x990 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x8fb as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x994 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x90b as ::core::ffi::c_int as isize)
                as *mut uint16);
            UpdateBackgroundDataRow(game);
        }
    }
}
pub unsafe fn CalculateBgScrollAndLayerPositionBlocks(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x907 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x90b as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = (v0 as ::core::ffi::c_int | 0xf000 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x8f7 as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v1 = (v1 as ::core::ffi::c_int | 0xf000 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x8fb as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0x909 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb3 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x90d as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int | 0xf000 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x8f9 as ::core::ffi::c_int as isize)
        as *mut uint16) = v2;
    let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x919 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v3 = (v3 as ::core::ffi::c_int | 0xf000 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x8fd as ::core::ffi::c_int as isize)
        as *mut uint16) = v3;
}
pub unsafe fn HandleAutoscrolling_X(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: int16 = 0;
    let mut v6: int16 = 0;
    let mut v4: uint16 = 0;
    let mut var933: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        let mut v0: uint16 = swap16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16,
        );
        if (v0 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) = v0;
        }
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int16;
        let mut v2: uint16 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + v1 as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(v2 as isize)
            != 0
        {
            if *(&raw mut game.g_ram as *mut uint8)
                .offset(0xcd20 as ::core::ffi::c_int as isize)
                .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16 as isize)
                != 0
            {
                return;
            }
            var933 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int) as uint16;
            let mut v5: uint16 = (var939 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16;
            if ((v5 as ::core::ffi::c_int
                - (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v4 = var933;
            } else {
                var939 = v5;
                v6 = *(&raw mut var939 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int16;
                if *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20 as ::core::ffi::c_int as isize)
                    .offset(
                        (Mult8x8(
                            ((*((&raw mut game.g_ram as *mut uint8)
                                .offset(0x915 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                + 128 as ::core::ffi::c_int) as uint16
                                as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int) as uint8,
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x7a9 as ::core::ffi::c_int as isize)
                                as *mut uint16) as uint8,
                        ) as ::core::ffi::c_int
                            + v6 as ::core::ffi::c_int) as uint16 as isize,
                    )
                    != 0
                {
                    v4 = var939;
                } else {
                    v4 = ((var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                        + 256 as ::core::ffi::c_int) as uint16;
                }
            }
        } else {
            var933 = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >= var933 as ::core::ffi::c_int
            {
                v4 = var933;
            } else {
                var939 = (var939 as ::core::ffi::c_int
                    + (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xda2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int)) as uint16;
                v3 = (*(&raw mut var939 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint8 as int16;
                if *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20 as ::core::ffi::c_int as isize)
                    .offset(
                        (Mult8x8(
                            ((*((&raw mut game.g_ram as *mut uint8)
                                .offset(0x915 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                + 128 as ::core::ffi::c_int) as uint16
                                as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int) as uint8,
                            *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x7a9 as ::core::ffi::c_int as isize)
                                as *mut uint16) as uint8,
                        ) as ::core::ffi::c_int
                            + v3 as ::core::ffi::c_int) as uint16 as isize,
                    )
                    != 0
                {
                    v4 = var939;
                } else {
                    v4 = (var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) as uint16;
                }
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = v4;
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollRight(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x90f as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut v0: uint16 = swap16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16,
    );
    if v0 as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int16;
        let mut RegWord: uint16 = Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(
                (RegWord as ::core::ffi::c_int + 1 as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
                    as uint16 as isize,
            )
            == 0
        {
            let mut var933: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int) as uint16;
            let mut v4: uint16 = (var939 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16;
            if ((v4 as ::core::ffi::c_int
                - (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v4 = var933;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) = v4;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
    };
}
pub unsafe fn HandleScrollingWhenTriggeringScrollLeft(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x90f as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int16;
        let mut prod: uint16 = Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset((prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16 as isize)
            == 0
        {
            let mut var933: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xda2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int >= var933 as ::core::ffi::c_int {
                v1 = var933;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) = v1;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn HandleAutoscrolling_Y(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v10: int16 = 0;
    let mut v8: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let mut v0: uint16 = 0 as uint16;
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        let mut r20: uint16 = (Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + v1 as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(r20 as isize) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        {
            v0 = 31 as uint16;
        }
        let mut var933: uint16 = v0;
        let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        *(&raw mut v2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8;
        *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = (*((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint8;
        let mut v3: uint16 = (var933 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
        if (v3 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) = v3;
        }
        v4 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        let mut v5: uint16 = (Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + v4 as ::core::ffi::c_int) as uint16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(v5 as isize)
            == 0
        {
            let mut var935: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v6: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xda6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v6 as ::core::ffi::c_int >= var935 as ::core::ffi::c_int {
                v8 = var935;
            } else {
                var939 = (var939 as ::core::ffi::c_int
                    + (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xda6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int)) as uint16;
                v7 = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 128 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as int16;
                let mut prod: uint16 = Mult8x8(
                    (*(&raw mut v6 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as uint8,
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x7a9 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint8,
                );
                if *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20 as ::core::ffi::c_int as isize)
                    .offset(
                        (prod as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint16 as isize,
                    )
                    != 0
                {
                    v8 = var939;
                } else {
                    v8 = (var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) as uint16;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) = v8;
            return;
        }
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + v5 as ::core::ffi::c_int) as uint16 as isize,
            )
            == 0
        {
            let mut var937: uint16 = (var933 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as uint16;
            if (var937 as ::core::ffi::c_int)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                let mut v9: uint16 = (var939 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xda6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int) as uint16;
                if ((v9 as ::core::ffi::c_int - var937 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v8 = var937;
                } else {
                    var939 = v9;
                    let mut prod_0: uint16 = Mult8x8(
                        *(&raw mut v9 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ),
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x7a9 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint8,
                    );
                    v10 = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        + 128 as ::core::ffi::c_int) as uint16
                        as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int) as int16;
                    if *(&raw mut game.g_ram as *mut uint8)
                        .offset(0xcd20 as ::core::ffi::c_int as isize)
                        .offset(
                            (prod_0 as ::core::ffi::c_int + v10 as ::core::ffi::c_int) as uint16
                                as isize,
                        )
                        != 0
                    {
                        v8 = var939;
                    } else {
                        v8 = ((var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                            + 256 as ::core::ffi::c_int) as uint16;
                    }
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v8;
                return;
            }
        }
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollDown(game: &mut Game) {
    let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prod: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v1: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = (prod as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(r20 as isize) as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        v0 = 31 as ::core::ffi::c_int;
    }
    let mut var933: uint16 = v0 as uint16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb0e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x913 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut v2: uint16 = swap16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7ab as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16,
    );
    let mut var937: uint16 = (var933 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
    if ((var933 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + r20 as ::core::ffi::c_int) as uint16 as isize,
            )
            == 0
            && {
                var937 = (var933 as ::core::ffi::c_int
                    + (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int)) as uint16;
                (var937 as ::core::ffi::c_int)
                    < *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
            }
    {
        let mut v3: uint16 = (var939 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xda6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16;
        if ((v3 as ::core::ffi::c_int - var937 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v3 = var937;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = v3;
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollUp(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut var939: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb0e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x913 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut prod: uint16 = Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a9 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        v0 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset((prod as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16 as isize)
            == 0
        {
            let mut var933: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xda6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int >= var933 as ::core::ffi::c_int {
                v1 = var933;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) = v1;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn DebugScrollPosSaveLoad(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x40 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5d3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh22 = (*fresh22).wrapping_add(1);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d3 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5d5 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5d7 as ::core::ffi::c_int as isize)
            as *mut uint16);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5d5 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x5d7 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
    };
}
pub unsafe fn UploadBackgroundDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0x1c as uint16, game);
}
pub unsafe fn UploadLevelDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0 as uint16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataColumn(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return;
    }
    let mut prod: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x990 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    if k != 0 {
        v2 = (v2 as ::core::ffi::c_int + 0x9600 as ::core::ffi::c_int) as uint16;
    }
    let mut r54: *const uint16 = (&raw mut game.g_ram as *mut uint8)
        .offset(0x10000 as ::core::ffi::c_int as isize)
        .offset(v2 as ::core::ffi::c_int as isize)
        as *const uint16;
    let mut v3: uint16 = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x3c as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x958 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = v3;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x956 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
        ((v3 as ::core::ffi::c_int ^ 0x3f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as uint16;
    prod = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint8,
        0x40 as uint8,
    );
    let mut var935: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x994 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = (2 as ::core::ffi::c_int * var935 as ::core::ffi::c_int) as uint16;
    let mut var933: uint16 = (prod as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as uint16;
    let mut v5: uint16 = addr_unk_605000 as ::core::ffi::c_int as uint16;
    if var935 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        v5 = addr_unk_6053E0 as ::core::ffi::c_int as uint16;
    }
    if k != 0 {
        v5 = (v5 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x98e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    }
    let mut var937: uint16 = v5;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95a as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
        (var933 as ::core::ffi::c_int + v5 as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95c as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (var935 as ::core::ffi::c_int
        + var935 as ::core::ffi::c_int
        + var937 as ::core::ffi::c_int)
        as uint16;
    let mut v6: uint16 = ((&raw mut game.g_ram as *mut uint8)
        .offset(0xc8c8 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset_from(&raw mut game.g_ram as *mut uint8)
        as ::core::ffi::c_long as uint16;
    let mut v7: uint16 = 0 as uint16;
    if k != 0 {
        v6 = (&raw mut game.g_ram as *mut uint8)
            .offset(0xc9d0 as ::core::ffi::c_int as isize)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
        v7 = 264 as uint16;
    }
    let mut v8: uint16 = (*(((&raw mut game.g_ram as *mut uint8)
        .offset(0x956 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v6 as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95e as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = v8;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x960 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
        (v8 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    var937 = v7;
    let mut t2: uint16 = k;
    let mut v9: uint16 = 0 as uint16;
    let mut var939: uint16 = 16 as uint16;
    loop {
        let mut var93B: uint16 =
            *r54.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v10: uint16 =
            (var93B as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16;
        let mut v17: uint16 = v9;
        let mut v11: uint16 = var937;
        let mut v12: uint16 =
            (var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) as uint16;
        if var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if v12 as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
                let mut v14: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_right as ::core::ffi::c_int
                    ^ 0x4000 as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_left as ::core::ffi::c_int
                    ^ 0x4000 as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_right as ::core::ffi::c_int
                        ^ 0x4000 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left as ::core::ffi::c_int
                        ^ 0x4000 as ::core::ffi::c_int) as uint16;
            } else {
                let mut v15: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                let mut v16: uint16 = 0;
                if v12 as ::core::ffi::c_int == 2048 as ::core::ffi::c_int {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc8c8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc908 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc8c8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    v16 = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_right as ::core::ffi::c_int
                        ^ 0x8000 as ::core::ffi::c_int) as uint16;
                } else {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc8c8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc908 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc8c8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    v16 = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left as ::core::ffi::c_int
                        ^ 0xc000 as ::core::ffi::c_int) as uint16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) = v16;
            }
        } else {
            let mut v13: uint16 =
                (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa000 as ::core::ffi::c_int as isize)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc908 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa000 as ::core::ffi::c_int as isize)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_right;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .bottom_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc908 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .bottom_right;
        }
        var937 = (v11 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        v9 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            * 2 as ::core::ffi::c_int
            + v17 as ::core::ffi::c_int) as uint16;
        var939 = var939.wrapping_sub(1);
        if !(var939 != 0) {
            break;
        }
    }
    let ref mut fresh7 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x962 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(t2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh7 = (*fresh7).wrapping_add(1);
}
pub unsafe fn UpdateBackgroundDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0x1c as uint16, game);
}
pub unsafe fn UpdateLevelDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0 as uint16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataRow(mut v0: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return;
    }
    let mut prod: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x992 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x990 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    if v0 != 0 {
        v2 = (v2 as ::core::ffi::c_int - 27136 as ::core::ffi::c_int) as uint16;
    }
    let mut r54: *const uint16 = (&raw mut game.g_ram as *mut uint8)
        .offset(0x10000 as ::core::ffi::c_int as isize)
        .offset(v2 as ::core::ffi::c_int as isize)
        as *const uint16;
    let mut var933: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x994 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x964 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (4 as ::core::ffi::c_int
        * (16 as ::core::ffi::c_int - var933 as ::core::ffi::c_int))
        as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x966 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (4 as ::core::ffi::c_int
        * (var933 as ::core::ffi::c_int + 1 as ::core::ffi::c_int))
        as uint16;
    prod = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x996 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint8,
        0x40 as uint8,
    );
    let mut var935: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x994 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = (2 as ::core::ffi::c_int * var935 as ::core::ffi::c_int) as uint16;
    var933 = (prod as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
    let mut var937: uint16 = addr_unk_605400 as ::core::ffi::c_int as uint16;
    let mut v4: uint16 = addr_unk_605000 as ::core::ffi::c_int as uint16;
    if var935 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        var937 = addr_unk_605000 as ::core::ffi::c_int as uint16;
        v4 = addr_unk_6053E0 as ::core::ffi::c_int as uint16;
    }
    if v0 != 0 {
        v4 = (v4 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x98e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x968 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
        (var933 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as uint16;
    let mut v5: uint16 = var937;
    if v0 != 0 {
        v5 = (var937 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x98e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96a as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
        (prod as ::core::ffi::c_int + v5 as ::core::ffi::c_int) as uint16;
    let mut v6: uint16 = ((&raw mut game.g_ram as *mut uint8)
        .offset(0xc948 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset_from(&raw mut game.g_ram as *mut uint8)
        as ::core::ffi::c_long as uint16;
    let mut v7: uint16 = 0 as uint16;
    if v0 != 0 {
        v6 = (&raw mut game.g_ram as *mut uint8)
            .offset(0xca50 as ::core::ffi::c_int as isize)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
        v7 = 264 as uint16;
    }
    let mut v8: uint16 = (*(((&raw mut game.g_ram as *mut uint8)
        .offset(0x964 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v6 as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96c as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v8;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96e as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v8 as ::core::ffi::c_int + 68 as ::core::ffi::c_int) as uint16;
    var937 = v7;
    let mut t2: uint16 = v0;
    let mut v9: uint16 = 0 as uint16;
    let mut var939: uint16 = 17 as uint16;
    loop {
        let mut var93B: uint16 =
            *r54.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v10: uint16 =
            (var93B as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int) as uint16;
        let mut v17: uint16 = v9;
        let mut v11: uint16 = var937;
        let mut v12: uint16 =
            (var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) as uint16;
        if var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if v12 as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
                let mut v14: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc948 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_right as ::core::ffi::c_int
                    ^ 0x4000 as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc948 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left as ::core::ffi::c_int
                        ^ 0x4000 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .bottom_right
                    as ::core::ffi::c_int
                    ^ 0x4000 as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left as ::core::ffi::c_int
                        ^ 0x4000 as ::core::ffi::c_int) as uint16;
            } else {
                let mut v15: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                let mut v16: uint16 = 0;
                if v12 as ::core::ffi::c_int == 2048 as ::core::ffi::c_int {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc948 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc948 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc98c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_left as ::core::ffi::c_int
                            ^ 0x8000 as ::core::ffi::c_int) as uint16;
                    v16 = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_right as ::core::ffi::c_int
                        ^ 0x8000 as ::core::ffi::c_int) as uint16;
                } else {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc948 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc948 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_left as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc98c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v15 as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xa000 as ::core::ffi::c_int as isize)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_right as ::core::ffi::c_int
                            ^ 0xc000 as ::core::ffi::c_int) as uint16;
                    v16 = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000 as ::core::ffi::c_int as isize)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left as ::core::ffi::c_int
                        ^ 0xc000 as ::core::ffi::c_int) as uint16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) = v16;
            }
        } else {
            let mut v13: uint16 =
                (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc948 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa000 as ::core::ffi::c_int as isize)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc948 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_right;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc98c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xa000 as ::core::ffi::c_int as isize)
                as *mut TileTables))
                .tables[v10 as usize]
                .bottom_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc98c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000 as ::core::ffi::c_int as isize)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .bottom_right;
        }
        var937 = (v11 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        v9 = (v17 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        var939 = var939.wrapping_sub(1);
        if !(var939 != 0) {
            break;
        }
    }
    let ref mut fresh6 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x970 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(t2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh6 = (*fresh6).wrapping_add(1);
}
pub unsafe fn FixDoorsMovingUp(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x901 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh66 = (*fresh66).wrapping_add(1);
    let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x905 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh67 = (*fresh67).wrapping_add(1);
    DoorTransition_Up(game);
}
static kDoorTransitionSetupFuncs: [Option<Func_V>; 4] = unsafe {
    [
        Some(DoorTransitionScrollingSetup_Right as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Left as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Down as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Up as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn DoorTransitionScrollingSetup(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x927 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x929 as ::core::ffi::c_int as isize) as *mut uint16);
    kDoorTransitionSetupFuncs[(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x791 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Right(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh60 = (*fresh60 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16;
    CalculateLayer2Ypos(game);
    let ref mut fresh61 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh61 = (*fresh61 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16;
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8ff as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh62 = (*fresh62).wrapping_sub(1);
    let ref mut fresh63 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x903 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh63 = (*fresh63).wrapping_sub(1);
    DoorTransition_Right(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Left(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh56 = (*fresh56 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    CalculateLayer2Ypos(game);
    let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh57 = (*fresh57 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8ff as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh58 = (*fresh58).wrapping_add(1);
    let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x903 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh59 = (*fresh59).wrapping_add(1);
    DoorTransition_Left(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Down(game: &mut Game) {
    CalculateLayer2Xpos(game);
    CalculateLayer2Ypos(game);
    let ref mut fresh52 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x919 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh52 = (*fresh52 as ::core::ffi::c_int - 224 as ::core::ffi::c_int) as uint16;
    let ref mut fresh53 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh53 = (*fresh53 as ::core::ffi::c_int - 224 as ::core::ffi::c_int) as uint16;
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh54 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x901 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh54 = (*fresh54).wrapping_sub(1);
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x905 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_sub(1);
    DoorTransition_Down(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Up(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16);
    let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh46 = (*fresh46 as ::core::ffi::c_int + 31 as ::core::ffi::c_int) as uint16;
    CalculateLayer2Ypos(game);
    let ref mut fresh47 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x919 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh47 = (*fresh47 as ::core::ffi::c_int + 224 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    UpdateBgScrollOffsets(game);
    let ref mut fresh48 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x929 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh48 = (*fresh48 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x901 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh49 = (*fresh49).wrapping_add(1);
    let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x905 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh50 = (*fresh50).wrapping_add(1);
    let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh51 = (*fresh51).wrapping_sub(1);
    DoorTransition_Up(game);
}
pub unsafe fn UpdatePreviousLayerBlocks(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x8ff as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x903 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8fb as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x901 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8f9 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x905 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x8fd as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn UpdateBgScrollOffsets(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x91d as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x91f as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb3 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x921 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x923 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
}
static kDoorTransitionFuncs: [Option<Func_U8>; 4] = unsafe {
    [
        Some(DoorTransition_Right as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Left as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Down as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Up as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Irq_FollowDoorTransition(game: &mut Game) {
    if kDoorTransitionFuncs[(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x791 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x927 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x929 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x931 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh68 = (*fresh68 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DoorTransition_Right(game: &mut Game) -> uint8 {
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x925 as ::core::ffi::c_int as isize) as *mut uint16);
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x92d as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x92b as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16);
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh10 = (*fresh10 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh11 = (*fresh11 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    UpdateScrollVarsUpdateMap(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if v2 as ::core::ffi::c_int != 63 as ::core::ffi::c_int {
        return 0 as uint8;
    }
    UpdateScrollVarsUpdateMap(game);
    return 1 as uint8;
}
pub unsafe fn DoorTransition_Left(game: &mut Game) -> uint8 {
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x925 as ::core::ffi::c_int as isize) as *mut uint16);
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
            as *mut uint16,
        -(((*((&raw mut game.g_ram as *mut uint8).offset(0x92d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x92b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16);
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x917 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    UpdateScrollVarsUpdateMap(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    return (v2 as ::core::ffi::c_int == 63 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn DoorTransition_Down(game: &mut Game) -> uint8 {
    let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x925 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            < 0x39 as ::core::ffi::c_int
        {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x92d as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x92b as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh0 = (*fresh0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x919 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh1 = (*fresh1 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            UpdateScrollVarsUpdateMap(game);
        }
    } else {
        let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh2 = (*fresh2 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as uint16;
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh3 = (*fresh3 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as uint16;
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x901 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_sub(1);
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x905 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh5 = (*fresh5).wrapping_sub(1);
        UpdateScrollVarsUpdateMap(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) = v5;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if ((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        < 0x39 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    UpdateScrollVarsUpdateMap(game);
    return 1 as uint8;
}
pub unsafe fn DoorTransition_Up(game: &mut Game) -> uint8 {
    let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x925 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
                as *mut uint16,
            -(((*((&raw mut game.g_ram as *mut uint8).offset(0x92d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x92b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh12 = (*fresh12 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh13 = (*fresh13 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >= 5 as ::core::ffi::c_int
        {
            UpdateScrollVarsUpdateMap(game);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x91d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x91f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x921 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x917 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x923 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        }
    } else {
        let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh14 = (*fresh14 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x901 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh16 = (*fresh16).wrapping_add(1);
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x905 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh17 = (*fresh17).wrapping_add(1);
        UpdateScrollVarsUpdateMap(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x919 as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) = v5;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x925 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    return (v6 as ::core::ffi::c_int == 56 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn ConfigureMode7RotationMatrix(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a as ::core::ffi::c_int as isize)
                as *mut uint16) = kSinCosTable8bit_Sext[(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x785 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int)
                as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7c as ::core::ffi::c_int as isize)
                as *mut uint16) = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x7a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x78 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                kSinCosTable8bit_Sext[((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x785 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int) as uint8
                    as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int) as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x78 as ::core::ffi::c_int as isize)
                as *mut uint16);
            let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x785 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh21 = (*fresh21).wrapping_add(1);
        }
    }
}

unsafe fn DecompNextByte(game: &mut Game) -> uint8 {
    let mut b: uint8 = *RomPtr(game.decompress_src as uint32_t, game);
    let fresh31 = game.decompress_src;
    game.decompress_src = game.decompress_src.wrapping_add(1);
    if fresh31 & 0xffff as uint32 == 0xffff as uint32 {
        game.decompress_src = game.decompress_src.wrapping_add(0x8000 as uint32);
    }
    return b;
}
pub unsafe fn DecompressToMem(mut src: uint32, mut decompress_dst: *mut uint8, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: ::core::ffi::c_int = 0;
    let mut dst_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut len: ::core::ffi::c_int = 0;
        let mut cmd: uint8 = 0;
        let mut b: uint8 = 0;
        b = DecompNextByte(game);
        if b as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
            break;
        }
        if b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int == 0xe0 as ::core::ffi::c_int {
            cmd = (8 as ::core::ffi::c_int * b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                as uint8;
            len = ((b as ::core::ffi::c_int & 3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | DecompNextByte(game) as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int;
        } else {
            cmd = (b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int) as uint8;
            len = (b as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int;
        }
        if cmd as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            let mut want_xor: uint8 =
                (if cmd as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                    0xff as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as uint8;
            if cmd as ::core::ffi::c_int >= 0xc0 as ::core::ffi::c_int {
                src_pos = dst_pos - DecompNextByte(game) as ::core::ffi::c_int;
            } else {
                src_pos = DecompNextByte(game) as ::core::ffi::c_int;
                src_pos += DecompNextByte(game) as ::core::ffi::c_int * 256 as ::core::ffi::c_int;
            }
            loop {
                let fresh23 = src_pos;
                src_pos = src_pos + 1;
                b = (*decompress_dst.offset(fresh23 as isize) as ::core::ffi::c_int
                    ^ want_xor as ::core::ffi::c_int) as uint8;
                let fresh24 = dst_pos;
                dst_pos = dst_pos + 1;
                *decompress_dst.offset(fresh24 as isize) = b;
                len -= 1;
                if !(len != 0) {
                    break;
                }
            }
        } else {
            match cmd as ::core::ffi::c_int {
                32 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh25 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh25 as isize) = b;
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                64 => {
                    b = DecompNextByte(game);
                    let mut b2: uint8 = DecompNextByte(game);
                    loop {
                        let fresh26 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh26 as isize) = b;
                        len -= 1;
                        if len == 0 {
                            break;
                        }
                        let fresh27 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh27 as isize) = b2;
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                96 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh28 = b;
                        b = b.wrapping_add(1);
                        let fresh29 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh29 as isize) = fresh28;
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                _ => loop {
                    b = DecompNextByte(game);
                    let fresh30 = dst_pos;
                    dst_pos = dst_pos + 1;
                    *decompress_dst.offset(fresh30 as isize) = b;
                    len -= 1;
                    if !(len != 0) {
                        break;
                    }
                },
            }
        }
    }
}
unsafe fn ReadPpuByte(mut addr: uint16, game: &mut Game) -> uint8 {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        (addr as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16, game);
    let mut data: uint16 = ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16, game);
    return (if addr as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (data as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
    } else {
        data as ::core::ffi::c_int
    }) as uint8;
}
pub unsafe fn DecompressToVRAM(mut src: uint32, mut dst_addr: uint16, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: ::core::ffi::c_int = 0;
    let mut dst_pos: ::core::ffi::c_int = dst_addr as ::core::ffi::c_int;
    loop {
        let mut len: ::core::ffi::c_int = 0;
        let mut b: uint8 = DecompNextByte(game);
        let mut cmd: uint8 = 0;
        if b as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
            break;
        }
        if b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int == 0xe0 as ::core::ffi::c_int {
            cmd = (8 as ::core::ffi::c_int * b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                as uint8;
            len = ((b as ::core::ffi::c_int & 3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | DecompNextByte(game) as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int;
        } else {
            cmd = (b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int) as uint8;
            len = (b as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int;
        }
        if cmd as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            let mut want_xor: uint8 =
                (if cmd as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                    0xff as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as uint8;
            if cmd as ::core::ffi::c_int >= 0xc0 as ::core::ffi::c_int {
                src_pos = dst_pos - DecompNextByte(game) as ::core::ffi::c_int;
            } else {
                src_pos = DecompNextByte(game) as ::core::ffi::c_int;
                src_pos += DecompNextByte(game) as ::core::ffi::c_int * 256 as ::core::ffi::c_int;
                src_pos += dst_addr as ::core::ffi::c_int;
            }
            loop {
                let fresh32 = src_pos;
                src_pos = src_pos + 1;
                b = (ReadPpuByte(fresh32 as uint16, game) as ::core::ffi::c_int
                    ^ want_xor as ::core::ffi::c_int) as uint8;
                WriteRegWord(
                    VMADDL as ::core::ffi::c_int as uint16,
                    (dst_pos >> 1 as ::core::ffi::c_int) as uint16,
                    game,
                );
                let fresh33 = dst_pos;
                dst_pos = dst_pos + 1;
                WriteReg(
                    (VMDATAL as ::core::ffi::c_int + (fresh33 & 1 as ::core::ffi::c_int)) as uint16,
                    b,
                    game,
                );
                len -= 1;
                if !(len != 0) {
                    break;
                }
            }
        } else {
            match cmd as ::core::ffi::c_int {
                32 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh34 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh34 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b,
                            game,
                        );
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                64 => {
                    b = DecompNextByte(game);
                    let mut b2: uint8 = DecompNextByte(game);
                    loop {
                        let fresh35 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh35 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b,
                            game,
                        );
                        len -= 1;
                        if len == 0 {
                            break;
                        }
                        let fresh36 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh36 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b2,
                            game,
                        );
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                96 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh37 = dst_pos;
                        dst_pos = dst_pos + 1;
                        let fresh38 = b;
                        b = b.wrapping_add(1);
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh37 & 1 as ::core::ffi::c_int))
                                as uint16,
                            fresh38,
                            game,
                        );
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
                _ => loop {
                    b = DecompNextByte(game);
                    let fresh39 = dst_pos;
                    dst_pos = dst_pos + 1;
                    let fresh40 = b;
                    b = b.wrapping_add(1);
                    WriteReg(
                        (VMDATAL as ::core::ffi::c_int + (fresh39 & 1 as ::core::ffi::c_int))
                            as uint16,
                        fresh40,
                        game,
                    );
                    len -= 1;
                    if !(len != 0) {
                        break;
                    }
                },
            }
        }
    }
}
pub unsafe fn LoadFromLoadStation(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e75 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    let mut L: *const LoadStationList = RomPtr_80(
        (*(game.g_rom.offset(
            (((0x80c4b5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x80c4b5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            + 14 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut LoadStationList;
    *((&raw mut game.g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
        as *mut uint16) = (*L).room_ptr_ as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d as ::core::ffi::c_int as isize)
        as *mut uint16) = (*L).door_ptr as uint16;
    let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh77 = (*L).screen_x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x91d as ::core::ffi::c_int as isize)
        as *mut uint16) = *fresh77;
    let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh78 = (*L).screen_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x91f as ::core::ffi::c_int as isize)
        as *mut uint16) = *fresh78;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + (*L).samus_y_offset as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x911 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        + (*L).samus_x_offset as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*get_RoomDefHeader(
        *((&raw mut game.g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    ))
    .area_index_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn SetElevatorsAsUsed(game: &mut Game) {
    let mut v0: *const uint8 = RomPtr_80(
        (*(game.g_rom.offset(
            (((0x80cd46 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x80cd46 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int
                * ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x793 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int)) as uint16_t,
        game,
    );
    let ref mut fresh92 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8 as ::core::ffi::c_int as isize)
        .offset(*v0.offset(0 as ::core::ffi::c_int as isize) as isize);
    *fresh92 = (*fresh92 as ::core::ffi::c_int
        | *v0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint8;
    let ref mut fresh93 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8 as ::core::ffi::c_int as isize)
        .offset(*v0.offset(2 as ::core::ffi::c_int as isize) as isize);
    *fresh93 = (*fresh93 as ::core::ffi::c_int
        | *v0.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint8;
}
