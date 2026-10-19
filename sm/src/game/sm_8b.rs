pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_91::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    memset7E, CheckEventHappened, ClearOamExt, ClearUnusedOam, CopySuperMetroidString,
    DecompressToMem, DisableIrqInterrupts, EnableNMI, HandleFadeIn, HandleFadeOut, HasQueuedMusic,
    QueueMode7Transfers, QueueMusic_Delayed8, QueueMusic_DelayedY, QueueSfx1_Max15, QueueSfx2_Max6,
    QueueSfx3_Max6, SetupDmaTransfer, WaitForNMI_Async,
};
use crate::game::sm_81::{DrawSpritemap, DrawSpritemapOffScreen, SaveToSram};
use crate::game::sm_88::{
    CinematicFunction_Intro_Func133, DisableHdmaObjects, EnableHdmaObjects,
    SpawnIntroCutsceneCrossfadeHdma, SpawnPowerBombExplosion, SpawnTitleScreenGradientObjs,
    WaitUntilEndOfVblankAndClearHdma,
};
use crate::game::sm_8d::{
    ClearPaletteFXObjects, DisablePaletteFx, EnablePaletteFx, PaletteFxHandler, SpawnPalfxObject,
};
use crate::game::sm_90::{
    DrawSamusAndProjectiles, HandleControllerInputForGamePhysics, HandleSamusMovementAndPause,
    KillProjectile, UpdateBeamTilesAndPalette,
};
use crate::game::sm_91::{
    EnableDemoInput, LoadDemoInputObject, ResetDemoData, SamusFunc_F433, Samus_Initialize,
    Samus_SetAnimationFrameIfPoseChanged,
};
use crate::game::sm_93::DrawPlayerExplosions2;
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game_state::Game;
use crate::rtl::AddToHiLo;
use crate::rtl::{Mult8x8, RomPtr, SnesDivide, Unreachable, WriteReg, WriteRegWord};
pub use crate::snes::registers::*;
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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t,
    CoroutineRet, Func_V, LongPtr, OamEnt, Point16U, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7ObjectDef {
    pub field_0: VoidP,
    pub field_2: VoidP,
    pub field_4: VoidP,
}
pub use crate::types::Buttons;
pub use crate::types::Consts_8B;
pub use crate::types::SamusPose;
pub use crate::types::{
    Consts_60, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram3800, Ram4000,
    Ram4000_Backups, StartDmaCopy,
};
pub use crate::types::{GameState, Mode7VramWriteQueue};
pub const addr_kCinematicBgObjectDef_8BF74E: Consts_8B = 63310;
pub const addr_kCinematicBgObjectDef_8BF748: Consts_8B = 63304;
pub const addr_stru_8BF6F8: Consts_8B = 63224;
pub const addr_kCinematicSpriteObjectDef_8BEF93: Consts_8B = 61331;
pub const addr_kCinematicSpriteObjectDef_8BEF8D: Consts_8B = 61325;
pub const addr_kCinematicSpriteObjectDef_8BEF87: Consts_8B = 61319;
pub const addr_kCinematicSpriteObjectDef_8BEF81: Consts_8B = 61313;
pub const addr_kCinematicSpriteObjectDef_8BEF7B: Consts_8B = 61307;
pub const addr_kCinematicSpriteObjectDef_8BEF75: Consts_8B = 61301;
pub const addr_kCinematicSpriteObjectDef_8BEF6F: Consts_8B = 61295;
pub const addr_kCinematicSpriteObjectDef_8BEF69: Consts_8B = 61289;
pub const addr_kCinematicSpriteObjectDef_8BEF63: Consts_8B = 61283;
pub const addr_kCinematicSpriteObjectDef_8BEF5D: Consts_8B = 61277;
pub const addr_kCinematicSpriteObjectDef_8BEF57: Consts_8B = 61271;
pub const addr_kCinematicSpriteObjectDef_8BEF51: Consts_8B = 61265;
pub const addr_kCinematicSpriteObjectDef_8BEF4B: Consts_8B = 61259;
pub const addr_kCinematicSpriteObjectDef_8BEF45: Consts_8B = 61253;
pub const addr_kCinematicSpriteObjectDef_8BEF3F: Consts_8B = 61247;
pub const addr_kCinematicSpriteObjectDef_8BEF39: Consts_8B = 61241;
pub const addr_kCinematicSpriteObjectDef_8BEF33: Consts_8B = 61235;
pub const addr_kCinematicSpriteObjectDef_8BEF2D: Consts_8B = 61229;
pub const addr_kCinematicSpriteObjectDef_8BEF27: Consts_8B = 61223;
pub const addr_kCinematicSpriteObjectDef_8BEF1B: Consts_8B = 61211;
pub const addr_kCinematicSpriteObjectDef_8BEF15: Consts_8B = 61205;
pub const addr_kCinematicSpriteObjectDef_8BEF0F: Consts_8B = 61199;
pub const addr_kCinematicSpriteObjectDef_8BEF09: Consts_8B = 61193;
pub const addr_kCinematicSpriteObjectDef_8BEF03: Consts_8B = 61187;
pub const addr_kCinematicSpriteObjectDef_8BEEFD: Consts_8B = 61181;
pub const addr_kCinematicSpriteObjectDef_8BEEF7: Consts_8B = 61175;
pub const addr_kCinematicSpriteObjectDef_8BEEF1: Consts_8B = 61169;
pub const addr_kCinematicSpriteObjectDef_8BEEEB: Consts_8B = 61163;
pub const addr_kCinematicSpriteObjectDef_8BEEE5: Consts_8B = 61157;
pub const addr_kCinematicSpriteObjectDef_8BEEDF: Consts_8B = 61151;
pub const addr_kCinematicSpriteObjectDef_8BEED9: Consts_8B = 61145;
pub const addr_kCinematicSpriteObjectDef_8BEED3: Consts_8B = 61139;
pub const addr_kCinematicSpriteObjectDef_8BEECD: Consts_8B = 61133;
pub const addr_kCinematicSpriteObjectDef_8BEEC7: Consts_8B = 61127;
pub const addr_kCinematicSpriteObjectDef_8BEEC1: Consts_8B = 61121;
pub const addr_kCinematicSpriteObjectDef_8BEEBB: Consts_8B = 61115;
pub const addr_kCinematicSpriteObjectDef_8BEEB5: Consts_8B = 61109;
pub const addr_kCinematicSpriteObjectDef_8BEEAF: Consts_8B = 61103;
pub const addr_kCinematicSpriteObjectDef_8BEEA9: Consts_8B = 61097;
pub const addr_kCinematicSpriteObjectDef_8BEEA3: Consts_8B = 61091;
pub const addr_kCinematicSpriteObjectDef_8BEE9D: Consts_8B = 61085;
pub const addr_off_8BEE9B: Consts_8B = 61083;
pub const addr_word_8BED9D: Consts_8B = 60829;
pub const addr_word_8BED95: Consts_8B = 60821;
pub const addr_loc_8BE1B4: Consts_8B = 57780;
pub const addr_kMode7ObjectDef_8BD437: Consts_8B = 54327;
pub const addr_kMode7ObjectDef_8BD431: Consts_8B = 54321;
pub const addr_kMode7ObjectDef_8BD42B: Consts_8B = 54315;
pub const addr_kMode7ObjectDef_8BD425: Consts_8B = 54309;
pub const addr_kMode7ObjectDef_8BD41F: Consts_8B = 54303;
pub const addr_kMode7ObjectDef_8BD419: Consts_8B = 54297;
pub const addr_kMode7ObjectDef_8BD413: Consts_8B = 54291;
pub const addr_kMode7ObjectDef_8BD40D: Consts_8B = 54285;
pub const addr_kMode7ObjectDef_8BD407: Consts_8B = 54279;
pub const addr_kMode7ObjectDef_8BD401: Consts_8B = 54273;
pub const addr_off_8BD3E7: Consts_8B = 54247;
pub const addr_word_8BD389: Consts_8B = 54153;
pub const addr_word_8BD371: Consts_8B = 54129;
pub const addr_word_8BD30D: Consts_8B = 54029;
pub const addr_word_8BD2D5: Consts_8B = 53973;
pub const addr_word_8BD2A5: Consts_8B = 53925;
pub const addr_word_8BD259: Consts_8B = 53849;
pub const addr_word_8BD215: Consts_8B = 53781;
pub const addr_word_8BD1B9: Consts_8B = 53689;
pub const addr_word_8BD15D: Consts_8B = 53597;
pub const addr_word_8BD0F9: Consts_8B = 53497;
pub const addr_word_8BD0E1: Consts_8B = 53473;
pub const addr_word_8BD085: Consts_8B = 53381;
pub const addr_word_8BD055: Consts_8B = 53333;
pub const addr_word_8BCFFD: Consts_8B = 53245;
pub const addr_word_8BCFBD: Consts_8B = 53181;
pub const addr_word_8BCF75: Consts_8B = 53109;
pub const addr_kCinematicBgObjectDef_8BCF69: Consts_8B = 53097;
pub const addr_kCinematicBgObjectDef_8BCF63: Consts_8B = 53091;
pub const addr_kCinematicBgObjectDef_8BCF5D: Consts_8B = 53085;
pub const addr_kCinematicBgObjectDef_8BCF57: Consts_8B = 53079;
pub const addr_kCinematicBgObjectDef_8BCF51: Consts_8B = 53073;
pub const addr_kCinematicBgObjectDef_8BCF4B: Consts_8B = 53067;
pub const addr_kCinematicBgObjectDef_8BCF45: Consts_8B = 53061;
pub const addr_kCinematicBgObjectDef_8BCF3F: Consts_8B = 53055;
pub const addr_kCinematicSpriteObjectDef_8BCF39: Consts_8B = 53049;
pub const addr_kCinematicSpriteObjectDef_8BCF33: Consts_8B = 53043;
pub const addr_kCinematicSpriteObjectDef_8BCF2D: Consts_8B = 53037;
pub const addr_kCinematicSpriteObjectDef_8BCF27: Consts_8B = 53031;
pub const addr_kCinematicSpriteObjectDef_8BCF21: Consts_8B = 53025;
pub const addr_kCinematicSpriteObjectDef_8BCF1B: Consts_8B = 53019;
pub const addr_kCinematicSpriteObjectDef_8BCF15: Consts_8B = 53013;
pub const addr_kCinematicSpriteObjectDef_8BCF0F: Consts_8B = 53007;
pub const addr_kCinematicSpriteObjectDef_8BCF09: Consts_8B = 53001;
pub const addr_kCinematicSpriteObjectDef_8BCF03: Consts_8B = 52995;
pub const addr_kCinematicSpriteObjectDef_8BCEFD: Consts_8B = 52989;
pub const addr_kCinematicSpriteObjectDef_8BCEF7: Consts_8B = 52983;
pub const addr_kCinematicSpriteObjectDef_8BCEF1: Consts_8B = 52977;
pub const addr_kCinematicSpriteObjectDef_8BCEEB: Consts_8B = 52971;
pub const addr_kCinematicSpriteObjectDef_8BCEE5: Consts_8B = 52965;
pub const addr_kCinematicSpriteObjectDef_8BCEDF: Consts_8B = 52959;
pub const addr_kCinematicSpriteObjectDef_8BCED9: Consts_8B = 52953;
pub const addr_kCinematicSpriteObjectDef_8BCED3: Consts_8B = 52947;
pub const addr_kCinematicSpriteObjectDef_8BCECD: Consts_8B = 52941;
pub const addr_kCinematicSpriteObjectDef_8BCEC7: Consts_8B = 52935;
pub const addr_kCinematicSpriteObjectDef_8BCEC1: Consts_8B = 52929;
pub const addr_kCinematicSpriteObjectDef_8BCEBB: Consts_8B = 52923;
pub const addr_kCinematicSpriteObjectDef_8BCEB5: Consts_8B = 52917;
pub const addr_kCinematicSpriteObjectDef_8BCEAF: Consts_8B = 52911;
pub const addr_kCinematicSpriteObjectDef_8BCEA3: Consts_8B = 52899;
pub const addr_kCinematicSpriteObjectDef_8BCE91: Consts_8B = 52881;
pub const addr_kCinematicSpriteObjectDef_8BCE8B: Consts_8B = 52875;
pub const addr_kCinematicSpriteObjectDef_8BCE85: Consts_8B = 52869;
pub const addr_kCinematicSpriteObjectDef_8BCE7F: Consts_8B = 52863;
pub const addr_kCinematicSpriteObjectDef_8BCE79: Consts_8B = 52857;
pub const addr_kCinematicSpriteObjectDef_8BCE73: Consts_8B = 52851;
pub const addr_kCinematicSpriteObjectDef_8BCE6D: Consts_8B = 52845;
pub const addr_kCinematicSpriteObjectDef_8BCE67: Consts_8B = 52839;
pub const addr_kCinematicSpriteObjectDef_8BCE61: Consts_8B = 52833;
pub const addr_kCinematicSpriteObjectDef_8BCE5B: Consts_8B = 52827;
pub const addr_kCinematicSpriteObjectDef_8BCE55: Consts_8B = 52821;
pub const addr_off_8BCE53: Consts_8B = 52819;
pub const addr_word_8BCD71: Consts_8B = 52593;
pub const addr_word_8BCC23: Consts_8B = 52259;
pub const addr_unk_8BCC03: Consts_8B = 52227;
pub const addr_word_8BCBFB: Consts_8B = 52219;
pub const addr_word_8BCB3B: Consts_8B = 52027;
pub const addr_off_8BCB19: Consts_8B = 51993;
pub const addr_kCinematicFunction_Intro_Func76_M7_2: Consts_8B = 50170;
pub const addr_kCinematicFunction_Intro_Func76_M7_1: Consts_8B = 50160;
pub const addr_kCinematicFunction_Intro_Func76_M7_0: Consts_8B = 50150;
pub const addr_kCinematicFunction_Intro_Func56_M7: Consts_8B = 48756;
pub const addr_kMode7ObjBabyMetroidInTitle: Consts_8B = 41813;
pub const addr_kCinematicSpriteObjectDef_8BA125: Consts_8B = 41253;
pub const addr_kCinematicSpriteObjectDef_8BA119: Consts_8B = 41241;
pub const addr_kCinematicSpriteObjectDef_8BA113: Consts_8B = 41235;
pub const addr_kCinematicSpriteObjectDef_8BA107: Consts_8B = 41223;
pub const addr_kCinematicSpriteObjectDef_8BA101: Consts_8B = 41217;
pub const addr_kCinematicSpriteObjectDef_8BA0FB: Consts_8B = 41211;
pub const addr_kCinematicSpriteObjectDef_8BA0F5: Consts_8B = 41205;
pub const addr_kCinematicSpriteObjectDef_8BA0EF: Consts_8B = 41199;
pub use crate::types::Consts_8C;
pub const addr_kSpriteMap_NintendoLogo: Consts_8C = 32955;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_91;
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
unsafe fn RomPtr_8B(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x8b0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
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
unsafe fn get_Mode7ObjectDef(mut a: uint16, game: &mut Game) -> *mut Mode7ObjectDef {
    return RomPtr(
        (0x8b0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut Mode7ObjectDef;
}
pub const fnProcessCinematicBgObject_Nothing: uint32 = 9144393;
pub const fnProcessCinematicBgObject_DrawChar: uint32 = 9144397;
pub const fnProcessCinematicBgObject_DrawToTextTilemap: uint32 = 9144503;
pub const fnProcessCinematicBgObject_DrawToBgTilemap: uint32 = 9144573;
pub const fnProcessCinematicBgObject_Unk1: uint32 = 9144683;
pub const fnProcessCinematicBgObject_Unk2: uint32 = 9144783;
pub const fnCinematicFunction_nullsub_116: ::core::ffi::c_int = 0x8b93d9;
pub const fnCinematicSprInstr_Delete: uint32 = 9147448;
pub const fnCinematicSprInstr_Sleep: uint32 = 9147458;
pub const fnCinematicSprInstr_SetPreInstr: uint32 = 9147468;
pub const fnCinematicSprInstr_ClearPreInstr: uint32 = 9147479;
pub const fnCinematicFunction_nullsub_298: ::core::ffi::c_int = 0x8b945f;
pub const fnCinematicSprInstr_GotoRel: uint32 = 9147554;
pub const fnCinematicSprInstr_Goto: uint32 = 9147580;
pub const fnCinematicSprInstr_DecTimerGoto: uint32 = 9147587;
pub const fnCinematicSprInstr_DecTimerGotoRel: uint32 = 9147597;
pub const fnCinematicSprInstr_SetTimer: uint32 = 9147606;
pub const fnMode7Instr_Delete: uint32 = 9147755;
pub const fnMode7Instr_SetPreInstr: uint32 = 9147762;
pub const fnMode7Instr_ClearPreInstr: uint32 = 9147773;
pub const fnMode7Instr_Goto: uint32 = 9147782;
pub const fnMode7Instr_DecTimerAndGoto: uint32 = 9147789;
pub const fnMode7Instr_SetTimer: uint32 = 9147799;
pub const fnCinematicBgInstr_Delete: uint32 = 9148056;
pub const fnCinematicBgInstr_Goto: uint32 = 9148190;
pub const fnIntroObject_Delete: uint32 = 9148926;
pub const fnIntroObject_Goto: uint32 = 9148934;
pub const fnIntroObject_DecTimerGoto: uint32 = 9148941;
pub const fnIntroObject_SetTimer: uint32 = 9148951;
pub const fnCinematicFunctionNone: ::core::ffi::c_int = 0x8b9a47;
pub const fnCinematicSpriteInit_7: uint32 = 9149210;
pub const fnCinematicSpriteInit_8: uint32 = 9149229;
pub const fnCinematicSpriteInit_9: uint32 = 9149248;
pub const fnCinematicFunctionOpening: ::core::ffi::c_int = 0x8b9b68;
pub const fnCinematicSpriteInit_0: uint32 = 9149628;
pub const fnsub_8B9CCF: uint32 = 9149647;
pub const fnCinematicSprInstr_9CE1: uint32 = 9149665;
pub const fnCinematicFunc_Func2: ::core::ffi::c_int = 0x8b9d17;
pub const fnCinematicSpriteInit_1: uint32 = 9149770;
pub const fnCinematicSprInstr_9D5D: uint32 = 9149789;
pub const fnCinematicFunc_Func4: ::core::ffi::c_int = 0x8b9d90;
pub const fnCinematicSpriteInit_2: uint32 = 9149891;
pub const fnCinematicSprInstr_9DD6: uint32 = 9149910;
pub const fnCinematicFunc_Func5: ::core::ffi::c_int = 0x8b9e12;
pub const fnCinematicSpriteInit_3: uint32 = 9150021;
pub const fnCinematicSprInstr_Func8: uint32 = 9150040;
pub const fnCinematicFunc_Func7: ::core::ffi::c_int = 0x8b9e8b;
pub const fnCinematicSpriteInit_4: uint32 = 9150131;
pub const fnCinematicSprInstr_Func9: uint32 = 9150157;
pub const fnCinematicSpriteInit_5: uint32 = 9150166;
pub const fnCinematicSprInstr_sub_8B9EF0: uint32 = 9150192;
pub const fnCinematicSpriteInit_6: uint32 = 9150207;
pub const fnCinematicSprInstr_9F19: uint32 = 9150233;
pub const fnnullsub_117: ::core::ffi::c_int = 0x8b9f28;
pub const fnCinematicFunc_Func1: ::core::ffi::c_int = 0x8b9f29;
pub const fnCinematicFunc_Func10: ::core::ffi::c_int = 0x8b9f52;
pub const fnCinematicFunc_Func9: ::core::ffi::c_int = 0x8b9fae;
pub const fnPlayBabyMetroidCry1: uint32 = 9151067;
pub const fnPlayBabyMetroidCry2: uint32 = 9151075;
pub const fnPlayBabyMetroidCry3: uint32 = 9151083;
pub const fnCinematicSetPal1: uint32 = 9151091;
pub const fnCinematicSetPal2: uint32 = 9151108;
pub const fnCinematicSetPal3: uint32 = 9151125;
pub const fnCinematicSetPal4: uint32 = 9151142;
pub const fnnullsub_120: ::core::ffi::c_int = 0x8ba38f;
pub const fnnullsub_121: ::core::ffi::c_int = 0x8ba390;
pub const fnCinematicFunc_Nothing: ::core::ffi::c_int = 0x8ba391;
pub const fnCinematicFunction_Intro_Initial: uint32 = 9151381;
pub const fnCinematicFunction_Intro_FadeIn: ::core::ffi::c_int = 0x8ba5a7;
pub const fnCinematicFunction_Intro_DrawInitJpn: ::core::ffi::c_int = 0x8ba5bd;
pub const fnCinematicFunction_Intro_LastMetroidCaptivity: ::core::ffi::c_int = 0x8ba5f8;
pub const fnCinematicFunc_Intro_QueueGalaxyIsAtPeace: ::core::ffi::c_int = 0x8ba613;
pub const fnCinematicFunc_Intro_WaitForQueueWait4secs: ::core::ffi::c_int = 0x8ba639;
pub const fnCinematicFunc_Intro_FadeOut: ::core::ffi::c_int = 0x8ba64c;
pub const fnCinematicFunc_Intro_WaitForFadeOut: ::core::ffi::c_int = 0x8ba663;
pub const fnCinematicFunc_Intro_SetupTextPage2: ::core::ffi::c_int = 0x8ba66f;
pub const fnCinematicFunc_Intro_WaitFadeinShowText: ::core::ffi::c_int = 0x8ba82b;
pub const fnCinematicFunc_Intro_WaitForFadeinSleep: ::core::ffi::c_int = 0x8ba84a;
pub const fnSetSomeStuffForSpriteObject_2: uint32 = 9152705;
pub const fnnullsub_122: uint32 = 9152724;
pub const fnSetSomeStuffForSpriteObject_3: uint32 = 9152725;
pub const fnSetSomeStuffForSpriteObject_4_MetroidEgg: uint32 = 9152744;
pub const fnCinematicSprPreInstr_A903: uint32 = 9152771;
pub const fnCinematicSprInstr_SpawnMetroidEggParticles: uint32 = 9152792;
pub const fnSetSomeStuffForSpriteObject_6: uint32 = 9152856;
pub const fnSetSomeStuffForSpriteObject_7: uint32 = 9152916;
pub const fnSetSomeStuffForSpriteObject_8: uint32 = 9153178;
pub const fnSetSomeStuffForSpriteObject_9: uint32 = 9153203;
pub const fnSetSomeStuffForSpriteObject_10: uint32 = 9153877;
pub const fnSetSomeStuffForSpriteObject_11: uint32 = 9153896;
pub const fnSetSomeStuffForSpriteObject_12: uint32 = 9153939;
pub const fnSetSomeStuffForSpriteObject_13: uint32 = 9153958;
pub const fnCinematicBgInstr_SetSomeStuffForSpriteObject_14: uint32 = 9154004;
pub const fnSetSomeStuffForSpriteObject_17: uint32 = 9154055;
pub const fnSetSomeStuffForSpriteObject_18: uint32 = 9154074;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page1: uint32 = 9154115;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page1: uint32 = 9154139;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page2: uint32 = 9154169;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page2: uint32 = 9154193;
pub const fnCinematicFunction_Intro_WaitInputSetupMotherBrainFight: ::core::ffi::c_int = 0x8baeb8;
pub const fnCinematicFunction_Intro_WaitInputSetupBabyMetroid: ::core::ffi::c_int = 0x8baf6c;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page3: uint32 = 9154676;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page3: uint32 = 9154700;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page4: uint32 = 9154739;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page4: uint32 = 9154763;
pub const fnCinematicFunction_Intro_Func11: ::core::ffi::c_int = 0x8bb0f2;
pub const fnCinematicFunction_Intro_Func12: ::core::ffi::c_int = 0x8bb123;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page5: uint32 = 9154971;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page5: uint32 = 9154995;
pub const fnCinematicFunction_Intro_Func15: ::core::ffi::c_int = 0x8bb1da;
pub const fnCinematicFunction_Intro_Func15b: ::core::ffi::c_int = 0x8bb207;
pub const fnCinematicBgInstr_Func16: uint32 = 9155112;
pub const fnCinematicBgInstr_Func17: uint32 = 9155136;
pub const fnCinematicFunction_Intro_XfadeGameplayFade: ::core::ffi::c_int = 0x8bb250;
pub const fnCinematicFunction_Intro_XfadeScientistFade: ::core::ffi::c_int = 0x8bb2d2;
pub const fnCinematicSprInstr_StartIntroPage2: uint32 = 9155382;
pub const fnCinematicSprInstr_StartIntroPage3: uint32 = 9155390;
pub const fnCinematicSprInstr_StartIntroPage4: uint32 = 9155398;
pub const fnCinematicSprInstr_StartIntroPage5: uint32 = 9155406;
pub const fnCinematicFunction_Intro_Page2: ::core::ffi::c_int = 0x8bb35f;
pub const fnCinematicFunction_Intro_Page3: ::core::ffi::c_int = 0x8bb370;
pub const fnCinematicFunction_Intro_Page4: ::core::ffi::c_int = 0x8bb381;
pub const fnCinematicFunction_Intro_Page5: ::core::ffi::c_int = 0x8bb392;
pub const fnCinematicFunction_Intro_Fadestuff: ::core::ffi::c_int = 0x8bb3f4;
pub const fnCinematicFunction_Intro_Fadestuff2: ::core::ffi::c_int = 0x8bb458;
pub const fnCinematicBgPreInstr_SamusBlink: uint32 = 9155772;
pub const fnsub_8BB51E: uint32 = 9155870;
pub const fnEnableCinematicBgTilemapUpdates__0: uint32 = 9155919;
pub const fnCinematicFunction_Intro_Func21: uint32 = 9155924;
pub const fnCinematicFunction_Intro_Func22: uint32 = 9155973;
pub const fnCinematicFunction_Intro_ThenWaitInputSetupBabyMetroid: uint32 = 9156024;
pub const fnCinematicFunction_Intro_Func23: uint32 = 9156035;
pub const fnCinematicFunction_Intro_Func24: uint32 = 9156078;
pub const fnCinematicFunction_Intro_Func25: uint32 = 9156123;
pub const fnCinematicFunction_Intro_Func26: uint32 = 9156134;
pub const fnCinematicFunction_Intro_Func27: uint32 = 9156183;
pub const fnCinematicFunction_Intro_Func28: uint32 = 9156234;
pub const fnCinematicFunction_Intro_Func29: uint32 = 9156245;
pub const fnCinematicFunction_Intro_Func30: uint32 = 9156288;
pub const fnCinematicFunction_Intro_Func31: uint32 = 9156333;
pub const fnCinematicFunction_Intro_Func32: uint32 = 9156344;
pub const fnEnableCinematicBgTilemapUpdates__: uint32 = 9156363;
pub const fnCinematicFunction_Intro_Func34: ::core::ffi::c_int = 0x8bb72f;
pub const fnCinematicFunction_Intro_Func35: uint32 = 9156467;
pub const fnCinematicFunction_Intro_Func36: uint32 = 9156486;
pub const fnCinematicFunction_Intro_Func39: ::core::ffi::c_int = 0x8bb80f;
pub const fnCinematicSprPreInstr_B82E: uint32 = 9156654;
pub const fnCinematicFunction_Intro_Func42: uint32 = 9156758;
pub const fnCinematicSprInstr_Func43: uint32 = 9156805;
pub const fnCinematicFunction_Intro_Func44: ::core::ffi::c_int = 0x8bb8d8;
pub const fnCinematicFunction_Intro_Func45: ::core::ffi::c_int = 0x8bb93b;
pub const fnCinematicFunction_Intro_Func46: uint32 = 9157005;
pub const fnCinematicFunction_Intro_Func47: uint32 = 9157076;
pub const fnCinematicFunction_Intro_Func48: uint32 = 9157135;
pub const fnCinematicSprInstr_SpawnIntroRinkas01: uint32 = 9157153;
pub const fnCinematicSprInstr_SpawnIntroRinkas23: uint32 = 9157174;
pub const fnCinematicFunction_Intro_Func49: uint32 = 9157195;
pub const fnCinematicFunction_Intro_Func50: uint32 = 9157214;
pub const fnCinematicFunction_Intro_Func51: ::core::ffi::c_int = 0x8bba73;
pub const fnCinematicFunction_Intro_Func52: ::core::ffi::c_int = 0x8bbb0d;
pub const fnCinematicFunction_Intro_Func53: ::core::ffi::c_int = 0x8bbb24;
pub const fnCinematicFunction_Intro_Func54: ::core::ffi::c_int = 0x8bbca0;
pub const fnCinematicFunction_Intro_Func55: ::core::ffi::c_int = 0x8bbde4;
pub const fnCinematicFunction_Intro_Func56: ::core::ffi::c_int = 0x8bbdf9;
pub const fnCinematicFunction_Intro_Func57: uint32 = 9158270;
pub const fnCinematicFunction_Intro_Func58: uint32 = 9158325;
pub const fnCinematicFunction_Intro_Func59: uint32 = 9158434;
pub const fnCinematicFunction_Intro_Func60: uint32 = 9158453;
pub const fnCinematicFunction_Intro_Func61: uint32 = 9158476;
pub const fnCinematicFunction_Intro_Func62: uint32 = 9158495;
pub const fnCinematicFunction_Intro_Func63: uint32 = 9158518;
pub const fnCinematicFunction_Intro_Func64: uint32 = 9158537;
pub const fnCinematicFunction_Intro_Func65: uint32 = 9158560;
pub const fnCinematicFunction_Intro_Func66: ::core::ffi::c_int = 0x8bbfc6;
pub const fnCinematicSprPreInstr_nullsub_300: ::core::ffi::c_int = 0x8bbfd9;
pub const fnCinematicFunction_Intro_Func67: ::core::ffi::c_int = 0x8bbfda;
pub const fnCinematicFunction_Intro_Func68: uint32 = 9158787;
pub const fnCinematicCommonInstr_Func69: uint32 = 9158806;
pub const fnCinematicCommonInstr_Func70: uint32 = 9158818;
pub const fnCinematicFunction_Intro_Func71: uint32 = 9158834;
pub const fnCinematicFunction_Intro_Func72: ::core::ffi::c_int = 0x8bc0c5;
pub const fnCinematicFunction_Intro_Func73: ::core::ffi::c_int = 0x8bc100;
pub const fnCinematicFunctionBlackoutFromCeres: uint32 = 9158939;
pub const fnCinematicFunction_Intro_Func74: ::core::ffi::c_int = 0x8bc2e4;
pub const fnCinematicFunction_Intro_Func75: ::core::ffi::c_int = 0x8bc2f1;
pub const fnCinematicFunction_Intro_Func76: ::core::ffi::c_int = 0x8bc345;
pub const fnCinematicSprInstr_SpawnCeresExplosions1: uint32 = 9159684;
pub const fnCinematicFunction_Intro_Func78: uint32 = 9159732;
pub const fnCinematicSprPreInstr_C489: uint32 = 9159817;
pub const fnCinematicFunction_Intro_Func80: uint32 = 9159865;
pub const fnCinematicSprInstr_SpawnCeresExplosions3: uint32 = 9159948;
pub const fnCinematicFunction_Intro_Func81: uint32 = 9159987;
pub const fnCinematicFunction_Intro_Func82: uint32 = 9160066;
pub const fnCinematicFunction_Intro_Func83: uint32 = 9160105;
pub const fnCinematicFunction_Intro_Func77: ::core::ffi::c_int = 0x8bc5ca;
pub const fnCinematicFunction_Intro_Func84: ::core::ffi::c_int = 0x8bc610;
pub const fnCinematicFunction_Intro_Func85: ::core::ffi::c_int = 0x8bc627;
pub const fnCinematicFunction_Intro_Func86: ::core::ffi::c_int = 0x8bc699;
pub const fnCinematicFunction_Intro_Func87: ::core::ffi::c_int = 0x8bc79c;
pub const fnCinematicFunction_Intro_Func88: ::core::ffi::c_int = 0x8bc7ca;
pub const fnCinematicFunction_Intro_Func89: uint32 = 9160763;
pub const fnCinematicFunction_Intro_Func90: uint32 = 9160782;
pub const fnCinematicFunction_Intro_Func91: ::core::ffi::c_int = 0x8bc85d;
pub const fnCinematicFunction_Intro_Func92: uint32 = 9160855;
pub const fnCinematicFunction_Intro_Func93: uint32 = 9160874;
pub const fnCinematicFunction_Intro_Func94: ::core::ffi::c_int = 0x8bc8b9;
pub const fnCinematicFunction_Intro_Func96: uint32 = 9160953;
pub const fnCinematicFunction_Intro_Func97: ::core::ffi::c_int = 0x8bc908;
pub const fnCinematicFunction_Intro_Func98: uint32 = 9161026;
pub const fnCinematicFunction_Intro_Func99: uint32 = 9161046;
pub const fnCinematicFunction_Intro_Func100: uint32 = 9161066;
pub const fnCinematicFunction_Intro_Func101: uint32 = 9161086;
pub const fnCinematicFunction_Intro_Func102: uint32 = 9161106;
pub const fnCinematicSprInstr_C9A5: uint32 = 9161125;
pub const fnCinematicSprInstr_C9AF: uint32 = 9161135;
pub const fnCinematicSprInstr_C9BD: uint32 = 9161149;
pub const fnCinematicSprInstr_C9C7: uint32 = 9161159;
pub const fnCinematicFunction_Intro_Func104: uint32 = 9161190;
pub const fnCinematicFunction_Intro_Func105: ::core::ffi::c_int = 0x8bc9f9;
pub const fnCinematicFunction_Intro_Func106: ::core::ffi::c_int = 0x8bca36;
pub const fnCinematicFunction_Intro_Func107: ::core::ffi::c_int = 0x8bca85;
pub const fnCinematicFunction_Intro_Func108: ::core::ffi::c_int = 0x8bcad0;
pub const fnnullsub_124: ::core::ffi::c_int = 0x8bcade;
pub const fnCinematicFunction_Intro_Func95: ::core::ffi::c_int = 0x8bcadf;
pub const fnCinematicFunctionEscapeFromCebes: uint32 = 9163904;
pub const fnCinematicFunction_Intro_Func109: ::core::ffi::c_int = 0x8bd6d7;
pub const fnCinematicFunction_Intro_Func110: ::core::ffi::c_int = 0x8bd6f2;
pub const fnCinematicFunction_Intro_Func111: ::core::ffi::c_int = 0x8bd701;
pub const fnCinematicFunction_Intro_Func112: ::core::ffi::c_int = 0x8bd731;
pub const fnCinematicFunction_Intro_Func113: ::core::ffi::c_int = 0x8bd7f8;
pub const fnCinematicFunction_Intro_Func114: ::core::ffi::c_int = 0x8bd807;
pub const fnCinematicFunction_Intro_Func115: ::core::ffi::c_int = 0x8bd837;
pub const fnCinematicFunction_Intro_Func116: ::core::ffi::c_int = 0x8bda02;
pub const fnCinematicFunction_Intro_Func117: ::core::ffi::c_int = 0x8bda1a;
pub const fnCinematicFunction_Intro_Func118: ::core::ffi::c_int = 0x8bdad3;
pub const fnnullsub_125: ::core::ffi::c_int = 0x8bdb9d;
pub const fnCinematicFunction_Intro_Func119: ::core::ffi::c_int = 0x8bdb9e;
pub const fnCinematicFunction_Intro_Func120: ::core::ffi::c_int = 0x8bdbc4;
pub const fnCinematicFunction_Intro_Func121: ::core::ffi::c_int = 0x8bdca5;
pub const fnCinematicFunction_Intro_Func123: ::core::ffi::c_int = 0x8bdd42;
pub const fnCinematicFunction_Intro_Func124: ::core::ffi::c_int = 0x8bddcd;
pub const fnnullsub_126: ::core::ffi::c_int = 0x8bde63;
pub const fnCinematicFunction_Intro_Func125: ::core::ffi::c_int = 0x8bde64;
pub const fnCinematicFunction_Intro_Func126: ::core::ffi::c_int = 0x8bde80;
pub const fnCinematicFunction_Intro_Func129: ::core::ffi::c_int = 0x8be0f4;
pub const fnCinematicFunction_Intro_Func130: ::core::ffi::c_int = 0x8be110;
pub const fnCinematicFunction_Intro_Func131: ::core::ffi::c_int = 0x8be158;
pub const fnCinematicFunction_Intro_Func132: ::core::ffi::c_int = 0x8be190;
pub const fnCinematicFunction_Intro_Func134: ::core::ffi::c_int = 0x8be1d2;
pub const fnCinematicFunction_Intro_Func136: ::core::ffi::c_int = 0x8be265;
pub const fnCinematicFunction_Intro_Func135: ::core::ffi::c_int = 0x8be293;
pub const fnCinematicFunction_Intro_Func137: ::core::ffi::c_int = 0x8be2dd;
pub const fnCinematicFunction_Intro_Func138: ::core::ffi::c_int = 0x8be314;
pub const fnCinematicFunction_Intro_Func145: ::core::ffi::c_int = 0x8be342;
pub const fnCinematicFunction_Intro_Func139: ::core::ffi::c_int = 0x8be3ae;
pub const fnCinematicFunction_Intro_Func141: ::core::ffi::c_int = 0x8be48a;
pub const fnCinematicFunction_Intro_Func143: ::core::ffi::c_int = 0x8be504;
pub const fnCinematicFunction_Intro_Func144: ::core::ffi::c_int = 0x8be58a;
pub const fnCalcItemPercentageCount: uint32 = 9168423;
pub const fnCinematicFunction_Intro_Func146: uint32 = 9168745;
pub const fnCinematicFunction_Intro_Func147: uint32 = 9168768;
pub const fnCinematicFunction_Intro_Func148: ::core::ffi::c_int = 0x8be797;
pub const fnnullsub_127: ::core::ffi::c_int = 0x8be7ba;
pub const fnCinematicFunction_Intro_Func150: uint32 = 9170841;
pub const fnCinematicFunction_Intro_Func151: uint32 = 9170866;
pub const fnCinematicFunction_Intro_Func152: uint32 = 9170922;
pub const fnCinematicFunction_Intro_Func153: uint32 = 9170941;
pub const fnCinematicFunction_Intro_Func154: uint32 = 9170949;
pub const fnCinematicFunction_Intro_Func155: uint32 = 9170968;
pub const fnCinematicFunction_Intro_Func156: uint32 = 9170987;
pub const fnCinematicFunction_Intro_Func157: uint32 = 9171006;
pub const fnCinematicFunction_Intro_Func159: uint32 = 9171038;
pub const fnCinematicFunction_Intro_Func161: uint32 = 9171053;
pub const fnCinematicFunction_Intro_Func162: uint32 = 9171068;
pub const fnCinematicFunction_Intro_Func163: uint32 = 9171077;
pub const fnCinematicFunction_Intro_Func164: uint32 = 9171092;
pub const fnCinematicFunction_Intro_Func165: uint32 = 9171122;
pub const fnCinematicFunction_Intro_Func166: uint32 = 9171169;
pub const fnCinematicFunction_Intro_Func167: uint32 = 9171177;
pub const fnCinematicFunction_Intro_Func168: uint32 = 9171185;
pub const fnCinematicFunction_Intro_Func169: uint32 = 9171193;
pub const fnCinematicFunction_Intro_Func170: uint32 = 9171230;
pub const fnCinematicFunction_Intro_Func171: uint32 = 9171267;
pub const fnCinematicFunction_Intro_Func172: uint32 = 9171286;
pub const fnCinematicFunction_Intro_Func173: uint32 = 9171305;
pub const fnCinematicFunction_Intro_Func174: uint32 = 9171324;
pub const fnCinematicFunction_Intro_Func175: uint32 = 9171343;
pub const fnCinematicFunction_Intro_Func176: uint32 = 9171368;
pub const fnCinematicFunction_Intro_Func177: uint32 = 9171393;
pub const fnCinematicFunction_Intro_Func178: uint32 = 9171412;
pub const fnCinematicFunction_Intro_Func179: uint32 = 9171431;
pub const fnCinematicFunction_Intro_Func180: uint32 = 9171495;
pub const fnCinematicSprInstr_Func181: uint32 = 9171550;
pub const fnCinematicSprInstr_Func182: uint32 = 9171588;
pub const fnCinematicSprInstr_Func183: uint32 = 9171605;
pub const fnCinematicFunction_Intro_Func184: uint32 = 9171621;
pub const fnCinematicSprInstr_Func185: uint32 = 9171639;
pub const fnCinematicSprInstr_Func186: uint32 = 9171706;
pub const fnCinematicSprInstr_Func187: uint32 = 9171755;
pub const fnCinematicSprPreInstr_F35A: uint32 = 9171802;
pub const fnCinematicFunction_Intro_Func189: ::core::ffi::c_int = 0x8bf375;
pub const fnCinematicFunction_Intro_Func189b: uint32 = 9171867;
pub const fnCinematicSprInstr_Func190: uint32 = 9171888;
pub const fnCinematicFunction_Intro_Func191: uint32 = 9171897;
pub const fnCinematicSprInstr_Func192: uint32 = 9171918;
pub const fnCinematicSprInstr_Func193: uint32 = 9171995;
pub const fnCinematicSprInstr_Func194: uint32 = 9172004;
pub const fnCinematicSprInstr_Func195: uint32 = 9172013;
pub const fnCinematicSprInstr_Func196: uint32 = 9172022;
pub const fnCinematicSprInstr_Func197: uint32 = 9172031;
pub const fnCinematicSprInstr_Func198: uint32 = 9172040;
pub const fnCinematicFunction_Intro_Func199: uint32 = 9172053;
pub const fnCinematicFunction_Intro_Func200: ::core::ffi::c_int = 0x8bf464;
pub const fnCinematicFunction_Intro_Func201: uint32 = 9172088;
pub const fnCinematicFunction_Intro_Func202: ::core::ffi::c_int = 0x8bf487;
pub const fnCinematicFunction_Intro_Func203: uint32 = 9172123;
pub const fnCinematicFunction_Intro_Func204: ::core::ffi::c_int = 0x8bf4aa;
pub const fnCinematicFunction_Intro_Func205: uint32 = 9172177;
pub const fnCinematicFunction_Intro_Func206: ::core::ffi::c_int = 0x8bf4e0;
pub const fnCinematicFunction_Intro_Func207: uint32 = 9172231;
pub const fnnullsub_128: uint32 = 9172252;
pub const fnCinematicSprInstr_Func208: uint32 = 9172253;
pub const fnCinematicSprPreInstr_F528: uint32 = 9172264;
pub const fnCinematicSprInstr_Func211: uint32 = 9172308;
pub const fnCinematicSprPreInstr_F57F: uint32 = 9172351;
pub const fnCinematicSprInstr_Func213: uint32 = 9172375;
pub const fnCinematicSprInstr_Func214: uint32 = 9172410;
pub const fnCinematicFunction_Intro_Func215: ::core::ffi::c_int = 0x8bf5dd;
pub const fnCinematicSprInstr_Func217: uint32 = 9172484;
pub const fnCinematicSprInstr_Func218: uint32 = 9172561;
pub const fnCinematicFunction_Intro_Func219: uint32 = 9172734;
pub const fnSamus_FrameHandlerAlfa_Func12: ::core::ffi::c_int = 0x90e6c9;
pub const fnSamus_Func15: ::core::ffi::c_int = 0x90e833;
pub const fnSamusDrawHandler_Default: ::core::ffi::c_int = 0x90eb52;
pub const fnlocret_8B9585: ::core::ffi::c_int = 0x8b9585;
pub unsafe fn SetupPpuForTitleSequence(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(BGMODE as uint16, 7 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
    WriteReg(M7SEL as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
    WriteReg(BG1SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 0 as uint8;
    WriteReg(BG2SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
    WriteReg(BG3SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x10 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(SETINI as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
    memset(
        (&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0x7fe as size_t,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x731) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x741) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x745) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x751) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x759) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_Intro(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(BGMODE as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 80 as uint8;
    WriteReg(BG1SC as uint16, 0x50 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 72 as uint8;
    WriteReg(BG2SC as uint16, 0x48 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 76 as uint8;
    WriteReg(BG3SC as uint16, 0x4c as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    WriteReg(BG34NBA as uint16, 4 as uint8, game);
    WriteReg(TM as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 4 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(SETINI as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut uint16,
        0x2f as uint16,
        0x7fe as uint16,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_3_Mode7(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(BGMODE as uint16, 7 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
    WriteReg(M7SEL as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
    WriteReg(BG1SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 0 as uint8;
    WriteReg(BG2SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
    WriteReg(BG3SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_4_Mode1(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(BGMODE as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 92 as uint8;
    WriteReg(BG1SC as uint16, 0x5c as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 0 as uint8;
    WriteReg(BG2SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
    WriteReg(BG3SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 6 as uint8;
    WriteReg(BG12NBA as uint16, 6 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_5_Mode7(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 0xa3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = -(93 as ::core::ffi::c_int) as uint8;
    WriteReg(BGMODE as uint16, 7 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
    WriteReg(BG1SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 0 as uint8;
    WriteReg(BG2SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
    WriteReg(BG3SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(SETINI as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_6_Mode1(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 2 as uint8;
    WriteReg(BGMODE as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 112 as uint8;
    WriteReg(BG1SC as uint16, 0x70 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 120 as uint8;
    WriteReg(BG2SC as uint16, 0x78 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
    WriteReg(BG3SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 68 as uint8;
    WriteReg(BG12NBA as uint16, 0x44 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x10 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
}
pub unsafe fn SetupPpu_7_Mode1(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 0 as uint8;
    WriteReg(BGMODE as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 72 as uint8;
    WriteReg(BG1SC as uint16, 0x48 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 76 as uint8;
    WriteReg(BG2SC as uint16, 0x4c as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 36 as uint8;
    WriteReg(BG3SC as uint16, 0x24 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 84 as uint8;
    WriteReg(BG12NBA as uint16, 0x54 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 2 as uint8;
    WriteReg(BG34NBA as uint16, 2 as uint8, game);
    WriteReg(TM as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 1 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 128 as uint16;
    WriteRegWord(M7Y as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
}
pub unsafe fn HandleMode7TransformationNoRotation(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
}
unsafe fn Smult16x16(mut r38: uint16, mut r40: uint16) -> uint32 {
    let mut t: uint32 =
        (r38 as int16 as ::core::ffi::c_int * r40 as int16 as ::core::ffi::c_int) as uint32;
    return t;
}
pub unsafe fn HandleMode7TransformationMatrix(game: &mut Game) {
    let ref mut fresh152 = *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16);
    *fresh152 = (Smult16x16(
        kSinCosTable8bit_Sext[((*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
            as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as usize] as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16),
    ) >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = *fresh152;
    let mut v1: uint16 = (Smult16x16(
        kSinCosTable8bit_Sext[(*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
            as uint8 as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as usize] as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16),
    ) >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) =
        -(v1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
}
pub unsafe fn SetPpuBackdropSomeColor(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = -(127 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 56 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 88 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
}
pub unsafe fn SetPpuBackdropBlack(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
}
pub unsafe fn UpdateCinematicBgTilemap(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x55) as ::core::ffi::c_int
        != 7 as ::core::ffi::c_int
    {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        (*gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .size = 1920 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 12288 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let fresh157 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
            fresh157 as ::core::ffi::c_int,
            game,
        ))
        .size as *mut uint8)
            .offset(0) = 126 as uint8;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
            *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn UpdateSamusEyesTilemap(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x55) as ::core::ffi::c_int
        != 7 as ::core::ffi::c_int
    {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        (*gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .size = 128 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
            (((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
                + 832 as ::core::ffi::c_long) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let fresh156 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
            fresh156 as ::core::ffi::c_int,
            game,
        ))
        .size as *mut uint8)
            .offset(0) = 126 as uint8;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 18848 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicUpdateSomeBg(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x55) as ::core::ffi::c_int
        != 7 as ::core::ffi::c_int
    {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        (*gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .size = 2048 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
            ((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
                as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let fresh4 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
            fresh4 as ::core::ffi::c_int,
            game,
        ))
        .size as *mut uint8)
            .offset(0) = 126 as uint8;
        (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
            *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CallProcessCinematicBgObject(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    match ea {
        9144393 => {
            ProcessCinematicBgObject_Nothing(k, j, r18);
            return;
        }
        9144397 => {
            ProcessCinematicBgObject_DrawChar(k, j, r18, game);
            return;
        }
        9144503 => {
            ProcessCinematicBgObject_DrawToTextTilemap(k, j, r18, game);
            return;
        }
        9144573 => {
            ProcessCinematicBgObject_DrawToBgTilemap(k, j, r18, game);
            return;
        }
        9144683 => {
            ProcessCinematicBgObject_Unk1(k, j, r18, game);
            return;
        }
        9144783 => {
            ProcessCinematicBgObject_Unk2(k, j, r18, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ProcessCinematicBgObject(mut v0: uint16, mut r18: uint16, game: &mut Game) {
    let mut j: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x19b5) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    CallProcessCinematicBgObject(
        (*(RomPtr_8C(j as uint16_t, game) as *mut uint16) as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        v0,
        j,
        r18,
        game,
    );
}
pub unsafe fn ProcessCinematicBgObject_DrawChar(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut v3: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as uint16;
    SpawnTextGlowObject(j, r18, game);
    let mut v2: *const uint8 = RomPtr_8C(
        *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as uint16_t,
        game,
    );
    if (*(v2 as *mut int16).offset(3) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(15) =
            8 as uint16;
        v3 = (8 as ::core::ffi::c_int
            * (*v2.offset(3) as ::core::ffi::c_int + 2 as ::core::ffi::c_int))
            as int16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(15) =
            (8 as ::core::ffi::c_int * *v2.offset(8) as ::core::ffi::c_int) as uint16;
        v3 = (8 as ::core::ffi::c_int * *v2.offset(9) as ::core::ffi::c_int) as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(15) =
        (v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    if *(v2.offset(4) as *mut uint16) as ::core::ffi::c_int != 0xd67d as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16)
            as ::core::ffi::c_int
            != 0
    {
        QueueSfx3_Max6(0xd as uint16, game);
    }
    ProcessCinematicBgObject_DrawToTextTilemap(0x1e as uint16, j, r18, game);
}
pub unsafe fn ProcessCinematicBgObject_DrawToTextTilemap(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut jorg: uint16 = j;
    let mut offs: uint16 = CinematicGetTilemapOffsetForTile(r18);
    let mut r22: uint16 = offs;
    let mut v3: *const uint8 = RomPtr_8C(j as uint16_t, game);
    let mut m: uint16 = *v3.offset(2) as uint16;
    let mut morg: uint16 = m;
    let mut n: uint16 = *v3.offset(3) as uint16;
    loop {
        loop {
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
                .offset(offs as ::core::ffi::c_int as isize) as *mut uint16) =
                *(RomPtr_8C(j as uint16_t, game) as *mut uint16).offset(2);
            j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            offs = (offs as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            m = m.wrapping_sub(1);
            if !(m != 0) {
                break;
            }
        }
        m = morg;
        n = n.wrapping_sub(1);
        if n == 0 {
            break;
        }
        r22 = (r22 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        offs = r22;
    }
}
pub fn ProcessCinematicBgObject_Nothing(mut k: uint16, mut j: uint16, mut r18: uint16) {}
pub unsafe fn ProcessCinematicBgObject_DrawToBgTilemap(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut jorg: uint16 = j;
    let mut offs: uint16 = CinematicGetTilemapOffsetForTile(r18);
    let mut r22: uint16 = offs;
    let mut v3: *const uint8 = RomPtr_8C(j as uint16_t, game);
    let mut m: uint16 = *v3.offset(2) as uint16;
    let mut morg: uint16 = m;
    let mut n: ::core::ffi::c_int = *v3.offset(3) as ::core::ffi::c_int;
    loop {
        loop {
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .cinematic_bg_tilemap as *mut uint16 as *mut uint8)
                .offset(offs as ::core::ffi::c_int as isize) as *mut uint16) =
                *(RomPtr_8C(j as uint16_t, game) as *mut uint16).offset(2);
            j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            offs = (offs as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            m = m.wrapping_sub(1);
            if !(m != 0) {
                break;
            }
        }
        m = morg;
        n -= 1;
        if n == 0 {
            break;
        }
        r22 = (r22 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        offs = r22;
    }
}
pub unsafe fn CinematicGetTilemapOffsetForTile(mut r18: uint16) -> uint16 {
    return (2 as ::core::ffi::c_int * r18 as uint8 as ::core::ffi::c_int
        + *(&raw mut r18 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            * 0x40 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ProcessCinematicBgObject_Unk1(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut r22: uint16 = Mult0x80Add(r18);
    let mut v2: *const uint8 = RomPtr_8C(j as uint16_t, game);
    let mut r18x: uint16 = *v2.offset(2) as uint16;
    let mut n: uint16 = *v2.offset(3) as uint16;
    let mut v3: uint16 = (j as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16);
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = 128 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = v3;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset(v4 as isize) as *mut uint8 as *mut uint16) = 139 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = r18x;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = r22;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        v4 = (v4 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + r18x as ::core::ffi::c_int) as uint16;
        r22 = (r22 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
        n = n.wrapping_sub(1);
        if !(n != 0) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16) = v4;
}
pub unsafe fn ProcessCinematicBgObject_Unk2(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut r22: uint16 = Mult0x80Add(r18);
    let mut v2: *const uint8 = RomPtr_8C(j as uint16_t, game);
    let mut n: ::core::ffi::c_int = *v2.offset(2) as ::core::ffi::c_int;
    let mut r20: uint16 = *v2.offset(3) as uint16;
    let mut v3: uint16 = (j as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16);
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = 128 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = v3;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset(v4 as isize) as *mut uint8 as *mut uint16) = 139 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = r20;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = r22;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut uint8)
            .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) = 2 as uint16;
        v4 = (v4 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        r22 = r22.wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16) = v4;
}
pub unsafe fn Mult0x80Add(mut r18: uint16) -> uint16 {
    return (r18 as uint8 as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int
            * *(&raw mut r18 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_CalcPos_Mode7(game: &mut Game) {
    let mut r34: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut r36: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut r26: uint16 = (Smult16x16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = (r26
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    r26 = (Smult16x16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16),
        r34,
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) as ::core::ffi::c_int
            - r26 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CalcExplosion_Mode7(mut k: uint16, game: &mut Game) -> Point16U {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r34: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut r36: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    let mut r26: uint16 = (Smult16x16(
        r34,
        *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    let mut r20: uint16 = (r26 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    r26 = (Smult16x16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16),
        r34,
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
        as ::core::ffi::c_int
        - r26 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    return Point16U { x: r20, y: r18 };
}
pub unsafe fn CalcCeresSteamPos_Mode7(mut pt: Point16U, game: &mut Game) -> Point16U {
    let mut r34: uint16 = (pt.x as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut r36: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
        as ::core::ffi::c_int
        - pt.y as ::core::ffi::c_int) as uint16;
    let mut r26: uint16 = (Smult16x16(
        (pt.x as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    pt.x = (r26 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    r26 = (Smult16x16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16),
        r34,
    ) >> 8 as ::core::ffi::c_int) as uint16;
    r26 = (r26 as uint32).wrapping_add(
        Smult16x16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16),
            r36,
        ) >> 8 as ::core::ffi::c_int,
    ) as uint16 as uint16;
    pt.y = (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
        as ::core::ffi::c_int
        - r26 as ::core::ffi::c_int) as uint16;
    return pt;
}
pub unsafe fn CopyPalettesToFadingPalettes(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 256 as ::core::ffi::c_int)
                as isize,
        ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v0 -= 1;
        if !(v0 != 0) {
            break;
        }
    }
}
pub unsafe fn DecomposePaletteDataForFading(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut v1: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
    loop {
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 256 as ::core::ffi::c_int) as isize);
        let mut v3: ::core::ffi::c_int = r18 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 512 as ::core::ffi::c_int) as isize) =
            (v3 << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1280 as ::core::ffi::c_int) as isize) =
            (v3 << 3 as ::core::ffi::c_int) as uint16;
        let mut v4: ::core::ffi::c_int =
            (r18 as ::core::ffi::c_int & 0x3e0 as ::core::ffi::c_int) >> 5 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 768 as ::core::ffi::c_int) as isize) =
            (v4 << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1536 as ::core::ffi::c_int) as isize) =
            (v4 << 3 as ::core::ffi::c_int) as uint16;
        let mut v6: ::core::ffi::c_int =
            (r18 as ::core::ffi::c_int & 0x7c00 as ::core::ffi::c_int) >> 10 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1024 as ::core::ffi::c_int) as isize) =
            (v6 << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1792 as ::core::ffi::c_int) as isize) =
            (v6 << 3 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 -= 1;
        if !(v1 != 0) {
            break;
        }
    }
}
pub unsafe fn ClearYColorsFromIndexX(mut k: uint16, mut j: uint16, game: &mut Game) {
    loop {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 256 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 512 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 768 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1024 as ::core::ffi::c_int) as isize) = 0 as uint16;
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = j.wrapping_sub(1);
        if !(j != 0) {
            break;
        }
    }
}
pub unsafe fn FadeOutYColorsFromIndexX(mut k: uint16, mut j: uint16, game: &mut Game) {
    loop {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 512 as ::core::ffi::c_int) as isize);
        *fresh16 = (*fresh16 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1280 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 768 as ::core::ffi::c_int) as isize);
        *fresh17 = (*fresh17 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1536 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1024 as ::core::ffi::c_int) as isize);
        *fresh18 = (*fresh18 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1792 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = j.wrapping_sub(1);
        if !(j != 0) {
            break;
        }
    }
}
pub unsafe fn FadeInYColorsFromIndexX(mut k: uint16, mut j: uint16, game: &mut Game) {
    loop {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 512 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1280 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v2 + 512 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                & 0x1fff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 768 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1536 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v2 + 768 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                & 0x1fff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 1024 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1792 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v2 + 1024 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                & 0x1fff as ::core::ffi::c_int) as uint16;
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = j.wrapping_sub(1);
        if !(j != 0) {
            break;
        }
    }
}
pub unsafe fn ComposeFadingPalettes(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut v1: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
    loop {
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut r18: ::core::ffi::c_int = *(((&raw mut game.g_ram as *mut uint8).offset(0x2000)
            as *mut uint16)
            .offset((v2 + 512 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int;
        r18 |= *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 768 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int
            & 0x3e0 as ::core::ffi::c_int;
        let mut v3: uint16 = (r18
            | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 1024 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                & 0x7c00 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(v2 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v2 as isize) =
            v3;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 -= 1;
        if !(v1 != 0) {
            break;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func20(mut j: uint16, game: &mut Game) {
    let mut tt: *mut uint16 = RomPtr_8B(j as uint16_t, game) as *mut uint16;
    let mut v2: uint16 = *tt.offset(0);
    let mut r18: ::core::ffi::c_int = *tt.offset(1) as ::core::ffi::c_int;
    let mut dst: *mut uint16 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
        as *mut Ram4000))
        .intro_japanese_text_tiles as *mut uint16;
    loop {
        memcpy(
            dst.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut game.g_ram as *mut uint8).offset(
                (*tt.offset(2) as ::core::ffi::c_int + 0x1a000 as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16 as *const ::core::ffi::c_void,
            16 as size_t,
        );
        memcpy(
            dst.offset(
                (v2 as ::core::ffi::c_int + 768 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                    as isize,
            ) as *mut ::core::ffi::c_void,
            (&raw mut game.g_ram as *mut uint8).offset(
                (*tt.offset(3) as ::core::ffi::c_int + 0x1a000 as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16 as *const ::core::ffi::c_void,
            16 as size_t,
        );
        v2 = (v2 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        tt = tt.offset(2);
        r18 -= 1;
        if !(r18 != 0) {
            break;
        }
    }
}
pub unsafe fn TransferJapaneseTextTilesToVram(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1536 as uint16;
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 0x4000 as uint16;
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh21 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh21 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 126 as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 16768 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn HandleSamusDuringIntro(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) != 0 {
        HandleControllerInputForGamePhysics(game);
        HandleSamusMovementAndPause(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) != 0 {
            let ref mut fresh160 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16);
            *fresh160 = (*fresh160).wrapping_sub(1);
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) != 0 {
            let ref mut fresh161 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16);
            *fresh161 = (*fresh161).wrapping_sub(1);
        }
    }
}
pub unsafe fn DrawIntroSprites(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            DrawSamusAndProjectiles(game);
            DrawPlayerExplosions2(game);
            DrawCinematicSpriteObjects_Intro(game);
        } else {
            DrawCinematicSpriteObjects_Intro(game);
            DrawSamusAndProjectiles(game);
            DrawPlayerExplosions2(game);
        }
    } else {
        DrawCinematicSpriteObjects_Intro(game);
    };
}
pub unsafe fn CalcSinMult(mut k: uint16, mut r24: uint16) -> uint16 {
    let mut r18: uint16 = (Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const int16).offset(64) as *const int16
            as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        r24 as uint8,
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    return (r18 as ::core::ffi::c_int
        + *((&raw const kSinCosTable8bit_Sext as *const int16).offset(64) as *const int16
            as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1) as ::core::ffi::c_int
            * r24 as uint8 as ::core::ffi::c_int) as uint16;
}
static kMoveUnusedSpritesMasks: [uint16; 8] = [
    0x5555 as uint16,
    0x5554 as uint16,
    0x5550 as uint16,
    0x5540 as uint16,
    0x5500 as uint16,
    0x5400 as uint16,
    0x5000 as uint16,
    0x4000 as uint16,
];
pub unsafe fn MoveUnusedSpritesOffScreen(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int
        >= 0x200 as ::core::ffi::c_int
    {
        return;
    }
    let mut j: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x590)
        as *mut uint16) as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x590)
        as *mut uint16) as ::core::ffi::c_int
        >> 5 as ::core::ffi::c_int;
    let fresh11 = i;
    i = i + 1;
    let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
        .offset(fresh11 as isize);
    *fresh12 = (*fresh12 as ::core::ffi::c_int
        | kMoveUnusedSpritesMasks[(j & 7 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    while i < 16 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16).offset(i as isize) =
            0x5555 as uint16;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x590)
        as *mut uint16) as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int;
    while i_0 < 0x80 as ::core::ffi::c_int {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x370) as *mut OamEnt)
            .offset(i_0 as isize))
        .xcoord as *mut uint16) = 0x80 as uint16;
        i_0 += 1;
    }
}
pub unsafe fn AdvanceFastScreenFadeOut(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if ((*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16)
                as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x51) =
                ((*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16)
                        as ::core::ffi::c_int) as uint8;
            return 0 as uint8;
        }
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    }
    return 1 as uint8;
}
pub unsafe fn AdvanceSlowScreenFadeOut(game: &mut Game) -> uint8 {
    let mut v0: bool = ((*((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16)
        as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8).offset(0) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) as uint8
        as ::core::ffi::c_int
        != 0
        && !v0
    {
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x51) =
                ((*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int) as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8)
                .offset(0) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) as uint8;
            return 0 as uint8;
        }
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8)
            .offset(0) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16 as *mut uint8)
            .offset(0) = 0 as uint8;
    }
    return 1 as uint8;
}
pub unsafe fn AdvanceFastScreenFadeIn(game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) as ::core::ffi::c_int
        + *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        - 15 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16)
                as ::core::ffi::c_int
                + *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint8;
        return 0 as uint8;
    } else {
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
        return 1 as uint8;
    };
}
pub unsafe fn AdvanceSlowScreenFadeIn(game: &mut Game) -> uint8 {
    let mut v0: bool = ((*((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16)
        as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8).offset(0) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) as uint8
        as ::core::ffi::c_int
        != 0
        && !v0
    {
        return 0 as uint8;
    }
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        - 15 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) =
            (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8)
            .offset(0) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) as uint8;
        return 0 as uint8;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16 as *mut uint8).offset(0) =
        0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16 as *mut uint8).offset(0) =
        0 as uint8;
    return 1 as uint8;
}
pub unsafe fn InitializeIoDisplayLogo_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x612);
    's_645: {
        let mut current_block_182: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                WriteReg(INIDISP as uint16, 0x8f as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x8f as uint8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
                ClearOamExt(game);
                ClearUnusedOam(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x71f) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x721) as *mut uint16) = 0 as uint16;
                WriteReg(NMITIMEN as uint16, 1 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x84) = 1 as uint8;
                WriteReg(WRIO as uint16, 0 as uint8, game);
                WriteReg(WRMPYA as uint16, 0 as uint8, game);
                WriteReg(WRMPYB as uint16, 0 as uint8, game);
                WriteReg(WRDIVL as uint16, 0 as uint8, game);
                WriteReg(WRDIVH as uint16, 0 as uint8, game);
                WriteReg(WRDIVB as uint16, 0 as uint8, game);
                WriteReg(HTIMEL as uint16, 0 as uint8, game);
                WriteReg(HTIMEH as uint16, 0 as uint8, game);
                WriteReg(VTIMEL as uint16, 0 as uint8, game);
                WriteReg(VTIMEH as uint16, 0 as uint8, game);
                WriteReg(MDMAEN as uint16, 0 as uint8, game);
                WriteReg(HDMAEN as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
                WriteReg(MEMSEL as uint16, 1 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x86) = 1 as uint8;
                WriteReg(OBSEL as uint16, 3 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
                WriteReg(OAMADDL as uint16, 0 as uint8, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8)
                    .offset(0) = 0 as uint8;
                WriteReg(OAMADDH as uint16, 0x80 as uint8, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) = 0x80 as uint8;
                WriteReg(OAMDATA as uint16, 0 as uint8, game);
                WriteReg(OAMDATA as uint16, 0 as uint8, game);
                WriteReg(BGMODE as uint16, 1 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
                WriteReg(MOSAIC as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
                WriteReg(BG1SC as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
                WriteReg(BG2SC as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 0 as uint8;
                WriteReg(BG3SC as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
                WriteReg(BG4SC as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
                WriteReg(BG12NBA as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
                WriteReg(BG34NBA as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
                WriteReg(BG1HOFS as uint16, 0 as uint8, game);
                WriteReg(BG1HOFS as uint16, 0 as uint8, game);
                WriteReg(BG1VOFS as uint16, 0 as uint8, game);
                WriteReg(BG1VOFS as uint16, 0 as uint8, game);
                WriteReg(BG2HOFS as uint16, 0 as uint8, game);
                WriteReg(BG2HOFS as uint16, 0 as uint8, game);
                WriteReg(BG2VOFS as uint16, 0 as uint8, game);
                WriteReg(BG2VOFS as uint16, 0 as uint8, game);
                WriteReg(BG3HOFS as uint16, 0 as uint8, game);
                WriteReg(BG3HOFS as uint16, 0 as uint8, game);
                WriteReg(BG3VOFS as uint16, 0 as uint8, game);
                WriteReg(BG3VOFS as uint16, 0 as uint8, game);
                WriteReg(BG4HOFS as uint16, 0 as uint8, game);
                WriteReg(BG4HOFS as uint16, 0 as uint8, game);
                WriteReg(BG4VOFS as uint16, 0 as uint8, game);
                WriteReg(BG4VOFS as uint16, 0 as uint8, game);
                WriteReg(VMAIN as uint16, 0 as uint8, game);
                WriteReg(M7SEL as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
                WriteReg(M7A as uint16, 0 as uint8, game);
                WriteReg(M7B as uint16, 0 as uint8, game);
                WriteReg(M7C as uint16, 0 as uint8, game);
                WriteReg(M7D as uint16, 0 as uint8, game);
                WriteReg(M7X as uint16, 0 as uint8, game);
                WriteReg(M7Y as uint16, 0 as uint8, game);
                WriteReg(W12SEL as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
                WriteReg(W34SEL as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
                WriteReg(WOBJSEL as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
                WriteReg(WH0 as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x63) = 0 as uint8;
                WriteReg(WH1 as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x64) = 0 as uint8;
                WriteReg(WH2 as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x65) = 0 as uint8;
                WriteReg(WH3 as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x66) = 0 as uint8;
                WriteReg(WBGLOG as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x67) = 0 as uint8;
                WriteReg(WOBJLOG as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x68) = 0 as uint8;
                WriteReg(TM as uint16, 0x10 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
                WriteReg(TMW as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
                WriteReg(TS as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
                WriteReg(TSW as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
                WriteReg(CGWSEL as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
                WriteReg(CGADSUB as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
                WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
                WriteReg(SETINI as uint16, 0 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
                DecompressToMem(
                    0x9580d8 as uint32,
                    (&raw mut game.g_ram as *mut uint8).offset(0x15000),
                    game,
                );
                WriteReg(VMADDL as uint16, 0 as uint8, game);
                WriteReg(VMADDH as uint16, 0x60 as uint8, game);
                WriteReg(VMAIN as uint16, 0x80 as uint8, game);
                static unk_8B92B9: StartDmaCopy = StartDmaCopy {
                    chan: 1 as uint8,
                    dmap: 1 as uint8,
                    bbad: 0x18 as uint8,
                    a1: LongPtr {
                        addr: (0x7f5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                            as VoidP,
                        bank: (0x7f5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
                    },
                    das: 0x4000 as VoidP,
                };
                SetupDmaTransfer(&raw const unk_8B92B9 as *const ::core::ffi::c_void, game);
                WriteReg(MDMAEN as uint16, 2 as uint8, game);
                *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
                let mut v0: uint16 = 0 as uint16;
                loop {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        *(game.g_rom.offset(
                            (((0x8ce1e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x8ce1e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int)
                    {
                        break;
                    }
                }
                EnableNMI(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
                current_block_182 = 17485376261910781866;
            }
            1 => {
                current_block_182 = 3156160545233843657;
            }
            2 => {
                current_block_182 = 6110428972393212497;
            }
            3 => {
                current_block_182 = 2371591747199196472;
            }
            4 => {
                current_block_182 = 17628988452808763652;
            }
            5 => {
                current_block_182 = 17952583510507278934;
            }
            6 => {
                current_block_182 = 2484129932066553195;
            }
            _ => {
                current_block_182 = 2222055338596505704;
            }
        }
        loop {
            match current_block_182 {
                2222055338596505704 => {
                    break 's_645;
                }
                17628988452808763652 => {
                    let mut _coret__2: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret__2 != 0 {
                        *_state_variable_ = 4 as uint8;
                        return _coret__2 as CoroutineRet;
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) =
                        1 as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) =
                        1 as uint16;
                    current_block_182 = 7019009297990327870;
                }
                6110428972393212497 => {
                    let mut _coret__0: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret__0 != 0 {
                        *_state_variable_ = 2 as uint8;
                        return _coret__0 as CoroutineRet;
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) =
                        120 as uint16;
                    current_block_182 = 18425699056680496821;
                }
                17485376261910781866 => {
                    ClearOamExt(game);
                    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) =
                        0 as uint16;
                    AddNintentoLogoToOam(game);
                    if AdvanceFastScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                        != 0
                    {
                        ClearUnusedOam(game);
                        current_block_182 = 6110428972393212497;
                        continue;
                    } else {
                        ClearUnusedOam(game);
                        current_block_182 = 3156160545233843657;
                        continue;
                    }
                }
                3156160545233843657 => {
                    let mut _coret_: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    current_block_182 = 17485376261910781866;
                    continue;
                }
                2371591747199196472 => {
                    let mut _coret__1: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret__1 != 0 {
                        *_state_variable_ = 3 as uint8;
                        return _coret__1 as CoroutineRet;
                    }
                    current_block_182 = 18425699056680496821;
                }
                17952583510507278934 => {
                    let mut _coret__3: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret__3 != 0 {
                        *_state_variable_ = 5 as uint8;
                        return _coret__3 as CoroutineRet;
                    }
                    current_block_182 = 7019009297990327870;
                }
                _ => {
                    let mut _coret__4: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret__4 != 0 {
                        *_state_variable_ = 6 as uint8;
                        return _coret__4 as CoroutineRet;
                    }
                    WriteReg(INIDISP as uint16, 0x8f as uint8, game);
                    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x8f as uint8;
                    current_block_182 = 2222055338596505704;
                    continue;
                }
            }
            match current_block_182 {
                18425699056680496821 => {
                    ClearOamExt(game);
                    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) =
                        0 as uint16;
                    AddNintentoLogoToOam(game);
                    let ref mut fresh168 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
                    *fresh168 = (*fresh168).wrapping_sub(1);
                    if *fresh168 == 0 {
                        ClearUnusedOam(game);
                        current_block_182 = 17628988452808763652;
                    } else {
                        ClearUnusedOam(game);
                        current_block_182 = 2371591747199196472;
                    }
                }
                _ => {
                    ClearOamExt(game);
                    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) =
                        0 as uint16;
                    AddNintentoLogoToOam(game);
                    if AdvanceFastScreenFadeOut(game) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        != 0
                    {
                        ClearUnusedOam(game);
                        current_block_182 = 2484129932066553195;
                    } else {
                        ClearUnusedOam(game);
                        current_block_182 = 17952583510507278934;
                    }
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn AddNintentoLogoToOam(game: &mut Game) {
    DrawSpritemap(
        0x8c as uint8,
        addr_kSpriteMap_NintendoLogo as uint16,
        128 as uint16,
        112 as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn SpawnCinematicSpriteObject(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) = a;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)
            == 0
        {
            return SpawnCimenaticSpriteObjectInner(j, i as uint16, game);
        }
        i -= 2 as ::core::ffi::c_int;
    }
    return 1 as uint8;
}
pub unsafe fn SpawnCinematicSpriteObjectAtSlot(
    j: uint16,
    slot: uint16,
    param: uint16,
    game: &mut Game,
) -> uint8 {
    game.g_ram[0x1b9d..0x1b9f].copy_from_slice(&param.to_le_bytes());
    SpawnCimenaticSpriteObjectInner(j, slot, game)
}
pub unsafe fn SpawnCinematicSpriteObjectToR18(
    mut j: uint16,
    mut r18: uint16,
    game: &mut Game,
) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) = 0 as uint16;
    return SpawnCimenaticSpriteObjectInner(j, r18, game);
}
pub unsafe fn SpawnCimenaticSpriteObjectInner(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v2: *const uint8 = RomPtr_8B(k as uint16_t, game);
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v3 as isize) =
        *(v2.offset(2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v3 as isize) =
        *(v2.offset(4) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v3 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    CallCinematicSpriteObjectSetup(
        (*(v2 as *mut uint16) as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
        j,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn ClearCinematicSprites(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn HandleCinematicSprites(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a59) as *mut uint16) = i as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            ProcessCinematicSpriteInstructionList(i as uint16, game);
            i = *((&raw mut game.g_ram as *mut uint8).offset(0x1a59) as *mut uint16)
                as ::core::ffi::c_int;
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ProcessCinematicSpriteInstructionList(mut k: uint16, game: &mut Game) {
    CallCinematicSprPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a59) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1a59)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh22 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize);
    let fresh23 = *fresh22;
    *fresh22 = (*fresh22).wrapping_sub(1);
    if fresh23 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
            .offset(v2 as isize);
        let mut v6: uint16 = 0;
        loop {
            let mut v5: *const uint16 = RomPtr_8B(v4 as uint16_t, game) as *const uint16;
            v6 = *v5;
            if *v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            v4 = CallCinematicSprInstr(
                (v6 as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
                v1,
                (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let mut v7: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v7 as isize) =
            v6;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v7 as isize) =
            *(RomPtr_8B(v4 as uint16_t, game) as *mut uint16).offset(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v7 as isize) =
            (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    return 0 as uint16;
}
pub unsafe fn CinematicSprInstr_Sleep(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    return 0 as uint16;
}
pub unsafe fn CinematicSprInstr_SetPreInstr(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicSprInstr_ClearPreInstr(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x8b945f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicSprInstr_GotoRel(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return (j as ::core::ffi::c_int + *RomPtr_8B(j as uint16_t, game) as int8 as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn CinematicSprInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
}
pub unsafe fn CinematicSprInstr_DecTimerGoto(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh26 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize);
    let fresh27 = *fresh26;
    *fresh26 = (*fresh26).wrapping_sub(1);
    if fresh27 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        return CinematicSprInstr_Goto(k, j, game);
    };
}
pub unsafe fn CinematicSprInstr_DecTimerGotoRel(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh24 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize);
    let fresh25 = *fresh24;
    *fresh24 = (*fresh24).wrapping_sub(1);
    if fresh25 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    } else {
        return CinematicSprInstr_GotoRel(k, j, game);
    };
}
pub unsafe fn CinematicSprInstr_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub fn sub_8B94E1() {}
pub unsafe fn SpawnMode7Object(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    let mut Mode7ObjectDef: *mut Mode7ObjectDef = ::core::ptr::null_mut::<Mode7ObjectDef>();
    *((&raw mut game.g_ram as *mut uint8).offset(0x19b3) as *mut uint16) = a;
    let mut v3: uint16 = 2 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 1 as uint8;
        }
    }
    Mode7ObjectDef = get_Mode7ObjectDef(j, game);
    let mut v6: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16).offset(v6 as isize) =
        (*Mode7ObjectDef).field_2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16).offset(v6 as isize) =
        (*Mode7ObjectDef).field_4 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v6 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19ad) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    CallCinematicSpriteInit(
        ((*Mode7ObjectDef).field_0 as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int)
            as uint32,
        v6 as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn HandleMode7Objects(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x19b1) as *mut uint16) = i as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            ProcessMode7ObjectInstructions(i as uint16, game);
            i = *((&raw mut game.g_ram as *mut uint8).offset(0x19b1) as *mut uint16)
                as ::core::ffi::c_int;
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ProcessMode7ObjectInstructions(mut k: uint16, game: &mut Game) {
    CallMode7PreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x19b1) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x19b1)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh19 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v2 as isize);
    let fresh20 = *fresh19;
    *fresh19 = (*fresh19).wrapping_sub(1);
    if fresh20 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16)
            .offset(v2 as isize);
        let mut v6: uint16 = 0;
        loop {
            let mut v5: *const uint16 = RomPtr_8B(v4 as uint16_t, game) as *const uint16;
            v6 = *v5;
            if *v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            v4 = CallMode7Instr(
                (v6 as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
                v1,
                (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v6;
        let mut v7: *const uint8 = RomPtr_8B(v4 as uint16_t, game);
        QueueMode7Transfers(0x8b as uint8, *(v7.offset(2) as *mut uint16), game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Mode7Instr_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint16;
}
pub unsafe fn Mode7Instr_SetPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Mode7Instr_ClearPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x8b9585 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn Mode7Instr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
}
pub unsafe fn Mode7Instr_DecTimerAndGoto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh0 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19ad) as *mut uint16).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        return Mode7Instr_Goto(k, j, game);
    };
}
pub unsafe fn Mode7Instr_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19ad) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8B(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EnableCinematicBgObjects(game: &mut Game) {
    let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x19f1) as *mut uint16);
    *fresh13 = (*fresh13 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableCinematicBgObjects(game: &mut Game) {
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x19f1) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn EnableCinematicBgTilemapUpdates(game: &mut Game) {
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x19f3) as *mut uint16);
    *fresh14 = (*fresh14 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableCinematicBgTilemapUpdates(game: &mut Game) {
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x19f3) as *mut uint16);
    *fresh10 = (*fresh10 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ClearCinematicBgObjects(mut a: uint16, game: &mut Game) {
    let mut j: uint16 = 0;
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = a;
        i -= 2 as ::core::ffi::c_int;
    }
    j = 6 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19b5) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
}
pub unsafe fn CallCinematicBgObjectSetup(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        9147353 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn SpawnCinematicBgObject(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = a;
    let mut v3: uint16 = 6 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 1 as uint8;
        }
    }
    let mut v5: *const uint8 = RomPtr_8B(j as uint16_t, game);
    let mut v6: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19d5) as *mut uint16).offset(v6 as isize) =
        *(v5.offset(2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16).offset(v6 as isize) =
        *(v5.offset(4) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19dd) as *mut uint16).offset(v6 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19b5) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19e5) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    CallCinematicBgObjectSetup(
        (*(v5 as *mut uint16) as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
        v3,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn HandleCinematicBgObjects(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x19f1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x19ed) as *mut uint16) = i as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                ProcessCinematicBgObjectsInstrs(i as uint16, game);
                i = *((&raw mut game.g_ram as *mut uint8).offset(0x19ed) as *mut uint16)
                    as ::core::ffi::c_int;
            }
            i -= 2 as ::core::ffi::c_int;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x19f3) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            UpdateCinematicBgTilemap(game);
        }
        UpdateSamusEyesTilemap(game);
    }
}
pub unsafe fn ProcessCinematicBgObjectsInstrs(mut k: uint16, game: &mut Game) {
    CallCinematicBgPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x19d5) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x19ed) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x19ed)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh158 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19dd) as *mut uint16).offset(v2 as isize);
    let fresh159 = *fresh158;
    *fresh158 = (*fresh158).wrapping_sub(1);
    if fresh159 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16)
            .offset(v2 as isize);
        let mut v6: uint16 = 0;
        loop {
            let mut v5: *const uint16 = RomPtr_8C(v4 as uint16_t, game) as *const uint16;
            v6 = *v5;
            if *v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            v4 = CallCinematicBgInstr(
                (v6 as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
                v1,
                (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let mut v7: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19dd) as *mut uint16).offset(v7 as isize) =
            v6;
        let mut v8: *const uint8 = RomPtr_8C(v4 as uint16_t, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x19b5) as *mut uint16).offset(v7 as isize) =
            *(v8.offset(4) as *mut uint16);
        ProcessCinematicBgObject(v1, *(v8.offset(2) as *mut uint16), game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16).offset(v7 as isize) =
            (v4 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicBgInstr_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19b5) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    return 0 as uint16;
}
pub unsafe fn CinematicBgInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_8C(j as uint16_t, game) as *mut uint16);
}
pub fn sub_8B9743() {}
pub unsafe fn DrawCinematicSpriteObjects_Intro(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v1 as isize)
            != 0
        {
            let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d)
                as *mut uint16)
                .offset(v1 as isize);
            let mut chr: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1abd)
                as *mut uint16)
                .offset(v1 as isize);
            let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a7d)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a9d)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            if y as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                if ((y as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int)
                    < 0x1ff as ::core::ffi::c_int
                {
                    DrawSpritemapOffScreen(v2, x, y, chr, game);
                }
            } else if ((y as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int)
                < 0x1ff as ::core::ffi::c_int
            {
                DrawSpritemap(0x8c as uint8, v2, x, y, chr, game);
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn DrawCinematicSpriteObjects_Ending(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d) as *mut uint16).offset(v1 as isize)
            != 0
        {
            let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a5d)
                as *mut uint16)
                .offset(v1 as isize);
            let mut chr: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1abd)
                as *mut uint16)
                .offset(v1 as isize);
            let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a7d)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a9d)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            if y as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                if ((y as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int)
                    < 0x1ff as ::core::ffi::c_int
                {
                    DrawSpritemapOffScreen(v2, x, y, chr, game);
                }
            } else if ((y as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int)
                < 0x1ff as ::core::ffi::c_int
            {
                DrawSpritemap(0x8c as uint8, v2, x, y, chr, game);
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        - (0x8bda02 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        MoveUnusedSpritesOffScreen(game);
    }
}
pub unsafe fn SpawnTextGlowObject(mut j: uint16, mut r18: uint16, game: &mut Game) {
    let mut v1: uint16 = 14 as uint16;
    while *(((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16 as *mut uint8)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16 as *mut uint8)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = j;
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((v2 + 10 as ::core::ffi::c_int) as isize) = r18 as uint8 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16 as *mut uint8)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(&raw mut r18 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x1a37) as *mut uint16 as *mut uint8)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
}
pub unsafe fn HandleTextGlowObjects(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16) = i as uint16;
        if *(((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16)
            != 0
        {
            ProcessTextGlowObject(game);
            i = *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16)
                as ::core::ffi::c_int;
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ProcessTextGlowObject(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1a47)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh153 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    let fresh154 = *fresh153;
    *fresh153 = (*fresh153).wrapping_sub(1);
    if fresh154 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut R28: uint16 =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x1a37) as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        let mut v3: uint16 =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        let mut r22: uint16 = (2 as ::core::ffi::c_int
            * *(((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
                .offset((v1 + 10 as ::core::ffi::c_int) as isize) as *mut uint16
                as *mut uint8)
                .offset(0) as ::core::ffi::c_int
            + Mult8x8(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize),
                0x40 as uint8,
            ) as ::core::ffi::c_int) as uint16;
        let mut v4: uint16 = r22;
        let mut v5: *const uint8 = RomPtr_8C(v3 as uint16_t, game);
        let mut n: ::core::ffi::c_int = *v5.offset(2) as ::core::ffi::c_int;
        let mut norg: ::core::ffi::c_int = n;
        let mut m: ::core::ffi::c_int = *v5.offset(3) as ::core::ffi::c_int;
        loop {
            loop {
                *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                    .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
                    .offset(v4 as ::core::ffi::c_int as isize) as *mut uint16) =
                    (R28 as ::core::ffi::c_int
                        | *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000)
                            as *mut Ram3000))
                            .pause_menu_map_tilemap as *mut uint16
                            as *mut uint8)
                            .offset(v4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            & 0xe3ff as ::core::ffi::c_int) as uint16;
                v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                n -= 1;
                if !(n != 0) {
                    break;
                }
            }
            n = norg;
            m -= 1;
            if m == 0 {
                break;
            }
            r22 = (r22 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
            v4 = r22;
        }
        let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16);
        if *(((&raw mut game.g_ram as *mut uint8).offset(0x1a37) as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16) as ::core::ffi::c_int
            == 3072 as ::core::ffi::c_int
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16 as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16)
                        as ::core::ffi::c_int as isize,
                ) as *mut uint16) = 0 as uint16;
        } else {
            let ref mut fresh155 = *(((&raw mut game.g_ram as *mut uint8).offset(0x1a37)
                as *mut uint16 as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1a47) as *mut uint16)
                        as ::core::ffi::c_int as isize,
                ) as *mut uint16);
            *fresh155 = (*fresh155 as ::core::ffi::c_int + 1024 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(
                ((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 2 as ::core::ffi::c_int)
                    as isize,
            ) = 5 as uint16;
        }
    }
}
pub unsafe fn EnableTextGlowObjects_(game: &mut Game) {
    let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0x19ff) as *mut uint16);
    *fresh15 = (*fresh15 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableTextGlowObjects_(game: &mut Game) {
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x19ff) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func128(mut a: uint16, game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = a;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a01) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f9) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19fb) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19fd) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = 18432 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(0) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
}
pub unsafe fn CreditsObject_Init(mut j: uint16, game: &mut Game) {
    let mut v1: *const uint8 = RomPtr_8B(j as uint16_t, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19fd) as *mut uint16) =
        *(v1.offset(2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16) =
        *(v1.offset(4) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f9) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19fb) as *mut uint16) = 0 as uint16;
    CallCreditsObjectFunc(
        (*(v1 as *mut uint16) as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
        0 as uint16,
        game,
    );
}
pub unsafe fn CreditsObject_Process(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x19ff) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16) != 0 {
            CreditsObject_ProcessOne(game);
        }
        CinematicUpdateSomeBg(game);
    }
}
pub unsafe fn CreditsObject_ProcessOne(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    CallCreditsObjectFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x19fd) as *mut uint16) as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        0 as uint16,
        game,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        0x8000 as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(0)
            as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(0) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
        i = *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16)
            as ::core::ffi::c_int;
        loop {
            let mut v2: *const uint16 = RomPtr_8C(i as uint16_t, game) as *const uint16;
            let mut v3: uint16 = *v2;
            if *v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            i = CallIntroObjectInstr(
                (v3 as ::core::ffi::c_int | 0x8b0000 as ::core::ffi::c_int) as uint32,
                0 as uint16,
                (i + 2 as ::core::ffi::c_int) as uint16,
                game,
            ) as ::core::ffi::c_int;
            if i == 0 {
                return;
            }
        }
        CreditsObject_Func1(i as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a01) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1a01) as *mut uint16)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16) =
            (i + 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CreditsObject_Func1(mut j: uint16, game: &mut Game) {
    let mut RegWord: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a01) as *mut uint16) as uint8,
        0x40 as uint8,
    );
    let mut n: ::core::ffi::c_int = 31 as ::core::ffi::c_int;
    let mut src: *const uint16 = (&raw mut game.g_ram as *mut uint8).offset(0x10000).offset(
        *(RomPtr_8C(j as uint16_t, game) as *mut uint16).offset(1) as ::core::ffi::c_int as isize,
    ) as *const uint16;
    loop {
        let fresh6 = src;
        src = src.offset(1);
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(RegWord as ::core::ffi::c_int as isize) as *mut uint16) = *fresh6;
        RegWord = (RegWord as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn IntroObject_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f7) as *mut uint16) = 0 as uint16;
    return 0 as uint16;
}
pub unsafe fn IntroObject_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_8C(j as uint16_t, game) as *mut uint16);
}
pub unsafe fn IntroObject_DecTimerGoto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x19fb) as *mut uint16);
    *fresh8 = (*fresh8).wrapping_sub(1);
    if *fresh8 != 0 {
        return IntroObject_Goto(k, j, game);
    } else {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn IntroObject_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19fb) as *mut uint16) =
        *(RomPtr_8C(j as uint16_t, game) as *mut uint16);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn GameState_1_OpeningCinematic_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        game,
    );
    HandleCinematicSprites(game);
    HandleMode7Objects(game);
    PaletteFxHandler(game);
    DrawCinematicSpriteObjects_Intro(game);
    MaybeSkipCinematics(game);
    HandleCinematicsTransitions(game);
    HandleMode7TransformationNoRotation(game);
    return kCoroutineNone as CoroutineRet;
}
pub fn CinematicFunctionNone() {}
pub unsafe fn MaybeSkipCinematics(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        + 0x60d8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & (kButton_B as ::core::ffi::c_int
                | kButton_Start as ::core::ffi::c_int
                | kButton_A as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn HandleCinematicsTransitions_1(game: &mut Game) {
    if AdvanceFastScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9f28 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueMusic_Delayed8(6 as uint16, game);
    }
}
pub unsafe fn HandleCinematicsTransitions_2(game: &mut Game) {
    ClearCinematicSprites(game);
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA119 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA125 as uint16,
        0x0400, // Palette left in A by the first sprite initializer.
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) = 3 as uint16;
    let mut v2: uint16 = 256 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce1e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce1e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(201) =
        0x7fff as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(202) =
        32128 as uint16;
    ClearPaletteFXObjects(game);
    SpawnPalfxObject(addr_kPalfx_E1A0 as uint16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as uint16, game);
    SetPpuBackdropBlack(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    SpawnTitleScreenGradientObjs(game);
}
pub unsafe fn CinematicSpriteInit_7(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
}
pub unsafe fn CinematicSpriteInit_8(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        81 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicSpriteInit_9(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        196 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn HandleCinematicsTransitions_3(game: &mut Game) {
    if AdvanceFastScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16) = 900 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9f29 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
static off_8B9A7B: [Option<Func_V>; 4] = unsafe {
    [
        None,
        Some(HandleCinematicsTransitions_1 as unsafe fn(&mut Game) -> ()),
        Some(HandleCinematicsTransitions_2 as unsafe fn(&mut Game) -> ()),
        Some(HandleCinematicsTransitions_3 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleCinematicsTransitions(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) != 0 {
        off_8B9A7B[*((&raw mut game.g_ram as *mut uint8).offset(0x1a53) as *mut uint16) as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn CinematicFunctionOpening(game: &mut Game) {
    LoadTitleSequenceGraphics(game);
    QueueMusic_Delayed8(0xff03 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA0EF as uint16,
        (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
    QueueMusic_Delayed8(5 as uint16, game);
}
pub unsafe fn LoadTitleSequenceGraphics(game: &mut Game) {
    let mut v1: int16 = 0;
    SetupPpuForTitleSequence(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7) as *mut uint16) = 0 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce1e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce1e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x94e000 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x96fc04 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    DecompressToMem(
        0x9580d8 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x15000),
        game,
    );
    DecompressToMem(
        0x95a5e1 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x19000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8B9BFF: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8B9BFF as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    v1 = 0x4000 as int16;
    loop {
        WriteReg(VMDATAL as uint16, 0xff as uint8, game);
        v1 -= 1;
        if !(v1 != 0) {
            break;
        }
    }
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    static unk_8B9C37: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f4000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f4000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x1000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8B9C37 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8B9C57: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8B9C57 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 128 as uint16;
    WriteRegWord(M7Y as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 128 as uint16;
    EnableNMI(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    EnableHdmaObjects(game);
    EnablePaletteFx(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
    SpawnPalfxObject(addr_kPalfx_E1A0 as uint16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as uint16, game);
    SpawnMode7Object(
        addr_kMode7ObjBabyMetroidInTitle as uint16,
        0x000c, // The second title palette allocation leaves its slot offset in A.
        game,
    );
}
pub unsafe fn CinematicSpriteInit_0(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        129 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
}
pub unsafe fn sub_8B9CCF(mut k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub fn sub_8B9CDE() {}
pub unsafe fn CinematicSprInstr_9CE1(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9d17 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 72 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 315 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 225 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) =
        -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0x8000 as uint16;
    SetPpuBackdropSomeColor(game);
    return j;
}
pub unsafe fn CinematicFunc_Func2(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) as ::core::ffi::c_int
        + 7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA0F5 as uint16,
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_1(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        129 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
}
pub unsafe fn CinematicSprInstr_9D5D(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9d90 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 44 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(155 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) =
        -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0x8000 as uint16;
    return j;
}
pub unsafe fn CinematicFunc_Func4(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) as ::core::ffi::c_int
        + 176 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA0FB as uint16,
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_2(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        129 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
}
pub unsafe fn CinematicSprInstr_9DD6(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9e12 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        -(177 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(160 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0x8000 as uint16;
    SetPpuBackdropSomeColor(game);
    return j;
}
pub unsafe fn CinematicFunc_Func5(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) as ::core::ffi::c_int
        - 163 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA101 as uint16,
            (0x8b9a47 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_3(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        129 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
}
pub unsafe fn CinematicSprInstr_Func8(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9e8b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 67 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
    SpawnTitleScreenGradientObjs(game);
    return j;
}
pub unsafe fn CinematicFunc_Func7(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16)
            as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh149 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
            *fresh149 = (*fresh149).wrapping_add(1);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8b9f28 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BA107 as uint16,
                (0x8b9f28 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
                game,
            );
        }
    }
}
pub unsafe fn CinematicSpriteInit_4(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    SpawnPalfxObject(addr_kPalfx_E194 as uint16, game);
}
pub unsafe fn CinematicSprInstr_Func9(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0x9ecd as uint16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA113 as uint16, v0, game);
    return j;
}
pub unsafe fn CinematicSpriteInit_5(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        81 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
    SpawnPalfxObject(addr_kPalfx_E198 as uint16, game);
}
pub unsafe fn CinematicSprInstr_sub_8B9EF0(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA113 as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSpriteInit_6(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        196 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
    SpawnPalfxObject(addr_kPalfx_E19C as uint16, game);
}
pub unsafe fn CinematicSprInstr_9F19(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16) = 900 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8b9f29 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunc_Func1(game: &mut Game) {
    let mut current_block: u64;
    let ref mut fresh148 = *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16);
    *fresh148 = (*fresh148).wrapping_sub(1);
    let mut v0: bool =
        *fresh148 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9fae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        current_block = 5176254373955358112;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_B as ::core::ffi::c_int
            | kButton_Start as ::core::ffi::c_int
            | kButton_A as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9f52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        current_block = 5176254373955358112;
    } else {
        current_block = 4906268039856690917;
    }
    match current_block {
        5176254373955358112 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 2 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
        }
        _ => {}
    };
}
pub unsafe fn CinematicFunc_Func10(game: &mut Game) {
    let mut j: uint16 = 0;
    if AdvanceSlowScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        EnableNMI(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisablePaletteFx(game);
        ClearPaletteFXObjects(game);
        let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        j = 510 as uint16;
        while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0 as uint16;
            j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunc_Func9(game: &mut Game) {
    let mut j: uint16 = 0;
    if AdvanceSlowScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        EnableNMI(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisablePaletteFx(game);
        ClearPaletteFXObjects(game);
        let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        j = 510 as uint16;
        while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0 as uint16;
            j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 40 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f57) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn ConfigureTitleSequenceGradientHDMA(game: &mut Game) {
    let mut v0: uint16 = *(game.g_rom.offset(
        (((0x8cbc5d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x8cbc5d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16)
                as ::core::ffi::c_int
                & 0xf0 as ::core::ffi::c_int)
                >> 4 as ::core::ffi::c_int) as isize,
        );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut v2: uint16 = *(RomPtr_8C(v0 as uint16_t, game) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = v2;
        if v2 == 0 {
            break;
        }
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn PlayBabyMetroidCry1(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    QueueSfx3_Max6(0x23 as uint16, game);
    return j;
}
pub unsafe fn PlayBabyMetroidCry2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    QueueSfx3_Max6(0x26 as uint16, game);
    return j;
}
pub unsafe fn PlayBabyMetroidCry3(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    QueueSfx3_Max6(0x27 as uint16, game);
    return j;
}
static kCinematicPal1: [uint16; 16] = [
    0x1000 as uint16,
    0x6bf5 as uint16,
    0x2e41 as uint16,
    0x2da1 as uint16,
    0x2d01 as uint16,
    0x5e5f as uint16,
    0x183f as uint16,
    0x1014 as uint16,
    0x80a as uint16,
    0x404 as uint16,
    0x4f9f as uint16,
    0x3ed8 as uint16,
    0x2e12 as uint16,
    0x6f70 as uint16,
    0x7fff as uint16,
    0x5ee0 as uint16,
];
static kCinematicPal2: [uint16; 16] = [
    0x3800 as uint16,
    0x6bf5 as uint16,
    0x6e1 as uint16,
    0x641 as uint16,
    0x5a1 as uint16,
    0x5e5f as uint16,
    0x183f as uint16,
    0x1014 as uint16,
    0x80a as uint16,
    0x404 as uint16,
    0x4f9f as uint16,
    0x3ed8 as uint16,
    0x2e12 as uint16,
    0x6f70 as uint16,
    0x7fff as uint16,
    0x5ee0 as uint16,
];
static kCinematicPal3: [uint16; 16] = [
    0x3800 as uint16,
    0x77f8 as uint16,
    0x1344 as uint16,
    0x12a4 as uint16,
    0x1204 as uint16,
    0x6abf as uint16,
    0x249f as uint16,
    0x1c77 as uint16,
    0x146d as uint16,
    0x1067 as uint16,
    0x5bff as uint16,
    0x4b38 as uint16,
    0x3a72 as uint16,
    0x7bd3 as uint16,
    0x7fff as uint16,
    0x6b43 as uint16,
];
static kCinematicPal4: [uint16; 16] = [
    0x3800 as uint16,
    0x7ffb as uint16,
    0x1fa7 as uint16,
    0x1f07 as uint16,
    0x1e67 as uint16,
    0x771f as uint16,
    0x30ff as uint16,
    0x28da as uint16,
    0x20d0 as uint16,
    0x1cca as uint16,
    0x67ff as uint16,
    0x579b as uint16,
    0x46d5 as uint16,
    0x7bd6 as uint16,
    0x7fff as uint16,
    0x77a6 as uint16,
];
pub unsafe fn CinematicSetPal1(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 48 as ::core::ffi::c_int) as isize) =
            kCinematicPal1[(i >> 1 as ::core::ffi::c_int) as usize];
        i -= 2 as ::core::ffi::c_int;
    }
    return j;
}
pub unsafe fn CinematicSetPal2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 48 as ::core::ffi::c_int) as isize) =
            kCinematicPal2[(i >> 1 as ::core::ffi::c_int) as usize];
        i -= 2 as ::core::ffi::c_int;
    }
    return j;
}
pub unsafe fn CinematicSetPal3(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 48 as ::core::ffi::c_int) as isize) =
            kCinematicPal3[(i >> 1 as ::core::ffi::c_int) as usize];
        i -= 2 as ::core::ffi::c_int;
    }
    return j;
}
pub unsafe fn CinematicSetPal4(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 48 as ::core::ffi::c_int) as isize) =
            kCinematicPal4[(i >> 1 as ::core::ffi::c_int) as usize];
        i -= 2 as ::core::ffi::c_int;
    }
    return j;
}
pub unsafe fn GameState_37_CeresGoesBoomWithSamus_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh150 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16);
        *fresh150 = (*fresh150).wrapping_add(1);
    }
    let ref mut fresh151 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16);
    *fresh151 = (*fresh151).wrapping_add(1);
    HandleSamusDuringIntro(game);
    HandleCinematicSprites(game);
    HandleMode7Objects(game);
    HandleCinematicBgObjects(game);
    HandleTextGlowObjects(game);
    PaletteFxHandler(game);
    DrawIntroSprites(game);
    HandleMode7TransformationMatrix(game);
    return kCoroutineNone as CoroutineRet;
}
pub fn CinematicFunc_Nothing() {}
pub unsafe fn CinematicFunction_Intro_Initial(game: &mut Game) {
    let mut j: uint16 = 0;
    let mut k: uint16 = 0;
    SetupPpu_Intro(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a7) as *mut uint16) = 16 as uint16;
    Samus_Initialize(game);
    UpdateBeamTilesAndPalette(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) = 900 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) = 900 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    ResetButtonAssignmentsToDefault(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 0 as uint16;
    let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x95f90e as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x95d089 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x18000),
        game,
    );
    DecompressToMem(
        0x9788cc as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x19000),
        game,
    );
    DecompressToMem(
        0x96ff14 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x19800),
        game,
    );
    DecompressToMem(
        0x95e4c2 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1b800),
        game,
    );
    DecompressToMem(
        0x978d12 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA469: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x8000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA469 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA489: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x900 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA489 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x48 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA4A9: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f9000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f9000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA4A9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x4c as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA4C9: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fe000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fe000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA4C9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x50 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA4E9: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f9800 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f9800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA4E9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA509: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x9ad200 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x9ad200 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA509 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x6e as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA529: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fb800 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fb800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2400 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA529 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    DecompressToMem(
        0x95d713 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1a000),
        game,
    );
    EnableNMI(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 3 as uint16;
    EnableHdmaObjects(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) = 0 as uint16;
    j = 1023 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .cinematic_bg_tilemap[j as usize] =
            *((&raw mut game.g_ram as *mut uint8).offset(0x16402).offset(
                (j as ::core::ffi::c_int * 2 as ::core::ffi::c_int + 11262 as ::core::ffi::c_int)
                    as isize,
            ) as *mut uint8 as *mut uint16);
        j = j.wrapping_sub(1);
    }
    k = 127 as uint16;
    while k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[k as usize] = *((game.g_rom.offset(
            (((0x8cd81b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8cd81b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset((k as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as isize)
            as *mut uint16);
        k = k.wrapping_sub(1);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8ba5a7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff3f as uint16, game);
}
pub unsafe fn CinematicFunction_Intro_FadeIn(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba5bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_DrawInitJpn(game: &mut Game) {
    if AdvanceSlowScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba5f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 60 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
            let mut v0: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            (*gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .size = 256 as uint16;
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 12288 as uint16;
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            let fresh146 = v0;
            v0 = v0.wrapping_add(1);
            *(&raw mut (*(gVramWriteEntry
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
                fresh146 as ::core::ffi::c_int,
                game,
            ))
            .size as *mut uint8)
                .offset(0) = 126 as uint8;
            (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 20192 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_LastMetroidCaptivity(game: &mut Game) {
    let ref mut fresh145 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh145 = (*fresh145).wrapping_sub(1);
    let mut v0: bool =
        *fresh145 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba613 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 200 as uint16;
        QueueMusic_Delayed8(5 as uint16, game);
    }
}
pub unsafe fn CinematicFunc_Intro_QueueGalaxyIsAtPeace(game: &mut Game) {
    let ref mut fresh143 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh143 = (*fresh143).wrapping_sub(1);
    let mut v0: bool =
        *fresh143 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(0xff42 as uint16, game);
        QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba639 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForQueueWait4secs(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba64c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 240 as uint16;
    }
}
pub unsafe fn CinematicFunc_Intro_FadeOut(game: &mut Game) {
    let ref mut fresh141 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh141 = (*fresh141).wrapping_sub(1);
    let mut v0: bool =
        *fresh141 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba663 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForFadeOut(game: &mut Game) {
    if AdvanceSlowScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba66f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunc_Intro_SetupTextPage2(game: &mut Game) {
    ClearJapaneseTextTiles(game);
    WriteReg(VMADDL as uint16, 0x80 as uint8, game);
    WriteReg(VMADDH as uint16, 0x41 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BA689: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e4000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e4000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BA689 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(TM as uint16, 0x16 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 22 as uint8;
    ClearCinematicBgObjects(0x2f as uint16, game);
    EnableCinematicBgObjects(game);
    EnableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) =
        addr_unk_604C00 as uint16;
    let mut i: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[i as usize] = 15401 as uint16;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 896 as ::core::ffi::c_int) as usize] = 15401 as uint16;
        i -= 1;
    }
    let mut v1: uint16 = 768 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[v1 as usize] = *(game.g_rom.offset(
            (((0x8ba72b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ba72b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 = v1.wrapping_add(1);
        if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1792 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .menu
        .menu_tilemap as *mut uint8)
        .offset(286) as *mut uint8 as *mut uint32) = 472456233 as uint32;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCE6D as uint16,
        0x1c29 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 127 as uint16;
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF63 as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 8 as uint16;
    ClearEnglishTextTilemap(game);
    SetSomeStuffForSpriteObject_16(game);
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff36 as uint16, game);
    QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8ba82b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunc_Intro_WaitFadeinShowText(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba84a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
        SpawnCinematicBgObject(
            addr_kCinematicBgObjectDef_8BCF3F as uint16,
            0x4c00 as uint16,
            game,
        );
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForFadeinSleep(game: &mut Game) {
    if AdvanceSlowScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba391 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ClearEnglishTextTilemap(game: &mut Game) {
    let mut v0: uint16 = 128 as uint16;
    loop {
        let fresh5 = v0;
        v0 = v0.wrapping_add(1);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[fresh5 as usize] = 47 as uint16;
        if !(((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn ClearJapaneseTextTiles(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [v0 as usize] = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(7822) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7824) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7826) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7828) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7830) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7832) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7834) as *mut uint8 as *mut uint16);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(7836) as *mut uint8 as *mut uint16);
        v0 = (v0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn SetSomeStuffForSpriteObject_2(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_3(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        155 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_4_MetroidEgg(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - 169 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_word_8BCB3B as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicSprPreInstr_A903(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) == 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicSprInstr_SpawnMetroidEggParticles(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCECD as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCED3 as uint16,
        1 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCED9 as uint16,
        2 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEDF as uint16,
        3 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEE5 as uint16,
        4 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEEB as uint16,
        5 as uint16,
        game,
    );
    QueueSfx2_Max6(0xb as uint16, game);
    return j;
}
static kSetSomeStuffForSpriteObject_6_Tab: [int16; 12] = [
    0x5c as int16,
    0x58 as int16,
    0x63 as int16,
    0x58 as int16,
    0x59 as int16,
    0x5d as int16,
    0x60 as int16,
    0x5b as int16,
    0x66 as int16,
    0x5e as int16,
    0x63 as int16,
    0x60 as int16,
];
pub unsafe fn SetSomeStuffForSpriteObject_6(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (kSetSomeStuffForSpriteObject_6_Tab[v3 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kSetSomeStuffForSpriteObject_6_Tab[(v3 + 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            + 59 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        3584 as uint16;
}
static kSetSomeStuffForSpriteObject_7_Tab0: [int16; 12] = [
    -(1 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    0 as int16,
    0x4000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    0 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    0 as int16,
    0x2000 as int16,
];
static kSetSomeStuffForSpriteObject_7_Tab1: [uint16; 82] = [
    0xfffe as uint16,
    0 as uint16,
    0xfffe as uint16,
    0xc000 as uint16,
    0xfffe as uint16,
    0xa000 as uint16,
    0xfffe as uint16,
    0x8000 as uint16,
    0xfffe as uint16,
    0x6000 as uint16,
    0xfffe as uint16,
    0x4000 as uint16,
    0xfffe as uint16,
    0x2000 as uint16,
    0xffff as uint16,
    0 as uint16,
    0xffff as uint16,
    0xc000 as uint16,
    0xffff as uint16,
    0xa000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x6000 as uint16,
    0xffff as uint16,
    0x4000 as uint16,
    0xffff as uint16,
    0x2000 as uint16,
    0 as uint16,
    0 as uint16,
    0 as uint16,
    0x2000 as uint16,
    0 as uint16,
    0x4000 as uint16,
    0 as uint16,
    0x6000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0xa000 as uint16,
    0 as uint16,
    0xc000 as uint16,
    0 as uint16,
    0xe000 as uint16,
    1 as uint16,
    0 as uint16,
    1 as uint16,
    0x2000 as uint16,
    1 as uint16,
    0x4000 as uint16,
    1 as uint16,
    0x6000 as uint16,
    1 as uint16,
    0x8000 as uint16,
    1 as uint16,
    0xa000 as uint16,
    1 as uint16,
    0xc000 as uint16,
    1 as uint16,
    0xe000 as uint16,
    2 as uint16,
    0 as uint16,
    2 as uint16,
    0x2000 as uint16,
    2 as uint16,
    0x4000 as uint16,
    2 as uint16,
    0x6000 as uint16,
    2 as uint16,
    0x8000 as uint16,
    2 as uint16,
    0xa000 as uint16,
    2 as uint16,
    0xc000 as uint16,
    2 as uint16,
    0xe000 as uint16,
    0x9dad as uint16,
    0x991b as uint16,
    0x1b7d as uint16,
    0x97ad as uint16,
    0x991a as uint16,
    0x1a7d as uint16,
];
pub unsafe fn SetSomeStuffForSpriteObject_7(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *(((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int;
    '_c2rust_label: {
        if (v3 + 1 as ::core::ffi::c_int) < 12 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"v3 + 1 < 12\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_8b.c\0" as *const u8 as *const ::core::ffi::c_char,
                2389 as ::core::ffi::c_uint,
                b"void SetSomeStuffForSpriteObject_7(uint16)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (kSetSomeStuffForSpriteObject_7_Tab0[v3 as usize] as uint32) << 16 as ::core::ffi::c_int
            | kSetSomeStuffForSpriteObject_7_Tab0[(v3 + 1 as ::core::ffi::c_int) as usize] as uint16
                as uint32,
    );
    let mut v6: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *(((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int;
    '_c2rust_label_0: {
        if (v6 + 1 as ::core::ffi::c_int) < 82 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"v6 + 1 < 82\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_8b.c\0" as *const u8 as *const ::core::ffi::c_char,
                2392 as ::core::ffi::c_uint,
                b"void SetSomeStuffForSpriteObject_7(uint16)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (kSetSomeStuffForSpriteObject_7_Tab1[v6 as usize] as uint32) << 16 as ::core::ffi::c_int
            | kSetSomeStuffForSpriteObject_7_Tab1[(v6 + 1 as ::core::ffi::c_int) as usize]
                as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 168 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize);
        *fresh66 = (*fresh66 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_8(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(13);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(13);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
static kSetSomeStuffForSpriteObject_9_Tab0: [int16; 10] = [
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    1 as int16,
    0 as int16,
    0 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
];
static kSetSomeStuffForSpriteObject_9_Tab2: [int16; 124] = [
    -(2 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x6000 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x6000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    0 as int16,
    0 as int16,
    0 as int16,
    0x2000 as int16,
    0 as int16,
    0x4000 as int16,
    0 as int16,
    0x6000 as int16,
    0 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    1 as int16,
    0 as int16,
    1 as int16,
    0x2000 as int16,
    1 as int16,
    0x4000 as int16,
    1 as int16,
    0x6000 as int16,
    1 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    2 as int16,
    0 as int16,
    2 as int16,
    0x2000 as int16,
    2 as int16,
    0x4000 as int16,
    2 as int16,
    0x6000 as int16,
    2 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    3 as int16,
    0 as int16,
    3 as int16,
    0x2000 as int16,
    3 as int16,
    0x4000 as int16,
    3 as int16,
    0x6000 as int16,
    3 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    4 as int16,
    0 as int16,
    4 as int16,
    0x2000 as int16,
    4 as int16,
    0x4000 as int16,
    4 as int16,
    0x6000 as int16,
    4 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    5 as int16,
    0 as int16,
    5 as int16,
    0x2000 as int16,
    5 as int16,
    0x4000 as int16,
    5 as int16,
    0x6000 as int16,
    5 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
];
static kSetSomeStuffForSpriteObject_9_Tab1: [int16; 138] = [
    -(3 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    0x6000 as int16,
    -(3 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    -(3 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x6000 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x6000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x4000 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0x2000 as int16,
    0 as int16,
    0 as int16,
    0 as int16,
    0x2000 as int16,
    0 as int16,
    0x4000 as int16,
    0 as int16,
    0x6000 as int16,
    0 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    1 as int16,
    0 as int16,
    1 as int16,
    0x2000 as int16,
    1 as int16,
    0x4000 as int16,
    1 as int16,
    0x6000 as int16,
    1 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    1 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    2 as int16,
    0 as int16,
    2 as int16,
    0x2000 as int16,
    2 as int16,
    0x4000 as int16,
    2 as int16,
    0x6000 as int16,
    2 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    2 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    3 as int16,
    0 as int16,
    3 as int16,
    0x2000 as int16,
    3 as int16,
    0x4000 as int16,
    3 as int16,
    0x6000 as int16,
    3 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    3 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    4 as int16,
    0 as int16,
    4 as int16,
    0x2000 as int16,
    4 as int16,
    0x4000 as int16,
    4 as int16,
    0x6000 as int16,
    4 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
    5 as int16,
    0 as int16,
    5 as int16,
    0x2000 as int16,
    5 as int16,
    0x4000 as int16,
    5 as int16,
    0x6000 as int16,
    5 as int16,
    -(0x8000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x6000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x4000 as ::core::ffi::c_int) as int16,
    5 as int16,
    -(0x2000 as ::core::ffi::c_int) as int16,
];
pub unsafe fn SetSomeStuffForSpriteObject_9(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *(((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int;
    '_c2rust_label: {
        if (v3 + 1 as ::core::ffi::c_int) < 10 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"v3 + 1 < 10\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_8b.c\0" as *const u8 as *const ::core::ffi::c_char,
                2442 as ::core::ffi::c_uint,
                b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (kSetSomeStuffForSpriteObject_9_Tab0[v3 as usize] as uint32) << 16 as ::core::ffi::c_int
            | kSetSomeStuffForSpriteObject_9_Tab0[(v3 + 1 as ::core::ffi::c_int) as usize] as uint16
                as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v12: ::core::ffi::c_int = 2 as ::core::ffi::c_int
            * *(((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int;
        '_c2rust_label_0: {
            if (v12 + 1 as ::core::ffi::c_int) < 124 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"v12 + 1 < 124\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/sm_8b.c\0" as *const u8 as *const ::core::ffi::c_char,
                    2446 as ::core::ffi::c_uint,
                    b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            (kSetSomeStuffForSpriteObject_9_Tab2[v12 as usize] as uint32)
                << 16 as ::core::ffi::c_int
                | kSetSomeStuffForSpriteObject_9_Tab2[(v12 + 1 as ::core::ffi::c_int) as usize]
                    as uint16 as uint32,
        );
    } else {
        let mut v6: ::core::ffi::c_int = 2 as ::core::ffi::c_int
            * *(((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int;
        '_c2rust_label_1: {
            if (v6 + 1 as ::core::ffi::c_int) < 138 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"v6 + 1 < 138\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/sm_8b.c\0" as *const u8 as *const ::core::ffi::c_char,
                    2450 as ::core::ffi::c_uint,
                    b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        };
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            (kSetSomeStuffForSpriteObject_9_Tab1[v6 as usize] as uint32)
                << 16 as ::core::ffi::c_int
                | kSetSomeStuffForSpriteObject_9_Tab1[(v6 + 1 as ::core::ffi::c_int) as usize]
                    as uint16 as uint32,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 168 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_word_8BCD71 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize);
        *fresh65 = (*fresh65 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_10(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        84 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        139 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_11(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) != 0 {
                let ref mut fresh63 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16);
                *fresh63 = (*fresh63).wrapping_sub(1);
                let ref mut fresh64 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d)
                    as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                *fresh64 = (*fresh64).wrapping_add(1);
            }
        }
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_12(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        111 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_13(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let ref mut fresh61 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
                *fresh61 = (*fresh61).wrapping_add(1);
                let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d)
                    as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                *fresh62 = (*fresh62).wrapping_sub(1);
            }
        }
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    };
}
pub unsafe fn CinematicBgInstr_SetSomeStuffForSpriteObject_14(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(15) =
        addr_unk_8BCC03 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(15) = 1 as uint16;
    return j;
}
pub unsafe fn SetSomeStuffForSpriteObject_15(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(15) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(15) = 248 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_16(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(15) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(15) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(15) =
        addr_word_8BCBFB as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(15) = 1 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_17(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        216 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn SetSomeStuffForSpriteObject_18(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) as ::core::ffi::c_int
        == 59 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_word_8BCC23 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) as ::core::ffi::c_int
        == 127 as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v2 as isize) =
            addr_off_8BCE53 as uint16;
    }
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page1(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD401 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page1(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8baeb8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page2(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD407 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page2(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD40D as uint16, 0 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8baf6c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_WaitInputSetupMotherBrainFight(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) != 0 {
        let ref mut fresh139 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16);
        *fresh139 = (*fresh139).wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 80 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
            kPose_02_FaceL_Normal as uint16;
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = 155 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) = 155 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = 115 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = 115 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 127 as uint16;
        SpawnCinematicSpriteObjectAtSlot(
            addr_kCinematicSpriteObjectDef_8BCE55 as uint16,
            0 as uint16,
            0x007f, // The preceding $1A4B store leaves $007F in A.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF27 as uint16,
            0x0e00, // Palette left in A by the first sprite initializer.
            game,
        );
        let mut v1: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0x8cbec3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cbec3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v1 as ::core::ffi::c_int - 448 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut v2: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(v2 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
            v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
            (0x90e6c9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
            (0x90e833 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        ResetDemoData(game);
        EnableDemoInput(game);
        // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
        LoadDemoInputObject(0x8000, addr_stru_918784 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        CinematicFunction_Intro_SetupTransitionToGameplay(game);
    }
}
pub unsafe fn CinematicFunction_Intro_WaitInputSetupBabyMetroid(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) != 0 {
        let ref mut fresh138 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16);
        *fresh138 = (*fresh138).wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 84 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) = 32 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a7) as *mut uint16) = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
            kPose_02_FaceL_Normal as uint16;
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = 376 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) = 376 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = 147 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = 147 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 127 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE5B as uint16,
            0x7f as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE79 as uint16,
            0x0e00, // Palette left in A by the first sprite initializer.
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16) = 0 as uint16;
        let mut v1: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0x8cc083 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cc083 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v1 as ::core::ffi::c_int - 768 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
            (0x90e6c9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
            (0x90e833 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        ResetDemoData(game);
        EnableDemoInput(game);
        // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
        LoadDemoInputObject(0x8000, addr_stru_91877E as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) = 1 as uint16;
        CinematicFunction_Intro_SetupTransitionToGameplay(game);
    }
}
pub unsafe fn CinematicFunction_Intro_SetupTransitionToGameplay(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 6 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    SpawnIntroCutsceneCrossfadeHdma(game);
    SetSomeStuffForSpriteObject_15(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb250 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 256 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x28 as uint16, 3 as uint16, game);
    ClearYColorsFromIndexX(0xe0 as uint16, 0x10 as uint16, game);
    ClearYColorsFromIndexX(0x180 as uint16, 0x20 as uint16, game);
    ClearYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
    ComposeFadingPalettes(game);
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page3(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD413 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page3(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD419 as uint16, 0 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bb0f2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page4(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD41F as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page4(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD425 as uint16, 0 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bb123 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func11(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) != 0 {
        let ref mut fresh137 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16);
        *fresh137 = (*fresh137).wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 88 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 32 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 127 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE61 as uint16,
            0x7f as uint16,
            game,
        );
        CinematicFunction_Intro_Func13(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func12(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) != 0 {
        let ref mut fresh136 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16);
        *fresh136 = (*fresh136).wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 92 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
            -(24 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 127 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE67 as uint16,
            0x7f as uint16,
            game,
        );
        CinematicFunction_Intro_Func13(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func13(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 6 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    SpawnIntroCutsceneCrossfadeHdma(game);
    SetSomeStuffForSpriteObject_15(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb2d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 256 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
    ClearYColorsFromIndexX(0x1c0 as uint16, 9 as uint16, game);
    ComposeFadingPalettes(game);
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page5(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD42B as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page5(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD431 as uint16, 0 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bb1da as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func15b(game: &mut Game) {
    EnableCinematicBgTilemapUpdates(game);
    ClearEnglishTextTilemap(game);
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF5D as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8ba390 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    SetSomeStuffForSpriteObject_16(game);
    QueueMusic_Delayed8(0 as uint16, game);
}
pub unsafe fn CinematicFunction_Intro_Func15(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) != 0 {
        let ref mut fresh135 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16);
        *fresh135 = (*fresh135).wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 127 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8bb207 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            DisableCinematicBgTilemapUpdates(game);
            ClearJapaneseTextTiles(game);
            TransferJapaneseTextTilesToVram(game);
        } else {
            CinematicFunction_Intro_Func15b(game);
        }
    }
}
pub unsafe fn CinematicBgInstr_Func16(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ba1) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD437 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicBgInstr_Func17(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb72f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_XfadeGameplayFade(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        FadeOutYColorsFromIndexX(0 as uint16, 0x14 as uint16, game);
        FadeOutYColorsFromIndexX(0x60 as uint16, 0x10 as uint16, game);
        FadeOutYColorsFromIndexX(0x1d2 as uint16, 6 as uint16, game);
        FadeInYColorsFromIndexX(0x28 as uint16, 3 as uint16, game);
        FadeInYColorsFromIndexX(0xe0 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x180 as uint16, 0x20 as uint16, game);
        FadeInYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
    }
    let ref mut fresh133 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh133 = (*fresh133).wrapping_sub(1);
    if *fresh133 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 21 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        let mut v0: uint16 = 128 as uint16;
        loop {
            let fresh134 = v0;
            v0 = v0.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[fresh134 as usize] = 47 as uint16;
            if !(((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        ClearJapaneseTextTiles(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba391 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            SpawnPalfxObject(addr_kPalfx_E1BC as uint16, game);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_XfadeScientistFade(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        FadeOutYColorsFromIndexX(0 as uint16, 0x14 as uint16, game);
        FadeOutYColorsFromIndexX(0x60 as uint16, 0x10 as uint16, game);
        FadeOutYColorsFromIndexX(0x1d2 as uint16, 6 as uint16, game);
        FadeInYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x1c0 as uint16, 9 as uint16, game);
        ComposeFadingPalettes(game);
    }
    let ref mut fresh131 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh131 = (*fresh131).wrapping_sub(1);
    if *fresh131 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 21 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        let mut v0: uint16 = 128 as uint16;
        loop {
            let fresh132 = v0;
            v0 = v0.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[fresh132 as usize] = 47 as uint16;
            if !(((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        ClearJapaneseTextTiles(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba391 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicSprInstr_StartIntroPage2(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb35f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Instr_StartIntroPage_Common(game);
    return j;
}
pub unsafe fn CinematicSprInstr_StartIntroPage3(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb370 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Instr_StartIntroPage_Common(game);
    return j;
}
pub unsafe fn CinematicSprInstr_StartIntroPage4(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb381 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Instr_StartIntroPage_Common(game);
    return j;
}
pub unsafe fn CinematicSprInstr_StartIntroPage5(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb392 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Instr_StartIntroPage_Common(game);
    return j;
}
pub unsafe fn Instr_StartIntroPage_Common(game: &mut Game) {
    DisableCinematicBgTilemapUpdates(game);
    ClearJapaneseTextTiles(game);
    TransferJapaneseTextTilesToVram(game);
}
pub unsafe fn CinematicFunction_Intro_SetupStuff(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 6 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 256 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0 as uint16, 0x10 as uint16, game);
    ClearYColorsFromIndexX(0x60 as uint16, 0x10 as uint16, game);
    ClearYColorsFromIndexX(0x1d2 as uint16, 6 as uint16, game);
    ComposeFadingPalettes(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 127 as uint16;
    EnableCinematicBgTilemapUpdates(game);
    SetSomeStuffForSpriteObject_16(game);
}
pub unsafe fn CinematicFunction_Intro_Page2(game: &mut Game) {
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF45 as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb3f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page3(game: &mut Game) {
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF4B as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb3f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page4(game: &mut Game) {
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF51 as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb458 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page5(game: &mut Game) {
    SpawnCinematicBgObject(
        addr_kCinematicBgObjectDef_8BCF57 as uint16,
        0x4c00 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb458 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Fadestuff(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        FadeInYColorsFromIndexX(0 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x60 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x1d2 as uint16, 6 as uint16, game);
        FadeOutYColorsFromIndexX(0x28 as uint16, 3 as uint16, game);
        FadeOutYColorsFromIndexX(0xe0 as uint16, 0x10 as uint16, game);
        FadeOutYColorsFromIndexX(0x180 as uint16, 0x20 as uint16, game);
        FadeOutYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
    }
    let ref mut fresh130 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16);
    *fresh130 = (*fresh130).wrapping_sub(1);
    if *fresh130 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 22 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba391 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Fadestuff2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        FadeInYColorsFromIndexX(0 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x60 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x1d2 as uint16, 6 as uint16, game);
        FadeOutYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
        FadeOutYColorsFromIndexX(0x1c0 as uint16, 9 as uint16, game);
        ComposeFadingPalettes(game);
    }
    let ref mut fresh129 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16);
    *fresh129 = (*fresh129).wrapping_sub(1);
    if *fresh129 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 22 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        let mut v0: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 224 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(
                (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 224 as ::core::ffi::c_int) as isize,
                );
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v0 as ::core::ffi::c_int - 14 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba391 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicBgPreInstr_SamusBlink(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bb207 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16)
            as ::core::ffi::c_int
            == 0x8ba390 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19cd) as *mut uint16).offset(v1 as isize) =
            -(10733 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19dd) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19d5) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CallCreditsObjectFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        9147353 => return,
        9155772 => {
            CinematicBgPreInstr_SamusBlink(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CinematicFunction_Intro_Func19(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(15)
        as ::core::ffi::c_int
        + 0x33fd as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x19dd) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func18(game: &mut Game) {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    CinematicFunction_Intro_Func20(addr_word_8BD389 as uint16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD389 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
}
pub unsafe fn sub_8BB51E(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BCF75 as uint16, game);
    CinematicFunction_Intro_Func20(addr_word_8BCFBD as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn EnableCinematicBgTilemapUpdates__0(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func21(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BCFFD as uint16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD055 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func22(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16).offset(v1 as isize) =
            -(11355 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD085 as uint16, game);
        CinematicFunction_Intro_Func20(addr_word_8BD0E1 as uint16, game);
        TransferJapaneseTextTilesToVram(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) = 60 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_ThenWaitInputSetupBabyMetroid(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8baf6c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func23(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD0F9 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func24(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16).offset(v1 as isize) =
            -(11333 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD15D as uint16, game);
        TransferJapaneseTextTilesToVram(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) = 60 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func25(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb0f2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func26(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD1B9 as uint16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD215 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func27(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16).offset(v1 as isize) =
            -(11311 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD259 as uint16, game);
        CinematicFunction_Intro_Func20(addr_word_8BD2A5 as uint16, game);
        TransferJapaneseTextTilesToVram(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) = 60 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func28(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb123 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func29(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19) =
        *(game.g_rom.offset(
            (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(19);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD2D5 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func30(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a1) as *mut uint16).offset(v1 as isize) =
            addr_off_8BD3E7 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a9) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19a5) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD30D as uint16, game);
        TransferJapaneseTextTilesToVram(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ba3) as *mut uint16) = 60 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func31(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bb1da as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func32(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableCinematicBgTilemapUpdates(game);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD371 as uint16, game);
    TransferJapaneseTextTilesToVram(game);
    return j;
}
pub unsafe fn EnableCinematicBgTilemapUpdates__(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    EnableCinematicBgTilemapUpdates(game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func33(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh169 = *(&raw mut game.g_ram as *mut uint8).offset(0x69);
        *fresh169 = (*fresh169 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint8;
    } else {
        let ref mut fresh170 = *(&raw mut game.g_ram as *mut uint8).offset(0x69);
        *fresh170 = (*fresh170 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint8;
    };
}
pub unsafe fn CinematicFunction_Intro_Func34(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bbca0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        RevertButtonConfig(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func35(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        56 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        111 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func36(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func37(k, game);
    let mut v1: uint16 = 8 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        != 256 as ::core::ffi::c_int
    {
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        - 84 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        KillProjectile(v1, game);
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v2 as isize) =
            8 as uint16;
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
            v3;
        if v3 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
                .offset(v2 as isize) =
                (0x8bb80f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF1B as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF1B as uint16,
                1 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF1B as uint16,
                2 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF15 as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF15 as uint16,
                1 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF15 as uint16,
                2 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF15 as uint16,
                3 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF15 as uint16,
                4 as uint16,
                game,
            );
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func39(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func37(k, game);
    CinematicFunction_Intro_Func40(game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCB19 as uint16;
    }
}
pub unsafe fn CinematicSprPreInstr_B82E(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func40(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) == 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func37(mut k: uint16, game: &mut Game) {
    let mut j: uint16 = 0;
    let mut v3: uint16 = 0;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if v1 != 0 {
        if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((i >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int) as isize,
                ) = *(game.g_rom.offset(
                    (((0x8ce3e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8ce3e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(((i >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int) as isize);
                i -= 2 as ::core::ffi::c_int;
            }
            v3 = k;
        } else {
            j = 30 as uint16;
            while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 240 as ::core::ffi::c_int) as isize,
                ) = 0x7fff as uint16;
                j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            }
            v3 = k;
        }
        let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh60 = (*fresh60).wrapping_sub(1);
    }
}
pub unsafe fn CinematicFunction_Intro_Func40(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
        *fresh58 = (*fresh58 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
        *fresh59 = (*fresh59 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    };
}
static kCinematicFunction_Intro_Func42_Tab0: [uint16; 4] = [
    0x70 as uint16,
    0xc0 as uint16,
    0x80 as uint16,
    0xe8 as uint16,
];
static kCinematicFunction_Intro_Func42_Tab1: [uint16; 4] = [
    0x50 as uint16,
    0x40 as uint16,
    0x38 as uint16,
    0x58 as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func42(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        kCinematicFunction_Intro_Func42_Tab0[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func42_Tab1[v3 as usize] as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicSprInstr_Func43(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bb93b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bb8d8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func44(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x8000 as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x8000 as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
        - (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(0)
            as ::core::ffi::c_int
            != 0x8bb80f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            return;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 11 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 11 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
        addr_off_8BCE53 as uint16;
}
static kCinematicFunction_Intro_Func45_Tab0: [uint16; 4] =
    [0 as uint16, 0xffff as uint16, 0 as uint16, 0xffff as uint16];
pub unsafe fn CinematicFunction_Intro_Func45(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (kCinematicFunction_Intro_Func45_Tab0[v2 as usize] as uint32) << 16 as ::core::ffi::c_int
            | 0x8000 as uint16 as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x8000 as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 208 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(0)
            as ::core::ffi::c_int
            == 0x8bb80f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    }
}
static kCinematicFunction_Intro_Func46_Tab0: [uint16; 5] = [
    0 as uint16,
    0x10 as uint16,
    0xfff0 as uint16,
    0xfff8 as uint16,
    8 as uint16,
];
static kCinematicFunction_Intro_Func46_Tab1: [uint16; 5] = [
    0 as uint16,
    0xfff0 as uint16,
    8 as uint16,
    0xfff0 as uint16,
    8 as uint16,
];
static kCinematicFunction_Intro_Func46_Tab2: [uint16; 5] = [
    1 as uint16,
    0x10 as uint16,
    0x20 as uint16,
    0x30 as uint16,
    0x40 as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func46(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func46_Tab0[v3 as usize] as ::core::ffi::c_int
            + 56 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func46_Tab1[v3 as usize] as ::core::ffi::c_int
            + 111 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
        kCinematicFunction_Intro_Func46_Tab2[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
static kCinematicFunction_Intro_Func47_Tab0: [uint16; 3] =
    [0x10 as uint16, 0xfff0 as uint16, 0xfff0 as uint16];
static kCinematicFunction_Intro_Func47_Tab1: [uint16; 3] =
    [0 as uint16, 4 as uint16, 0xfff8 as uint16];
static kCinematicFunction_Intro_Func47_Tab2: [uint16; 3] =
    [1 as uint16, 8 as uint16, 0x10 as uint16];
pub unsafe fn CinematicFunction_Intro_Func47(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func47_Tab0[v3 as usize] as ::core::ffi::c_int
            + 56 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func47_Tab1[v3 as usize] as ::core::ffi::c_int
            + 111 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
        kCinematicFunction_Intro_Func47_Tab2[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func48(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) == 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    }
}
pub unsafe fn CinematicSprInstr_SpawnIntroRinkas01(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF21 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF21 as uint16,
        1 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_SpawnIntroRinkas23(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF21 as uint16,
        2 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF21 as uint16,
        3 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func49(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        155 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func50(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + 0x3487 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bba73 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func51(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        == 145 as ::core::ffi::c_int
    {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEF1 as uint16,
            0 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEF1 as uint16,
            1 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEF1 as uint16,
            2 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEF1 as uint16,
            3 as uint16,
            game,
        );
        QueueSfx3_Max6(0x23 as uint16, game);
    }
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16)
            as ::core::ffi::c_int
            + 543 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            let ref mut fresh56 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16);
            *fresh56 = (*fresh56 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16)
        as ::core::ffi::c_int
        - 544 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16);
        *fresh57 = (*fresh57 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) as int16
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bbb0d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func52(mut k: uint16, game: &mut Game) {
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_sub(1);
    let mut v1: bool =
        *fresh55 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) == 0 || v1 {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v2 as isize) =
            (0x8bbb24 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
            0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func53(mut k: uint16, game: &mut Game) {
    let mut kh: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(kh as isize)
            as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d)
                as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
                .offset(kh as isize) = v3;
            if v3 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                QueueSfx3_Max6(0x23 as uint16, game);
            }
        }
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
                as ::core::ffi::c_int
                + 639 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let ref mut fresh50 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
                *fresh50 = (*fresh50 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
            as ::core::ffi::c_int
            - 640 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh51 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
            *fresh51 = (*fresh51 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) =
            (if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                1 as ::core::ffi::c_int
            } else {
                -(1 as ::core::ffi::c_int)
            }) as uint16;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(kh as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(kh as isize)
                as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) as int16
                as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint32,
        );
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16)
                as ::core::ffi::c_int
                + 543 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let ref mut fresh52 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16);
                *fresh52 = (*fresh52 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16)
            as ::core::ffi::c_int
            - 544 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh53 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16);
            *fresh53 = (*fresh53 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        }
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(kh as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(kh as isize)
                as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) as int16
                as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint32,
        );
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(kh as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(kh as isize) =
            addr_off_8BCE53 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a57) as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn ResetButtonAssignmentsToDefault(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xd82) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd84) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd86) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd88) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b8) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd8a) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ba) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd8c) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9bc) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd8e) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9be) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9aa) as *mut uint16) = kButton_Up as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ac) as *mut uint16) = kButton_Down as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ae) as *mut uint16) = kButton_Left as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b0) as *mut uint16) = kButton_Right as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) = kButton_X as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16) = kButton_A as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) = kButton_B as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b8) as *mut uint16) = kButton_Y as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ba) as *mut uint16) = kButton_Select as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9bc) as *mut uint16) = kButton_L as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9be) as *mut uint16) = kButton_R as uint16;
}
pub unsafe fn RevertButtonConfig(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd82) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd84) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd86) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9b8) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd88) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9ba) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8a) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9bc) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8c) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9be) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8e) as *mut uint16);
}
pub unsafe fn CinematicFunction_Intro_Func54(game: &mut Game) {
    let mut v1: int16 = 0;
    SetupPpu_3_Mode7(game);
    ClearCinematicSprites(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7) as *mut uint16) = 0 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce5e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce5e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x95a82f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x96fe69 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    DecompressToMem(
        0x96d10a as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x15000),
        game,
    );
    WriteRegWord(VMADDL as uint16, 0 as uint16, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BBD04: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BBD04 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteRegWord(VMADDL as uint16, 0 as uint16, game);
    v1 = 0x4000 as int16;
    loop {
        WriteReg(VMDATAL as uint16, 0x8c as uint8, game);
        v1 -= 1;
        if !(v1 != 0) {
            break;
        }
    }
    WriteRegWord(VMADDL as uint16, 0 as uint16, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    static unk_8BBD3C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f4000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f4000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x300 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BBD3C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(VMADDL as uint16, 0x6000 as uint16, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BBD5C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BBD5C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x38 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 56 as uint16;
    WriteRegWord(M7Y as uint16, 0x18 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        -(72 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(104 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 224 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 512 as uint16;
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    ClearCinematicBgObjects(0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = 23552 as uint16;
    EnableNMI(game);
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF0F as uint16,
        0 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bbde4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(0xff2d as uint16, game);
    QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
}
pub unsafe fn CinematicFunction_Intro_Func55(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bbdf9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func56(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 49 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 63 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 95 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
            -(97 as ::core::ffi::c_int) as uint8;
        ClearCinematicSprites(game);
        QueueMode7Transfers(
            0x8b as uint8,
            addr_kCinematicFunction_Intro_Func56_M7 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
            -(32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
            -(128 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 32 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF39 as uint16,
            0x0020, // The preceding $198D store leaves $0020 in A.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE85 as uint16,
            0x0800, // Palette left in A by the preceding sprite initializer.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE8B as uint16,
            0x0c00, // Palette left in A by the preceding sprite initializer.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE91 as uint16,
            1 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF0F as uint16,
            1 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bbfda as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnPalfxObject(addr_kPalfx_E1A8 as uint16, game);
        SpawnPalfxObject(addr_kPalfx_E1AC as uint16, game);
    } else {
        let ref mut fresh128 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh128 = (*fresh128 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func57(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bbfc6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            -(32 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
            -(1024 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            112 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        87 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func58(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bbdf9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize);
        *fresh49 = (*fresh49 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
        let mut d: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize) as int16 as int32)
            << 8 as ::core::ffi::c_int;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            d as uint32,
        );
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            d as uint32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
            d as uint32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
            d as uint32,
        );
    }
}
pub unsafe fn CinematicFunction_Intro_Func59(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        80 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        159 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func60(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x4000 as uint32,
    );
    let ref mut fresh48 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh48 = (*fresh48 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func61(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        116 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        160 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func62(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        4096 as uint32,
    );
    let ref mut fresh47 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh47 = (*fresh47 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func63(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func64(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        2048 as uint32,
    );
    let ref mut fresh46 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh46 = (*fresh46 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func65(mut j: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 224 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            112 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bbfd9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        87 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func66(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -(0x2000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func67(game: &mut Game) {
    let ref mut fresh123 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0);
    *fresh123 = (*fresh123).wrapping_sub(1);
    if *fresh123 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    }
    let ref mut fresh124 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1);
    *fresh124 = (*fresh124).wrapping_sub(1);
    if *fresh124 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    }
    let ref mut fresh125 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2);
    *fresh125 = (*fresh125).wrapping_sub(1);
    if *fresh125 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        -(0x2000 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 3072 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh126 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh126 = (*fresh126 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16)
        as ::core::ffi::c_int
        - 0x2000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh127 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh127 = (*fresh127 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    } else {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 92 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 6 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
        EnableCinematicBgObjects(game);
        EnableCinematicBgTilemapUpdates(game);
        SpawnCinematicBgObject(
            addr_kCinematicBgObjectDef_8BCF69 as uint16,
            0x5c00 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8ba38f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func68(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        124 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        186 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicCommonInstr_Func69(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) == 0 {
        j = (j as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    }
    return j;
}
pub unsafe fn CinematicCommonInstr_Func70(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bc0c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func71(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        124 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        204 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func72(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc100 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func73(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
        kGameState_31_SetUpNewGame as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
        kGameState_31_SetUpNewGame as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) = 6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
    SaveToSram(
        *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
        game,
    );
}
pub unsafe fn CinematicFunctionBlackoutFromCeres(game: &mut Game) {
    let mut j: int16 = 0;
    let mut v3: int16 = 0;
    SetupPpu_3_Mode7(game);
    j = 656 as int16;
    while j as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(j as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = 0 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce5e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce5e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x95a82f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x96fe69 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    DecompressToMem(
        0x96d10a as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x15000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BC19C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC19C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    v3 = 0x4000 as int16;
    loop {
        WriteReg(VMDATAL as uint16, 0x8c as uint8, game);
        v3 -= 1;
        if !(v3 != 0) {
            break;
        }
    }
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    static unk_8BC1D4: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f4600 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f4600 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x600 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC1D4 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BC1F4: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC1F4 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BC214: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x9ad200 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x9ad200 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x1a00 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC214 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x34 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 52 as uint16;
    WriteRegWord(M7Y as uint16, 0x30 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        -(44 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(112 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    ClearCinematicBgObjects(0x2f as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = 0 as uint16;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCE7F as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObjectAtSlot(
        addr_kCinematicSpriteObjectDef_8BCE8B as uint16,
        2 as uint16,
        0x0002,
        game,
    );
    SpawnCinematicSpriteObjectAtSlot(
        addr_kCinematicSpriteObjectDef_8BCE91 as uint16,
        0 as uint16,
        0x0000,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF33 as uint16,
        0x0800, // Palette left in A by the preceding sprite initializer.
        game,
    );
    SpawnPalfxObject(addr_kPalfx_E1B8 as uint16, game);
    EnableNMI(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bc2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff2d as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        == kGameState_37_CeresGoesBoomWithSamus as ::core::ffi::c_int
    {
        QueueMusic_DelayedY(8 as uint16, 0xe as uint16, game);
    } else {
        QueueMusic_DelayedY(7 as uint16, 0xe as uint16, game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func74(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc2f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func75(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        4096 as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        -(0x4000 as ::core::ffi::c_int) as uint32,
    );
    let ref mut fresh122 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh122 = (*fresh122).wrapping_add(1);
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc345 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func76(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        4096 as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        -(0x4000 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 640 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh121 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh121 = (*fresh121).wrapping_add(1);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        EnableHdmaObjects(game);
        SpawnPowerBombExplosion(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 55 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 768 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF2D as uint16,
            0 as uint16,
            game,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            == 37 as ::core::ffi::c_int
        {
            QueueMode7Transfers(
                0x8b as uint8,
                addr_kCinematicFunction_Intro_Func76_M7_2 as uint16,
                game,
            );
        } else {
            QueueMode7Transfers(
                0x8b as uint8,
                addr_kCinematicFunction_Intro_Func76_M7_0 as uint16,
                game,
            );
        }
        QueueMode7Transfers(
            0x8b as uint8,
            addr_kCinematicFunction_Intro_Func76_M7_1 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc5ca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicSprInstr_SpawnCeresExplosions1(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEBB as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEBB as uint16,
        1 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEBB as uint16,
        2 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEBB as uint16,
        3 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEBB as uint16,
        4 as uint16,
        game,
    );
    return j;
}
static kCinematicFunction_Intro_Func78_Tab0: [uint16; 5] = [
    1 as uint16,
    16 as uint16,
    32 as uint16,
    48 as uint16,
    64 as uint16,
];
static kCinematicFunction_Intro_Func78_Tab1: [int16; 5] = [
    16 as int16,
    -(16 as ::core::ffi::c_int) as int16,
    16 as int16,
    -(16 as ::core::ffi::c_int) as int16,
    0 as int16,
];
static kCinematicFunction_Intro_Func78_Tab2: [int16; 5] = [
    -(16 as ::core::ffi::c_int) as int16,
    16 as int16,
    16 as int16,
    -(16 as ::core::ffi::c_int) as int16,
    0 as int16,
];
pub unsafe fn CinematicFunction_Intro_Func78(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
        kCinematicFunction_Intro_Func78_Tab0[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func78_Tab1[v3 as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func78_Tab2[v3 as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicSprPreInstr_C489(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bc5ca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
        *fresh45 = (*fresh45).wrapping_sub(1);
        let mut v1: bool = *fresh45 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v1 {
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEC1 as uint16,
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16),
                game,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 12 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    & 7 as ::core::ffi::c_int) as uint16;
        }
    };
}
static g_word_8BC4EB: [int16; 16] = [
    14 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as int16,
    12 as int16,
    -(16 as ::core::ffi::c_int) as int16,
    12 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(14 as ::core::ffi::c_int) as int16,
    0 as int16,
    0 as int16,
    16 as int16,
    14 as int16,
    -(12 as ::core::ffi::c_int) as int16,
    4 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
];
static kCinematicFunction_Intro_Func81_Tab0: [int16; 4] =
    [1 as int16, 4 as int16, 8 as int16, 16 as int16];
static kCinematicFunction_Intro_Func81_Tab1: [int16; 4] = [
    8 as int16,
    12 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static kCinematicFunction_Intro_Func81_Tab2: [int16; 4] = [
    -(4 as ::core::ffi::c_int) as int16,
    8 as int16,
    -(10 as ::core::ffi::c_int) as int16,
    12 as int16,
];
pub unsafe fn CinematicFunction_Intro_Func80(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v3: ::core::ffi::c_int = 2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (g_word_8BC4EB[v3 as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (g_word_8BC4EB[(v3 + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicSprInstr_SpawnCeresExplosions3(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEC7 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEC7 as uint16,
        1 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEC7 as uint16,
        2 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCEC7 as uint16,
        3 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func81(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
        kCinematicFunction_Intro_Func81_Tab0[v1 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func81_Tab1[v1 as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        (kCinematicFunction_Intro_Func81_Tab2[v1 as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func82(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -(4096 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x4000 as uint32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func83(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func77(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 55 as uint8;
    let ref mut fresh119 = *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16);
    *fresh119 = (*fresh119 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint8 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc610 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 192 as uint16;
    } else {
        let ref mut fresh120 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh120 = (*fresh120 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func84(game: &mut Game) {
    let ref mut fresh118 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh118 = (*fresh118).wrapping_sub(1);
    let mut v0: bool =
        *fresh118 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc627 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func85(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            == kGameState_37_CeresGoesBoomWithSamus as ::core::ffi::c_int
        {
            QueueMusic_Delayed8(0 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 0 as uint16;
            WaitUntilEndOfVblankAndClearHdma(game);
            DisableIrqInterrupts(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
            let mut i: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
                    .offset(((i >> 1 as ::core::ffi::c_int) + 8 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                kGameState_25_SamusNoHealth as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8bc699 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func86(game: &mut Game) {
    SetupPpu_4_Mode1(game);
    let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    DecompressToMem(
        0x978adb as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x19000),
        game,
    );
    DecompressToMem(
        0x96ec76 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x15000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    static unk_8BC6E4: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f4300 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f4300 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x300 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC6E4 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x5c as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BC704: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f9000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f9000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC704 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BC724: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BC724 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x38 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 56 as uint16;
    WriteRegWord(M7Y as uint16, 0x18 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = -(127 as ::core::ffi::c_int) as uint8;
    SpawnPalfxObject(addr_kPalfx_E1A8 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bc79c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    EnableNMI(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func87(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh116 = *(&raw mut game.g_ram as *mut uint8).offset(0x57);
        *fresh116 = (*fresh116 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint8;
    }
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bc7ca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func88(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh115 = *(&raw mut game.g_ram as *mut uint8).offset(0x57);
        *fresh115 = (*fresh115 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint8;
        if *(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
            & 0xf0 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 128 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
                -(104 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 32 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 256 as uint16;
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEA3 as uint16,
                0x0100, // The preceding $198F store leaves $0100 in A.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEF7 as uint16,
                0x0e00, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEFD as uint16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF03 as uint16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF09 as uint16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEAF as uint16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8ba38f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func89(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        136 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        111 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func90(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bcade as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bc85d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func91(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh43 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize);
    *fresh43 = (*fresh43 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -((*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func92(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        127 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func93(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bcade as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bc8b9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func94(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh42 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize);
    *fresh42 = (*fresh42 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -((*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bcadf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func96(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bcade as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bc908 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func97(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh41 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize);
    *fresh41 = (*fresh41 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -((*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BCE53 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func98(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        47 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func99(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        208 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        47 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func100(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        207 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func101(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        208 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        207 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func102(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        186 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicSprInstr_C9A5(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnPalfxObject(addr_kPalfx_E1B0 as uint16, game);
    return j;
}
pub unsafe fn CinematicSprInstr_C9AF(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEB5 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16),
            game,
        );
    }
    return j;
}
pub unsafe fn CinematicSprInstr_C9BD(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnPalfxObject(addr_loc_8BE1B4 as uint16, game);
    return j;
}
pub unsafe fn CinematicSprInstr_C9C7(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 62 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(112 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bc9f9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func104(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        204 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func105(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        0x2000 as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        -(0x8000 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 1152 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh114 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh114 = (*fresh114 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bca36 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func106(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        0x2000 as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        -(0x8000 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bca85 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh113 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh113 = (*fresh113 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
            as ::core::ffi::c_int
            != 224 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint8 as uint16;
        }
    };
}
pub unsafe fn CinematicFunction_Intro_Func107(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16,
        0x2000 as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        0x2000 as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 0x2000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh112 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh112 = (*fresh112 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    } else {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bcad0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 64 as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func108(game: &mut Game) {
    let ref mut fresh111 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh111 = (*fresh111).wrapping_sub(1);
    let mut v0: bool =
        *fresh111 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bcade as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func95(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
        kGameState_6_LoadingGameData as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16);
}
pub unsafe fn GameState_39_EndingAndCredits_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
            | 0x8b0000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh162 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b9f) as *mut uint16);
        *fresh162 = (*fresh162).wrapping_add(1);
    }
    let ref mut fresh163 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16);
    *fresh163 = (*fresh163).wrapping_add(1);
    HandleCinematicSprites(game);
    MaybeHandleCinematicBgObjects(game);
    CreditsObject_Process(game);
    PaletteFxHandler(game);
    DrawCinematicSpriteObjects_Ending(game);
    CinematicFunction_Intro_Func149(game);
    HandleMode7TransformationMatrix(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn MaybeHandleCinematicBgObjects(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        + 0x1afc as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        HandleCinematicBgObjects(game);
    }
}
pub unsafe fn CinematicFunctionEscapeFromCebes(game: &mut Game) {
    let mut j: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7) as *mut uint16) = 0 as uint16;
    SetupPpu_5_Mode7(game);
    j = 656 as int16;
    while j as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(j as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
    }
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8cede9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8cede9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x98bcd6 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x99a56f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    DecompressToMem(
        0x99d17e as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x18000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD4FC: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD4FC as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x20 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD51C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD51C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD53C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f4000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f4000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD53C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD55C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD55C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    DecompressToMem(
        0x988304 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x18000),
        game,
    );
    DecompressToMem(
        0x95a82f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x96fe69 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    let mut v3: uint16 = 768 as uint16;
    loop {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x14000) as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) = 0x8c8c as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v3 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x98b5c1 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e000),
        game,
    );
    DecompressToMem(
        0x98b857 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e800),
        game,
    );
    DecompressToMem(
        0x98baed as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1f000),
        game,
    );
    DecompressToMem(
        0x98bccd as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1f800),
        game,
    );
    DecompressToMem(
        0x97e7de as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0xa000),
        game,
    );
    DecompressToMem(
        0x99d65b as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x2000),
        game,
    );
    DecompressToMem(
        0x99d932 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x6000),
        game,
    );
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 128 as uint16;
    WriteRegWord(M7Y as uint16, 0x80 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 64 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = 0 as uint16;
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    EnableNMI(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(1) = 0 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 17 as uint8;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEEB as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEF1 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEEB as uint16,
        1 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEF1 as uint16,
        1 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bd6d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff33 as uint16, game);
    QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
}
pub unsafe fn CinematicFunction_Intro_Func109(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        SpawnPalfxObject(addr_kPalfx_E1D8 as uint16, game);
        SpawnPalfxObject(addr_kPalfx_E1DC as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bd6f2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func110(game: &mut Game) {
    CinematicFunction_Intro_Func111(game);
    if AdvanceFastScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bd701 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func111(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint8 as uint16;
    }
    let ref mut fresh109 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh109 = (*fresh109 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 384 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if AdvanceFastScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8bd731 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func112(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD746: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e2000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e2000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD746 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x20 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD766: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e2000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e2000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD766 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    DecompressToMem(
        0x98ed4f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x2000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD79B: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7e2000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e2000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD79B as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    ClearPaletteFXObjects(game);
    ClearCinematicSprites(game);
    SpawnPalfxObject(addr_kPalfx_E1E0 as uint16, game);
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEED3 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEED9 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEDF as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEE5 as uint16,
        0 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 64 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bd7f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func114(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint8 as uint16;
    }
    let ref mut fresh108 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh108 = (*fresh108 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 384 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if AdvanceFastScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8bd837 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func113(game: &mut Game) {
    CinematicFunction_Intro_Func114(game);
    if AdvanceFastScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bd807 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func115(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD84C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e6000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e6000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD84C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x20 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD86C: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e6000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e6000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD86C as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    DecompressToMem(
        0x999101 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x6000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD8A1: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 0 as uint8,
        bbad: 0x19 as uint8,
        a1: LongPtr {
            addr: (0x7e6000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e6000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD8A1 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD8C1: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f8000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f8000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x6000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD8C1 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x70 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD8E1: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fe000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fe000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD8E1 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x74 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD901: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fe800 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fe800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD901 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x78 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD921: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7ff000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7ff000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD921 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x7c as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD941: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7ff800 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7ff800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD941 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x50 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BD961: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7ea000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7ea000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x1000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BD961 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 2 as uint8;
    ClearPaletteFXObjects(game);
    ClearCinematicSprites(game);
    let mut v0: uint16 = 256 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 = *(game.g_rom.offset(
            (((0x8cebe9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8cebe9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            v2;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEE9D as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEAF as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEA3 as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEA9 as uint16,
        0 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 64 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 33 as uint8;
    CopyPalettesToFadingPalettes(game);
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x1a0 as uint16, 0x10 as uint16, game);
    ClearYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
    ComposeFadingPalettes(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 63 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bda02 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func116(game: &mut Game) {
    CinematicFunction_Intro_Func117(game);
    if AdvanceFastScreenFadeIn(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bda1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func117(game: &mut Game) {
    let ref mut fresh106 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh106 = (*fresh106 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        FadeOutYColorsFromIndexX(0xe0 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x1a0 as uint16, 0x10 as uint16, game);
        FadeInYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
    }
    let ref mut fresh107 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh107 = (*fresh107).wrapping_sub(1);
    if *fresh107 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut v0: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0x8cebe9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cebe9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v0 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 112 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 120 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 68 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1999) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x199b) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x199d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x199f) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdad3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 0 as uint16;
    }
}
static kCinematicFunction_Intro_Func118_Tab0: [uint16; 16] = [
    0 as uint16,
    0x800 as uint16,
    0x1000 as uint16,
    0x1800 as uint16,
    0x2000 as uint16,
    0x2800 as uint16,
    0x3000 as uint16,
    0x3800 as uint16,
    0x4000 as uint16,
    0x4800 as uint16,
    0x5000 as uint16,
    0x5800 as uint16,
    0x6000 as uint16,
    0x6800 as uint16,
    0x7000 as uint16,
    0x7800 as uint16,
];
static kCinematicFunction_Intro_Func118_Tab1: [uint16; 16] = [
    0 as uint16,
    0x800 as uint16,
    0x1000 as uint16,
    0x1800 as uint16,
    0x2000 as uint16,
    0x2800 as uint16,
    0x3000 as uint16,
    0x3800 as uint16,
    0 as uint16,
    0x800 as uint16,
    0x1000 as uint16,
    0x1800 as uint16,
    0x2000 as uint16,
    0x2800 as uint16,
    0x3000 as uint16,
    0x3800 as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func118(game: &mut Game) {
    let mut v2: ::core::ffi::c_uint = 0;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16);
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16) as ::core::ffi::c_uint;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16) = 192 as uint16;
        let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut uint8)
            .offset(v2 as isize) as *mut uint16) =
            kCinematicFunction_Intro_Func118_Tab0[v3 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 2048 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v1 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = kCinematicFunction_Intro_Func118_Tab1[v3 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut uint8)
            .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = 128 as uint16;
    } else {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16) = 128 as uint16;
        let mut v4: ::core::ffi::c_int = v0 as ::core::ffi::c_int;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut uint8)
            .offset(v2 as isize) as *mut uint16) =
            kCinematicFunction_Intro_Func118_Tab0[v4 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 2048 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut uint8)
            .offset((v1 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = kCinematicFunction_Intro_Func118_Tab1[v4 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut uint8)
            .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x334) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
    let ref mut fresh105 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh105 = (*fresh105).wrapping_add(1);
    if *fresh105 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdb9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func119(game: &mut Game) {
    let ref mut fresh104 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh104 = (*fresh104).wrapping_sub(1);
    let mut v0: bool =
        *fresh104 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(0xff3c as uint16, game);
        QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdbc4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func120(game: &mut Game) {
    if HasQueuedMusic(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 48 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 63 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 95 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
            -(97 as ::core::ffi::c_int) as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(0) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(16) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(128) =
            0 as uint16;
        SpawnPalfxObject(addr_kPalfx_E1D4 as uint16, game);
        SpawnPalfxObject(addr_kPalfx_E1E4 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 56 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 24 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
            -(72 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
            -(104 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 3072 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
            -(112 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 192 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdca5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func122(game: &mut Game) {
    let ref mut fresh96 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh96 = (*fresh96).wrapping_sub(1);
    let mut v0: bool =
        *fresh96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        let ref mut fresh97 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0);
        *fresh97 = (*fresh97).wrapping_sub(1);
        if *fresh97 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
        }
        let ref mut fresh98 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1);
        *fresh98 = (*fresh98).wrapping_sub(1);
        if *fresh98 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
        }
        let ref mut fresh99 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2);
        *fresh99 = (*fresh99).wrapping_sub(1);
        if *fresh99 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) as ::core::ffi::c_int
            == 32 as ::core::ffi::c_int
            && *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) as ::core::ffi::c_int
                == 64 as ::core::ffi::c_int
            && *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 8 as uint16;
    }
}
static kCinematicFunction_Intro_Func121_Tab0: [uint16; 32] = [
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0 as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
    0xffff as uint16,
    0x8000 as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func121(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) != 0 {
        let ref mut fresh101 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16);
        *fresh101 = (*fresh101).wrapping_sub(1);
    } else {
        CinematicFunction_Intro_Func122(game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint8 as uint16;
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
            as ::core::ffi::c_int;
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        (kCinematicFunction_Intro_Func121_Tab0[v0 as usize] as uint32) << 16 as ::core::ffi::c_int
            | kCinematicFunction_Intro_Func121_Tab0[(v0 + 1 as ::core::ffi::c_int) as usize]
                as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint16;
    let ref mut fresh102 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh102 = (*fresh102 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 1456 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdd42 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
static kCinematicFunction_Intro_Func123_Tab0: [uint16; 16] = [
    1 as uint16,
    0 as uint16,
    1 as uint16,
    0 as uint16,
    1 as uint16,
    0 as uint16,
    0xffff as uint16,
    0 as uint16,
    0xffff as uint16,
    0 as uint16,
    1 as uint16,
    0 as uint16,
    1 as uint16,
    0 as uint16,
    0xffff as uint16,
    0 as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func123(game: &mut Game) {
    CinematicFunction_Intro_Func122(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) as ::core::ffi::c_int
        != 224 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint8 as uint16;
    }
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
            as ::core::ffi::c_int;
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        (kCinematicFunction_Intro_Func123_Tab0[v0 as usize] as uint32) << 16 as ::core::ffi::c_int
            | kCinematicFunction_Intro_Func123_Tab0[(v0 + 1 as ::core::ffi::c_int) as usize]
                as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh100 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh100 = (*fresh100 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 1184 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bddcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 0 as uint16;
        if CheckEventHappened(0xf as uint16, game) != 0 {
            SpawnCinematicSpriteObjectToR18(0xef21 as uint16, 4 as uint16, game);
        }
    }
}
static g_word_8BDE43: [uint16; 16] = [
    0 as uint16,
    0x6f7b as uint16,
    0x4a52 as uint16,
    0x1ce7 as uint16,
    0 as uint16,
    0x5ad6 as uint16,
    0x4a52 as uint16,
    0x35ad as uint16,
    0x2529 as uint16,
    0x7fff as uint16,
    0x56b5 as uint16,
    0x294a as uint16,
    0x4210 as uint16,
    0x2d6b as uint16,
    0x18c6 as uint16,
    0x7fff as uint16,
];
pub unsafe fn CinematicFunction_Intro_Func124(game: &mut Game) {
    CinematicFunction_Intro_Func122(game);
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16,
        -(256 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x1991) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 384 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1a51) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
            as ::core::ffi::c_int
            != 16 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16)
                as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint8 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEEC7 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16),
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bde63 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 80 as ::core::ffi::c_int) as isize) =
                g_word_8BDE43[(i >> 1 as ::core::ffi::c_int) as usize];
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        let ref mut fresh94 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh94 = (*fresh94 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func125(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    if AdvanceFastScreenFadeOut(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bde80 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func126(game: &mut Game) {
    SetupPpu_7_Mode1(game);
    ClearCinematicSprites(game);
    ClearPaletteFXObjects(game);
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce9e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce9e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    DecompressToMem(
        0x979803 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    DecompressToMem(
        0x97b957 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x2000),
        game,
    );
    DecompressToMem(
        0x97d7fc as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x6000),
        game,
    );
    DecompressToMem(
        0x97e7de as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1c000),
        game,
    );
    DecompressToMem(
        0x9796f4 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e000),
        game,
    );
    DecompressToMem(
        0x97f987 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x14000),
        game,
    );
    DecompressToMem(
        0x99da9f as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e800),
        game,
    );
    DecompressToMem(
        0x99dab1 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1f000),
        game,
    );
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x20 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDF39: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fe800 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fe800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x100 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDF39 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x24 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDF59: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7ff000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7ff000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDF59 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDF79: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fc000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fc000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x1000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDF79 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDF99: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7e6000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7e6000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x4000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDF99 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x50 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDFB9: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x2000 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDFB9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x4c as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8BDFD9: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x7fe000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x7fe000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    SetupDmaTransfer(&raw const unk_8BDFD9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    DecompressToMem(
        0x99e089 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x6000),
        game,
    );
    DecompressToMem(
        0x99ecc4 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x8000),
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        WriteReg(VMADDL as uint16, 0 as uint8, game);
        WriteReg(VMADDH as uint16, 0 as uint8, game);
        WriteReg(VMAIN as uint16, 0x80 as uint8, game);
        static unk_8BE027: StartDmaCopy = StartDmaCopy {
            chan: 1 as uint8,
            dmap: 1 as uint8,
            bbad: 0x18 as uint8,
            a1: LongPtr {
                addr: (0x7e2000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
                bank: (0x7e2000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
            },
            das: 0x4000 as VoidP,
        };
        SetupDmaTransfer(&raw const unk_8BE027 as *const ::core::ffi::c_void, game);
        WriteReg(MDMAEN as uint16, 2 as uint8, game);
    } else {
        WriteReg(VMADDL as uint16, 0 as uint8, game);
        WriteReg(VMADDH as uint16, 0 as uint8, game);
        WriteReg(VMAIN as uint16, 0x80 as uint8, game);
        static unk_8BE04B: StartDmaCopy = StartDmaCopy {
            chan: 1 as uint8,
            dmap: 1 as uint8,
            bbad: 0x18 as uint8,
            a1: LongPtr {
                addr: (0x7f0000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
                bank: (0x7f0000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
            },
            das: 0x4000 as VoidP,
        };
        SetupDmaTransfer(&raw const unk_8BE04B as *const ::core::ffi::c_void, game);
        WriteReg(MDMAEN as uint16, 2 as uint8, game);
    }
    WriteRegWord(M7A as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7B as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7C as uint16, 0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
    WriteRegWord(M7D as uint16, 0x100 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    WriteRegWord(M7X as uint16, 0x68 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 104 as uint16;
    WriteRegWord(M7Y as uint16, 0x70 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        -(8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(24 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 2304 as uint16;
    EnableNMI(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = 0 as uint16;
    EnableHdmaObjects(game);
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    CinematicFunction_Intro_Func127(game);
    DecompressToMem(
        0x97eeff as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x10000),
        game,
    );
    ClearCinematicBgObjects(0x7f as uint16, game);
    CinematicFunction_Intro_Func128(0x7f as uint16, game);
    EnableTextGlowObjects_(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 3 as uint16;
    CreditsObject_Init(addr_stru_8BF6F8 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bdb9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func129(game: &mut Game) {
    let ref mut fresh92 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh92 = (*fresh92).wrapping_sub(1);
    let mut v0: bool =
        *fresh92 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be110 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 120 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
    }
}
pub unsafe fn CinematicFunction_Intro_Func130(game: &mut Game) {
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 34 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        CopyPalettesToFadingPalettes(game);
        DecomposePaletteDataForFading(game);
        ClearYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 32 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be158 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func131(game: &mut Game) {
    FadeInYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
    ComposeFadingPalettes(game);
    let ref mut fresh90 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh90 = (*fresh90).wrapping_sub(1);
    let mut v0: bool =
        *fresh90 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 18 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 2 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be190 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CinematicFunction_Intro_Func133(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func132(game: &mut Game) {
    let ref mut fresh86 = *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16);
    *fresh86 = (*fresh86 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh88 = (*fresh88).wrapping_sub(1);
    let mut v0: bool =
        *fresh88 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 288 as ::core::ffi::c_int) as usize] =
                *(game.g_rom.offset(
                    (((0x8cdc9b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cdc9b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize);
            v1 = v1.wrapping_add(1);
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 576 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        CinematicUpdateSomeBg(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 1 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be1d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd9c) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func134(game: &mut Game) {
    let ref mut fresh85 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh85 = (*fresh85).wrapping_sub(1);
    if *fresh85 == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 127 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            CopyPalettesToFadingPalettes(game);
            DecomposePaletteDataForFading(game);
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF27 as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF2D as uint16,
                0 as uint16,
                game,
            );
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16)
                as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF4B as uint16,
                    0 as uint16,
                    game,
                );
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF45 as uint16,
                    0 as uint16,
                    game,
                );
                *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                    (0x8be293 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
                *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
                return;
            }
            CopyPalettesToFadingPalettes(game);
            DecomposePaletteDataForFading(game);
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF51 as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF45 as uint16,
                0 as uint16,
                game,
            );
            ClearYColorsFromIndexX(0x1c0 as uint16, 0x10 as uint16, game);
        }
        ClearYColorsFromIndexX(0x1a0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 34 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be265 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func136(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CinematicFunction_Intro_Func135(game);
    } else {
        FadeOutYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            FadeInYColorsFromIndexX(0x1c0 as uint16, 0x10 as uint16, game);
        }
        FadeInYColorsFromIndexX(0x1a0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
        CinematicFunction_Intro_Func135(game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func135(game: &mut Game) {
    let ref mut fresh84 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh84 = (*fresh84).wrapping_sub(1);
    if *fresh84 as ::core::ffi::c_int - 65 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let mut i: ::core::ffi::c_int = 574 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16)
                .offset(288) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 127 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize] =
                *(game.g_rom.offset(
                    (((0x8cdedb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cdedb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize);
            v1 = v1.wrapping_add(1);
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 128 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        CinematicUpdateSomeBg(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 1 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be2dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func137(game: &mut Game) {
    let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16);
    *fresh83 = (*fresh83).wrapping_sub(1);
    let mut v0: bool =
        *fresh83 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16) == 0 || v0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16)
                as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 34 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8be314 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (0x8be342 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func138(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CinematicFunction_Intro_Func145(game);
    } else {
        FadeOutYColorsFromIndexX(0x40 as uint16, 0x10 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            FadeInYColorsFromIndexX(0x1c0 as uint16, 0x10 as uint16, game);
        }
        FadeInYColorsFromIndexX(0x1a0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
        CinematicFunction_Intro_Func145(game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func145(game: &mut Game) {
    let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh81 = (*fresh81).wrapping_sub(1);
    if *fresh81 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 7 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8bdb9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF33 as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF39 as uint16,
                0 as uint16,
                game,
            );
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16)
                as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF69 as uint16,
                    0 as uint16,
                    game,
                );
            } else {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF63 as uint16,
                    0 as uint16,
                    game,
                );
            }
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF5D as uint16,
                0 as uint16,
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEF57 as uint16,
                0 as uint16,
                game,
            );
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func140(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) != 0 {
        let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16);
        *fresh77 = (*fresh77).wrapping_sub(1);
        FadeOutYColorsFromIndexX(0x1e0 as uint16, 0x10 as uint16, game);
        ComposeFadingPalettes(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func139(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) != 0 {
        let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
        *fresh78 = (*fresh78).wrapping_sub(1);
        FadeOutYColorsFromIndexX(0x180 as uint16, 0x10 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) != 0 {
            ComposeFadingPalettes(game);
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) != 0 {
        let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
        *fresh79 = (*fresh79).wrapping_sub(1);
    } else {
        CinematicFunction_Intro_Func140(game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int) as uint8 as uint16;
    let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
    *fresh80 = (*fresh80 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) = 24 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be48a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func142(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) == 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
            as ::core::ffi::c_int
            - 6 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v0: uint16 = (4 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            let mut v1: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
                    as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            (*gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .size = *(game.g_rom.offset(
                (((0x8be45a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8be45a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size =
                *(game.g_rom.offset(
                    (((0x8be45a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8be45a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize);
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            let fresh75 = v1;
            v1 = v1.wrapping_add(1);
            *(&raw mut (*(gVramWriteEntry
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
                fresh75 as ::core::ffi::c_int,
                game,
            ))
            .size as *mut uint8)
                .offset(0) = *(game.g_rom.offset(
                (((0x8be45a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8be45a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize)
                as uint8;
            (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size =
                *(game.g_rom.offset(
                    (((0x8be45a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8be45a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 + 3 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            let ref mut fresh76 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
            *fresh76 = (*fresh76).wrapping_add(1);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func141(game: &mut Game) {
    CinematicFunction_Intro_Func140(game);
    CinematicFunction_Intro_Func142(game);
    let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh74 = (*fresh74).wrapping_sub(1);
    if *fresh74 == 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 1 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 49 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 63 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 95 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
            -(97 as ::core::ffi::c_int) as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 78 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 84 as uint8;
        let mut v0: uint16 = 384 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0x8ce7e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8ce7e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v0 as ::core::ffi::c_int - 416 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be504 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 32 as uint16;
        EnableCinematicBgObjects(game);
        EnableCinematicBgTilemapUpdates(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = 19456 as uint16;
        let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16)
                .offset(384) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 127 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        CinematicUpdateSomeBg(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func143(game: &mut Game) {
    let mut j: uint16 = 0;
    let ref mut fresh69 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0);
    *fresh69 = (*fresh69).wrapping_sub(1);
    if *fresh69 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    }
    let ref mut fresh70 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1);
    *fresh70 = (*fresh70).wrapping_sub(1);
    if *fresh70 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    }
    let ref mut fresh71 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2);
    *fresh71 = (*fresh71).wrapping_sub(1);
    if *fresh71 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    }
    let ref mut fresh73 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh73 = (*fresh73).wrapping_sub(1);
    let mut v0: bool =
        *fresh73 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be7ba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19f5) as *mut uint16) = 20480 as uint16;
        CinematicUpdateSomeBg(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = 256 as uint16;
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEF81 as uint16,
            0 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEF87 as uint16,
            0 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEF8D as uint16,
            0 as uint16,
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEF93 as uint16,
            0 as uint16,
            game,
        );
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 16 as ::core::ffi::c_int) as isize) =
                0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        j = 30 as uint16;
        while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(
                (((0x8cefe9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8cefe9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func144(game: &mut Game) {
    let mut j: uint16 = 0;
    let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16)
            as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v5: uint16 = *(game.g_rom.offset(
        (((0x8be5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x8be5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v0 + 1 as ::core::ffi::c_int) as isize);
    let mut v1: uint16 = *(game.g_rom.offset(
        (((0x8be5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x8be5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v0 as isize);
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 16 as ::core::ffi::c_int) as isize) =
            *(RomPtr_8C(v1 as uint16_t, game) as *mut uint16);
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v3: uint16 = v5;
    j = 30 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
            ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int)
                as isize,
        ) = *(RomPtr_8C(v3 as uint16_t, game) as *mut uint16);
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16);
    *fresh68 = (*fresh68).wrapping_add(1);
    if *fresh68 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be7ba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        ClearCinematicSprites(game);
        SpawnCinematicBgObject(
            addr_kCinematicBgObjectDef_8BF74E as uint16,
            0x4c00 as uint16,
            game,
        );
    }
}
pub unsafe fn CalcItemPercentageCount(
    mut k: uint16,
    mut instrp: uint16,
    game: &mut Game,
) -> uint16 {
    let mut current_block: u64;
    let mut j: uint16 = 0;
    let mut v0: uint16 = 4 as uint16;
    let mut r18: uint16 = 0 as uint16;
    loop {
        let mut v1: *const uint16 = RomPtr_RAM(
            *(game.g_rom.offset(
                (((0x8be70d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8be70d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize) as uint16_t,
            game,
        ) as *const uint16;
        r18 = (r18 as ::core::ffi::c_int
            + SnesDivide(
                *v1,
                *(game.g_rom.offset(
                    (((0x8be717 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8be717 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v0 as isize) as uint8,
            ) as ::core::ffi::c_int) as uint16;
        v0 = v0.wrapping_sub(1);
        if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut i: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *(game.g_rom.offset(
            (((0x8be721 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8be721 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16)
                as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            r18 = r18.wrapping_add(1);
        }
        i -= 2 as ::core::ffi::c_int;
    }
    j = 8 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if *(game.g_rom.offset(
            (((0x8be737 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8be737 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16)
                as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            r18 = r18.wrapping_add(1);
        }
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    let mut RegWord: uint16 = (r18 as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = (r18 as ::core::ffi::c_int % 10 as ::core::ffi::c_int) as uint16;
    r18 = (RegWord as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = (RegWord as ::core::ffi::c_int % 10 as ::core::ffi::c_int) as uint16;
    if r18 != 0 {
        let mut v5: ::core::ffi::c_int = (4 as ::core::ffi::c_int * r18 as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[462 as ::core::ffi::c_int as usize] =
            *(game.g_rom.offset(
                (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v5 as isize);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[494 as ::core::ffi::c_int as usize] =
            *(game.g_rom.offset(
                (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v5 + 1 as ::core::ffi::c_int) as isize);
    }
    let mut v6: uint16 = r20;
    if r20 != 0 {
        current_block = 18317007320854588510;
    } else if r18 != 0 {
        v6 = r20;
        current_block = 18317007320854588510;
    } else {
        current_block = 2719512138335094285;
    }
    match current_block {
        18317007320854588510 => {
            let mut v7: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v6 as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[463 as ::core::ffi::c_int as usize] =
                *(game.g_rom.offset(
                    (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v7 as isize);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[495 as ::core::ffi::c_int as usize] =
                *(game.g_rom.offset(
                    (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v7 + 1 as ::core::ffi::c_int) as isize);
        }
        _ => {}
    }
    let mut v8: ::core::ffi::c_int = (4 as ::core::ffi::c_int * r22 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[464 as ::core::ffi::c_int as usize] = *(game.g_rom.offset(
        (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v8 as isize);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[496 as ::core::ffi::c_int as usize] = *(game.g_rom.offset(
        (((0x8be741 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x8be741 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v8 + 1 as ::core::ffi::c_int) as isize);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[465 as ::core::ffi::c_int as usize] = 14442 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[497 as ::core::ffi::c_int as usize] = 14458 as uint16;
    return instrp;
}
pub unsafe fn CinematicFunction_Intro_Func146(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        let mut i: ::core::ffi::c_int = 63 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(i + 736 as ::core::ffi::c_int) as usize] =
                *(game.g_rom.offset(
                    (((0x8cdf5b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x8cdf5b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(i as isize);
            i -= 1;
        }
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func147(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(736) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 127 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8be797 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func148(game: &mut Game) {
    let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16);
    *fresh67 = (*fresh67 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) as ::core::ffi::c_int
        + 79 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        SpawnCinematicBgObject(
            addr_kCinematicBgObjectDef_8BF748 as uint16,
            0x4c00 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (0x8be7ba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CopySuperMetroidString(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func127(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v4: int16 = 0;
    let mut v0: uint16 = (&raw mut game.g_ram as *mut uint8)
        .offset(0xe0c)
        .offset_from(&raw mut game.g_ram as *mut uint8)
        as ::core::ffi::c_long as uint16;
    v1 = 0 as int16;
    loop {
        v4 = v1;
        let mut v2: *mut uint8 = RomPtr_RAM(v0 as uint16_t, game);
        *(v2 as *mut uint16) = v1 as uint16;
        *(v2 as *mut uint16).offset(6) = 0 as uint16;
        *(v2 as *mut uint16).offset(7) = 0 as uint16;
        let mut v3: uint16 = *(game.g_rom.offset(
            (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((8 as ::core::ffi::c_int * v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                    + 3 as ::core::ffi::c_int) as isize,
            );
        if v3 != 0 {
            *(v2 as *mut uint16).offset(5) = v3;
            let ref mut fresh93 = *(v2 as *mut uint16);
            *fresh93 = (*fresh93 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
        } else {
            *(v2 as *mut uint16).offset(5) = 32 as uint16;
        }
        *(v2 as *mut uint16).offset(1) = 128 as uint16;
        *(v2 as *mut uint16).offset(3) = 128 as uint16;
        *(v2 as *mut uint16).offset(2) = 0 as uint16;
        *(v2 as *mut uint16).offset(4) = 0 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        v1 += 1;
        if !(((v4 as ::core::ffi::c_int - 39 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func149(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    let mut v12: int16 = 0;
    let mut v17: int16 = 0;
    let mut v18: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v19: int16 = 0;
    let mut v20: int16 = 0;
    let mut v22: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(1) != 0 {
        let mut n: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
        let mut v0: uint16 = 0xe0c as uint16;
        loop {
            let mut v3: *mut uint8 = RomPtr_RAM(v0 as uint16_t, game);
            if (*(v3 as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                let mut v1: *mut uint8 = v3;
                v2 = (*(v1.offset(10) as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                *(v1 as *mut uint16).offset(5) = v2 as uint16;
                if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    *(v1 as *mut uint16).offset(5) = 32 as uint16;
                    *(v1 as *mut uint16) = *v1 as uint16;
                }
            } else {
                let mut v4: ::core::ffi::c_int = 8 as ::core::ffi::c_int
                    * *(v3 as *mut uint16) as uint8 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int;
                if (*(v3 as *mut uint16) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                    - 1024 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    let ref mut fresh164 = *(v3 as *mut uint16).offset(6);
                    *fresh164 = (*fresh164 as ::core::ffi::c_int
                        + *(game.g_rom.offset(
                            (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int)
                        as uint16;
                    v5 = *(v3.offset(14) as *mut uint16) as int16;
                } else {
                    let ref mut fresh165 = *(v3 as *mut uint16).offset(6);
                    *fresh165 = (*fresh165 as ::core::ffi::c_int
                        + (*(game.g_rom.offset(
                            (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            + *(game.g_rom.offset(
                                (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                    << 15 as ::core::ffi::c_int
                                    | 0x8be9cf as ::core::ffi::c_int
                                        & 0x7fff as ::core::ffi::c_int)
                                    & 0x3fffff as ::core::ffi::c_int)
                                    as isize,
                            ) as *const uint8 as *mut uint16)
                                .offset(v4 as isize)
                                as ::core::ffi::c_int)) as uint16;
                    v5 = (*(game.g_rom.offset(
                        (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v4 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        + *(v3 as *mut uint16).offset(7) as ::core::ffi::c_int)
                        as int16;
                }
                *(v3 as *mut uint16).offset(7) =
                    (*(game.g_rom.offset(
                        (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v4 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        + v5 as ::core::ffi::c_int) as uint16;
                let mut v23: uint16 = *(v3.offset(12) as *mut uint16);
                let mut r18: uint16 = *(&raw mut v23 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as uint16;
                let mut r20: uint16 = ((v23 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int) as uint16;
                v8 = *(v3.offset(4) as *mut uint16) as int16;
                let mut v9: bool = u16::overflowing_add(r20, v8 as uint16).1;
                *(v3 as *mut uint16).offset(2) =
                    (r20 as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint16;
                let ref mut fresh166 = *(v3 as *mut uint16).offset(1);
                *fresh166 = (*fresh166 as ::core::ffi::c_int
                    + (r18 as ::core::ffi::c_int + v9 as ::core::ffi::c_int))
                    as uint16;
                let mut v24: uint16 = *(v3.offset(14) as *mut uint16);
                r18 = *(&raw mut v24 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as int8 as uint16;
                r20 = ((v24 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int) as uint16;
                v12 = *(v3.offset(8) as *mut uint16) as int16;
                v9 = u16::overflowing_add(r20, v12 as uint16).1;
                *(v3 as *mut uint16).offset(4) =
                    (r20 as ::core::ffi::c_int + v12 as ::core::ffi::c_int) as uint16;
                let ref mut fresh167 = *(v3 as *mut uint16).offset(3);
                *fresh167 = (*fresh167 as ::core::ffi::c_int
                    + (r18 as ::core::ffi::c_int + v9 as ::core::ffi::c_int))
                    as uint16;
            }
            v0 = (v0 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        let mut n2: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
        let mut v13: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16);
        let mut v14: uint16 = 3596 as uint16;
        loop {
            let mut v15: *mut uint8 = RomPtr_RAM(v14 as uint16_t, game);
            let mut v16: *mut uint8 = v15;
            if *(v15 as *mut int16) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                v17 = *(v15.offset(2) as *mut uint16) as int16;
                if *v15.offset(3) as ::core::ffi::c_int != 0
                    || v17 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    || {
                        v18 = gOamEnt(v13 as ::core::ffi::c_int, game);
                        *(&raw mut (*v18).xcoord as *mut uint16) =
                            (v17 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
                        v19 = *(v16.offset(6) as *mut uint16) as int16;
                        *v16.offset(7) as ::core::ffi::c_int != 0
                    }
                    || v19 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    let mut v21: *mut uint8 = v15;
                    *(v21 as *mut uint16).offset(5) = 32 as uint16;
                    *(v21 as *mut uint16).offset(1) = 128 as uint16;
                    *(v21 as *mut uint16).offset(3) = 128 as uint16;
                    *(v21 as *mut uint16).offset(2) = 0 as uint16;
                    *(v21 as *mut uint16).offset(4) = 0 as uint16;
                    *(v21 as *mut uint16).offset(6) = 0 as uint16;
                    *(v21 as *mut uint16).offset(7) = 0 as uint16;
                    *(v21 as *mut uint16) = *v21 as uint16;
                } else {
                    *(&raw mut (*v18).ycoord as *mut uint16) =
                        (v19 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
                    v20 = (*(v16.offset(10) as *mut uint16) as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int) as int16;
                    *(v16 as *mut uint16).offset(5) = v20 as uint16;
                    if v20 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
                        v22 = *(v16 as *mut uint16) as int16;
                        *(v16 as *mut uint16).offset(5) = *(game.g_rom.offset(
                            (((0x8be9cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x8be9cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        )
                            as *const uint8
                            as *mut uint16)
                            .offset(
                                ((8 as ::core::ffi::c_int
                                    * *(v16 as *mut uint16) as uint8 as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                    + 2 as ::core::ffi::c_int)
                                    as isize,
                            );
                        *(v16 as *mut uint16) =
                            (v22 as ::core::ffi::c_int + 512 as ::core::ffi::c_int) as uint16;
                    }
                    if *v16.offset(1) != 0 {
                        *(&raw mut (*(gOamEnt
                            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut OamEnt)(
                            v13 as ::core::ffi::c_int,
                            game,
                        ))
                        .charnum as *mut uint16) = *(game.g_rom.offset(
                            (((0x8be9a7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x8be9a7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8
                            as *mut uint16)
                            .offset(
                                (((*(v16 as *mut uint16) as ::core::ffi::c_int
                                    & 0xff00 as ::core::ffi::c_int)
                                    >> 8 as ::core::ffi::c_int)
                                    as uint16
                                    as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                    as isize,
                            );
                        v13 = (v13 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
            v14 = (v14 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
            n2 -= 1;
            if !(n2 != 0) {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = v13;
    }
}
pub unsafe fn CinematicFunction_Intro_Func150(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
        260 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func151(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        128 as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize);
    let ref mut fresh40 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh40 = (*fresh40).wrapping_add(1);
    if *fresh40 as ::core::ffi::c_int - 272 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func152(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func153(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        -(128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func154(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func155(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func156(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func157(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        160 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        512 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func158(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        160 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func159(mut j: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 156 as uint16;
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func161(mut j: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 164 as uint16;
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func162(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 172 as uint16;
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func163(mut j: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 180 as uint16;
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func164(mut j: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 188 as uint16;
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func160(mut j: uint16, mut a: uint16, game: &mut Game) {
    let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh3 = (*fresh3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int * a as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func165(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        -(96 as ::core::ffi::c_int) as uint16;
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_F0B8(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            384 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
            2560 as uint16;
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16)
            .offset(v1 as isize);
        *fresh2 = (*fresh2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
            2560 as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func166(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        -(32 as ::core::ffi::c_int) as uint16;
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func167(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 288 as uint16;
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func168(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 352 as uint16;
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func169(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        320 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            448 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            192 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func170(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        -(64 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b9d) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            -(192 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            64 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func171(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        136 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func172(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        152 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func173(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        124 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        108 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3072 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func174(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        121 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        107 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func175(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        530 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        231 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
        8 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func176(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        246 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        519 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
        8 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func177(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        385 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        366 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func178(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
        391 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
        384 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
}
pub unsafe fn CinematicFunction_Intro_Func179(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh38 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh38 = (*fresh38 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 395 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            394 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            367 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnPalfxObject(addr_kPalfx_E200 as uint16, game);
    } else {
        let ref mut fresh39 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize);
        *fresh39 = (*fresh39 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicFunction_Intro_Func180(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh36 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh36 = (*fresh36 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 382 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let ref mut fresh37 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16)
            .offset(v1 as isize);
        *fresh37 = (*fresh37 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize) =
            382 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            383 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CinematicSprInstr_Func181(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 34 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 100 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 84 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8be58a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicSprInstr_Func182(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnPalfxObject(addr_kPalfx_E1C4 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(254) = 1 as uint16;
    return j;
}
pub unsafe fn CinematicSprInstr_Func183(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0 as uint16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEB5 as uint16, v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(0) =
        0x7fff as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func184(mut j: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(15) == 0 {
        let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v2 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func185(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 17 as uint8;
    *(&raw mut v0 as *mut uint8).offset(0) = 124 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 124 as uint8;
    SpawnCinematicSpriteObjectToR18(
        addr_kCinematicSpriteObjectDef_8BEEBB as uint16,
        0 as uint16,
        game,
    );
    SpawnCinematicSpriteObjectToR18(
        addr_kCinematicSpriteObjectDef_8BEEF7 as uint16,
        2 as uint16,
        game,
    );
    SpawnPalfxObject(addr_kPalfx_E1C8 as uint16, game);
    SpawnPalfxObject(addr_kPalfx_E1D0 as uint16, game);
    SpawnPalfxObject(addr_kPalfx_E1E8 as uint16, game);
    return j;
}
pub unsafe fn CinematicSprInstr_Func186(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 18 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 51 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 116 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 120 as uint8;
    SpawnPalfxObject(addr_kPalfx_E1CC as uint16, game);
    SpawnCinematicSpriteObjectToR18(
        addr_kCinematicSpriteObjectDef_8BEEC1 as uint16,
        6 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func187(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(0) =
        0x7fff as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(128) =
        0x7fff as uint16;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 16 as ::core::ffi::c_int) as isize) =
            0x7fff as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bdb9e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 120 as uint16;
    return j;
}
pub unsafe fn CinematicSprPreInstr_F35A(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bdca5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bf375 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize) =
            0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func189(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        -(32 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1b7d) as *mut uint16).offset(v1 as isize)
            as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    CinematicFunction_Intro_Func189b(k, game);
}
pub unsafe fn CinematicFunction_Intro_Func189b(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bde63 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v5 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v5 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func190(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0 as uint16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEECD as uint16, v0, game);
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func191(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bde80 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func192(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEEFD as uint16,
        0 as uint16,
        game,
    );
    let mut hours1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9e0)
        as *mut uint16) as ::core::ffi::c_int
        / 10 as ::core::ffi::c_int;
    let mut hours2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9e0)
        as *mut uint16) as ::core::ffi::c_int
        % 10 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = hours1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = hours2 as uint16;
    let mut min1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9de)
        as *mut uint16) as ::core::ffi::c_int
        / 10 as ::core::ffi::c_int;
    let mut min2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9de)
        as *mut uint16) as ::core::ffi::c_int
        % 10 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf0) as *mut uint16) = min1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) = min2 as uint16;
    return j;
}
pub unsafe fn CinematicSprInstr_Func193(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEF03 as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func194(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEF09 as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func195(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEF0F as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func196(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEF15 as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func197(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BEF1B as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprInstr_Func198(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8bde64 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func199(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 176 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bf464 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func200(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize);
    let ref mut fresh35 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh35 = (*fresh35).wrapping_add(1);
}
pub unsafe fn CinematicFunction_Intro_Func201(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 176 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bf487 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func202(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize);
    let ref mut fresh34 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh34 = (*fresh34).wrapping_sub(1);
}
pub unsafe fn CinematicFunction_Intro_Func203(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bf4aa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func204(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize);
    let ref mut fresh32 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh32 = (*fresh32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize);
    let ref mut fresh33 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh33 = (*fresh33).wrapping_sub(1);
}
pub unsafe fn CinematicFunction_Intro_Func205(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16) as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (0x8bf4e0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func206(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize);
    let ref mut fresh30 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize);
    *fresh30 = (*fresh30 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v1 as isize);
    let ref mut fresh31 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16).offset(v1 as isize);
    *fresh31 = (*fresh31).wrapping_add(1);
}
pub unsafe fn CinematicFunction_Intro_Func207(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bdb9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func208(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    SpawnCinematicSpriteObjectToR18(
        addr_kCinematicSpriteObjectDef_8BEF3F as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprPreInstr_F528(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_word_8BED95 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8bf5dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16).offset(v1 as isize) =
            3072 as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func211(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
        - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        SpawnCinematicSpriteObjectToR18(
            addr_kCinematicSpriteObjectDef_8BEF7B as uint16,
            2 as uint16,
            game,
        );
    } else {
        SpawnCinematicSpriteObjectToR18(
            addr_kCinematicSpriteObjectDef_8BEF75 as uint16,
            2 as uint16,
            game,
        );
    }
    SpawnCinematicSpriteObjectToR18(
        addr_kCinematicSpriteObjectDef_8BEF6F as uint16,
        0 as uint16,
        game,
    );
    return j;
}
pub unsafe fn CinematicSprPreInstr_F57F(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_off_8BEE9B as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func213(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
        - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 120 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 118 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 120 as uint16;
    return j;
}
pub unsafe fn CinematicSprInstr_Func214(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
        - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 121 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 116 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a7d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 120 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 114 as uint16;
    }
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func215(mut k: uint16, game: &mut Game) {
    CinematicFunction_Intro_Func216(game);
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 136 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize) =
            136 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b5d) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v1 as isize) =
            addr_word_8BED9D as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b3d) as *mut uint16).offset(v1 as isize) =
            (0x8b93d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CinematicSprInstr_Func217(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8be3ae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = -(127 as ::core::ffi::c_int) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        -(4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) =
        -(8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4f) as *mut uint16) = 32 as uint16;
    CopyPalettesToFadingPalettes(game);
    DecomposePaletteDataForFading(game);
    QueueSfx1_Max15(0x22 as uint16, game);
    return j;
}
pub unsafe fn CinematicSprInstr_Func218(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) =
        -(16 as ::core::ffi::c_int) as uint16;
    return j;
}
pub unsafe fn CinematicFunction_Intro_Func210_EndingSamusFall(mut k: uint16, game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16,
        0x3800 as uint32,
    );
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16) as uint32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func216(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v0: uint16 = (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        (*gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .size = 2048 as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size = *(game.g_rom.offset(
            (((0x8bf6b8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8bf6b8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v2 as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let fresh28 = v1;
        v1 = v1.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
            fresh28 as ::core::ffi::c_int,
            game,
        ))
        .size as *mut uint8)
            .offset(0) = 127 as uint8;
        (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size = *(game.g_rom.offset(
            (((0x8bf6d8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x8bf6d8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4d) as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
    }
}
pub unsafe fn CinematicFunction_Intro_Func219(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    DisableTextGlowObjects_(game);
    let mut v0: uint16 = 8 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0x8ce7e9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x8ce7e9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(1) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (0x8be0f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a49) as *mut uint16) = 60 as uint16;
    return j;
}
pub unsafe fn CallCinematicFunction(mut ea: uint32, game: &mut Game) {
    match ea {
        9148999 => {
            CinematicFunctionNone();
            return;
        }
        9149288 => {
            CinematicFunctionOpening(game);
            return;
        }
        9149719 => {
            CinematicFunc_Func2(game);
            return;
        }
        9149840 => {
            CinematicFunc_Func4(game);
            return;
        }
        9149970 => {
            CinematicFunc_Func5(game);
            return;
        }
        9150091 => {
            CinematicFunc_Func7(game);
            return;
        }
        9150248 => return,
        9150249 => {
            CinematicFunc_Func1(game);
            return;
        }
        9150290 => {
            CinematicFunc_Func10(game);
            return;
        }
        9150382 => {
            CinematicFunc_Func9(game);
            return;
        }
        9151375 => return,
        9151376 => return,
        9151377 => {
            CinematicFunc_Nothing();
            return;
        }
        9151381 => {
            CinematicFunction_Intro_Initial(game);
            return;
        }
        9151911 => {
            CinematicFunction_Intro_FadeIn(game);
            return;
        }
        9151933 => {
            CinematicFunction_Intro_DrawInitJpn(game);
            return;
        }
        9151992 => {
            CinematicFunction_Intro_LastMetroidCaptivity(game);
            return;
        }
        9152019 => {
            CinematicFunc_Intro_QueueGalaxyIsAtPeace(game);
            return;
        }
        9152057 => {
            CinematicFunc_Intro_WaitForQueueWait4secs(game);
            return;
        }
        9152076 => {
            CinematicFunc_Intro_FadeOut(game);
            return;
        }
        9152099 => {
            CinematicFunc_Intro_WaitForFadeOut(game);
            return;
        }
        9152111 => {
            CinematicFunc_Intro_SetupTextPage2(game);
            return;
        }
        9152555 => {
            CinematicFunc_Intro_WaitFadeinShowText(game);
            return;
        }
        9152586 => {
            CinematicFunc_Intro_WaitForFadeinSleep(game);
            return;
        }
        9154232 => {
            CinematicFunction_Intro_WaitInputSetupMotherBrainFight(game);
            return;
        }
        9154412 => {
            CinematicFunction_Intro_WaitInputSetupBabyMetroid(game);
            return;
        }
        9154802 => {
            CinematicFunction_Intro_Func11(game);
            return;
        }
        9154851 => {
            CinematicFunction_Intro_Func12(game);
            return;
        }
        9155034 => {
            CinematicFunction_Intro_Func15(game);
            return;
        }
        9155079 => {
            CinematicFunction_Intro_Func15b(game);
            return;
        }
        9155152 => {
            CinematicFunction_Intro_XfadeGameplayFade(game);
            return;
        }
        9155282 => {
            CinematicFunction_Intro_XfadeScientistFade(game);
            return;
        }
        9155423 => {
            CinematicFunction_Intro_Page2(game);
            return;
        }
        9155440 => {
            CinematicFunction_Intro_Page3(game);
            return;
        }
        9155457 => {
            CinematicFunction_Intro_Page4(game);
            return;
        }
        9155474 => {
            CinematicFunction_Intro_Page5(game);
            return;
        }
        9155572 => {
            CinematicFunction_Intro_Fadestuff(game);
            return;
        }
        9155672 => {
            CinematicFunction_Intro_Fadestuff2(game);
            return;
        }
        9156399 => {
            CinematicFunction_Intro_Func34(game);
            return;
        }
        9157792 => {
            CinematicFunction_Intro_Func54(game);
            return;
        }
        9158116 => {
            CinematicFunction_Intro_Func55(game);
            return;
        }
        9158137 => {
            CinematicFunction_Intro_Func56(game);
            return;
        }
        9158618 => {
            CinematicFunction_Intro_Func67(game);
            return;
        }
        9158853 => {
            CinematicFunction_Intro_Func72(game);
            return;
        }
        9158912 => {
            CinematicFunction_Intro_Func73(game);
            return;
        }
        9158939 => {
            CinematicFunctionBlackoutFromCeres(game);
            return;
        }
        9159396 => {
            CinematicFunction_Intro_Func74(game);
            return;
        }
        9159409 => {
            CinematicFunction_Intro_Func75(game);
            return;
        }
        9159493 => {
            CinematicFunction_Intro_Func76(game);
            return;
        }
        9160138 => {
            CinematicFunction_Intro_Func77(game);
            return;
        }
        9160208 => {
            CinematicFunction_Intro_Func84(game);
            return;
        }
        9160231 => {
            CinematicFunction_Intro_Func85(game);
            return;
        }
        9160345 => {
            CinematicFunction_Intro_Func86(game);
            return;
        }
        9160604 => {
            CinematicFunction_Intro_Func87(game);
            return;
        }
        9160650 => {
            CinematicFunction_Intro_Func88(game);
            return;
        }
        9161209 => {
            CinematicFunction_Intro_Func105(game);
            return;
        }
        9161270 => {
            CinematicFunction_Intro_Func106(game);
            return;
        }
        9161349 => {
            CinematicFunction_Intro_Func107(game);
            return;
        }
        9161424 => {
            CinematicFunction_Intro_Func108(game);
            return;
        }
        9161438 => return,
        9161439 => {
            CinematicFunction_Intro_Func95(game);
            return;
        }
        9163904 => {
            CinematicFunctionEscapeFromCebes(game);
            return;
        }
        9164503 => {
            CinematicFunction_Intro_Func109(game);
            return;
        }
        9164530 => {
            CinematicFunction_Intro_Func110(game);
            return;
        }
        9164545 => {
            CinematicFunction_Intro_Func111(game);
            return;
        }
        9164593 => {
            CinematicFunction_Intro_Func112(game);
            return;
        }
        9164792 => {
            CinematicFunction_Intro_Func113(game);
            return;
        }
        9164807 => {
            CinematicFunction_Intro_Func114(game);
            return;
        }
        9164855 => {
            CinematicFunction_Intro_Func115(game);
            return;
        }
        9165314 => {
            CinematicFunction_Intro_Func116(game);
            return;
        }
        9165338 => {
            CinematicFunction_Intro_Func117(game);
            return;
        }
        9165523 => {
            CinematicFunction_Intro_Func118(game);
            return;
        }
        9165725 => return,
        9165726 => {
            CinematicFunction_Intro_Func119(game);
            return;
        }
        9165764 => {
            CinematicFunction_Intro_Func120(game);
            return;
        }
        9165989 => {
            CinematicFunction_Intro_Func121(game);
            return;
        }
        9166146 => {
            CinematicFunction_Intro_Func123(game);
            return;
        }
        9166285 => {
            CinematicFunction_Intro_Func124(game);
            return;
        }
        9166435 => return,
        9166436 => {
            CinematicFunction_Intro_Func125(game);
            return;
        }
        9166464 => {
            CinematicFunction_Intro_Func126(game);
            return;
        }
        9167092 => {
            CinematicFunction_Intro_Func129(game);
            return;
        }
        9167120 => {
            CinematicFunction_Intro_Func130(game);
            return;
        }
        9167192 => {
            CinematicFunction_Intro_Func131(game);
            return;
        }
        9167248 => {
            CinematicFunction_Intro_Func132(game);
            return;
        }
        9167314 => {
            CinematicFunction_Intro_Func134(game);
            return;
        }
        9167461 => {
            CinematicFunction_Intro_Func136(game);
            return;
        }
        9167507 => {
            CinematicFunction_Intro_Func135(game);
            return;
        }
        9167581 => {
            CinematicFunction_Intro_Func137(game);
            return;
        }
        9167636 => {
            CinematicFunction_Intro_Func138(game);
            return;
        }
        9167682 => {
            CinematicFunction_Intro_Func145(game);
            return;
        }
        9167790 => {
            CinematicFunction_Intro_Func139(game);
            return;
        }
        9168010 => {
            CinematicFunction_Intro_Func141(game);
            return;
        }
        9168132 => {
            CinematicFunction_Intro_Func143(game);
            return;
        }
        9168266 => {
            CinematicFunction_Intro_Func144(game);
            return;
        }
        9168791 => {
            CinematicFunction_Intro_Func148(game);
            return;
        }
        9168826 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallCinematicSprInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        9147448 => return CinematicSprInstr_Delete(k, j, game),
        9147458 => return CinematicSprInstr_Sleep(k, j, game),
        9147468 => return CinematicSprInstr_SetPreInstr(k, j, game),
        9147479 => return CinematicSprInstr_ClearPreInstr(k, j, game),
        9147554 => return CinematicSprInstr_GotoRel(k, j, game),
        9147580 => return CinematicSprInstr_Goto(k, j, game),
        9147587 => return CinematicSprInstr_DecTimerGoto(k, j, game),
        9147597 => return CinematicSprInstr_DecTimerGotoRel(k, j, game),
        9147606 => return CinematicSprInstr_SetTimer(k, j, game),
        9149665 => return CinematicSprInstr_9CE1(k, j, game),
        9149789 => return CinematicSprInstr_9D5D(k, j, game),
        9149910 => return CinematicSprInstr_9DD6(k, j, game),
        9150040 => return CinematicSprInstr_Func8(k, j, game),
        9150157 => return CinematicSprInstr_Func9(k, j, game),
        9150192 => return CinematicSprInstr_sub_8B9EF0(k, j, game),
        9150233 => return CinematicSprInstr_9F19(k, j, game),
        9152792 => return CinematicSprInstr_SpawnMetroidEggParticles(k, j, game),
        9155382 => return CinematicSprInstr_StartIntroPage2(k, j, game),
        9155390 => return CinematicSprInstr_StartIntroPage3(k, j, game),
        9155398 => return CinematicSprInstr_StartIntroPage4(k, j, game),
        9155406 => return CinematicSprInstr_StartIntroPage5(k, j, game),
        9156805 => return CinematicSprInstr_Func43(k, j, game),
        9157153 => return CinematicSprInstr_SpawnIntroRinkas01(k, j, game),
        9157174 => return CinematicSprInstr_SpawnIntroRinkas23(k, j, game),
        9158806 => return CinematicCommonInstr_Func69(k, j, game),
        9158818 => return CinematicCommonInstr_Func70(k, j, game),
        9159684 => return CinematicSprInstr_SpawnCeresExplosions1(k, j, game),
        9159948 => return CinematicSprInstr_SpawnCeresExplosions3(k, j, game),
        9161125 => return CinematicSprInstr_C9A5(k, j, game),
        9161135 => return CinematicSprInstr_C9AF(k, j, game),
        9161149 => return CinematicSprInstr_C9BD(k, j, game),
        9161159 => return CinematicSprInstr_C9C7(k, j, game),
        9171550 => return CinematicSprInstr_Func181(k, j, game),
        9171588 => return CinematicSprInstr_Func182(k, j, game),
        9171605 => return CinematicSprInstr_Func183(k, j, game),
        9171639 => return CinematicSprInstr_Func185(k, j, game),
        9171706 => return CinematicSprInstr_Func186(k, j, game),
        9171755 => return CinematicSprInstr_Func187(k, j, game),
        9171888 => return CinematicSprInstr_Func190(k, j, game),
        9171918 => return CinematicSprInstr_Func192(k, j, game),
        9171995 => return CinematicSprInstr_Func193(k, j, game),
        9172004 => return CinematicSprInstr_Func194(k, j, game),
        9172013 => return CinematicSprInstr_Func195(k, j, game),
        9172022 => return CinematicSprInstr_Func196(k, j, game),
        9172031 => return CinematicSprInstr_Func197(k, j, game),
        9172040 => return CinematicSprInstr_Func198(k, j, game),
        9172253 => return CinematicSprInstr_Func208(k, j, game),
        9172308 => return CinematicSprInstr_Func211(k, j, game),
        9172375 => return CinematicSprInstr_Func213(k, j, game),
        9172410 => return CinematicSprInstr_Func214(k, j, game),
        9172484 => return CinematicSprInstr_Func217(k, j, game),
        9172561 => return CinematicSprInstr_Func218(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn CallCinematicSprPreInstr(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        9158617 => return,
        9147353 => return,
        9147487 => return,
        9149647 => {
            sub_8B9CCF(j, game);
            return;
        }
        9152724 => return,
        9152744 => {
            SetSomeStuffForSpriteObject_4_MetroidEgg(j, game);
            return;
        }
        9152771 => {
            CinematicSprPreInstr_A903(j, game);
            return;
        }
        9152916 => {
            SetSomeStuffForSpriteObject_7(j, game);
            return;
        }
        9153203 => {
            SetSomeStuffForSpriteObject_9(j, game);
            return;
        }
        9153896 => {
            SetSomeStuffForSpriteObject_11(j, game);
            return;
        }
        9153958 => {
            SetSomeStuffForSpriteObject_13(j, game);
            return;
        }
        9154074 => {
            SetSomeStuffForSpriteObject_18(j, game);
            return;
        }
        9156486 => {
            CinematicFunction_Intro_Func36(j, game);
            return;
        }
        9156623 => {
            CinematicFunction_Intro_Func39(j, game);
            return;
        }
        9156654 => {
            CinematicSprPreInstr_B82E(j, game);
            return;
        }
        9156824 => {
            CinematicFunction_Intro_Func44(j, game);
            return;
        }
        9156923 => {
            CinematicFunction_Intro_Func45(j, game);
            return;
        }
        9157135 => {
            CinematicFunction_Intro_Func48(j, game);
            return;
        }
        9157214 => {
            CinematicFunction_Intro_Func50(j, game);
            return;
        }
        9157235 => {
            CinematicFunction_Intro_Func51(j, game);
            return;
        }
        9157389 => {
            CinematicFunction_Intro_Func52(j, game);
            return;
        }
        9157412 => {
            CinematicFunction_Intro_Func53(j, game);
            return;
        }
        9158325 => {
            CinematicFunction_Intro_Func58(j, game);
            return;
        }
        9158453 => {
            CinematicFunction_Intro_Func60(j, game);
            return;
        }
        9158495 => {
            CinematicFunction_Intro_Func62(j, game);
            return;
        }
        9158537 => {
            CinematicFunction_Intro_Func64(j, game);
            return;
        }
        9158598 => {
            CinematicFunction_Intro_Func66(j, game);
            return;
        }
        9159817 => {
            CinematicSprPreInstr_C489(j, game);
            return;
        }
        9160066 => {
            CinematicFunction_Intro_Func82(j, game);
            return;
        }
        9160782 => {
            CinematicFunction_Intro_Func90(j, game);
            return;
        }
        9160797 => {
            CinematicFunction_Intro_Func91(j, game);
            return;
        }
        9160874 => {
            CinematicFunction_Intro_Func93(j, game);
            return;
        }
        9160889 => {
            CinematicFunction_Intro_Func94(j, game);
            return;
        }
        9160953 => {
            CinematicFunction_Intro_Func96(j, game);
            return;
        }
        9160968 => {
            CinematicFunction_Intro_Func97(j, game);
            return;
        }
        9170866 => {
            CinematicFunction_Intro_Func151(j, game);
            return;
        }
        9171431 => {
            CinematicFunction_Intro_Func179(j, game);
            return;
        }
        9171495 => {
            CinematicFunction_Intro_Func180(j, game);
            return;
        }
        9171621 => {
            CinematicFunction_Intro_Func184(j, game);
            return;
        }
        9171802 => {
            CinematicSprPreInstr_F35A(j, game);
            return;
        }
        9171829 => {
            CinematicFunction_Intro_Func189(j, game);
            return;
        }
        9171867 => {
            CinematicFunction_Intro_Func189b(j, game);
            return;
        }
        9171897 => {
            CinematicFunction_Intro_Func191(j, game);
            return;
        }
        9172053 => {
            CinematicFunction_Intro_Func199(j, game);
            return;
        }
        9172068 => {
            CinematicFunction_Intro_Func200(j, game);
            return;
        }
        9172088 => {
            CinematicFunction_Intro_Func201(j, game);
            return;
        }
        9172103 => {
            CinematicFunction_Intro_Func202(j, game);
            return;
        }
        9172123 => {
            CinematicFunction_Intro_Func203(j, game);
            return;
        }
        9172138 => {
            CinematicFunction_Intro_Func204(j, game);
            return;
        }
        9172177 => {
            CinematicFunction_Intro_Func205(j, game);
            return;
        }
        9172192 => {
            CinematicFunction_Intro_Func206(j, game);
            return;
        }
        9172231 => {
            CinematicFunction_Intro_Func207(j, game);
            return;
        }
        9172252 => return,
        9172264 => {
            CinematicSprPreInstr_F528(j, game);
            return;
        }
        9172351 => {
            CinematicSprPreInstr_F57F(j, game);
            return;
        }
        9172445 => {
            CinematicFunction_Intro_Func215(j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallCinematicBgPreInstr(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        9147353 => return,
        9155772 => {
            CinematicBgPreInstr_SamusBlink(j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallCinematicBgInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        9148056 => return CinematicBgInstr_Delete(k, j, game),
        9148190 => return CinematicBgInstr_Goto(k, j, game),
        9154004 => return CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game),
        9154115 => return CinematicBgInstr_HandleCreateJpnText_Page1(k, j, game),
        9154139 => return CinematicBgInstr_SpawnMarkerWaitInput_Page1(k, j, game),
        9154169 => return CinematicBgInstr_HandleCreateJpnText_Page2(k, j, game),
        9154193 => return CinematicBgInstr_SpawnMarkerWaitInput_Page2(k, j, game),
        9154676 => return CinematicBgInstr_HandleCreateJpnText_Page3(k, j, game),
        9154700 => return CinematicBgInstr_SpawnMarkerWaitInput_Page3(k, j, game),
        9154739 => return CinematicBgInstr_HandleCreateJpnText_Page4(k, j, game),
        9154763 => return CinematicBgInstr_SpawnMarkerWaitInput_Page4(k, j, game),
        9154971 => return CinematicBgInstr_HandleCreateJpnText_Page5(k, j, game),
        9154995 => return CinematicBgInstr_SpawnMarkerWaitInput_Page5(k, j, game),
        9155112 => return CinematicBgInstr_Func16(k, j, game),
        9155136 => return CinematicBgInstr_Func17(k, j, game),
        9158806 => return CinematicCommonInstr_Func69(k, j, game),
        9158818 => return CinematicCommonInstr_Func70(k, j, game),
        9168423 => return CalcItemPercentageCount(k, j, game),
        9168745 => return CinematicFunction_Intro_Func146(k, j, game),
        9168768 => return CinematicFunction_Intro_Func147(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn CallCinematicSpriteObjectSetup(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        9147353 => return,
        9149210 => {
            CinematicSpriteInit_7(j, game);
            return;
        }
        9149229 => {
            CinematicSpriteInit_8(j, game);
            return;
        }
        9149248 => {
            CinematicSpriteInit_9(j, game);
            return;
        }
        9149628 => {
            CinematicSpriteInit_0(j, game);
            return;
        }
        9149770 => {
            CinematicSpriteInit_1(j, game);
            return;
        }
        9149891 => {
            CinematicSpriteInit_2(j, game);
            return;
        }
        9150021 => {
            CinematicSpriteInit_3(j, game);
            return;
        }
        9150131 => {
            CinematicSpriteInit_4(j, game);
            return;
        }
        9150166 => {
            CinematicSpriteInit_5(j, game);
            return;
        }
        9150207 => {
            CinematicSpriteInit_6(j, game);
            return;
        }
        9152705 => {
            SetSomeStuffForSpriteObject_2(j, game);
            return;
        }
        9152725 => {
            SetSomeStuffForSpriteObject_3(j, game);
            return;
        }
        9152856 => {
            SetSomeStuffForSpriteObject_6(j, game);
            return;
        }
        9153178 => {
            SetSomeStuffForSpriteObject_8(j, game);
            return;
        }
        9153877 => {
            SetSomeStuffForSpriteObject_10(j, game);
            return;
        }
        9153939 => {
            SetSomeStuffForSpriteObject_12(j, game);
            return;
        }
        9154055 => {
            SetSomeStuffForSpriteObject_17(j, game);
            return;
        }
        9156467 => {
            CinematicFunction_Intro_Func35(j, game);
            return;
        }
        9156758 => {
            CinematicFunction_Intro_Func42(j, game);
            return;
        }
        9157005 => {
            CinematicFunction_Intro_Func46(j, game);
            return;
        }
        9157076 => {
            CinematicFunction_Intro_Func47(j, game);
            return;
        }
        9157195 => {
            CinematicFunction_Intro_Func49(j, game);
            return;
        }
        9158270 => {
            CinematicFunction_Intro_Func57(j, game);
            return;
        }
        9158434 => {
            CinematicFunction_Intro_Func59(j, game);
            return;
        }
        9158476 => {
            CinematicFunction_Intro_Func61(j, game);
            return;
        }
        9158518 => {
            CinematicFunction_Intro_Func63(j, game);
            return;
        }
        9158560 => {
            CinematicFunction_Intro_Func65(j, game);
            return;
        }
        9158787 => {
            CinematicFunction_Intro_Func68(j, game);
            return;
        }
        9158834 => {
            CinematicFunction_Intro_Func71(j, game);
            return;
        }
        9159732 => {
            CinematicFunction_Intro_Func78(j, game);
            return;
        }
        9159865 => {
            CinematicFunction_Intro_Func80(j, game);
            return;
        }
        9159987 => {
            CinematicFunction_Intro_Func81(j, game);
            return;
        }
        9160105 => {
            CinematicFunction_Intro_Func83(j, game);
            return;
        }
        9160763 => {
            CinematicFunction_Intro_Func89(j, game);
            return;
        }
        9160855 => {
            CinematicFunction_Intro_Func92(j, game);
            return;
        }
        9161026 => {
            CinematicFunction_Intro_Func98(j, game);
            return;
        }
        9161046 => {
            CinematicFunction_Intro_Func99(j, game);
            return;
        }
        9161066 => {
            CinematicFunction_Intro_Func100(j, game);
            return;
        }
        9161086 => {
            CinematicFunction_Intro_Func101(j, game);
            return;
        }
        9161106 => {
            CinematicFunction_Intro_Func102(j, game);
            return;
        }
        9161190 => {
            CinematicFunction_Intro_Func104(j, game);
            return;
        }
        9170841 => {
            CinematicFunction_Intro_Func150(j, game);
            return;
        }
        9170922 => {
            CinematicFunction_Intro_Func152(j, game);
            return;
        }
        9170941 => {
            CinematicFunction_Intro_Func153(j, game);
            return;
        }
        9170949 => {
            CinematicFunction_Intro_Func154(j, game);
            return;
        }
        9170968 => {
            CinematicFunction_Intro_Func155(j, game);
            return;
        }
        9170987 => {
            CinematicFunction_Intro_Func156(j, game);
            return;
        }
        9171006 => {
            CinematicFunction_Intro_Func157(j, game);
            return;
        }
        9171038 => {
            CinematicFunction_Intro_Func159(j, game);
            return;
        }
        9171053 => {
            CinematicFunction_Intro_Func161(j, game);
            return;
        }
        9171068 => {
            CinematicFunction_Intro_Func162(j, game);
            return;
        }
        9171077 => {
            CinematicFunction_Intro_Func163(j, game);
            return;
        }
        9171092 => {
            CinematicFunction_Intro_Func164(j, game);
            return;
        }
        9171122 => {
            CinematicFunction_Intro_Func165(j, game);
            return;
        }
        9171169 => {
            CinematicFunction_Intro_Func166(j, game);
            return;
        }
        9171177 => {
            CinematicFunction_Intro_Func167(j, game);
            return;
        }
        9171185 => {
            CinematicFunction_Intro_Func168(j, game);
            return;
        }
        9171193 => {
            CinematicFunction_Intro_Func169(j, game);
            return;
        }
        9171230 => {
            CinematicFunction_Intro_Func170(j, game);
            return;
        }
        9171267 => {
            CinematicFunction_Intro_Func171(j, game);
            return;
        }
        9171286 => {
            CinematicFunction_Intro_Func172(j, game);
            return;
        }
        9171305 => {
            CinematicFunction_Intro_Func173(j, game);
            return;
        }
        9171324 => {
            CinematicFunction_Intro_Func174(j, game);
            return;
        }
        9171343 => {
            CinematicFunction_Intro_Func175(j, game);
            return;
        }
        9171368 => {
            CinematicFunction_Intro_Func176(j, game);
            return;
        }
        9171393 => {
            CinematicFunction_Intro_Func177(j, game);
            return;
        }
        9171412 => {
            CinematicFunction_Intro_Func178(j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallIntroObjectInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        9148926 => return IntroObject_Delete(k, j, game),
        9148934 => return IntroObject_Goto(k, j, game),
        9148941 => return IntroObject_DecTimerGoto(k, j, game),
        9148951 => return IntroObject_SetTimer(k, j, game),
        9172734 => return CinematicFunction_Intro_Func219(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn CallCinematicSpriteInit(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        9149210 => {
            CinematicSpriteInit_7(j, game);
            return;
        }
        9149229 => {
            CinematicSpriteInit_8(j, game);
            return;
        }
        9149248 => {
            CinematicSpriteInit_9(j, game);
            return;
        }
        9149628 => {
            CinematicSpriteInit_0(j, game);
            return;
        }
        9149770 => {
            CinematicSpriteInit_1(j, game);
            return;
        }
        9149891 => {
            CinematicSpriteInit_2(j, game);
            return;
        }
        9150021 => {
            CinematicSpriteInit_3(j, game);
            return;
        }
        9150131 => {
            CinematicSpriteInit_4(j, game);
            return;
        }
        9150166 => {
            CinematicSpriteInit_5(j, game);
            return;
        }
        9150207 => {
            CinematicSpriteInit_6(j, game);
            return;
        }
        9147353 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallMode7Instr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        9147755 => return Mode7Instr_Delete(k, j, game),
        9147762 => return Mode7Instr_SetPreInstr(k, j, game),
        9147773 => return Mode7Instr_ClearPreInstr(k, j, game),
        9147782 => return Mode7Instr_Goto(k, j, game),
        9147789 => return Mode7Instr_DecTimerAndGoto(k, j, game),
        9147799 => return Mode7Instr_SetTimer(k, j, game),
        9151067 => return PlayBabyMetroidCry1(k, j, game),
        9151075 => return PlayBabyMetroidCry2(k, j, game),
        9151083 => return PlayBabyMetroidCry3(k, j, game),
        9151091 => return CinematicSetPal1(k, j, game),
        9151108 => return CinematicSetPal2(k, j, game),
        9151125 => return CinematicSetPal3(k, j, game),
        9151142 => return CinematicSetPal4(k, j, game),
        9155870 => return sub_8BB51E(k, j, game),
        9155919 => return EnableCinematicBgTilemapUpdates__0(k, j, game),
        9155924 => return CinematicFunction_Intro_Func21(k, j, game),
        9156024 => return CinematicFunction_Intro_ThenWaitInputSetupBabyMetroid(k, j, game),
        9156035 => return CinematicFunction_Intro_Func23(k, j, game),
        9156123 => return CinematicFunction_Intro_Func25(k, j, game),
        9156134 => return CinematicFunction_Intro_Func26(k, j, game),
        9156234 => return CinematicFunction_Intro_Func28(k, j, game),
        9156245 => return CinematicFunction_Intro_Func29(k, j, game),
        9156333 => return CinematicFunction_Intro_Func31(k, j, game),
        9156344 => return CinematicFunction_Intro_Func32(k, j, game),
        9156363 => return EnableCinematicBgTilemapUpdates__(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn CallMode7PreInstr(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        9147353 => return,
        9155973 => {
            CinematicFunction_Intro_Func22(k, game);
            return;
        }
        9156078 => {
            CinematicFunction_Intro_Func24(k, game);
            return;
        }
        9156183 => {
            CinematicFunction_Intro_Func27(k, game);
            return;
        }
        9156288 => {
            CinematicFunction_Intro_Func30(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
