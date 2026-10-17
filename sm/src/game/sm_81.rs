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
    static mut g_ram: [uint8; 131072];
    static mut g_sram: *mut uint8;
    static mut g_rom: *const uint8;
    fn Unreachable() -> bool;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn WriteReg(reg: uint16, value: uint8);
    fn RtlWriteSram();
    fn HasQueuedMusic() -> uint8;
    fn HandleFadeIn();
    fn HandleFadeOut();
    fn LoadFromLoadStation();
    fn LoadMirrorOfExploredMapTiles();
    fn QueueMusic_Delayed8(a: uint16);
    fn QueueSfx1_Max6(a: uint16);
    fn QueueSfx3_Max6(a: uint16);
    fn ScreenOff();
    fn ScreenOn();
    fn SetupDmaTransfer(p: *const ::core::ffi::c_void);
    fn AdvanceGradualColorChangeOfPalette(k: uint16, j: uint16) -> uint8;
    fn DisplayMapElevatorDestinations();
    fn DrawBorderAroundDataClearMode();
    fn DrawBorderAroundDataCopyMode();
    fn DrawBorderAroundSamusData();
    fn DrawFileCopyArrow();
    fn DrawFileSelectMapIcons();
    fn DrawMapScrollArrowAndCheckToScroll(db: uint8, k: uint16);
    fn DrawMenuSelectionMissile();
    fn DrawRoomSelectMap();
    fn DrawRoomSelectMapAreaLabel(dst: *mut uint16);
    fn HandleGameOverBabyMetroid();
    fn MapScrolling();
    fn SetupMapScrollingForFileSelectMap();
    fn SpawnHdmaObject(db: uint8, p: *const ::core::ffi::c_void) -> uint16;
    fn DisableHdmaObjects();
    fn EnableHdmaObjects();
    fn WaitUntilEndOfVblankAndClearHdma();
}
pub type __int8_t = i8;
pub type __uint8_t = u8;
pub type __int16_t = i16;
pub type __uint16_t = u16;
pub type __int32_t = i32;
pub type __uint32_t = u32;
pub type int8_t = __int8_t;
pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type size_t = usize;
pub type uint8 = uint8_t;
pub type int8 = int8_t;
pub type uint16 = uint16_t;
pub type int16 = int16_t;
pub type uint32 = uint32_t;
pub type int32 = int32_t;
pub type VoidP = uint16;
pub use crate::types::LongPtr;
pub type Func_V = unsafe extern "C" fn() -> ();
pub use crate::types::OamEnt;
pub use crate::types::VramWriteEntry;
pub type SnesRegs = ::core::ffi::c_uint;
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
pub struct SpawnHdmaObject_Args {
    pub field_0: uint8,
    pub field_1: uint8,
    pub addr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefHeader {
    pub semiunique_room_number: uint8,
    pub area_index_: uint8,
    pub x_coordinate_on_map: uint8,
    pub y_coordinate_on_map: uint8,
    pub width: uint8,
    pub height: uint8,
    pub up_scroller_: uint8,
    pub down_scroller_: uint8,
    pub cre_bitset_: uint8,
    pub ptr_to_doorout: VoidP,
}
pub type GameState = ::core::ffi::c_uint;
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
pub type Buttons = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct StartDmaCopy {
    pub chan: uint8,
    pub dmap: uint8,
    pub bbad: uint8,
    pub a1: LongPtr,
    pub das: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExpandingSquareTransitionHdma {
    pub field_0: uint8,
    pub field_1: uint8,
    pub field_2: uint8,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MapIconDataPointers {
    pub crateria: VoidP,
    pub brinstar: VoidP,
    pub norfair: VoidP,
    pub wrecked_ship: VoidP,
    pub maraidia: VoidP,
    pub tourian: VoidP,
    pub ceres: VoidP,
    pub debug: VoidP,
}
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
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MapScrollArrowData {
    pub xpos: uint16,
    pub ypos: uint16,
    pub pause_screen_animation_id: uint16,
    pub joypad_input: uint16,
    pub map_scroll_dir: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_MsgBox {
    pub pad: [uint8; 188],
    pub msg_box_anim_y_radius_neg: [uint16; 30],
    pub msg_box_anim_y_radius: [uint16; 30],
    pub field_134: [uint8; 204],
    pub tilemap: [uint16; 192],
    pub indirect_hdma: [uint8; 7],
    pub field_387: [uint8; 99],
    pub backup_of_enabled_hdma_channels: uint8,
    pub backup_of_bg3_tilemap_and_size: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Misc {
    pub field_0: [uint8; 472],
    pub pause_screen_samus_witeframe_tilemap: [uint16; 264],
    pub field_3E8: [uint8; 24],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram3000_Menu {
    pub field_0: [uint8; 768],
    pub palette_backup_in_menu: [uint16; 256],
    pub backup_of_io_registers_in_gameover: [uint8; 54],
    pub field_536: [uint8; 202],
    pub menu_tilemap: [uint8; 512],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram4000_Backups {
    pub field_0: [uint8; 256],
    pub backup_of_vram_0x5880_msgbox: [uint16; 896],
    pub field_800: [uint8; 2048],
    pub backup_of_0x3e00_in_kraid_pause: [uint8; 1024],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram4000 {
    pub xray_tilemaps: [uint16; 6144],
    pub bg2_tilemap: [uint16; 2048],
    pub decomp_buffer_kraid: [uint16; 1024],
    pub bg2_room_select_map_tilemap: [uint16; 1024],
    pub intro_japanese_text_tiles: [uint16; 768],
    pub backups: Ram4000_Backups,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Ram3000 {
    pub pause_menu_map_tilemap: [uint16; 1024],
    pub cinematic_bg_tilemap: [uint16; 1024],
    pub msgbox: Ram3000_MsgBox,
    pub msgbox_y_scroll_hdma: [uint8; 240],
    pub misc: Ram3000_Misc,
    pub menu: Ram3000_Menu,
}
pub type Consts_81 = ::core::ffi::c_uint;
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
pub type Consts_82 = ::core::ffi::c_uint;
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe extern "C" fn RomPtr_81(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x810000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtr_82(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x820000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtr_8C(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x8c0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtr_92(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x920000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtr_93(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x930000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
    );
}
#[inline]
unsafe extern "C" fn gOamEnt(mut v: ::core::ffi::c_int) -> *mut OamEnt {
    return (&raw mut g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe extern "C" fn gVramWriteEntry(mut a: ::core::ffi::c_int) -> *mut VramWriteEntry {
    return (&raw mut g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe extern "C" fn get_RoomDefHeader(mut a: uint16) -> *mut RoomDefHeader {
    return RomPtr((0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut RoomDefHeader;
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
static mut kFileSelectMap_AreaIndexes: [uint16; 6] = [
    0 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    5 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
];
#[no_mangle]
pub unsafe extern "C" fn SoftReset() {
    *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16) =
        0xffff as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SaveToSram(mut a: uint16) {
    let mut v7: uint16 = 0;
    let mut v11: uint16 = 0;
    let mut r20: uint16 = 0 as uint16;
    let mut r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xd7c0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            *(((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        * 256 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut g_ram as *mut uint8)
                .offset(0x7f7 as ::core::ffi::c_int as isize)
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
    PackMapToSave();
    *((&raw mut g_ram as *mut uint8).offset(0xd916 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x78b as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xd918 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v5: uint16 = *(g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v6: *mut uint16 =
        (&raw mut g_ram as *mut uint8).offset(0xd7c0 as ::core::ffi::c_int as isize) as *mut uint16;
    loop {
        let fresh94 = v6;
        v6 = v6.offset(1);
        v7 = *fresh94;
        *(g_sram.offset(
            (2 as ::core::ffi::c_int * (v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
                as isize,
        ) as *mut uint8 as *mut uint16) = v7;
        r20 = (r20 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint16;
        v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(v6
            != (&raw mut g_ram as *mut uint8).offset(0xde1c as ::core::ffi::c_int as isize)
                as *mut uint16)
        {
            break;
        }
    }
    let mut v8: uint16 = r18;
    let mut v9: uint16 = r20;
    let mut v10: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *(g_sram.offset((2 as ::core::ffi::c_int * v10) as isize) as *mut uint8 as *mut uint16) = r20;
    *(g_sram.offset((2 as ::core::ffi::c_int * v10 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9;
    v11 = !(v9 as ::core::ffi::c_int) as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v10 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    *(g_sram.offset((2 as ::core::ffi::c_int * v10 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    RtlWriteSram();
}
#[no_mangle]
pub unsafe extern "C" fn LoadFromSram(mut a: uint16) -> uint8 {
    let mut r20: uint16 = 0 as uint16;
    let mut r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = *(g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((a as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize);
    let mut v2: *mut uint16 =
        (&raw mut g_ram as *mut uint8).offset(0xd7c0 as ::core::ffi::c_int as isize) as *mut uint16;
    loop {
        let mut v3: uint16 = *(g_sram.offset(0 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let fresh1 = v2;
        v2 = v2.offset(1);
        *fresh1 = v3;
        r20 = (r20 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(v2
            != (&raw mut g_ram as *mut uint8).offset(0xde1c as ::core::ffi::c_int as isize)
                as *mut uint16)
        {
            break;
        }
    }
    let mut v4: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if r20 as ::core::ffi::c_int
        == *(g_sram.offset(0 as ::core::ffi::c_int as isize) as *mut uint16).offset(v4 as isize)
            as ::core::ffi::c_int
        && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
            == *(g_sram.offset(0x8 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
        || r20 as ::core::ffi::c_int
            == *(g_sram.offset(0x1ff0 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
            && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
                == *(g_sram.offset(0x1ff8 as ::core::ffi::c_int as isize) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xd7c0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        UnpackMapFromSave();
        *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xd916 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xd918 as ::core::ffi::c_int as isize)
            as *mut uint16);
        return 0 as uint8;
    } else {
        r20 = 0 as uint16;
        let mut v7: uint16 = *(g_rom.offset(
            (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v8: uint16 = ((&raw mut g_ram as *mut uint8)
            .offset(0xd7c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8)
            as ::core::ffi::c_long as uint16;
        let mut v9: uint16 = 0 as uint16;
        loop {
            *(g_sram.offset(0 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v9;
            v9 = (v9 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
            r20 = v9;
            v7 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v8 = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(v8 as ::core::ffi::c_long
                != ((&raw mut g_ram as *mut uint8).offset(0xde1c as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset_from(&raw mut g_ram as *mut uint8)
                    as ::core::ffi::c_long)
            {
                break;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn UnpackMapFromSave() {
    let mut i: ::core::ffi::c_int = 1792 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 6 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = *(g_rom.offset(
            (((0x818131 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818131 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(g_rom.offset(
                (((0x8182d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8182d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(i_0 as isize) as uint16_t,
        );
        let mut v3: ::core::ffi::c_int = *(g_rom.offset(
            (((0x818138 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818138 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let mut unpacked: *mut uint8 = (&raw mut g_ram as *mut uint8)
            .offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8;
        loop {
            let fresh2 = r0;
            r0 = r0.offset(1);
            let mut v4: uint16 =
                (i_0 * 256 as ::core::ffi::c_int + *fresh2 as ::core::ffi::c_int) as uint16;
            let fresh3 = v3;
            v3 = v3 + 1;
            *unpacked.offset(v4 as isize) = *(&raw mut g_ram as *mut uint8)
                .offset(0xd91c as ::core::ffi::c_int as isize)
                .offset(fresh3 as isize);
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PackMapToSave() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = *(g_rom.offset(
            (((0x818131 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818131 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(i as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(g_rom.offset(
                (((0x8182d6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8182d6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(i as isize) as uint16_t,
        );
        let mut v1: ::core::ffi::c_int = *(g_rom.offset(
            (((0x818138 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x818138 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i as isize) as ::core::ffi::c_int;
        let mut unpacked: *mut uint8 = (&raw mut g_ram as *mut uint8)
            .offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8;
        loop {
            let fresh95 = r0;
            r0 = r0.offset(1);
            let mut v3: ::core::ffi::c_int =
                i * 256 as ::core::ffi::c_int + *fresh95 as ::core::ffi::c_int;
            let fresh96 = v1;
            v1 = v1 + 1;
            *(&raw mut g_ram as *mut uint8)
                .offset(0xd91c as ::core::ffi::c_int as isize)
                .offset(fresh96 as isize) = *unpacked.offset(v3 as isize);
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DrawSpritemap(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r22: uint16,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int && idx < 0x200 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int = y_r18 as uint8 as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        if if *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
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
            | *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh9 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
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
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawSpritemapOffScreen(
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r22: uint16,
) {
    let mut pp: *const uint8 = RomPtr_8C(j as uint16_t);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int && idx < 0x200 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int = y_r18 as uint8 as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        if if *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
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
            | *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh10 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
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
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawMenuSpritemap(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    mut chr_r3: uint16,
) {
    let mut pp: *const uint8 = RomPtr_82(
        *(g_rom.offset(
            (((0x82c569 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x82c569 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(a as isize) as uint16_t,
    );
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: uint16 =
            (k as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh4 = (*fresh4 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord = (j as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r3 as ::core::ffi::c_int
            | *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawSamusSpritemap(mut a: uint16, mut x_pos: uint16, mut y_pos: uint16) {
    if *(g_rom.offset(
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
        *(g_rom.offset(
            (((0x92808d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x92808d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(a as isize) as uint16_t,
    );
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut x: uint16 =
            (x_pos as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut v9: *mut OamEnt = gOamEnt(idx);
        (*v9).xcoord = x as uint8;
        let ref mut fresh8 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh8 = (*fresh8 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v9).ycoord = (y_pos as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint8;
        *(&raw mut (*v9).charnum as *mut uint16) =
            *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16);
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawBeamGrappleSpritemap(
    mut a: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *(g_rom.offset(
                (((0x93a1a1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x93a1a1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(a as isize) as uint16_t,
        ),
        x_r20,
        y_r18,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DrawProjectileSpritemap(
    mut k: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *((&raw mut g_ram as *mut uint8).offset(0xcb8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as uint16_t,
        ),
        x_r20,
        y_r18,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DrawGrappleOrProjectileSpritemap(
    mut pp: *const uint8,
    mut x_r20: uint16,
    mut y_r18: uint16,
) {
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut v4: *mut OamEnt = gOamEnt(idx);
        (*v4).xcoord = x as uint8;
        let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh5 = (*fresh5 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v4).ycoord = (y_r18 as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint8;
        *(&raw mut (*v4).charnum as *mut uint16) =
            *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16);
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawSpritemapWithBaseTile(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
) {
    if j as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return;
    }
    let mut pp: *mut uint8 = RomPtrWithBank(db, j as uint16_t) as *mut uint8;
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: uint16 = (r20_x as ::core::ffi::c_int
            + *(pp.offset(0 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh11 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord = (r18_y as ::core::ffi::c_int
            + *pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int
                + *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawSpritemapWithBaseTile2(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: uint16 =
            (r20_x as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh12 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh12 = (*fresh12 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int = *pp.offset(2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                == 0) as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int
                + *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawSpritemapWithBaseTileOffscreen(
    mut db: uint8,
    mut j: uint16,
    mut r20_x: uint16,
    mut r18_y: uint16,
    mut r3: uint16,
    mut r0: uint16,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: ::core::ffi::c_int =
            r20_x as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        (*oam).xcoord = x as uint8;
        let ref mut fresh13 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh13 = (*fresh13 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int = *pp.offset(2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                == 0) as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int
                + *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawEprojSpritemapWithBaseTile(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r26: uint16,
    mut chr_r28: uint16,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh6 = (*fresh6 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int = *pp.offset(2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                == 0) as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int
                + *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawEprojSpritemapWithBaseTileOffscreen(
    mut db: uint8,
    mut j: uint16,
    mut x_r20: uint16,
    mut y_r18: uint16,
    mut chr_r26: uint16,
    mut chr_r28: uint16,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j as uint16_t);
    let mut idx: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 as ::core::ffi::c_int {
        let mut oam: *mut OamEnt = gOamEnt(idx);
        let mut x: uint16 =
            (x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int) as uint16;
        (*oam).xcoord = x as uint8;
        let ref mut fresh7 = *((&raw mut g_ram as *mut uint8)
            .offset(0x570 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh7 = (*fresh7 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | ((*(pp as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let mut y: ::core::ffi::c_int = *pp.offset(2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                == 0) as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int
                + *(pp.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5 as ::core::ffi::c_int as isize);
        n -= 1;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16) =
        idx as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_0_FadeOutConfigGfx() {
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        ScreenOff();
        QueueSfx3_Max6(1 as uint16);
        DisableHdmaObjects();
        WaitUntilEndOfVblankAndClearHdma();
        let mut v0: uint16 = 0 as uint16;
        loop {
            (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
                as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v0 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize] =
                *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
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
            (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
                as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 640 as ::core::ffi::c_int) as usize] =
                *((&raw mut g_ram as *mut uint8)
                    .offset(0x51 as ::core::ffi::c_int as isize)
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
        MapVramForMenu();
        LoadInitialMenuTiles();
        *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbb as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        LoadMenuPalettes();
        let ref mut fresh89 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh89 = (*fresh89).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_24_FadeIn() {
    HandleFadeIn();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh90 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh90 = (*fresh90).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MapVramForMenu() {
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x52 as ::core::ffi::c_int as isize) = 3 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize) = 81 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize) = 88 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5a as ::core::ffi::c_int as isize) = 92 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x5e as ::core::ffi::c_int as isize) = 4 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn LoadInitialMenuTiles() {
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    static mut unk_818DF1: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x8e8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x8e8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x5600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818DF1 as *const ::core::ffi::c_void);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x30 as uint8);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    static mut unk_818E11: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0xb68000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0xb68000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E11 as *const ::core::ffi::c_void);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x60 as uint8);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    static mut unk_818E31: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0xb6c000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0xb6c000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E31 as *const ::core::ffi::c_void);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x40 as uint8);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    static mut unk_818E51: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x8ed600 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x8ed600 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_818E51 as *const ::core::ffi::c_void);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
}
#[no_mangle]
pub unsafe extern "C" fn LoadMenuPalettes() {
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *(g_rom.offset(
            (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
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
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_3_Main() {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let ref mut fresh91 = *((&raw mut g_ram as *mut uint8)
            .offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh91 = (*fresh91 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & (kButton_B as ::core::ffi::c_int
            | kButton_Start as ::core::ffi::c_int
            | kButton_A as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            let ref mut fresh92 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh92 = (*fresh92).wrapping_add(1);
        } else {
            SaveToSram(
                *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            SoftReset();
        }
        return;
    }
    v0 = 30720 as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        v0 = -(30720 as ::core::ffi::c_int) as int16;
    }
    v1 = (v0 as ::core::ffi::c_int | 0x28 as ::core::ffi::c_int) as int16;
    let mut v3: *mut OamEnt = gOamEnt(
        *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *(&raw mut (*v3).xcoord as *mut uint16) = v1 as uint16;
    *(&raw mut (*v3).charnum as *mut uint16) = 182 as uint16;
    let ref mut fresh93 = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh93 = (*fresh93 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn RestorePalettesAndIoAfterDebugGameover() {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
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
        *((&raw mut g_ram as *mut uint8)
            .offset(0x51 as ::core::ffi::c_int as isize)
            .offset((v1 * 2 as ::core::ffi::c_int) as isize) as *mut uint16) =
            (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
                as *mut Ram3000))
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
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_5_Continue() {
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x40 as uint8);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8);
    static mut unk_819093: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x9ab200 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x9ab200 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_819093 as *const ::core::ffi::c_void);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8);
    *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16) =
        16 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    RestorePalettesAndIoAfterDebugGameover();
}
static mut kGameOverMenuFuncs: [Option<Func_V>; 8] = unsafe {
    [
        Some(GameOverMenu_0_FadeOutConfigGfx as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_1_Init as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_2_PlayMusic as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_3_FadeIn as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_4_Main as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_5_FadeOutToGameMap as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_6_LoadGameMapView as unsafe extern "C" fn() -> ()),
        Some(GameOverMenu_7_FadeOutToSoftReset as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu() {
    kGameOverMenuFuncs[*((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16) as usize]
        .expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_3_FadeIn() {
    HandleGameOverBabyMetroid();
    DrawMenuSelectionMissile();
    HandleFadeIn();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh86 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh86 = (*fresh86).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_5_FadeOutToGameMap() {
    HandleGameOverBabyMetroid();
    DrawMenuSelectionMissile();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh83 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh83 = (*fresh83).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_7_FadeOutToSoftReset() {
    DrawMenuSelectionMissile();
    HandleGameOverBabyMetroid();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        SoftReset();
    }
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_6_LoadGameMapView() {
    DisableHdmaObjects();
    WaitUntilEndOfVblankAndClearHdma();
    *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16) =
        kGameState_5_FileSelectMap as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_4_Main() {
    HandleGameOverBabyMetroid();
    DrawMenuSelectionMissile();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16);
        let ref mut fresh84 = *((&raw mut g_ram as *mut uint8)
            .offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh84 = (*fresh84 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .instruction_timer = 180 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16) = 7 as uint16;
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0xd914 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kGameState_31_SetUpNewGame as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                    .offset(0xd914 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            } else {
                let ref mut fresh85 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh85 = (*fresh85).wrapping_add(1);
            }
            LoadFromSram(
                *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        }
        return;
    }
    let mut v0: uint16 = 160 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        v0 = 192 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 40 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = v0;
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_1_Init() {
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    QueueMusic_Delayed8(0 as uint16);
    QueueMusic_Delayed8(0xff03 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x3000 as ::core::ffi::c_int as isize) as *mut Ram3000))
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
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v1 + 7 as ::core::ffi::c_int) as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x156 as uint16,
        addr_kMenuTilemap_GameOver as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x38a as uint16,
        addr_kMenuTilemap_FindMetroidLarva as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x414 as uint16,
        addr_kMenuTilemap_TryAgain as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x4ce as uint16,
        addr_kMenuTilemap_YesReturnToGame as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x5ce as uint16,
        addr_kMenuTilemap_NoGoToTitle as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .current_instruction = 0 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .instruction_timer = 0 as uint16;
    HandleGameOverBabyMetroid();
    *((&raw mut g_ram as *mut uint8).offset(0x6f as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*(&raw mut g_ram as *mut uint8).offset(0x70 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
    DisableHdmaObjects();
    WaitUntilEndOfVblankAndClearHdma();
    EnableHdmaObjects();
    static mut unk_819254: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0x927d as VoidP,
    };
    static mut unk_81925C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0x928d as VoidP,
    };
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_819254 as *const ::core::ffi::c_void,
    );
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_81925C as *const ::core::ffi::c_void,
    );
    ScreenOn();
    let ref mut fresh88 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh88 = (*fresh88).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x723 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x725 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 40 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = 160 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn GameOverMenu_2_PlayMusic() {
    if HasQueuedMusic() == 0 {
        let ref mut fresh87 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh87 = (*fresh87).wrapping_add(1);
        QueueMusic_Delayed8(4 as uint16);
    }
}
static mut kFileSelectMenuFuncs: [Option<Func_V>; 34] = unsafe {
    [
        Some(FileSelectMenu_0_FadeOutConfigGfx as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_1_LoadFileSelectMenuBG2 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_2_InitMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_3_FadeInToMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_4_Main as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_6_FileCopyInit as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_8 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_9_InitializeSelectDest as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_10_FileCopySelectDest as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_11_InitializeConfirm as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_12_FileCopyConfirm as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_13_FileCopyDoIt as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_14_CopyCompleted as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_16 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_18 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_20_FileClearInit as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_22_FileClearSelectSlot as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_23_FileClearInitConfirm as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_24_FileClearConfirm as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_25_FileClearDoClear as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_26_ClearCompleted as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_16 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_30 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_31_TurnSamusHelmet as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_32_FadeOutToOptions as unsafe extern "C" fn() -> ()),
        Some(FileSelectMenu_33_FadeOutToTitle as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu() {
    kFileSelectMenuFuncs[*((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16)
        as usize]
        .expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_0_FadeOutConfigGfx() {
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    ScreenOff();
    QueueSfx3_Max6(1 as uint16);
    DisableHdmaObjects();
    WaitUntilEndOfVblankAndClearHdma();
    let ref mut fresh82 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh82 = (*fresh82).wrapping_add(1);
    MapVramForMenu();
    LoadInitialMenuTiles();
    *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb9 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbb as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    LoadFileSelectPalettes();
}
#[no_mangle]
pub unsafe extern "C" fn LoadFileSelectPalettes() {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut v1: ::core::ffi::c_int = v0 >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *(g_rom.offset(
            (((0x8ee400 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ee400 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
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
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_32_FadeOutToOptions() {
    DrawMenuSelectionMissile();
    DrawBorderAroundSamusData();
    DrawFileSelectSamusHelmets();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_2_GameOptionsMenu as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
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
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_33_FadeOutToTitle() {
    DrawBorderAroundSamusData();
    DrawFileSelectSamusHelmets();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        SoftReset();
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_5_FadeOutFromMain() {
    DrawMenuSelectionMissile();
    FileSelectMenu_15_FadeOutToMain();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_15_FadeOutToMain() {
    HandleFadeOut();
    *(&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize) =
        (*(&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        ScreenOff();
        let ref mut fresh48 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh48 = (*fresh48).wrapping_add(1);
    }
}
static mut kMenuSelectionMissileXY: [uint16; 12] = [
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
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_17_FadeInToMain() {
    let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    FileSelectMenu_7_FadeInFromMain();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_7_FadeInFromMain() {
    DrawMenuSelectionMissile();
    HandleFadeIn();
    *(&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize) =
        (*(&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh46 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh46 = (*fresh46).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_6_FileCopyInit() {
    let ref mut fresh76 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh76 = (*fresh76).wrapping_add(1);
    FileSelectMenu_Func1();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_Func1() {
    ClearMenuTilemap();
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x150 as uint16,
        addr_kMenuTilemap_CopyWhichData as ::core::ffi::c_int as uint16,
    );
    LoadMenuExitTilemap();
    DrawFileCopySaveFileInfo();
    SetInitialFileCopyMenuSelection();
    SetFileCopyMenuSelectionMissilePosition();
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(17 as ::core::ffi::c_int as isize) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn sub_819591() {
    SetInitialFileCopyMenuSelection();
}
#[no_mangle]
pub unsafe extern "C" fn SetInitialFileCopyMenuSelection() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x954 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
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
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize) = v1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn ClearMenuTilemap() {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadMenuExitTilemap() {
    LoadMenuTilemap(
        0x688 as uint16,
        addr_kMenuTilemap_Exit as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopySaveFileInfo() {
    LoadFromSram_(0 as uint16);
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotAInfo();
    LoadFromSram_(1 as uint16);
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotBInfo();
    LoadFromSram_(2 as uint16);
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotCInfo();
    QueueTransferOfMenuTilemapToVramBG1();
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopySaveSlotAInfo() {
    DrawFileSelectionHealth(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16,
        0x218 as uint16,
    );
    DrawFileSelectionTime(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16,
        0x272 as uint16,
    );
    LoadMenuTilemap(
        0x234 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x208 as uint16,
        addr_kMenuTilemap_SamusA as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopySaveSlotBInfo() {
    DrawFileSelectionHealth(
        (!(*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_int) as uint16,
        0x318 as uint16,
    );
    DrawFileSelectionTime(
        (!(*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_int) as uint16,
        0x372 as uint16,
    );
    LoadMenuTilemap(
        0x334 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x308 as uint16,
        addr_kMenuTilemap_SamusB as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopySaveSlotCInfo() {
    DrawFileSelectionHealth(
        (!(*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            & 4 as ::core::ffi::c_int) as uint16,
        0x418 as uint16,
    );
    DrawFileSelectionTime(
        (!(*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            & 4 as ::core::ffi::c_int) as uint16,
        0x472 as uint16,
    );
    LoadMenuTilemap(
        0x434 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x408 as uint16,
        addr_kMenuTilemap_SamusC as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn QueueTransferOfMenuTilemapToVramBG1() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
static mut kBitShl: [uint8; 3] = [
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
];
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_8() {
    let mut v0: uint8 = 0;
    DrawBorderAroundDataCopyMode();
    DrawMenuSelectionMissile();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        {
            let ref mut fresh73 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh73 = (*fresh73 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(16 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize);
            let ref mut fresh74 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh74 = (*fresh74).wrapping_add(1);
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh75 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh75 = (*fresh75 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16);
    } else {
        let mut current_block_18: u64;
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v0 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize) as uint8;
            loop {
                v0 = v0.wrapping_sub(1);
                if !(v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 1109700713171191020;
                    break;
                }
                if kBitShl[v0 as usize] as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 1398313740379482408;
                    break;
                }
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v0 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize) as uint8;
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
                        & *((&raw mut g_ram as *mut uint8)
                            .offset(0x954 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
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
                *(((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(15 as ::core::ffi::c_int as isize) as *mut uint16
                    as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize) = v0;
                QueueSfx1_Max6(0x37 as uint16);
                SetFileCopyMenuSelectionMissilePosition();
                return;
            }
        }
        SetFileCopyMenuSelectionMissilePosition();
    };
}
#[no_mangle]
pub unsafe extern "C" fn SetFileCopyMenuSelectionMissilePosition() {
    static mut kFileCopyMissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        211 as ::core::ffi::c_int as uint16,
    ];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) =
        kFileCopyMissileY[*((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 22 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_9_InitializeSelectDest() {
    DrawBorderAroundDataCopyMode();
    DrawFileCopySelectDestinationSaveFileInfo();
    let ref mut fresh72 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh72 = (*fresh72).wrapping_add(1);
    let mut v0: uint16 = 0 as uint16;
    while !(v0 as ::core::ffi::c_int
        != *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        v0 = v0.wrapping_add(1);
        if !(v0 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0)
        {
            break;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize) = v0;
    SetFileCopyMenuSelectionMissilePosition();
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopySelectDestinationSaveFileInfo() {
    ClearMenuTilemap();
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x148 as uint16,
        addr_kMenuTilemap_CopySamusToWhere as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap[944 as ::core::ffi::c_int as usize] =
        (*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    LoadMenuExitTilemap();
    LoadFromSram_(0 as uint16);
    let mut v0: uint16 = 1024 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize)
        != 0
    {
        v0 = 0 as uint16;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v0;
    DrawFileCopySaveSlotAInfo();
    LoadFromSram_(1 as uint16);
    let mut v1: uint16 = 1024 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        v1 = 0 as uint16;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v1;
    DrawFileCopySaveSlotBInfo();
    LoadFromSram_(2 as uint16);
    let mut v2: uint16 = 1024 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        v2 = 0 as uint16;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v2;
    DrawFileCopySaveSlotCInfo();
    QueueTransferOfMenuTilemapToVramBG1();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_10_FileCopySelectDest() {
    let mut current_block: u64;
    static mut kFileCopySelectDest_MissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        212 as ::core::ffi::c_int as uint16,
    ];
    let mut v0: uint16 = 0;
    DrawBorderAroundDataCopyMode();
    DrawMenuSelectionMissile();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(17 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize);
            let ref mut fresh69 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh69 = (*fresh69).wrapping_add(1);
            current_block = 16545170798630791882;
        } else {
            let ref mut fresh70 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh70 = (*fresh70 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
            current_block = 16203760046146113240;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x37 as uint16);
            v0 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize);
            loop {
                v0 = v0.wrapping_sub(1);
                if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 as ::core::ffi::c_int
                    != *((&raw mut g_ram as *mut uint8)
                        .offset(0x1997 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(16 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x37 as uint16);
            v0 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize);
            loop {
                v0 = v0.wrapping_add(1);
                if !(v0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 as ::core::ffi::c_int
                    != *((&raw mut g_ram as *mut uint8)
                        .offset(0x1997 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(16 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
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
                *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(15 as ::core::ffi::c_int as isize) = v0;
                current_block = 16545170798630791882;
            }
        }
    } else {
        let ref mut fresh71 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh71 = (*fresh71 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
        QueueSfx1_Max6(0x37 as uint16);
        FileSelectMenu_Func1();
        current_block = 16203760046146113240;
    }
    match current_block {
        16203760046146113240 => {
            return;
        }
        _ => {
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(10 as ::core::ffi::c_int as isize) = kFileCopySelectDest_MissileY
                [*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(15 as ::core::ffi::c_int as isize) as usize];
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(5 as ::core::ffi::c_int as isize) = 22 as uint16;
            return;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_11_InitializeConfirm() {
    DrawBorderAroundDataCopyMode();
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x144 as uint16,
        addr_kMenuTilemap_CopySamusToSamus as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap[942 as ::core::ffi::c_int as usize] =
        (*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap[955 as ::core::ffi::c_int as usize] =
        (*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    DrawFileCopyClearConfirmation();
    let ref mut fresh68 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh68 = (*fresh68).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x198f as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopyClearConfirmation() {
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x514 as uint16,
        addr_kMenuTilemap_IsThisOk as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x59c as uint16,
        addr_kMenuTilemap_Yes as ::core::ffi::c_int as uint16,
    );
    let mut v0: ::core::ffi::c_int = 832 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut g_ram as *mut uint8)
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
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x65c as uint16,
        addr_kMenuTilemap_No as ::core::ffi::c_int as uint16,
    );
    DrawFileCopyConfirmationSaveFileInfo();
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileCopyConfirmationSaveFileInfo() {
    LoadFromSram_(0 as uint16);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 0
        && *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0
    {
        v0 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v0 as uint16;
    DrawFileCopySaveSlotAInfo();
    LoadFromSram_(1 as uint16);
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        v1 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v1 as uint16;
    DrawFileCopySaveSlotBInfo();
    LoadFromSram_(2 as uint16);
    let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
    {
        v2 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = v2 as uint16;
    DrawFileCopySaveSlotCInfo();
    QueueTransferOfMenuTilemapToVramBG1();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_12_FileCopyConfirm() {
    DrawBorderAroundDataCopyMode();
    DrawMenuSelectionMissile();
    HandleFileCopyArrowPalette();
    DrawFileCopyArrow();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh64 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *fresh64 = (*fresh64 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16);
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh65 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh65 = (*fresh65 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(17 as ::core::ffi::c_int as isize);
            QueueSfx1_Max6(0x37 as uint16);
            return;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x38 as uint16);
            if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize)
                != 0
            {
                let ref mut fresh66 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh66 = (*fresh66 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
                FileSelectMenu_Func1();
            } else {
                let ref mut fresh67 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh67 = (*fresh67).wrapping_add(1);
            }
            return;
        }
    }
    let mut v0: uint16 = 184 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize)
        != 0
    {
        v0 = 208 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = v0;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 94 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn HandleFileCopyArrowPalette() {
    if *((&raw mut g_ram as *mut uint8).offset(0x198f as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh63 = *((&raw mut g_ram as *mut uint8)
            .offset(0x198f as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh63 = (*fresh63).wrapping_sub(1);
        if *fresh63 == 0 {
            *((&raw mut g_ram as *mut uint8).offset(0x198f as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
            let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(145 as ::core::ffi::c_int as isize);
            let mut v1: uint16 = 0 as uint16;
            loop {
                *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            + 145 as ::core::ffi::c_int) as isize,
                    ) = *((&raw mut g_ram as *mut uint8)
                    .offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
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
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(151 as ::core::ffi::c_int as isize) = v0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_13_FileCopyDoIt() {
    static mut kBitShl_16bit: [uint16; 3] = [
        1 as ::core::ffi::c_int as uint16,
        2 as ::core::ffi::c_int as uint16,
        4 as ::core::ffi::c_int as uint16,
    ];
    DrawBorderAroundDataCopyMode();
    DrawMenuSelectionMissile();
    HandleFileCopyArrowPalette();
    DrawFileCopyArrow();
    let mut src_addr: uint16 = *(g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(16 as ::core::ffi::c_int as isize) as isize,
        );
    let mut dst_addr: uint16 = *(g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(17 as ::core::ffi::c_int as isize) as isize,
        );
    memcpy(
        g_sram.offset(dst_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        g_sram.offset(src_addr as isize) as *mut uint8 as *const ::core::ffi::c_void,
        1628 as size_t,
    );
    let mut v2: ::core::ffi::c_int =
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    let mut v10: ::core::ffi::c_int = *(g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v9: ::core::ffi::c_int = *(g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v8: ::core::ffi::c_int = *(g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let mut v4: ::core::ffi::c_int =
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    *(g_sram.offset((2 as ::core::ffi::c_int * v4 + 8 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = *(g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16);
    *(g_sram.offset((2 as ::core::ffi::c_int * v4) as isize) as *mut uint8 as *mut uint16) =
        v8 as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v4 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9 as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v4 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v10 as uint16;
    let ref mut fresh61 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh61 = (*fresh61).wrapping_add(1);
    let mut v5: ::core::ffi::c_int = 640 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut g_ram as *mut uint8)
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
    let ref mut fresh62 = *((&raw mut g_ram as *mut uint8)
        .offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh62 = (*fresh62 as ::core::ffi::c_int
        | kBitShl_16bit[*((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as usize] as ::core::ffi::c_int)
        as uint16;
    let mut v6: ::core::ffi::c_int = ((4 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + 9 as ::core::ffi::c_int)
        << 6 as ::core::ffi::c_int)
        + 24 as ::core::ffi::c_int;
    let mut v7: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(736 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        v6 += 2 as ::core::ffi::c_int;
        v7 += 2 as ::core::ffi::c_int;
        if !(((v7 - 22 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x510 as uint16,
        addr_kMenuTilemap_CopyCompleted as ::core::ffi::c_int as uint16,
    );
    DrawFileCopyConfirmationSaveFileInfo();
    RtlWriteSram();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_14_CopyCompleted() {
    DrawBorderAroundDataCopyMode();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        QueueSfx1_Max6(0x37 as uint16);
        let ref mut fresh60 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh60 = (*fresh60).wrapping_add(1);
        let mut slot: ::core::ffi::c_int = *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        if slot & 0x8000 as ::core::ffi::c_int != 0
            || *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            || slot
                & *(g_sram.offset(0x1fee as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int
                != 0
        {
            slot = 0 as ::core::ffi::c_int;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) = slot as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_18() {
    let ref mut fresh59 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh59 = (*fresh59 as ::core::ffi::c_int - 14 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_20_FileClearInit() {
    DrawBorderAroundDataClearMode();
    let ref mut fresh58 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh58 = (*fresh58).wrapping_add(1);
    InitFileSelectMenuFileClear();
}
#[no_mangle]
pub unsafe extern "C" fn InitFileSelectMenuFileClear() {
    ClearMenuTilemap();
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x50 as uint16,
        addr_kMenuTilemap_DataClearMode as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearWhichData as ::core::ffi::c_int as uint16,
    );
    LoadMenuExitTilemap();
    DrawFileCopySaveFileInfo();
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) = 0 as uint16;
    SetInitialFileCopyMenuSelection();
    SetFileClearMenuMissilePos();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_22_FileClearSelectSlot() {
    static mut kBitShl_: [uint8; 3] = [
        1 as ::core::ffi::c_int as uint8,
        2 as ::core::ffi::c_int as uint8,
        4 as ::core::ffi::c_int as uint8,
    ];
    let mut v0: uint16 = 0;
    let mut v1: uint8 = 0;
    DrawBorderAroundDataClearMode();
    DrawMenuSelectionMissile();
    *(&raw mut v0 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 3 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(16 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize);
            let ref mut fresh56 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh56 = (*fresh56).wrapping_add(1);
            return;
        }
    } else if !(*((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int)
    {
        let mut current_block_18: u64;
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v1 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize) as uint8;
            loop {
                v1 = v1.wrapping_sub(1);
                if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block_18 = 11307063007268554308;
                    break;
                }
                if kBitShl_[v1 as usize] as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block_18 = 11325801268588962582;
                    break;
                }
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v1 = *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize) as uint8;
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
                        & *((&raw mut g_ram as *mut uint8)
                            .offset(0x954 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
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
                *(((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(15 as ::core::ffi::c_int as isize) as *mut uint16
                    as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize) = v1;
                *(&raw mut v0 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = 55 as uint8;
                QueueSfx1_Max6(v0);
                SetFileClearMenuMissilePos();
                return;
            }
        }
        SetFileClearMenuMissilePos();
        return;
    }
    QueueSfx1_Max6(0x37 as uint16);
    let ref mut fresh57 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh57 = (*fresh57 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SetFileClearMenuMissilePos() {
    static mut kFileClear_MissileY: [uint16; 4] = [
        72 as ::core::ffi::c_int as uint16,
        104 as ::core::ffi::c_int as uint16,
        136 as ::core::ffi::c_int as uint16,
        211 as ::core::ffi::c_int as uint16,
    ];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) =
        kFileClear_MissileY[*((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 22 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_23_FileClearInitConfirm() {
    DrawBorderAroundDataClearMode();
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearSamusA as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
        as *mut Ram3000))
        .pause_menu_map_tilemap[949 as ::core::ffi::c_int as usize] =
        (*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 8298 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(17 as ::core::ffi::c_int as isize) = 3 as uint16;
    DrawFileCopyClearConfirmation();
    let ref mut fresh55 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_24_FileClearConfirm() {
    let mut current_block: u64;
    let mut v0: uint16 = 0;
    DrawBorderAroundDataClearMode();
    DrawMenuSelectionMissile();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh52 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *fresh52 = (*fresh52 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x37 as uint16);
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            current_block = 18062911466027671358;
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
        {
            current_block = 504450154722759016;
        } else {
            QueueSfx1_Max6(0x38 as uint16);
            if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(15 as ::core::ffi::c_int as isize)
                != 0
            {
                current_block = 18062911466027671358;
            } else {
                let ref mut fresh54 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh54 = (*fresh54).wrapping_add(1);
                return;
            }
        }
        match current_block {
            504450154722759016 => {}
            _ => {
                let ref mut fresh53 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh53 = (*fresh53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(15 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                    .offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(16 as ::core::ffi::c_int as isize);
                QueueSfx1_Max6(0x37 as uint16);
                InitFileSelectMenuFileClear();
                return;
            }
        }
    }
    v0 = 184 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize)
        != 0
    {
        v0 = 208 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = v0;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 94 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_25_FileClearDoClear() {
    static mut kBitShl_Not: [uint16; 3] = [
        0xfffe as ::core::ffi::c_int as uint16,
        0xfffd as ::core::ffi::c_int as uint16,
        0xfffb as ::core::ffi::c_int as uint16,
    ];
    DrawBorderAroundDataClearMode();
    let mut sram_addr: ::core::ffi::c_int = *(g_rom.offset(
        (((0x81812b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81812b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(16 as ::core::ffi::c_int as isize) as isize,
        ) as ::core::ffi::c_int;
    memset(
        g_sram.offset(sram_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        1628 as size_t,
    );
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    let mut v2: ::core::ffi::c_int =
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    *(g_sram.offset((2 as ::core::ffi::c_int * v2) as isize) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 0 as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    *(g_sram.offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    let ref mut fresh50 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh50 = (*fresh50).wrapping_add(1);
    NewSaveFile();
    LoadFromSram(
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize),
    );
    *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
    LoadMirrorOfExploredMapTiles();
    let mut v3: uint16 = 640 as uint16;
    loop {
        // File-clear rows share the full menu tilemap at $3600.
        *((&raw mut g_ram as *mut uint8)
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
    let ref mut fresh51 = *((&raw mut g_ram as *mut uint8)
        .offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh51 = (*fresh51 as ::core::ffi::c_int
        & kBitShl_Not[*((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) as usize] as ::core::ffi::c_int)
        as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x500 as uint16,
        addr_kMenuTilemap_DataCleared as ::core::ffi::c_int as uint16,
    );
    DrawFileCopyConfirmationSaveFileInfo();
    RtlWriteSram();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_26_ClearCompleted() {
    DrawBorderAroundDataClearMode();
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        let mut current_block_7: u64;
        QueueSfx1_Max6(0x37 as uint16);
        let ref mut fresh49 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh49 = (*fresh49).wrapping_add(1);
        if !(LoadFromSram(0 as uint16) == 0) {
            if LoadFromSram(1 as uint16) != 0 {
                if LoadFromSram(2 as uint16) != 0 {
                    current_block_7 = 17037298857681653719;
                } else {
                    *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 2 as uint16;
                    current_block_7 = 1394248824506584008;
                }
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                current_block_7 = 1394248824506584008;
            }
            match current_block_7 {
                17037298857681653719 => {}
                _ => {
                    return;
                }
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_30() {
    DrawBorderAroundSamusData();
    let ref mut fresh45 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh45 = (*fresh45 as ::core::ffi::c_int - 26 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_31_TurnSamusHelmet() {
    DrawMenuSelectionMissile();
    DrawBorderAroundSamusData();
    DrawFileSelectSlotSamusHelmet(4 as uint16);
    DrawFileSelectSlotSamusHelmet(6 as uint16);
    DrawFileSelectSlotSamusHelmet(8 as uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        || *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        || *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
    {
        let ref mut fresh44 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh44 = (*fresh44).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileSelectSamusHelmets() {
    *((&raw mut g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(4 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(6 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    DrawFileSelectSlotSamusHelmet(8 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileSelectSlotSamusHelmet(mut k: uint16) {
    let mut v0: uint16 = k;
    let mut v1: int16 = 0;
    v1 = *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) as int16;
    if v1 != 0 {
        let mut v2: uint16 = (v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v2;
        if v2 == 0 {
            *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 8 as uint16;
            let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v4: uint16 = (*((&raw mut g_ram as *mut uint8)
                .offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - 7 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
                v4 = 7 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = v4;
        }
    }
    let mut v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v6: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as ::core::ffi::c_int) as uint16;
    static mut kDrawFileSlotHelmet_Spritemaps: [uint16; 9] = [
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
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 5 as ::core::ffi::c_int) as isize),
        *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 10 as ::core::ffi::c_int) as isize),
        3584 as uint16,
    );
}
static mut kFileSelectMapFuncs: [Option<Func_V>; 23] = unsafe {
    [
        Some(FileSelectMap_0 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_1 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_3_LoadAreaSelectBackgroundTilemap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_5_ExpandingSquare as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_6_AreaSelectMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_7_PrepExpandSquareTransToRoomMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_8_ExpandSquareTransToRoomSelectMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_9_InitRoomSelectMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_10_RoomSelectMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_11 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_11 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_13 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_14 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_15_ClearTileMap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_16_LoadPalettes as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_18 as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_20_SetupExpandingSquare as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_21_MoveExpandingSquare as unsafe extern "C" fn() -> ()),
        Some(FileSelectMap_22 as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap() {
    kFileSelectMapFuncs[*((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16)
        as usize]
        .expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_22() {
    DrawAreaSelectMapLabels();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_2_GameOptionsMenu as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_1_LoadFileSelectMenuBG2() {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    let menu_tilemap = (&raw mut g_ram as *mut uint8).add(0x3600).cast::<uint16>();
    let mut i: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *menu_tilemap.add(i as usize) = *(g_rom.offset(
            (((0x8edc00 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8edc00 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(i as isize);
        i -= 1;
    }
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    v2 = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh81 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh81 = (*fresh81).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_2_InitMain() {
    let mut v0: uint16 = 0;
    if *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || {
            v0 = *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint16);
            *(g_sram.offset(0x1fee as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                & *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        }
    {
        v0 = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize) as *mut uint16) =
        v0;
    FileSelectMenu_16();
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_16() {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x56 as uint16,
        addr_kMenuTilemap_SamusData as ::core::ffi::c_int as uint16,
    );
    LoadMenuTilemap(
        0x146 as uint16,
        addr_kMenuTilemap_SamusA as ::core::ffi::c_int as uint16,
    );
    let mut c: uint8 = LoadFromSram_(0 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x15c as uint16,
    );
    DrawFileSelectionTime(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x1b4 as uint16,
    );
    LoadMenuTilemap(
        0x176 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x286 as uint16,
        addr_kMenuTilemap_SamusB as ::core::ffi::c_int as uint16,
    );
    c = LoadFromSram_(1 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x29c as uint16,
    );
    DrawFileSelectionTime(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x2f4 as uint16,
    );
    LoadMenuTilemap(
        0x2b6 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x3c6 as uint16,
        addr_kMenuTilemap_SamusC as ::core::ffi::c_int as uint16,
    );
    c = LoadFromSram_(2 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16;
    DrawFileSelectionHealth(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x3dc as uint16,
    );
    DrawFileSelectionTime(
        (*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int) as uint16,
        0x434 as uint16,
    );
    LoadMenuTilemap(
        0x3f6 as uint16,
        addr_kMenuTilemap_TIME as ::core::ffi::c_int as uint16,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16) =
        (swap16(
            !(*((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .palette_index = 0 as uint16;
        LoadMenuTilemap(
            0x508 as uint16,
            addr_kMenuTilemap_DataCopy as ::core::ffi::c_int as uint16,
        );
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .palette_index = 0 as uint16;
        LoadMenuTilemap(
            0x5c8 as uint16,
            addr_kMenuTilemap_DataClear as ::core::ffi::c_int as uint16,
        );
    }
    (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x688 as uint16,
        addr_kMenuTilemap_Exit as ::core::ffi::c_int as uint16,
    );
    QueueTransferOfMenuTilemapToVramBG1();
    *((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x198f as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(6 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(11 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(7 as ::core::ffi::c_int as isize) = 100 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(8 as ::core::ffi::c_int as isize) = 100 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(9 as ::core::ffi::c_int as isize) = 100 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(12 as ::core::ffi::c_int as isize) = 47 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(13 as ::core::ffi::c_int as isize) = 87 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(14 as ::core::ffi::c_int as isize) = 127 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x723 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x725 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    ScreenOn();
    let ref mut fresh47 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh47 = (*fresh47).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(15 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(16 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(17 as ::core::ffi::c_int as isize) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn LoadFromSram_(mut a: uint16) -> uint8 {
    return LoadFromSram(a);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_3_FadeInToMain() {
    DrawFileSelectSamusHelmets();
    let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    DrawMenuSelectionMissile();
    DrawBorderAroundSamusData();
    HandleFadeIn();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        let ref mut fresh80 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh80 = (*fresh80).wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileSelectionHealth(mut a: uint16, mut k: uint16) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    if a != 0 {
        FileSelectClearRestOfMenuTilemapRow(k);
        LoadMenuTilemap(
            (k as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
            addr_kMenuTilemap_NoData as ::core::ffi::c_int as uint16,
        );
    } else {
        LoadMenuTilemap(k, addr_kMenuTilemap_Energy as ::core::ffi::c_int as uint16);
        v2 = (k as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as int16;
        let mut n: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            / 100 as ::core::ffi::c_int;
        let mut q: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            % 100 as ::core::ffi::c_int;
        let mut mh: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9c4 as ::core::ffi::c_int as isize)
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
            *(((&raw mut (*((&raw mut g_ram as *mut uint8)
                .offset(0x3000 as ::core::ffi::c_int as isize)
                as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16)
                .offset(768 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
                ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(0 as ::core::ffi::c_int as isize))
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
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(802 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | q + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(801 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | n + 8288 as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileSelectionTime(mut a: uint16, mut k: uint16) {
    if a == 0 {
        let mut r26: uint16 = k;
        let mut div_val: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9e0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            / 10 as ::core::ffi::c_int;
        let mut mod_val: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9e0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            % 10 as ::core::ffi::c_int;
        let mut v2: uint16 = r26;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(769 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | mod_val + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | div_val + 8288 as ::core::ffi::c_int) as uint16;
        LoadMenuTilemap(
            (r26 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16,
            addr_word_81B4A8 as ::core::ffi::c_int as uint16,
        );
        let mut div_min: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9de as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            / 10 as ::core::ffi::c_int;
        let mut mod_min: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x9de as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            % 10 as ::core::ffi::c_int;
        let mut v3: uint16 = r26;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(772 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | mod_min + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(771 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | div_min + 8288 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMenu_4_Main() {
    let mut current_block: u64;
    let mut v0: int16 = 0;
    let mut v1: uint16 = 0;
    let mut v2: uint16 = 0;
    let mut v3: ::core::ffi::c_int = 0;
    DrawBorderAroundSamusData();
    DrawMenuSelectionMissile();
    DrawFileSelectSlotSamusHelmet(4 as uint16);
    DrawFileSelectSlotSamusHelmet(6 as uint16);
    DrawFileSelectSlotSamusHelmet(8 as uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint16;
                if ((*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v1 = 5 as uint16;
                }
            } else {
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint16;
                if (*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
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
            *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                as *mut uint16) = v1;
            current_block = 14136749492126903395;
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & kButton_B as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x37 as uint16);
                *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 33 as uint16;
                QueueSfx1_Max6(0x37 as uint16);
            }
            current_block = 8908889263641380247;
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0x954 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                v2 = (*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 5 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v2 = 0 as uint16;
                }
            } else {
                v2 = (*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
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
            *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                as *mut uint16) = v2;
            current_block = 14136749492126903395;
        }
        match current_block {
            8908889263641380247 => {}
            _ => {
                QueueSfx1_Max6(0x37 as uint16);
            }
        }
    } else {
        v0 = *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16;
        if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            QueueSfx1_Max6(0x2a as uint16);
            let ref mut fresh77 = *((&raw mut g_ram as *mut uint8)
                .offset(0x727 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh77 = (*fresh77 as ::core::ffi::c_int + 27 as ::core::ffi::c_int) as uint16;
            *(((&raw mut g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (2 as ::core::ffi::c_int
                        * (*((&raw mut g_ram as *mut uint8)
                            .offset(0x952 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int)) as uint16
                        as ::core::ffi::c_int as isize,
                ) as *mut uint16) = 1 as uint16;
            *(g_sram.offset(0x1fec as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
                *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            *(g_sram.offset(0x1fee as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
                !(*((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            RtlWriteSram();
            if LoadFromSram(
                *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            ) != 0
            {
                NewSaveFile();
                *((&raw mut g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            } else {
                LoadMirrorOfExploredMapTiles();
            }
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x37 as uint16);
                let ref mut fresh78 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh78 = (*fresh78).wrapping_add(1);
                *((&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize)
                    as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                    .offset(0x57 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xff0c as ::core::ffi::c_int
                    | 3 as ::core::ffi::c_int) as uint16;
            } else {
                if *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == 4 as ::core::ffi::c_int
                {
                    QueueSfx1_Max6(0x37 as uint16);
                    let ref mut fresh79 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x727 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh79 =
                        (*fresh79 as ::core::ffi::c_int + 15 as ::core::ffi::c_int) as uint16;
                    v0 = (*((&raw mut g_ram as *mut uint8)
                        .offset(0x57 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0xff0c as ::core::ffi::c_int
                        | 3 as ::core::ffi::c_int) as int16;
                    *((&raw mut g_ram as *mut uint8).offset(0x57 as ::core::ffi::c_int as isize)
                        as *mut uint16) = v0 as uint16;
                }
                if v0 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                    *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 33 as uint16;
                }
            }
            return;
        }
    }
    v3 = (4 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x952 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(10 as ::core::ffi::c_int as isize) = kMenuSelectionMissileXY[v3 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(5 as ::core::ffi::c_int as isize) =
        kMenuSelectionMissileXY[(v3 + 1 as ::core::ffi::c_int) as usize];
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_0() {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    ClearMenuTilemap();
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc400 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    LoadFileSelectPalettes();
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(14 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(30 as ::core::ffi::c_int as isize) = 0 as uint16;
    let ref mut fresh43 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh43 = (*fresh43).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_1() {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    if AdvanceGradualColorChangeOfPalette(0 as uint16, 0x40 as uint16) != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
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
        *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        v1 = 0 as int16;
        loop {
            v2 = v1;
            if v1 as ::core::ffi::c_int
                == *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                LoadActiveAreaMapForegroundColors(v1 as uint16);
            } else {
                LoadInactiveAreaMapForegroundColors(v1 as uint16);
            }
            v1 += 1;
            if !(((v2 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let ref mut fresh42 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh42 = (*fresh42).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0x787 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 2 as uint8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn sub_81A3D1(mut k: uint16) {
    LoadActiveAreaMapForegroundColors(k);
}
static mut kActiveAreaMapForegroundColors: [uint16; 6] = [
    0 as ::core::ffi::c_int as uint16,
    0xa as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    0x16 as ::core::ffi::c_int as uint16,
    0x24 as ::core::ffi::c_int as uint16,
    0x2a as ::core::ffi::c_int as uint16,
];
static mut kInactiveAreaMapForegroundColors: [uint16; 6] = [
    0x30 as ::core::ffi::c_int as uint16,
    0x3a as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0x46 as ::core::ffi::c_int as uint16,
    0x54 as ::core::ffi::c_int as uint16,
    0x5a as ::core::ffi::c_int as uint16,
];
#[no_mangle]
pub unsafe extern "C" fn LoadActiveAreaMapForegroundColors(mut v0: uint16) {
    LoadAreaMapForegroundColors(kActiveAreaMapForegroundColors[v0 as usize]);
}
#[no_mangle]
pub unsafe extern "C" fn LoadInactiveAreaMapForegroundColors(mut v0: uint16) {
    LoadAreaMapForegroundColors(kInactiveAreaMapForegroundColors[v0 as usize]);
}
#[no_mangle]
pub unsafe extern "C" fn LoadAreaMapForegroundColors(mut j: uint16) {
    loop {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(g_rom.offset(
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
        let mut v2: uint16 = *(g_rom.offset(
            (((0x81a4e6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81a4e6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        let mut v3: uint16 = *(g_rom.offset(
            (((0x81a4e6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x81a4e6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        let mut n: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
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
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_2_LoadAreaSelectForegroundTilemap() {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr = addr_kAreaSelectForegroundTilemap as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 129 as uint16;
    (*v1).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*(&raw mut g_ram as *mut uint8).offset(0x52 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | 0xf as ::core::ffi::c_int) as uint16;
    let ref mut fresh18 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh18 = (*fresh18).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_18() {
    let ref mut fresh17 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh17 = (*fresh17).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_3_LoadAreaSelectBackgroundTilemap() {
    let ref mut fresh41 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh41 = (*fresh41).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn LoadAreaSelectBackgroundTilemap(mut j: uint16) {
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr = (8 as ::core::ffi::c_int * 256 as ::core::ffi::c_int * j as ::core::ffi::c_int
        + addr_kAreaSelectBackgroundTilemaps as ::core::ffi::c_int) as VoidP;
    (*v2).src.bank = 0x81 as uint8;
    (*v2).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x5a as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_4_SetupExpandingSquareTransition() {
    *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 4 as uint8;
    WriteReg(TS as ::core::ffi::c_int as uint16, 4 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 4 as uint8;
    WriteReg(TSW as ::core::ffi::c_int as uint16, 4 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x6f as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(CGWSEL as ::core::ffi::c_int as uint16, 2 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x72 as ::core::ffi::c_int as isize) = 37 as uint8;
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 0x25 as uint8);
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0x80 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 64 as uint8;
    *(&raw mut g_ram as *mut uint8)
        .offset(0x74 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 32 as uint8;
    SetupInitialExpandingSquareHDMA();
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8);
    ConfigureWindow1ForExpandingSquare();
    *((&raw mut g_ram as *mut uint8).offset(0x18b0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn ConfigureWindow1ForExpandingSquare() {
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    WriteReg(TM as ::core::ffi::c_int as uint16, 0x13 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 19 as uint8;
    WriteReg(TMW as ::core::ffi::c_int as uint16, 0x13 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 35 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x23 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 3 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 3 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 35 as uint8;
    WriteReg(WOBJSEL as ::core::ffi::c_int as uint16, 0x23 as uint8);
}
#[no_mangle]
pub unsafe extern "C" fn SetupInitialExpandingSquareHDMA() {
    *((&raw mut g_ram as *mut uint8).offset(0x9e20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e22 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = 127 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = -(127 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 = 111 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_1 = 34 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_0 = 1 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_1 = 50 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_0 = 1 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_1 = 50 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 111 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_1 = 32 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP2 as ::core::ffi::c_int as uint16, 0x40 as uint8);
    WriteReg(BBAD2 as ::core::ffi::c_int as uint16, 0x26 as uint8);
    WriteReg(A1T2L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A1T2H as ::core::ffi::c_int as uint16, 0x9e as uint8);
    WriteReg(A1B2 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(DAS20 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(DAS2L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(DAS2H as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A2A2L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A2A2H as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(NTRL2 as ::core::ffi::c_int as uint16, 0 as uint8);
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 = 111 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_1 = 32 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_0 = 1 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_1 = 54 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_0 = 1 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_1 = 54 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(2 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 111 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_1 = 32 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP3 as ::core::ffi::c_int as uint16, 0x40 as uint8);
    WriteReg(BBAD3 as ::core::ffi::c_int as uint16, 0x27 as uint8);
    WriteReg(A1T3L as ::core::ffi::c_int as uint16, 0x10 as uint8);
    WriteReg(A1T3H as ::core::ffi::c_int as uint16, 0x9e as uint8);
    WriteReg(A1B3 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(DAS30 as ::core::ffi::c_int as uint16, 0x7e as uint8);
    WriteReg(DAS3L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(DAS3H as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A2A3L as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(A2A3H as ::core::ffi::c_int as uint16, 0 as uint8);
    WriteReg(NTRL3 as ::core::ffi::c_int as uint16, 0 as uint8);
}
static mut kExpandingSquareTransitionSpeed: uint8 = 4 as uint8;
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_5_ExpandingSquare() {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    DrawAreaSelectMapLabels();
    let mut v0: uint8 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as ::core::ffi::c_int
        - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
    if (((*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as ::core::ffi::c_int
        - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as int8
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
        let ref mut fresh36 =
            *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize);
        *fresh36 = (*fresh36 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 0 as uint8;
        *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = 24 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x73b as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[v1 as usize] = *(g_rom.offset(
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
            (*((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[fresh37 as usize] = 10241 as uint16;
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*((&raw mut g_ram as *mut uint8)
                .offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .bg2_room_select_map_tilemap as *mut uint16)
                .offset(170 as ::core::ffi::c_int as isize),
        );
        let mut v2: uint16 = 320 as uint16;
        let mut v3: uint16 = 959 as uint16;
        loop {
            let fresh38 = v3;
            v3 = v3.wrapping_sub(1);
            (*((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
                as *mut Ram4000))
                .xray_tilemaps[fresh38 as usize] = *(g_rom.offset(
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
        let mut v4: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v5 = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v5).size = 2048 as uint16;
        (*v5).src.addr = ((&raw mut g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize) as *mut Ram4000
            as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8)
            as ::core::ffi::c_long as VoidP;
        (*v5).src.bank = 126 as uint8;
        (*v5).vram_dst = ((*(&raw mut g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v4 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh39 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0;
        *fresh39 = (*fresh39 as ::core::ffi::c_int
            - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3 as ::core::ffi::c_int as isize))
        .field_0 = v0;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0 = v0;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3 as ::core::ffi::c_int as isize))
        .field_0 = v0;
        let ref mut fresh40 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
        .field_0;
        *fresh40 = (*fresh40 as ::core::ffi::c_int
            + kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2 as ::core::ffi::c_int as isize))
        .field_0 = (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
        .field_0;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
        .field_0 = (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
        .field_0;
        (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2 as ::core::ffi::c_int as isize))
        .field_0 = (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
        .field_0;
        *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - kExpandingSquareTransitionSpeed as ::core::ffi::c_int) as uint8;
        *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) =
            (kExpandingSquareTransitionSpeed as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_6_AreaSelectMap() {
    let mut current_block: u64;
    let mut v0: int16 = 0;
    v0 = *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as int16;
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Left as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        v0 = *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            let mut R28: uint16 = DecAndWraparoundTo5(
                *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            if CheckIfFileSelectMapAreaCanBeSelected(R28) as ::core::ffi::c_int != 0
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28) as ::core::ffi::c_int != 0
                }
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28) as ::core::ffi::c_int != 0
                }
            {
                QueueSfx1_Max6(0x37 as uint16);
                SwitchActiveFileSelectMapArea(R28);
                DrawAreaSelectMapLabels();
                return;
            }
            current_block = 4115184416633590978;
        } else {
            current_block = 9480232856434185415;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & (kButton_Select as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
    {
        current_block = 9480232856434185415;
    } else {
        v0 = *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            current_block = 9480232856434185415;
        } else {
            let mut n: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
            let mut R28_0: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0x950 as ::core::ffi::c_int as isize)
                as *mut uint16);
            loop {
                R28_0 = WraparoundFrom6to0(R28_0);
                if CheckIfFileSelectMapAreaCanBeSelected(R28_0) != 0 {
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
                    SwitchActiveFileSelectMapArea(R28_0);
                    QueueSfx1_Max6(0x37 as uint16);
                    DrawAreaSelectMapLabels();
                    return;
                }
            }
        }
    }
    match current_block {
        9480232856434185415 => {
            if (v0 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 22 as uint16;
                DrawAreaSelectMapLabels();
                return;
            }
            if v0 as ::core::ffi::c_int
                & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x38 as uint16);
                SelectFileSelectMapArea();
                return;
            }
        }
        _ => {}
    }
    DrawAreaSelectMapLabels();
}
#[no_mangle]
pub unsafe extern "C" fn DecAndWraparoundTo5(mut a: uint16) -> uint16 {
    let mut result: uint16 = (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    if result as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        return 5 as uint16;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn WraparoundFrom6to0(mut a: uint16) -> uint16 {
    let mut result: uint16 = (a as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if result as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        return 0 as uint16;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn SelectFileSelectMapArea() {
    let mut v2: int8 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let ref mut fresh35 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh35 = (*fresh35).wrapping_add(1);
    if *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xd918 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xd916 as ::core::ffi::c_int as isize)
            as *mut uint16);
        DrawAreaSelectMapLabels();
        return;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16) =
        kFileSelectMap_AreaIndexes[*((&raw mut g_ram as *mut uint8)
            .offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16) as usize];
    let mut q: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xd8f8 as ::core::ffi::c_int as isize)
        .offset(
            (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as isize,
        ) as *mut uint8 as *mut uint16);
    let mut v0: uint16 = 0 as uint16;
    let mut r0: *const uint16 = RomPtr_82(
        *(RomPtr_82(
            (addr_kMapIconDataPointers as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16_t,
        ) as *mut uint16),
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
                Unreachable();
            }
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize) as *mut uint16) =
        v0;
    DrawAreaSelectMapLabels();
}
#[no_mangle]
pub unsafe extern "C" fn CheckIfFileSelectMapAreaCanBeSelected(mut a: uint16) -> uint16 {
    let mut v1: ::core::ffi::c_int =
        2 as ::core::ffi::c_int * kFileSelectMap_AreaIndexes[a as usize] as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8)
        .offset(0xd8f8 as ::core::ffi::c_int as isize)
        .offset(v1 as isize) as *mut uint8 as *mut uint16)
        != 0
    {
        return true_0 as uint16;
    }
    let mut t: uint16 = (*((&raw mut (*(g_rom.offset(
        (((0x82c7cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x82c7cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MapIconDataPointers)
        .offset(4 as ::core::ffi::c_int as isize))
    .crateria as *mut uint8)
        .offset(v1 as isize) as *mut uint16) as ::core::ffi::c_int
        + 64 as ::core::ffi::c_int) as uint16;
    return (t as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int) as ::core::ffi::c_int
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SwitchActiveFileSelectMapArea(mut R28: uint16) {
    LoadInactiveAreaMapForegroundColors(
        kFileSelectMap_AreaIndexes[*((&raw mut g_ram as *mut uint8)
            .offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16) as usize],
    );
    *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize) as *mut uint16) =
        R28;
    LoadActiveAreaMapForegroundColors(kFileSelectMap_AreaIndexes[R28 as usize]);
    LoadAreaSelectBackgroundTilemap(
        kFileSelectMap_AreaIndexes[*((&raw mut g_ram as *mut uint8)
            .offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16) as usize],
    );
}
static mut kAreaSelectMapLabelPositions: [uint16; 12] = [
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
#[no_mangle]
pub unsafe extern "C" fn DrawAreaSelectMapLabels() {
    let mut r3: uint16 = 0 as uint16;
    DrawMenuSpritemap(
        *(g_rom.offset(
            (((0x82c749 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x82c749 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize),
        0x80 as uint16,
        0x10 as uint16,
        r3,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        r3 = (if i
            == *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            512 as ::core::ffi::c_int
        }) as uint16;
        let mut v1: uint16 = (2 as ::core::ffi::c_int
            * kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int)
            as uint16;
        let mut r36: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xd8f8 as ::core::ffi::c_int as isize)
            .offset(v1 as isize) as *mut uint8 as *mut uint16);
        let mut v2: *const uint16 = RomPtr_82(
            *((&raw mut (*(g_rom.offset(
                (((0x82c7cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x82c7cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut MapIconDataPointers)
                .offset(4 as ::core::ffi::c_int as isize))
            .crateria as *mut uint8)
                .offset(v1 as ::core::ffi::c_int as isize) as *mut VoidP),
        ) as *const uint16;
        let mut R30: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        while *v2 as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
            let mut v4: ::core::ffi::c_int = r36 as ::core::ffi::c_int & 1 as ::core::ffi::c_int;
            r36 = (r36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            if !(v4 != 0 && *v2 as ::core::ffi::c_int != 0xfffe as ::core::ffi::c_int) {
                v2 = v2.offset(2 as ::core::ffi::c_int as isize);
                R30 -= 1;
                if !(R30 == 0) {
                    continue;
                }
                if !(*((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
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
                (*(g_rom.offset(
                    (((0x82c749 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x82c749 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16,
                kAreaSelectMapLabelPositions[j as usize],
                kAreaSelectMapLabelPositions[(j + 1 as ::core::ffi::c_int) as usize],
                r3,
            );
            break;
        }
        i += 1;
    }
}
static mut kRoomSelectMapExpandingSquareTimers: [uint16; 6] = [
    0x33 as ::core::ffi::c_int as uint16,
    0x35 as ::core::ffi::c_int as uint16,
    0x2d as ::core::ffi::c_int as uint16,
    0x33 as ::core::ffi::c_int as uint16,
    0x33 as ::core::ffi::c_int as uint16,
    0x22 as ::core::ffi::c_int as uint16,
];
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_7_PrepExpandSquareTransToRoomMap() {
    DrawAreaSelectMapLabels();
    SetupInitialExpandingSquareHDMA();
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 19 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 50 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x32 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 2 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x72 as ::core::ffi::c_int as isize) = 5 as uint8;
    WriteReg(CGADSUB as ::core::ffi::c_int as uint16, 5 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 34 as uint8;
    WriteReg(WOBJSEL as ::core::ffi::c_int as uint16, 0x22 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 48 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 0 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 0 as uint8;
    DrawRoomSelectMapAreaLabel(
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000))
            .bg2_room_select_map_tilemap as *mut uint16)
            .offset(170 as ::core::ffi::c_int as isize),
    );
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).size = 512 as uint16;
    (*v1).src.addr = ((&raw mut g_ram as *mut uint8).offset(0x4000 as ::core::ffi::c_int as isize)
        as *mut Ram4000 as *mut uint8)
        .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long
        as VoidP;
    (*v1).src.bank = 126 as uint8;
    (*v1).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e50 as ::core::ffi::c_int as isize)
        as *mut uint16) = kRoomSelectMapExpandingSquareTimers[*((&raw mut g_ram as *mut uint8)
        .offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as usize];
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
        as *mut uint16) = kAreaSelectMapLabelPositions[v2 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x9e32 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
        as *mut uint16) = kAreaSelectMapLabelPositions[(v2 + 1 as ::core::ffi::c_int) as usize];
    *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0x9e3a as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e30 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e34 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e38 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e3c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut vels: *const ExpandingSquareVels = (g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut ExpandingSquareVels;
    *((&raw mut g_ram as *mut uint8).offset(0x9e40 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).left_subvel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e42 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).left_vel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e44 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).right_subvel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e46 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).right_vel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e48 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).top_subvel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e4a as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).top_vel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e4c as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).bottom_subvel;
    *((&raw mut g_ram as *mut uint8).offset(0x9e4e as ::core::ffi::c_int as isize)
        as *mut uint16) = (*vels).bottom_vel;
    SetupRoomSelectMapExpandingSquareTransHDMA();
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8);
    let ref mut fresh34 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh34 = (*fresh34).wrapping_add(1);
    QueueSfx1_Max6(0x3b as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn SetupRoomSelectMapExpandingSquareTransHDMA() {
    let mut k: uint16 = 0 as uint16;
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9e3a as ::core::ffi::c_int as isize) as *mut uint16);
    AddExpandingSqTransLeftIndirHDMA(v0, k, 0x9e22 as uint16);
    k = AddExpandingSqTransRightIndirHDMA(v0, k, 0x9e20 as uint16);
    let mut v2: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x9e3e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
    {
        v2 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v2, k, 0x9e32 as uint16);
    k = AddExpandingSqTransRightIndirHDMA(v2, k, 0x9e36 as uint16);
    let mut v4: uint16 = (-(32 as ::core::ffi::c_int)
        - *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        == 0xe0 as ::core::ffi::c_int
    {
        v4 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v4, k, 0x9e22 as uint16);
    k = AddExpandingSqTransRightIndirHDMA(v4, k, 0x9e20 as uint16);
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn AddExpandingSqTransLeftIndirHDMA(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
) {
    if a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = (a as ::core::ffi::c_int
            - 127 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    } else {
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    };
}
#[no_mangle]
pub unsafe extern "C" fn AddExpandingSqTransRightIndirHDMA(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
) -> uint16 {
    if a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = (a as ::core::ffi::c_int
            - 127 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        return (k as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    } else {
        *(&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut g_ram as *mut uint8)
            .offset(0x9e10 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
            .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        return (k as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap() {
    if HandleRoomSelectMapExpandingSquareTrans() as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh33 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh33 = (*fresh33).wrapping_add(1);
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 2 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 0 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6b as ::core::ffi::c_int as isize) = 0 as uint8;
    }
    DrawAreaSelectMapLabels();
}
#[no_mangle]
pub unsafe extern "C" fn HandleRoomSelectMapExpandingSquareTrans() -> uint16 {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e30 as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x9e42 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e40 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e34 as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x9e46 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e44 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 255 as uint16;
    }
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e38 as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x9e4a as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e48 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e3c as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x9e4e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e4c as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 224 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
            as *mut uint16) = 224 as uint16;
    }
    SetupRoomSelectMapExpandingSquareTransHDMA();
    let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9e50 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_sub(1);
    return *fresh0;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_9_InitRoomSelectMap() {
    LoadMirrorOfExploredMapTiles();
    DrawRoomSelectMap();
    LoadFromLoadStation();
    DisableHdmaObjects();
    WaitUntilEndOfVblankAndClearHdma();
    let mut RoomDefHeader: *mut RoomDefHeader = get_RoomDefHeader(
        *((&raw mut g_ram as *mut uint8).offset(0x79b as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*RoomDefHeader).area_index_;
    *((&raw mut g_ram as *mut uint8).offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*RoomDefHeader).x_coordinate_on_map;
    *((&raw mut g_ram as *mut uint8).offset(0x7a3 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = (*RoomDefHeader).y_coordinate_on_map;
    SetupMapScrollingForFileSelectMap();
    let ref mut fresh30 = *((&raw mut g_ram as *mut uint8)
        .offset(0x5b0 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh30 = (*fresh30 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        24 as uint16;
    let ref mut fresh31 = *((&raw mut g_ram as *mut uint8)
        .offset(0x69 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh31 = (*fresh31 as ::core::ffi::c_int & !(4 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh32 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh32 = (*fresh32).wrapping_add(1);
    *((&raw mut g_ram as *mut uint8).offset(0x5fd as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5ff as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x776 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x778 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x77a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_10_RoomSelectMap() {
    let mut v1: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int8 = 0;
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    DrawFileSelectMapIcons();
    HandleFileSelectMapScrollArrows();
    MapScrolling();
    DisplayMapElevatorDestinations();
    if *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Select as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let mut current_block_32: u64;
        QueueSfx1_Max6(0x38 as uint16);
        let mut r24: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xd8f8 as ::core::ffi::c_int as isize)
            .offset(
                (2 as ::core::ffi::c_int
                    * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16);
        v1 = *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
            as *mut uint16) as int16;
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
                        * *((&raw mut g_ram as *mut uint8)
                            .offset(0x79f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16_t,
            ) as *mut uint16),
        ) as *const uint16;
        v3 = (4 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        let mut r18: uint16 =
            *r0.offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut r20: uint16 = *r0.offset(
            ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
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
                    let ref mut fresh26 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x78b as ::core::ffi::c_int as isize)
                        as *mut uint16);
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
                    let ref mut fresh27 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x78b as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh27 = (*fresh27).wrapping_add(1);
                    current_block_32 = 8236137900636309791;
                }
            }
            match current_block_32 {
                8236137900636309791 => {
                    v5 = *r0.offset(
                        (2 as ::core::ffi::c_int
                            * *((&raw mut g_ram as *mut uint8)
                                .offset(0x78b as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int)
                            as isize,
                    ) as int16;
                    if v5 as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x78b as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                        r24 = *((&raw mut g_ram as *mut uint8)
                            .offset(0xd8f8 as ::core::ffi::c_int as isize)
                            .offset(
                                (2 as ::core::ffi::c_int
                                    * *((&raw mut g_ram as *mut uint8)
                                        .offset(0x79f as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int)
                                    as isize,
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
                        * *((&raw mut g_ram as *mut uint8)
                            .offset(0x78b as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                    >= 0xfffe as ::core::ffi::c_int
            {
                current_block_32 = 12800627514080957624;
            } else {
                break;
            }
        }
        let mut v6: uint16 = (4 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut w7: uint16 = *r0.offset(
            (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x78b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as isize,
        );
        if w7 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || w7 as ::core::ffi::c_int
                - 256 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            v8 = (*((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *r0.offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as int16;
            if v8 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if (v8 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x5ac as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    v8 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x5ac as ::core::ffi::c_int as isize)
                        as *mut uint16) as int16;
                }
            } else {
                v8 = 0 as int16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
                as *mut uint16) = v8 as uint16;
        }
        let mut v9: uint16 = (v6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let mut v10: uint16 =
            *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        if v10 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || v10 as ::core::ffi::c_int
                - 161 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            let mut v11: uint16 = (*((&raw mut g_ram as *mut uint8)
                .offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int) as uint16;
            if (v11 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x5b0 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v11 = *((&raw mut g_ram as *mut uint8).offset(0x5b0 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            }
            *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) = v11;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh28 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh28 = (*fresh28 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kFileSelectMap_AreaIndexes
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if (v0 as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                return;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x950 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x3c as uint16);
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh29 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
        QueueSfx1_Max6(0x38 as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn sub_81AEC8() {
    HandleFileSelectMapScrollArrows();
}
#[no_mangle]
pub unsafe extern "C" fn HandleFileSelectMapScrollArrows() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5ac as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kLeftMapScrollArrowData as ::core::ffi::c_int as uint16,
        );
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x5ae as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 232 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kRightMapScrollArrowData as ::core::ffi::c_int as uint16,
        );
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x5b0 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kUpMapScrollArrowData as ::core::ffi::c_int as uint16,
        );
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x5b2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 145 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x5fd as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == (*(g_rom.offset(
                (((0x81af50 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x81af50 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut MapScrollArrowData))
                .map_scroll_dir as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0x5fb as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x5fd as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x5ff as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kDownMapScrollArrowData as ::core::ffi::c_int as uint16,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_11() {
    DrawFileSelectMapIcons();
    DisplayMapElevatorDestinations();
    let ref mut fresh25 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh25 = (*fresh25).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_13() {
    DrawFileSelectMapIcons();
    DisplayMapElevatorDestinations();
    HandleFadeOut();
    if *(&raw mut g_ram as *mut uint8).offset(0x51 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh24 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh24 = (*fresh24).wrapping_add(1);
        (*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos = 32 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_14() {
    let fresh22 = &raw mut (*((&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos;
    fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
    if fresh22.read_unaligned() == 0 {
        ScreenOff();
        let ref mut fresh23 = *((&raw mut g_ram as *mut uint8)
            .offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh23 = (*fresh23).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_15_ClearTileMap() {
    *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 18 as uint8;
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint8)
            .offset_from(&raw mut g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x58 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh21 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh21 = (*fresh21).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_16_LoadPalettes() {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    LoadMenuPalettes();
    v0 = 0 as int16;
    loop {
        v1 = v0;
        let fresh19 = v0;
        v0 = v0 + 1;
        LoadInactiveAreaMapForegroundColors(fresh19 as uint16);
        if !(((v1 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    LoadActiveAreaMapForegroundColors(
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    LoadAreaSelectBackgroundTilemap(
        *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let ref mut fresh20 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh20 = (*fresh20).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_20_SetupExpandingSquare() {
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 0 as uint8;
    QueueSfx1_Max6(0x3c as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e50 as ::core::ffi::c_int as isize)
        as *mut uint16) = (kRoomSelectMapExpandingSquareTimers[*((&raw mut g_ram as *mut uint8)
        .offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as usize] as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e40 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .left_subvel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e42 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .left_vel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e44 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .right_subvel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e46 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .right_vel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e48 as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .top_subvel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e4a as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .top_vel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e4c as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .bottom_subvel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e4e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut (*(g_rom.offset(
        (((0x81aa34 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x81aa34 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut ExpandingSquareVels)
        .offset(0 as ::core::ffi::c_int as isize))
    .bottom_vel as *mut uint8)
        .offset(
            (16 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int as isize,
        ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize)
        as *mut uint16) = 248 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize)
        as *mut uint16) = 216 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e30 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e34 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e38 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e3c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SetupRoomSelectMapExpandingSquareTransHDMA();
    let ref mut fresh16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x727 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh16 = (*fresh16).wrapping_add(1);
    *(&raw mut g_ram as *mut uint8).offset(0x85 as ::core::ffi::c_int as isize) = 12 as uint8;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0xc as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x60 as ::core::ffi::c_int as isize) = 50 as uint8;
    WriteReg(W12SEL as ::core::ffi::c_int as uint16, 0x32 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x61 as ::core::ffi::c_int as isize) = 2 as uint8;
    WriteReg(W34SEL as ::core::ffi::c_int as uint16, 2 as uint8);
    *(&raw mut g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 48 as uint8;
    *(&raw mut g_ram as *mut uint8).offset(0x62 as ::core::ffi::c_int as isize) = 34 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e00 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 0 as uint8;
    (*((&raw mut g_ram as *mut uint8).offset(0x9e10 as ::core::ffi::c_int as isize)
        as *mut ExpandingSquareTransitionHdma)
        .offset(3 as ::core::ffi::c_int as isize))
    .field_0 = 0 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb1 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FileSelectMap_21_MoveExpandingSquare() {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e32 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e30 as ::core::ffi::c_int as isize) as *mut uint16,
        -(((*((&raw mut g_ram as *mut uint8).offset(0x9e42 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e40 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e36 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e34 as ::core::ffi::c_int as isize) as *mut uint16,
        -(((*((&raw mut g_ram as *mut uint8).offset(0x9e46 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e44 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e3a as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e38 as ::core::ffi::c_int as isize) as *mut uint16,
        -(((*((&raw mut g_ram as *mut uint8).offset(0x9e4a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e48 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
    );
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9e3e as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0x9e3c as ::core::ffi::c_int as isize) as *mut uint16,
        -(((*((&raw mut g_ram as *mut uint8).offset(0x9e4e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9e4c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
    );
    SetupRoomSelectMapExpandingSquareTransHDMA();
    DrawAreaSelectMapLabels();
    let ref mut fresh14 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9e50 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh14 = (*fresh14).wrapping_sub(1);
    if *fresh14 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let ref mut fresh15 = *((&raw mut g_ram as *mut uint8)
            .offset(0x727 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as uint16;
        *(&raw mut g_ram as *mut uint8).offset(0x69 as ::core::ffi::c_int as isize) = 17 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6c as ::core::ffi::c_int as isize) = 0 as uint8;
        *(&raw mut g_ram as *mut uint8).offset(0x6d as ::core::ffi::c_int as isize) = 0 as uint8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn NewSaveFile() {
    *((&raw mut g_ram as *mut uint8).offset(0x9c4 as ::core::ffi::c_int as isize) as *mut uint16) =
        99 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16) =
        99 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9a8 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9a4 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9c0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d4 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d8 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9aa as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Up as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ac as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Down as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ae as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Left as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9b0 as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Right as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_A as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_B as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_X as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Y as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ba as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_Select as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9be as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_R as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9bc as ::core::ffi::c_int as isize) as *mut uint16) =
        kButton_L as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9da as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9dc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9de as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e4 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9ea as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9e8 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd830 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd870 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd870 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd8b0 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xd8f0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd908 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd8f8 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd8f8 as ::core::ffi::c_int as isize)
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
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd830 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd870 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd870 as ::core::ffi::c_int as isize)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xd8b0 as ::core::ffi::c_int as isize)
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
        *((&raw mut g_ram as *mut uint8).offset(0xcd52 as ::core::ffi::c_int as isize)
            as *mut uint16)
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
#[no_mangle]
pub unsafe extern "C" fn FileSelectClearRestOfMenuTilemapRow(mut v0: uint16) {
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int
        - ((v0 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int);
    loop {
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadMenuTilemap(mut k: uint16, mut j: uint16) {
    let mut v2: int16 = 0;
    let mut k_bak: ::core::ffi::c_int = k as ::core::ffi::c_int;
    loop {
        loop {
            v2 = *(RomPtr_81(j as uint16_t) as *mut uint16) as int16;
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
        *(((&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x3000 as ::core::ffi::c_int as isize)
            as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .palette_index as ::core::ffi::c_int
                | v2 as ::core::ffi::c_int) as uint16;
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
