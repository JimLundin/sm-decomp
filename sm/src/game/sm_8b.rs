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
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::AddToHiLo;
use crate::rtl::{Mult8x8, RomPtr, SnesDivide, Unreachable, WriteReg, WriteRegWord};
pub use crate::snes::registers::*;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::ptr::null_mut;
extern "C" {
    fn __assert_fail(
        __assertion: *const c_char,
        __file: *const c_char,
        __line: u32,
        __function: *const c_char,
    ) -> !;
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;

}
pub use crate::types::{
    size_t, CoroutineRet, Func_V, LongPtr, OamEnt, Point16U, SnesRegs, VoidP, VramWriteEntry,
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
pub const kCoroutineNone: i32 = 0;
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    '_c2rust_label: {
        if addr < 0x2000u16 {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const c_char,
                51u32,
                b"uint8 *RomPtr_RAM(uint16_t)\0" as *const u8 as *const c_char,
            );
        }
    };
    (&raw mut game.g_ram as *mut u8).offset(addr as i32 as isize)
}
const RomPtr_8B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8b>;
const RomPtr_8C: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8c>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370i32 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_Mode7ObjectDef(a: u16, game: &mut Game) -> *mut Mode7ObjectDef {
    RomPtr((0x8b0000u32) | ((a) as u32), game) as *mut Mode7ObjectDef
}
pub const fnProcessCinematicBgObject_Nothing: u32 = 9144393;
pub const fnProcessCinematicBgObject_DrawChar: u32 = 9144397;
pub const fnProcessCinematicBgObject_DrawToTextTilemap: u32 = 9144503;
pub const fnProcessCinematicBgObject_DrawToBgTilemap: u32 = 9144573;
pub const fnProcessCinematicBgObject_Unk1: u32 = 9144683;
pub const fnProcessCinematicBgObject_Unk2: u32 = 9144783;
pub const fnCinematicFunction_nullsub_116: i32 = 0x8b93d9;
pub const fnCinematicSprInstr_Delete: u32 = 9147448;
pub const fnCinematicSprInstr_Sleep: u32 = 9147458;
pub const fnCinematicSprInstr_SetPreInstr: u32 = 9147468;
pub const fnCinematicSprInstr_ClearPreInstr: u32 = 9147479;
pub const fnCinematicFunction_nullsub_298: i32 = 0x8b945f;
pub const fnCinematicSprInstr_GotoRel: u32 = 9147554;
pub const fnCinematicSprInstr_Goto: u32 = 9147580;
pub const fnCinematicSprInstr_DecTimerGoto: u32 = 9147587;
pub const fnCinematicSprInstr_DecTimerGotoRel: u32 = 9147597;
pub const fnCinematicSprInstr_SetTimer: u32 = 9147606;
pub const fnMode7Instr_Delete: u32 = 9147755;
pub const fnMode7Instr_SetPreInstr: u32 = 9147762;
pub const fnMode7Instr_ClearPreInstr: u32 = 9147773;
pub const fnMode7Instr_Goto: u32 = 9147782;
pub const fnMode7Instr_DecTimerAndGoto: u32 = 9147789;
pub const fnMode7Instr_SetTimer: u32 = 9147799;
pub const fnCinematicBgInstr_Delete: u32 = 9148056;
pub const fnCinematicBgInstr_Goto: u32 = 9148190;
pub const fnIntroObject_Delete: u32 = 9148926;
pub const fnIntroObject_Goto: u32 = 9148934;
pub const fnIntroObject_DecTimerGoto: u32 = 9148941;
pub const fnIntroObject_SetTimer: u32 = 9148951;
pub const fnCinematicFunctionNone: i32 = 0x8b9a47;
pub const fnCinematicSpriteInit_7: u32 = 9149210;
pub const fnCinematicSpriteInit_8: u32 = 9149229;
pub const fnCinematicSpriteInit_9: u32 = 9149248;
pub const fnCinematicFunctionOpening: i32 = 0x8b9b68;
pub const fnCinematicSpriteInit_0: u32 = 9149628;
pub const fnsub_8B9CCF: u32 = 9149647;
pub const fnCinematicSprInstr_9CE1: u32 = 9149665;
pub const fnCinematicFunc_Func2: i32 = 0x8b9d17;
pub const fnCinematicSpriteInit_1: u32 = 9149770;
pub const fnCinematicSprInstr_9D5D: u32 = 9149789;
pub const fnCinematicFunc_Func4: i32 = 0x8b9d90;
pub const fnCinematicSpriteInit_2: u32 = 9149891;
pub const fnCinematicSprInstr_9DD6: u32 = 9149910;
pub const fnCinematicFunc_Func5: i32 = 0x8b9e12;
pub const fnCinematicSpriteInit_3: u32 = 9150021;
pub const fnCinematicSprInstr_Func8: u32 = 9150040;
pub const fnCinematicFunc_Func7: i32 = 0x8b9e8b;
pub const fnCinematicSpriteInit_4: u32 = 9150131;
pub const fnCinematicSprInstr_Func9: u32 = 9150157;
pub const fnCinematicSpriteInit_5: u32 = 9150166;
pub const fnCinematicSprInstr_sub_8B9EF0: u32 = 9150192;
pub const fnCinematicSpriteInit_6: u32 = 9150207;
pub const fnCinematicSprInstr_9F19: u32 = 9150233;
pub const fnnullsub_117: i32 = 0x8b9f28;
pub const fnCinematicFunc_Func1: i32 = 0x8b9f29;
pub const fnCinematicFunc_Func10: i32 = 0x8b9f52;
pub const fnCinematicFunc_Func9: i32 = 0x8b9fae;
pub const fnPlayBabyMetroidCry1: u32 = 9151067;
pub const fnPlayBabyMetroidCry2: u32 = 9151075;
pub const fnPlayBabyMetroidCry3: u32 = 9151083;
pub const fnCinematicSetPal1: u32 = 9151091;
pub const fnCinematicSetPal2: u32 = 9151108;
pub const fnCinematicSetPal3: u32 = 9151125;
pub const fnCinematicSetPal4: u32 = 9151142;
pub const fnnullsub_120: i32 = 0x8ba38f;
pub const fnnullsub_121: i32 = 0x8ba390;
pub const fnCinematicFunc_Nothing: i32 = 0x8ba391;
pub const fnCinematicFunction_Intro_Initial: u32 = 9151381;
pub const fnCinematicFunction_Intro_FadeIn: i32 = 0x8ba5a7;
pub const fnCinematicFunction_Intro_DrawInitJpn: i32 = 0x8ba5bd;
pub const fnCinematicFunction_Intro_LastMetroidCaptivity: i32 = 0x8ba5f8;
pub const fnCinematicFunc_Intro_QueueGalaxyIsAtPeace: i32 = 0x8ba613;
pub const fnCinematicFunc_Intro_WaitForQueueWait4secs: i32 = 0x8ba639;
pub const fnCinematicFunc_Intro_FadeOut: i32 = 0x8ba64c;
pub const fnCinematicFunc_Intro_WaitForFadeOut: i32 = 0x8ba663;
pub const fnCinematicFunc_Intro_SetupTextPage2: i32 = 0x8ba66f;
pub const fnCinematicFunc_Intro_WaitFadeinShowText: i32 = 0x8ba82b;
pub const fnCinematicFunc_Intro_WaitForFadeinSleep: i32 = 0x8ba84a;
pub const fnSetSomeStuffForSpriteObject_2: u32 = 9152705;
pub const fnnullsub_122: u32 = 9152724;
pub const fnSetSomeStuffForSpriteObject_3: u32 = 9152725;
pub const fnSetSomeStuffForSpriteObject_4_MetroidEgg: u32 = 9152744;
pub const fnCinematicSprPreInstr_A903: u32 = 9152771;
pub const fnCinematicSprInstr_SpawnMetroidEggParticles: u32 = 9152792;
pub const fnSetSomeStuffForSpriteObject_6: u32 = 9152856;
pub const fnSetSomeStuffForSpriteObject_7: u32 = 9152916;
pub const fnSetSomeStuffForSpriteObject_8: u32 = 9153178;
pub const fnSetSomeStuffForSpriteObject_9: u32 = 9153203;
pub const fnSetSomeStuffForSpriteObject_10: u32 = 9153877;
pub const fnSetSomeStuffForSpriteObject_11: u32 = 9153896;
pub const fnSetSomeStuffForSpriteObject_12: u32 = 9153939;
pub const fnSetSomeStuffForSpriteObject_13: u32 = 9153958;
pub const fnCinematicBgInstr_SetSomeStuffForSpriteObject_14: u32 = 9154004;
pub const fnSetSomeStuffForSpriteObject_17: u32 = 9154055;
pub const fnSetSomeStuffForSpriteObject_18: u32 = 9154074;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page1: u32 = 9154115;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page1: u32 = 9154139;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page2: u32 = 9154169;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page2: u32 = 9154193;
pub const fnCinematicFunction_Intro_WaitInputSetupMotherBrainFight: i32 = 0x8baeb8;
pub const fnCinematicFunction_Intro_WaitInputSetupBabyMetroid: i32 = 0x8baf6c;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page3: u32 = 9154676;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page3: u32 = 9154700;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page4: u32 = 9154739;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page4: u32 = 9154763;
pub const fnCinematicFunction_Intro_Func11: i32 = 0x8bb0f2;
pub const fnCinematicFunction_Intro_Func12: i32 = 0x8bb123;
pub const fnCinematicBgInstr_HandleCreateJpnText_Page5: u32 = 9154971;
pub const fnCinematicBgInstr_SpawnMarkerWaitInput_Page5: u32 = 9154995;
pub const fnCinematicFunction_Intro_Func15: i32 = 0x8bb1da;
pub const fnCinematicFunction_Intro_Func15b: i32 = 0x8bb207;
pub const fnCinematicBgInstr_Func16: u32 = 9155112;
pub const fnCinematicBgInstr_Func17: u32 = 9155136;
pub const fnCinematicFunction_Intro_XfadeGameplayFade: i32 = 0x8bb250;
pub const fnCinematicFunction_Intro_XfadeScientistFade: i32 = 0x8bb2d2;
pub const fnCinematicSprInstr_StartIntroPage2: u32 = 9155382;
pub const fnCinematicSprInstr_StartIntroPage3: u32 = 9155390;
pub const fnCinematicSprInstr_StartIntroPage4: u32 = 9155398;
pub const fnCinematicSprInstr_StartIntroPage5: u32 = 9155406;
pub const fnCinematicFunction_Intro_Page2: i32 = 0x8bb35f;
pub const fnCinematicFunction_Intro_Page3: i32 = 0x8bb370;
pub const fnCinematicFunction_Intro_Page4: i32 = 0x8bb381;
pub const fnCinematicFunction_Intro_Page5: i32 = 0x8bb392;
pub const fnCinematicFunction_Intro_Fadestuff: i32 = 0x8bb3f4;
pub const fnCinematicFunction_Intro_Fadestuff2: i32 = 0x8bb458;
pub const fnCinematicBgPreInstr_SamusBlink: u32 = 9155772;
pub const fnsub_8BB51E: u32 = 9155870;
pub const fnEnableCinematicBgTilemapUpdates__0: u32 = 9155919;
pub const fnCinematicFunction_Intro_Func21: u32 = 9155924;
pub const fnCinematicFunction_Intro_Func22: u32 = 9155973;
pub const fnCinematicFunction_Intro_ThenWaitInputSetupBabyMetroid: u32 = 9156024;
pub const fnCinematicFunction_Intro_Func23: u32 = 9156035;
pub const fnCinematicFunction_Intro_Func24: u32 = 9156078;
pub const fnCinematicFunction_Intro_Func25: u32 = 9156123;
pub const fnCinematicFunction_Intro_Func26: u32 = 9156134;
pub const fnCinematicFunction_Intro_Func27: u32 = 9156183;
pub const fnCinematicFunction_Intro_Func28: u32 = 9156234;
pub const fnCinematicFunction_Intro_Func29: u32 = 9156245;
pub const fnCinematicFunction_Intro_Func30: u32 = 9156288;
pub const fnCinematicFunction_Intro_Func31: u32 = 9156333;
pub const fnCinematicFunction_Intro_Func32: u32 = 9156344;
pub const fnEnableCinematicBgTilemapUpdates__: u32 = 9156363;
pub const fnCinematicFunction_Intro_Func34: i32 = 0x8bb72f;
pub const fnCinematicFunction_Intro_Func35: u32 = 9156467;
pub const fnCinematicFunction_Intro_Func36: u32 = 9156486;
pub const fnCinematicFunction_Intro_Func39: i32 = 0x8bb80f;
pub const fnCinematicSprPreInstr_B82E: u32 = 9156654;
pub const fnCinematicFunction_Intro_Func42: u32 = 9156758;
pub const fnCinematicSprInstr_Func43: u32 = 9156805;
pub const fnCinematicFunction_Intro_Func44: i32 = 0x8bb8d8;
pub const fnCinematicFunction_Intro_Func45: i32 = 0x8bb93b;
pub const fnCinematicFunction_Intro_Func46: u32 = 9157005;
pub const fnCinematicFunction_Intro_Func47: u32 = 9157076;
pub const fnCinematicFunction_Intro_Func48: u32 = 9157135;
pub const fnCinematicSprInstr_SpawnIntroRinkas01: u32 = 9157153;
pub const fnCinematicSprInstr_SpawnIntroRinkas23: u32 = 9157174;
pub const fnCinematicFunction_Intro_Func49: u32 = 9157195;
pub const fnCinematicFunction_Intro_Func50: u32 = 9157214;
pub const fnCinematicFunction_Intro_Func51: i32 = 0x8bba73;
pub const fnCinematicFunction_Intro_Func52: i32 = 0x8bbb0d;
pub const fnCinematicFunction_Intro_Func53: i32 = 0x8bbb24;
pub const fnCinematicFunction_Intro_Func54: i32 = 0x8bbca0;
pub const fnCinematicFunction_Intro_Func55: i32 = 0x8bbde4;
pub const fnCinematicFunction_Intro_Func56: i32 = 0x8bbdf9;
pub const fnCinematicFunction_Intro_Func57: u32 = 9158270;
pub const fnCinematicFunction_Intro_Func58: u32 = 9158325;
pub const fnCinematicFunction_Intro_Func59: u32 = 9158434;
pub const fnCinematicFunction_Intro_Func60: u32 = 9158453;
pub const fnCinematicFunction_Intro_Func61: u32 = 9158476;
pub const fnCinematicFunction_Intro_Func62: u32 = 9158495;
pub const fnCinematicFunction_Intro_Func63: u32 = 9158518;
pub const fnCinematicFunction_Intro_Func64: u32 = 9158537;
pub const fnCinematicFunction_Intro_Func65: u32 = 9158560;
pub const fnCinematicFunction_Intro_Func66: i32 = 0x8bbfc6;
pub const fnCinematicSprPreInstr_nullsub_300: i32 = 0x8bbfd9;
pub const fnCinematicFunction_Intro_Func67: i32 = 0x8bbfda;
pub const fnCinematicFunction_Intro_Func68: u32 = 9158787;
pub const fnCinematicCommonInstr_Func69: u32 = 9158806;
pub const fnCinematicCommonInstr_Func70: u32 = 9158818;
pub const fnCinematicFunction_Intro_Func71: u32 = 9158834;
pub const fnCinematicFunction_Intro_Func72: i32 = 0x8bc0c5;
pub const fnCinematicFunction_Intro_Func73: i32 = 0x8bc100;
pub const fnCinematicFunctionBlackoutFromCeres: u32 = 9158939;
pub const fnCinematicFunction_Intro_Func74: i32 = 0x8bc2e4;
pub const fnCinematicFunction_Intro_Func75: i32 = 0x8bc2f1;
pub const fnCinematicFunction_Intro_Func76: i32 = 0x8bc345;
pub const fnCinematicSprInstr_SpawnCeresExplosions1: u32 = 9159684;
pub const fnCinematicFunction_Intro_Func78: u32 = 9159732;
pub const fnCinematicSprPreInstr_C489: u32 = 9159817;
pub const fnCinematicFunction_Intro_Func80: u32 = 9159865;
pub const fnCinematicSprInstr_SpawnCeresExplosions3: u32 = 9159948;
pub const fnCinematicFunction_Intro_Func81: u32 = 9159987;
pub const fnCinematicFunction_Intro_Func82: u32 = 9160066;
pub const fnCinematicFunction_Intro_Func83: u32 = 9160105;
pub const fnCinematicFunction_Intro_Func77: i32 = 0x8bc5ca;
pub const fnCinematicFunction_Intro_Func84: i32 = 0x8bc610;
pub const fnCinematicFunction_Intro_Func85: i32 = 0x8bc627;
pub const fnCinematicFunction_Intro_Func86: i32 = 0x8bc699;
pub const fnCinematicFunction_Intro_Func87: i32 = 0x8bc79c;
pub const fnCinematicFunction_Intro_Func88: i32 = 0x8bc7ca;
pub const fnCinematicFunction_Intro_Func89: u32 = 9160763;
pub const fnCinematicFunction_Intro_Func90: u32 = 9160782;
pub const fnCinematicFunction_Intro_Func91: i32 = 0x8bc85d;
pub const fnCinematicFunction_Intro_Func92: u32 = 9160855;
pub const fnCinematicFunction_Intro_Func93: u32 = 9160874;
pub const fnCinematicFunction_Intro_Func94: i32 = 0x8bc8b9;
pub const fnCinematicFunction_Intro_Func96: u32 = 9160953;
pub const fnCinematicFunction_Intro_Func97: i32 = 0x8bc908;
pub const fnCinematicFunction_Intro_Func98: u32 = 9161026;
pub const fnCinematicFunction_Intro_Func99: u32 = 9161046;
pub const fnCinematicFunction_Intro_Func100: u32 = 9161066;
pub const fnCinematicFunction_Intro_Func101: u32 = 9161086;
pub const fnCinematicFunction_Intro_Func102: u32 = 9161106;
pub const fnCinematicSprInstr_C9A5: u32 = 9161125;
pub const fnCinematicSprInstr_C9AF: u32 = 9161135;
pub const fnCinematicSprInstr_C9BD: u32 = 9161149;
pub const fnCinematicSprInstr_C9C7: u32 = 9161159;
pub const fnCinematicFunction_Intro_Func104: u32 = 9161190;
pub const fnCinematicFunction_Intro_Func105: i32 = 0x8bc9f9;
pub const fnCinematicFunction_Intro_Func106: i32 = 0x8bca36;
pub const fnCinematicFunction_Intro_Func107: i32 = 0x8bca85;
pub const fnCinematicFunction_Intro_Func108: i32 = 0x8bcad0;
pub const fnnullsub_124: i32 = 0x8bcade;
pub const fnCinematicFunction_Intro_Func95: i32 = 0x8bcadf;
pub const fnCinematicFunctionEscapeFromCebes: u32 = 9163904;
pub const fnCinematicFunction_Intro_Func109: i32 = 0x8bd6d7;
pub const fnCinematicFunction_Intro_Func110: i32 = 0x8bd6f2;
pub const fnCinematicFunction_Intro_Func111: i32 = 0x8bd701;
pub const fnCinematicFunction_Intro_Func112: i32 = 0x8bd731;
pub const fnCinematicFunction_Intro_Func113: i32 = 0x8bd7f8;
pub const fnCinematicFunction_Intro_Func114: i32 = 0x8bd807;
pub const fnCinematicFunction_Intro_Func115: i32 = 0x8bd837;
pub const fnCinematicFunction_Intro_Func116: i32 = 0x8bda02;
pub const fnCinematicFunction_Intro_Func117: i32 = 0x8bda1a;
pub const fnCinematicFunction_Intro_Func118: i32 = 0x8bdad3;
pub const fnnullsub_125: i32 = 0x8bdb9d;
pub const fnCinematicFunction_Intro_Func119: i32 = 0x8bdb9e;
pub const fnCinematicFunction_Intro_Func120: i32 = 0x8bdbc4;
pub const fnCinematicFunction_Intro_Func121: i32 = 0x8bdca5;
pub const fnCinematicFunction_Intro_Func123: i32 = 0x8bdd42;
pub const fnCinematicFunction_Intro_Func124: i32 = 0x8bddcd;
pub const fnnullsub_126: i32 = 0x8bde63;
pub const fnCinematicFunction_Intro_Func125: i32 = 0x8bde64;
pub const fnCinematicFunction_Intro_Func126: i32 = 0x8bde80;
pub const fnCinematicFunction_Intro_Func129: i32 = 0x8be0f4;
pub const fnCinematicFunction_Intro_Func130: i32 = 0x8be110;
pub const fnCinematicFunction_Intro_Func131: i32 = 0x8be158;
pub const fnCinematicFunction_Intro_Func132: i32 = 0x8be190;
pub const fnCinematicFunction_Intro_Func134: i32 = 0x8be1d2;
pub const fnCinematicFunction_Intro_Func136: i32 = 0x8be265;
pub const fnCinematicFunction_Intro_Func135: i32 = 0x8be293;
pub const fnCinematicFunction_Intro_Func137: i32 = 0x8be2dd;
pub const fnCinematicFunction_Intro_Func138: i32 = 0x8be314;
pub const fnCinematicFunction_Intro_Func145: i32 = 0x8be342;
pub const fnCinematicFunction_Intro_Func139: i32 = 0x8be3ae;
pub const fnCinematicFunction_Intro_Func141: i32 = 0x8be48a;
pub const fnCinematicFunction_Intro_Func143: i32 = 0x8be504;
pub const fnCinematicFunction_Intro_Func144: i32 = 0x8be58a;
pub const fnCalcItemPercentageCount: u32 = 9168423;
pub const fnCinematicFunction_Intro_Func146: u32 = 9168745;
pub const fnCinematicFunction_Intro_Func147: u32 = 9168768;
pub const fnCinematicFunction_Intro_Func148: i32 = 0x8be797;
pub const fnnullsub_127: i32 = 0x8be7ba;
pub const fnCinematicFunction_Intro_Func150: u32 = 9170841;
pub const fnCinematicFunction_Intro_Func151: u32 = 9170866;
pub const fnCinematicFunction_Intro_Func152: u32 = 9170922;
pub const fnCinematicFunction_Intro_Func153: u32 = 9170941;
pub const fnCinematicFunction_Intro_Func154: u32 = 9170949;
pub const fnCinematicFunction_Intro_Func155: u32 = 9170968;
pub const fnCinematicFunction_Intro_Func156: u32 = 9170987;
pub const fnCinematicFunction_Intro_Func157: u32 = 9171006;
pub const fnCinematicFunction_Intro_Func159: u32 = 9171038;
pub const fnCinematicFunction_Intro_Func161: u32 = 9171053;
pub const fnCinematicFunction_Intro_Func162: u32 = 9171068;
pub const fnCinematicFunction_Intro_Func163: u32 = 9171077;
pub const fnCinematicFunction_Intro_Func164: u32 = 9171092;
pub const fnCinematicFunction_Intro_Func165: u32 = 9171122;
pub const fnCinematicFunction_Intro_Func166: u32 = 9171169;
pub const fnCinematicFunction_Intro_Func167: u32 = 9171177;
pub const fnCinematicFunction_Intro_Func168: u32 = 9171185;
pub const fnCinematicFunction_Intro_Func169: u32 = 9171193;
pub const fnCinematicFunction_Intro_Func170: u32 = 9171230;
pub const fnCinematicFunction_Intro_Func171: u32 = 9171267;
pub const fnCinematicFunction_Intro_Func172: u32 = 9171286;
pub const fnCinematicFunction_Intro_Func173: u32 = 9171305;
pub const fnCinematicFunction_Intro_Func174: u32 = 9171324;
pub const fnCinematicFunction_Intro_Func175: u32 = 9171343;
pub const fnCinematicFunction_Intro_Func176: u32 = 9171368;
pub const fnCinematicFunction_Intro_Func177: u32 = 9171393;
pub const fnCinematicFunction_Intro_Func178: u32 = 9171412;
pub const fnCinematicFunction_Intro_Func179: u32 = 9171431;
pub const fnCinematicFunction_Intro_Func180: u32 = 9171495;
pub const fnCinematicSprInstr_Func181: u32 = 9171550;
pub const fnCinematicSprInstr_Func182: u32 = 9171588;
pub const fnCinematicSprInstr_Func183: u32 = 9171605;
pub const fnCinematicFunction_Intro_Func184: u32 = 9171621;
pub const fnCinematicSprInstr_Func185: u32 = 9171639;
pub const fnCinematicSprInstr_Func186: u32 = 9171706;
pub const fnCinematicSprInstr_Func187: u32 = 9171755;
pub const fnCinematicSprPreInstr_F35A: u32 = 9171802;
pub const fnCinematicFunction_Intro_Func189: i32 = 0x8bf375;
pub const fnCinematicFunction_Intro_Func189b: u32 = 9171867;
pub const fnCinematicSprInstr_Func190: u32 = 9171888;
pub const fnCinematicFunction_Intro_Func191: u32 = 9171897;
pub const fnCinematicSprInstr_Func192: u32 = 9171918;
pub const fnCinematicSprInstr_Func193: u32 = 9171995;
pub const fnCinematicSprInstr_Func194: u32 = 9172004;
pub const fnCinematicSprInstr_Func195: u32 = 9172013;
pub const fnCinematicSprInstr_Func196: u32 = 9172022;
pub const fnCinematicSprInstr_Func197: u32 = 9172031;
pub const fnCinematicSprInstr_Func198: u32 = 9172040;
pub const fnCinematicFunction_Intro_Func199: u32 = 9172053;
pub const fnCinematicFunction_Intro_Func200: i32 = 0x8bf464;
pub const fnCinematicFunction_Intro_Func201: u32 = 9172088;
pub const fnCinematicFunction_Intro_Func202: i32 = 0x8bf487;
pub const fnCinematicFunction_Intro_Func203: u32 = 9172123;
pub const fnCinematicFunction_Intro_Func204: i32 = 0x8bf4aa;
pub const fnCinematicFunction_Intro_Func205: u32 = 9172177;
pub const fnCinematicFunction_Intro_Func206: i32 = 0x8bf4e0;
pub const fnCinematicFunction_Intro_Func207: u32 = 9172231;
pub const fnnullsub_128: u32 = 9172252;
pub const fnCinematicSprInstr_Func208: u32 = 9172253;
pub const fnCinematicSprPreInstr_F528: u32 = 9172264;
pub const fnCinematicSprInstr_Func211: u32 = 9172308;
pub const fnCinematicSprPreInstr_F57F: u32 = 9172351;
pub const fnCinematicSprInstr_Func213: u32 = 9172375;
pub const fnCinematicSprInstr_Func214: u32 = 9172410;
pub const fnCinematicFunction_Intro_Func215: i32 = 0x8bf5dd;
pub const fnCinematicSprInstr_Func217: u32 = 9172484;
pub const fnCinematicSprInstr_Func218: u32 = 9172561;
pub const fnCinematicFunction_Intro_Func219: u32 = 9172734;
pub const fnSamus_FrameHandlerAlfa_Func12: i32 = 0x90e6c9;
pub const fnSamus_Func15: i32 = 0x90e833;
pub const fnSamusDrawHandler_Default: i32 = 0x90eb52;
pub const fnlocret_8B9585: i32 = 0x8b9585;
// Cinematic graphics loads keep source, destination, then transfer size together.
// RAM destinations are byte offsets; VRAM destinations are word addresses.
enum CinematicAsset {
    Decompress(u32, isize),
    VramWords(u32, u16, u16),
    VramLowBytes(u32, u16, u16),
    VramHighBytes(u32, u16, u16),
}

impl CinematicAsset {
    unsafe fn load(self, game: &mut Game) {
        let (transfer, destination, vmain) = match self {
            Self::Decompress(source, destination) => {
                DecompressToMem(
                    source,
                    (&raw mut game.g_ram as *mut u8).offset(destination),
                    game,
                );
                return;
            }
            Self::VramWords(source, destination, size) => {
                (StartDmaCopy::vram_words(1, source, size), destination, 0x80)
            }
            Self::VramLowBytes(source, destination, size) => (
                StartDmaCopy::vram_low_bytes(1, source, size),
                destination,
                0,
            ),
            Self::VramHighBytes(source, destination, size) => (
                StartDmaCopy::vram_high_bytes(1, source, size),
                destination,
                0x80,
            ),
        };
        WriteReg(VMADDL as u16, destination as u8, game);
        WriteReg(VMADDH as u16, (destination >> 8) as u8, game);
        WriteReg(VMAIN as u16, vmain, game);
        SetupDmaTransfer(&raw const transfer as *const c_void, game);
        WriteReg(MDMAEN as u16, 2u8, game);
    }
}

pub unsafe fn SetupPpuForTitleSequence(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0x80, 0x5f, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0, 0x59, false, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0, 0x5a, false, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5c, false, game);
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5d, false, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0, 0x5e, false, game);
    WriteReg(BG34NBA as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x10, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
    crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
    WriteReg(COLDATA as u16, 0xe0u8, game);
    crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    memset(
        (&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut c_void,
        0i32,
        0x7fe as size_t,
    );
    set_ram_word!(game.oam_next_ptr = 0u16);
    set_ram_word!(game.menu_option_index = 0u16);
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 0u16);
    set_ram_word!(game.pausemenu_start_lr_pressed_highlight_timer = 0u16);
    set_ram_word!(game.pausemenu_lr_animation_timer = 0u16);
    set_ram_word!(game.pausemenu_item_selector_animation_timer = 0u16);
    set_ram_word!(game.pausemenu_reserve_tank_animation_timer = 0u16);
    set_ram_word!(game.UNUSED_word_7E0731 = 0u16);
    set_ram_word!(game.UNUSED_word_7E073D = 0u16);
    set_ram_word!(game.pausemenu_lr_animation_frame = 0u16);
    set_ram_word!(game.pausemenu_item_selector_animation_frame = 0u16);
    set_ram_word!(game.pausemenu_reserve_tank_animation_frame = 0u16);
    set_ram_word!(game.UNUSED_word_7E0745 = 0u16);
    set_ram_word!(game.pausemenu_shoulder_button_highlight = 0u16);
    set_ram_word!(game.pausemenu_button_label_mode = 0u16);
    set_ram_word!(game.pausemenu_equipment_category_item = 0u16);
    set_ram_word!(game.pausemenu_reserve_tank_delay_ctr = 0u16);
    set_ram_word!(game.UNUSED_word_7E0759 = 0u16);
    set_ram_word!(game.samus_reserve_health = 0u16);
    set_ram_word!(game.samus_reserve_missiles = 0u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
}
pub unsafe fn SetupPpu_Intro(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 9, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0x50, 0x58, true, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0x48, 0x59, true, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0x4c, 0x5a, true, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x5c) = 0u8;
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5d, false, game);
    WriteReg(BG12NBA as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(BG34NBA as u16, 4, 0x5e, true, game);
    crate::rtl::write_register_and_mirror(TM as u16, 4, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
    crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
    WriteReg(COLDATA as u16, 0xe0u8, game);
    crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x57) = 0u8;
    memset7E(
        (&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut u16,
        0x2fu16,
        0x7feu16,
    );
    set_ram_word!(game.oam_next_ptr = 0u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
    set_ram_word!(game.fx_layer_blending_config_a = 0u16);
    set_ram_word!(game.reg_BG2HOFS = 0u16);
    set_ram_word!(game.reg_BG2VOFS = 0u16);
    set_ram_word!(game.reg_BG3HOFS = 0u16);
    set_ram_word!(game.reg_BG3VOFS = 0u16);
}
pub unsafe fn SetupPpu_3_Mode7(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0x80, 0x5f, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0, 0x59, false, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0, 0x5a, false, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5c, false, game);
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5d, false, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0, 0x5e, false, game);
    WriteReg(BG34NBA as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x11, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
    crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
    WriteReg(COLDATA as u16, 0xe0u8, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x57) = 0u8;
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
    set_ram_word!(game.fx_layer_blending_config_a = 0u16);
}
pub unsafe fn SetupPpu_4_Mode1(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 1, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0x5c, 0x58, true, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 0u8;
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0, 0x5a, false, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5c, false, game);
    WriteReg(BG4SC as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 6, 0x5d, true, game);
    crate::rtl::write_register_and_mirror(BG34NBA as u16, 0, 0x5e, true, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x11, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    set_ram_word!(game.reg_BG1HOFS = 0u16);
    set_ram_word!(game.reg_BG1VOFS = 0u16);
}
pub unsafe fn SetupPpu_5_Mode7(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    WriteReg(OBSEL as u16, 0xa3u8, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x52) = -93i32 as u8;
    crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0, 0x59, false, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0, 0x5a, false, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5c, false, game);
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5d, false, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0, 0x5e, false, game);
    WriteReg(BG34NBA as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x11, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
    crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    WriteReg(COLDATA as u16, 0xe0u8, game);
    crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    set_ram_word!(game.oam_next_ptr = 0u16);
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 0u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
}
pub unsafe fn SetupPpu_6_Mode1(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 2, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 1, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0x70, 0x58, true, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0x78, 0x59, true, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x5a) = 0u8;
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5c, false, game);
    WriteReg(BG4SC as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0x44, 0x5d, true, game);
    crate::rtl::write_register_and_mirror(BG34NBA as u16, 0, 0x5e, true, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x10, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
    crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
    WriteReg(COLDATA as u16, 0xe0u8, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x57) = 0u8;
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
    set_ram_word!(game.fx_layer_blending_config_a = 0u16);
    set_ram_word!(game.reg_BG2HOFS = 0u16);
    set_ram_word!(game.reg_BG2VOFS = 0u16);
}
pub unsafe fn SetupPpu_7_Mode1(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x80, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 0, 0x52, false, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 1, 0x55, false, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0x48, 0x58, true, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0x4c, 0x59, true, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0x24, 0x5a, true, game);
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5c, true, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0x54, 0x5d, true, game);
    crate::rtl::write_register_and_mirror(BG34NBA as u16, 2, 0x5e, true, game);
    crate::rtl::write_register_and_mirror(TM as u16, 1, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    set_ram_word!(game.oam_next_ptr = 0u16);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7X = 128u16);
    WriteRegWord(M7Y as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7Y = 128u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 1u16);
    set_ram_word!(game.reg_BG1HOFS = 0u16);
    set_ram_word!(game.reg_BG1VOFS = 0u16);
    set_ram_word!(game.reg_BG2HOFS = 0u16);
    set_ram_word!(game.reg_BG2VOFS = 0u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
}
pub unsafe fn HandleMode7TransformationNoRotation(game: &mut Game) {
    set_ram_word!(game.reg_M7A = ram_word!(game, cinematic_var6));
    set_ram_word!(game.reg_M7D = ram_word!(game, cinematic_var6));
    set_ram_word!(game.reg_M7B = 0u16);
    set_ram_word!(game.reg_M7C = 0u16);
    set_ram_word!(game.reg_BG1HOFS = ram_word!(game, cinematic_var8));
    set_ram_word!(game.reg_BG1VOFS = ram_word!(game, cinematic_var10));
}
unsafe fn Smult16x16(r38: u16, r40: u16) -> u32 {
    let t: u32 = ((r38 as i16) as u32).wrapping_mul((r40 as i16) as u32);
    t
}
pub unsafe fn HandleMode7TransformationMatrix(game: &mut Game) {
    let ref mut fresh152 = *((&raw mut game.g_ram as *mut u8).offset(0x78) as *mut u16);
    *fresh152 = (Smult16x16(
        kSinCosTable8bit_Sext[((((ram_word!(game, cinematic_var5)) as u8).wrapping_add(64u8))
            as i32
            + 64i32) as usize] as u16,
        ram_word!(game, cinematic_var6),
    ) >> 8i32) as u16;
    set_ram_word!(game.reg_M7D = *fresh152);
    let v1: u16 = (Smult16x16(
        kSinCosTable8bit_Sext[(ram_word!(game, cinematic_var5) as u8 as i32 + 64i32) as usize]
            as u16,
        ram_word!(game, cinematic_var6),
    ) >> 8i32) as u16;
    set_ram_word!(game.reg_M7B = v1);
    set_ram_word!(game.reg_M7C = -(v1 as i32) as u16);
    set_ram_word!(game.reg_BG1HOFS = ram_word!(game, cinematic_var8));
    set_ram_word!(game.reg_BG1VOFS = ram_word!(game, cinematic_var10));
}
pub unsafe fn SetPpuBackdropSomeColor(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = -127i32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 56u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 88u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
}
pub unsafe fn SetPpuBackdropBlack(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
}
pub unsafe fn UpdateCinematicBgTilemap(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut u8).offset(0x55) != 7u8 {
        let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 1920u16;
        v0 = (v0).wrapping_add(2u16);
        (*gVramWriteEntry(v0 as i32, game)).size = 12288u16;
        v0 = (v0).wrapping_add(2u16);
        let fresh157 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh157 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) = 126u8;
        (*gVramWriteEntry(v0 as i32, game)).size = ram_word!(game, cinematicbg_var3);
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
    }
}
pub unsafe fn UpdateSamusEyesTilemap(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut u8).offset(0x55) != 7u8 {
        let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 128u16;
        v0 = (v0).wrapping_add(2u16);
        (*gVramWriteEntry(v0 as i32, game)).size =
            (((&raw mut game.g_ram as *mut u8).offset(0x3800) as *mut Ram3800 as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8) as c_long
                + 832 as c_long) as u16;
        v0 = (v0).wrapping_add(2u16);
        let fresh156 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh156 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) = 126u8;
        (*gVramWriteEntry(v0 as i32, game)).size = 18848u16;
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
    }
}
pub unsafe fn CinematicUpdateSomeBg(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut u8).offset(0x55) != 7u8 {
        let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 2048u16;
        v0 = (v0).wrapping_add(2u16);
        (*gVramWriteEntry(v0 as i32, game)).size =
            ((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000 as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
        v0 = (v0).wrapping_add(2u16);
        let fresh4 = v0;
        v0 = v0.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh4 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) = 126u8;
        (*gVramWriteEntry(v0 as i32, game)).size = ram_word!(game, cinematicbg_var3);
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
    }
}
pub unsafe fn CallProcessCinematicBgObject(ea: u32, k: u16, j: u16, r18: u16, game: &mut Game) {
    match ea {
        9144393 => ProcessCinematicBgObject_Nothing(k, j, r18),
        9144397 => ProcessCinematicBgObject_DrawChar(k, j, r18, game),
        9144503 => ProcessCinematicBgObject_DrawToTextTilemap(k, j, r18, game),
        9144573 => ProcessCinematicBgObject_DrawToBgTilemap(k, j, r18, game),
        9144683 => ProcessCinematicBgObject_Unk1(k, j, r18, game),
        9144783 => ProcessCinematicBgObject_Unk2(k, j, r18, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn ProcessCinematicBgObject(v0: u16, r18: u16, game: &mut Game) {
    let j: u16 = ram_word!(game, cinematicbg_arr1[(v0 as i32 >> 1i32)]);
    CallProcessCinematicBgObject(
        ((*(RomPtr_8C(j, game) as *mut u16)) as u32) | (0x8b0000u32),
        v0,
        j,
        r18,
        game,
    );
}
pub unsafe fn ProcessCinematicBgObject_DrawChar(k: u16, j: u16, r18: u16, game: &mut Game) {
    let mut v3: i16 = 0;
    set_ram_word!(game.cinematicbg_var1 = (ram_word!(game, cinematicbg_var1) == 0u16) as u16);
    SpawnTextGlowObject(j, r18, game);
    let v2: *const u8 = RomPtr_8C(
        ram_word!(game, cinematicbg_instr_ptr[(k as i32 >> 1i32)]),
        game,
    );
    if *(v2 as *mut i16).offset(3) < 0i16 {
        set_ram_word!(game.cinematicbg_arr7[15] = 8u16);
        v3 = (8i32 * (*v2.offset(3) as i32 + 2i32)) as i16;
    } else {
        set_ram_word!(game.cinematicbg_arr7[15] = (8u16).wrapping_mul((*v2.offset(8)) as u16));
        v3 = (8i16).wrapping_mul((*v2.offset(9)) as i16);
    }
    set_ram_word!(game.cinematicbg_arr8[15] = ((v3) as u16).wrapping_sub(8u16));
    if *(v2.offset(4) as *mut u16) != 0xd67du16 && ram_word!(game, cinematicbg_var1) != 0u16 {
        QueueSfx3_Max6(0xdu16, game);
    }
    ProcessCinematicBgObject_DrawToTextTilemap(0x1eu16, j, r18, game);
}
pub unsafe fn ProcessCinematicBgObject_DrawToTextTilemap(
    k: u16,
    mut j: u16,
    r18: u16,
    game: &mut Game,
) {
    let jorg: u16 = j;
    let mut offs: u16 = CinematicGetTilemapOffsetForTile(r18);
    let mut r22: u16 = offs;
    let v3: *const u8 = RomPtr_8C(j, game);
    let mut m: u16 = *v3.offset(2) as u16;
    let morg: u16 = m;
    let mut n: u16 = *v3.offset(3) as u16;
    loop {
        loop {
            *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut u16 as *mut u8)
                .offset(offs as i32 as isize) as *mut u16) =
                *(RomPtr_8C(j, game) as *mut u16).offset(2);
            j = (j).wrapping_add(2u16);
            offs = (offs).wrapping_add(2u16);
            m = m.wrapping_sub(1);
            if (m == 0) {
                break;
            }
        }
        m = morg;
        n = n.wrapping_sub(1);
        if n == 0 {
            break;
        }
        r22 = (r22).wrapping_add(64u16);
        offs = r22;
    }
}
pub fn ProcessCinematicBgObject_Nothing(k: u16, j: u16, r18: u16) {}
pub unsafe fn ProcessCinematicBgObject_DrawToBgTilemap(
    k: u16,
    mut j: u16,
    r18: u16,
    game: &mut Game,
) {
    let jorg: u16 = j;
    let mut offs: u16 = CinematicGetTilemapOffsetForTile(r18);
    let mut r22: u16 = offs;
    let v3: *const u8 = RomPtr_8C(j, game);
    let mut m: u16 = *v3.offset(2) as u16;
    let morg: u16 = m;
    let mut n: i32 = *v3.offset(3) as i32;
    loop {
        loop {
            *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3800) as *mut Ram3800))
                .cinematic_bg_tilemap as *mut u16 as *mut u8)
                .offset(offs as i32 as isize) as *mut u16) =
                *(RomPtr_8C(j, game) as *mut u16).offset(2);
            j = (j).wrapping_add(2u16);
            offs = (offs).wrapping_add(2u16);
            m = m.wrapping_sub(1);
            if (m == 0) {
                break;
            }
        }
        m = morg;
        n -= 1;
        if n == 0 {
            break;
        }
        r22 = (r22).wrapping_add(64u16);
        offs = r22;
    }
}
pub unsafe fn CinematicGetTilemapOffsetForTile(mut r18: u16) -> u16 {
    (2i32 * r18 as u8 as i32 + *(&raw mut r18 as *mut u8).offset(1isize) as i32 * 0x40i32) as u16
}
pub unsafe fn ProcessCinematicBgObject_Unk1(k: u16, j: u16, r18: u16, game: &mut Game) {
    let mut r22: u16 = Mult0x80Add(r18);
    let v2: *const u8 = RomPtr_8C(j, game);
    let r18x: u16 = *v2.offset(2) as u16;
    let mut n: u16 = *v2.offset(3) as u16;
    let mut v3: u16 = (j).wrapping_add(4u16);
    let mut v4: u16 = ram_word!(game, mode7_vram_write_queue_tail);
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(v4 as i32 as isize) as *mut u16) = 128u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut u8)
            .offset(v4 as i32 as isize) as *mut u16) = v3;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset(v4 as isize) as *mut u16) = 139u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v4 as i32 + 1i32) as isize) as *mut u16) = r18x;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v4 as i32 + 3i32) as isize) as *mut u16) = r22;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut u8)
            .offset(v4 as i32 as isize) as *mut u16) = 0u16;
        v4 = (v4).wrapping_add(9u16);
        v3 = (v3).wrapping_add(r18x);
        r22 = (r22).wrapping_add(128u16);
        n = n.wrapping_sub(1);
        if (n == 0) {
            break;
        }
    }
    set_ram_word!(game.mode7_vram_write_queue_tail = v4);
}
pub unsafe fn ProcessCinematicBgObject_Unk2(k: u16, j: u16, r18: u16, game: &mut Game) {
    let mut r22: u16 = Mult0x80Add(r18);
    let v2: *const u8 = RomPtr_8C(j, game);
    let mut n: i32 = *v2.offset(2) as i32;
    let r20: u16 = *v2.offset(3) as u16;
    let mut v3: u16 = (j).wrapping_add(4u16);
    let mut v4: u16 = ram_word!(game, mode7_vram_write_queue_tail);
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(v4 as i32 as isize) as *mut u16) = 128u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut u8)
            .offset(v4 as i32 as isize) as *mut u16) = v3;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset(v4 as isize) as *mut u16) = 139u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v4 as i32 + 1i32) as isize) as *mut u16) = r20;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v4 as i32 + 3i32) as isize) as *mut u16) = r22;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut u8)
            .offset(v4 as i32 as isize) as *mut u16) = 2u16;
        v4 = (v4).wrapping_add(9u16);
        v3 = (v3).wrapping_add(r20);
        r22 = r22.wrapping_add(1);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    set_ram_word!(game.mode7_vram_write_queue_tail = v4);
}
pub unsafe fn Mult0x80Add(mut r18: u16) -> u16 {
    (r18 as u8 as i32 + 0x80i32 * *(&raw mut r18 as *mut u8).offset(1isize) as i32) as u16
}
pub unsafe fn Samus_CalcPos_Mode7(game: &mut Game) {
    let r34: u16 = (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, reg_M7X));
    let r36: u16 = (ram_word!(game, reg_M7Y)).wrapping_sub(ram_word!(game, samus_y_pos));
    let mut r26: u16 = (Smult16x16(
        (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, reg_M7X)),
        ram_word!(game, reg_M7A),
    ) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7B), r36) >> 8i32) as u16;
    set_ram_word!(game.samus_x_pos = (r26).wrapping_add(ram_word!(game, reg_M7X)));
    r26 = (Smult16x16(ram_word!(game, reg_M7C), r34) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7A), r36) >> 8i32) as u16;
    set_ram_word!(game.samus_y_pos = (ram_word!(game, reg_M7Y)).wrapping_sub(r26));
}
pub unsafe fn CalcExplosion_Mode7(k: u16, game: &mut Game) -> Point16U {
    let v1: i32 = k as i32 >> 1i32;
    let r34: u16 = (ram_word!(game, projectile_x_pos[v1])).wrapping_sub(ram_word!(game, reg_M7X));
    let r36: u16 = (ram_word!(game, reg_M7Y)).wrapping_sub(ram_word!(game, projectile_y_pos[v1]));
    let mut r26: u16 = (Smult16x16(r34, ram_word!(game, reg_M7A)) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7B), r36) >> 8i32) as u16;
    let r20: u16 = (r26 as i32 + ram_word!(game, reg_M7X) as i32
        - ram_word!(game, layer1_x_pos) as i32) as u16;
    r26 = (Smult16x16(ram_word!(game, reg_M7C), r34) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7A), r36) >> 8i32) as u16;
    let r18: u16 = (ram_word!(game, reg_M7Y) as i32
        - r26 as i32
        - ram_word!(game, layer1_y_pos) as i32) as u16;
    Point16U { x: r20, y: r18 }
}
pub unsafe fn CalcCeresSteamPos_Mode7(mut pt: Point16U, game: &mut Game) -> Point16U {
    let r34: u16 = (pt.x).wrapping_sub(ram_word!(game, reg_M7X));
    let r36: u16 = (ram_word!(game, reg_M7Y)).wrapping_sub(pt.y);
    let mut r26: u16 = (Smult16x16(
        (pt.x).wrapping_sub(ram_word!(game, reg_M7X)),
        ram_word!(game, reg_M7A),
    ) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7B), r36) >> 8i32) as u16;
    pt.x = (r26).wrapping_add(ram_word!(game, reg_M7X));
    r26 = (Smult16x16(ram_word!(game, reg_M7C), r34) >> 8i32) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(ram_word!(game, reg_M7A), r36) >> 8i32) as u16;
    pt.y = (ram_word!(game, reg_M7Y)).wrapping_sub(r26);
    pt
}
pub unsafe fn CopyPalettesToFadingPalettes(game: &mut Game) {
    let mut v0: i32 = 256i32;
    let mut v1: u16 = 0u16;
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x2000) as *mut u16)
            .offset(((v1 as i32 >> 1i32) + 256i32) as isize) =
            ram_word!(game, palette_buffer[(v1 as i32 >> 1i32)]);
        v1 = (v1).wrapping_add(2u16);
        v0 -= 1;
        if (v0 == 0) {
            break;
        }
    }
}
pub unsafe fn DecomposePaletteDataForFading(game: &mut Game) {
    let mut v0: u16 = 0u16;
    let mut v1: i32 = 256i32;
    loop {
        let v2: i32 = v0 as i32 >> 1i32;
        let r18: u16 = ram_word!(game, tilemap_stuff[(v2 + 256i32)]);
        let v3: i32 = r18 as i32 & 0x1fi32;
        set_ram_word!(game.tilemap_stuff[(v2 + 512i32)] = (v3 << 8i32) as u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 1280i32)] = (v3 << 3i32) as u16);
        let v4: i32 = (r18 as i32 & 0x3e0i32) >> 5i32;
        set_ram_word!(game.tilemap_stuff[(v2 + 768i32)] = (v4 << 8i32) as u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 1536i32)] = (v4 << 3i32) as u16);
        let v6: i32 = (r18 as i32 & 0x7c00i32) >> 10i32;
        set_ram_word!(game.tilemap_stuff[(v2 + 1024i32)] = (v6 << 8i32) as u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 1792i32)] = (v6 << 3i32) as u16);
        v0 = (v0).wrapping_add(2u16);
        v1 -= 1;
        if (v1 == 0) {
            break;
        }
    }
}
pub unsafe fn ClearYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    loop {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.tilemap_stuff[(v2 + 256i32)] = 0u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 512i32)] = 0u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 768i32)] = 0u16);
        set_ram_word!(game.tilemap_stuff[(v2 + 1024i32)] = 0u16);
        k = (k).wrapping_add(2u16);
        j = j.wrapping_sub(1);
        if (j == 0) {
            break;
        }
    }
}
pub unsafe fn FadeOutYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    loop {
        let v2: i32 = k as i32 >> 1i32;
        update_ram_word!(
            game.tilemap_stuff[(v2 + 512i32)],
            wrapping_sub(ram_word!(game, tilemap_stuff[(v2 + 1280i32)]))
        );
        update_ram_word!(
            game.tilemap_stuff[(v2 + 768i32)],
            wrapping_sub(ram_word!(game, tilemap_stuff[(v2 + 1536i32)]))
        );
        update_ram_word!(
            game.tilemap_stuff[(v2 + 1024i32)],
            wrapping_sub(ram_word!(game, tilemap_stuff[(v2 + 1792i32)]))
        );
        k = (k).wrapping_add(2u16);
        j = j.wrapping_sub(1);
        if (j == 0) {
            break;
        }
    }
}
pub unsafe fn FadeInYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    loop {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(
            game.tilemap_stuff[(v2 + 512i32)] = (ram_word!(game, tilemap_stuff[(v2 + 1280i32)])
                as i32
                + ram_word!(game, tilemap_stuff[(v2 + 512i32)]) as i32
                & 0x1fffi32) as u16
        );
        set_ram_word!(
            game.tilemap_stuff[(v2 + 768i32)] = (ram_word!(game, tilemap_stuff[(v2 + 1536i32)])
                as i32
                + ram_word!(game, tilemap_stuff[(v2 + 768i32)]) as i32
                & 0x1fffi32) as u16
        );
        set_ram_word!(
            game.tilemap_stuff[(v2 + 1024i32)] = (ram_word!(game, tilemap_stuff[(v2 + 1792i32)])
                as i32
                + ram_word!(game, tilemap_stuff[(v2 + 1024i32)]) as i32
                & 0x1fffi32) as u16
        );
        k = (k).wrapping_add(2u16);
        j = j.wrapping_sub(1);
        if (j == 0) {
            break;
        }
    }
}
pub unsafe fn ComposeFadingPalettes(game: &mut Game) {
    let mut v0: u16 = 0u16;
    let mut v1: i32 = 256i32;
    loop {
        let v2: i32 = v0 as i32 >> 1i32;
        let mut r18: i32 = *(((&raw mut game.g_ram as *mut u8).offset(0x2000) as *mut u16)
            .offset((v2 + 512i32) as isize) as *mut u8)
            .offset(1isize) as i32
            & 0x1fi32;
        r18 |= ram_word!(game, tilemap_stuff[(v2 + 768i32)]) as i32 >> 3i32 & 0x3e0i32;
        let v3: u16 = (r18
            | (ram_word!(game, tilemap_stuff[(v2 + 1024i32)]) as i32) << 2i32 & 0x7c00i32)
            as u16;
        set_ram_word!(game.tilemap_stuff[v2] = v3);
        set_ram_word!(game.palette_buffer[v2] = v3);
        v0 = (v0).wrapping_add(2u16);
        v1 -= 1;
        if (v1 == 0) {
            break;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func20(j: u16, game: &mut Game) {
    let mut tt: *mut u16 = RomPtr_8B(j, game) as *mut u16;
    let mut v2: u16 = *tt.offset(0);
    let mut r18: i32 = *tt.offset(1) as i32;
    let dst: *mut u16 = &raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x4000)
        as *mut Ram4000))
        .intro_japanese_text_tiles as *mut u16;
    loop {
        memcpy(
            dst.offset((v2 as i32 >> 1i32) as isize) as *mut c_void,
            (&raw mut game.g_ram as *mut u8).offset((*tt.offset(2) as i32 + 0x1a000i32) as isize)
                as *mut u16 as *const c_void,
            16 as size_t,
        );
        memcpy(
            dst.offset((v2 as i32 + 768i32 >> 1i32) as isize) as *mut c_void,
            (&raw mut game.g_ram as *mut u8).offset((*tt.offset(3) as i32 + 0x1a000i32) as isize)
                as *mut u16 as *const c_void,
            16 as size_t,
        );
        v2 = (v2).wrapping_add(16u16);
        tt = tt.offset(2);
        r18 -= 1;
        if (r18 == 0) {
            break;
        }
    }
}
pub unsafe fn TransferJapaneseTextTilesToVram(game: &mut Game) {
    let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 1536u16;
    v0 = (v0).wrapping_add(2u16);
    (*gVramWriteEntry(v0 as i32, game)).size = 0x4000u16;
    v0 = (v0).wrapping_add(2u16);
    let fresh21 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh21 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = 126u8;
    (*gVramWriteEntry(v0 as i32, game)).size = 16768u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
}
pub unsafe fn HandleSamusDuringIntro(game: &mut Game) {
    if ram_word!(game, cinematic_var15) != 0 {
        HandleControllerInputForGamePhysics(game);
        HandleSamusMovementAndPause(game);
        if ram_word!(game, samus_invincibility_timer) != 0 {
            update_ram_word!(game.samus_invincibility_timer, wrapping_sub(1));
        }
        if ram_word!(game, samus_knockback_timer) != 0 {
            update_ram_word!(game.samus_knockback_timer, wrapping_sub(1));
        }
    }
}
pub unsafe fn DrawIntroSprites(game: &mut Game) {
    if ram_word!(game, cinematic_var15) != 0 {
        if (ram_word!(game, cinematic_var15) as i32 & 0x8000i32) != 0i32 {
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
pub unsafe fn CalcSinMult(k: u16, r24: u16) -> u16 {
    let r18: u16 = (Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *mut u8)
            .offset(k as i32 as isize),
        r24 as u8,
    ) as i32
        >> 8i32) as u16;
    (r18 as i32
        + *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *mut u8)
            .offset(k as i32 as isize)
            .offset(1) as i32
            * r24 as u8 as i32) as u16
}
static kMoveUnusedSpritesMasks: [u16; 8] = [
    0x5555, 0x5554, 0x5550, 0x5540, 0x5500, 0x5400, 0x5000, 0x4000,
];
pub unsafe fn MoveUnusedSpritesOffScreen(game: &mut Game) {
    if ram_word!(game, oam_next_ptr) >= 0x200u16 {
        return;
    }
    let j: i32 = ram_word!(game, oam_next_ptr) as i32 >> 2i32;
    let mut i: i32 = ram_word!(game, oam_next_ptr) as i32 >> 5i32;
    let fresh11 = i;
    i = i + 1;
    update_ram_word!(game.oam_ext[fresh11], |fresh12| (fresh12)
        | (kMoveUnusedSpritesMasks[(j & 7i32) as usize]));
    while i < 16i32 {
        set_ram_word!(game.oam_ext[i] = 0x5555u16);
        i += 1;
    }
    let mut i_0: i32 = ram_word!(game, oam_next_ptr) as i32 >> 2i32;
    while i_0 < 0x80i32 {
        *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x370) as *mut OamEnt)
            .offset(i_0 as isize))
        .xcoord as *mut u16) = 0x80u16;
        i_0 += 1;
    }
}
pub unsafe fn AdvanceFastScreenFadeOut(game: &mut Game) -> u8 {
    if (*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32) != 0i32 {
        if (((*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32)
            - ram_word!(game, screen_fade_counter) as i32) as i8)
            > 0i8
        {
            *(&raw mut game.g_ram as *mut u8).offset(0x51) =
                ((*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32)
                    - ram_word!(game, screen_fade_counter) as i32) as u8;
            return 0u8;
        }
        *(&raw mut game.g_ram as *mut u8).offset(0x51) = 0x80u8;
    }
    1u8
}
pub unsafe fn AdvanceSlowScreenFadeOut(game: &mut Game) -> u8 {
    let v0: bool = ((ram_word!(game, screen_fade_delay)) as i8).wrapping_sub(1i8) < 0i8;
    *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) =
        ((ram_word!(game, screen_fade_delay)) as u8).wrapping_sub(1u8);
    if (ram_word!(game, screen_fade_delay) as u8) != 0u8 && !v0 {
        return 0u8;
    }
    if (*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32) != 0i32 {
        if (*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32) != 1i32 {
            *(&raw mut game.g_ram as *mut u8).offset(0x51) =
                ((*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 & 0xfi32) - 1i32) as u8;
            *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) =
                ram_word!(game, screen_fade_counter) as u8;
            return 0u8;
        }
        *(&raw mut game.g_ram as *mut u8).offset(0x51) = 0x80u8;
        *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) = 0u8;
        *((&raw mut game.g_ram as *mut u8).offset(0x725) as *mut u16 as *mut u8).offset(0) = 0u8;
    }
    1u8
}
pub unsafe fn AdvanceFastScreenFadeIn(game: &mut Game) -> u8 {
    if (ram_word!(game, screen_fade_counter) as i32
        + *(&raw mut game.g_ram as *mut u8).offset(0x51) as i32
        & 0x1fi32)
        - 15i32
        & 0x80i32
        != 0
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x51) = (ram_word!(game, screen_fade_counter)
            as i32
            + *(&raw mut game.g_ram as *mut u8).offset(0x51) as i32
            & 0x1fi32) as u8;
        0u8
    } else {
        *(&raw mut game.g_ram as *mut u8).offset(0x51) = 15u8;
        1u8
    }
}
pub unsafe fn AdvanceSlowScreenFadeIn(game: &mut Game) -> u8 {
    let v0: bool = ((ram_word!(game, screen_fade_delay)) as i8).wrapping_sub(1i8) < 0i8;
    *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) =
        ((ram_word!(game, screen_fade_delay)) as u8).wrapping_sub(1u8);
    if (ram_word!(game, screen_fade_delay) as u8) != 0u8 && !v0 {
        return 0u8;
    }
    if (*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 + 1i32 & 0x1fi32) - 15i32 & 0x80i32
        != 0
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x51) =
            (*(&raw mut game.g_ram as *mut u8).offset(0x51) as i32 + 1i32 & 0x1fi32) as u8;
        *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) =
            ram_word!(game, screen_fade_counter) as u8;
        return 0u8;
    }
    *(&raw mut game.g_ram as *mut u8).offset(0x51) = 15u8;
    *((&raw mut game.g_ram as *mut u8).offset(0x723) as *mut u16 as *mut u8).offset(0) = 0u8;
    *((&raw mut game.g_ram as *mut u8).offset(0x725) as *mut u16 as *mut u8).offset(0) = 0u8;
    1u8
}
pub unsafe fn InitializeIoDisplayLogo_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = (&raw mut game.g_ram as *mut u8).offset(0x612);
    's_645: {
        let mut current_block_182: u64;
        match *_state_variable_ as i32 {
            0 => {
                crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
                set_ram_word!(game.oam_next_ptr = 0u16);
                ClearOamExt(game);
                ClearUnusedOam(game);
                set_ram_word!(game.nmi_copy_samus_halves = 0u16);
                set_ram_word!(game.nmi_copy_samus_top_half_src = 0u16);
                set_ram_word!(game.nmi_copy_samus_bottom_half_src = 0u16);
                crate::rtl::write_register_and_mirror(NMITIMEN as u16, 1, 0x84, false, game);
                WriteReg(WRIO as u16, 0u8, game);
                WriteReg(WRMPYA as u16, 0u8, game);
                WriteReg(WRMPYB as u16, 0u8, game);
                WriteReg(WRDIVL as u16, 0u8, game);
                WriteReg(WRDIVH as u16, 0u8, game);
                WriteReg(WRDIVB as u16, 0u8, game);
                WriteReg(HTIMEL as u16, 0u8, game);
                WriteReg(HTIMEH as u16, 0u8, game);
                WriteReg(VTIMEL as u16, 0u8, game);
                WriteReg(VTIMEH as u16, 0u8, game);
                WriteReg(MDMAEN as u16, 0u8, game);
                crate::rtl::write_register_and_mirror(HDMAEN as u16, 0, 0x85, false, game);
                crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
                crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
                WriteReg(OAMADDL as u16, 0u8, game);
                *((&raw mut game.g_ram as *mut u8).offset(0x53) as *mut u16 as *mut u8).offset(0) =
                    0u8;
                WriteReg(OAMADDH as u16, 0x80u8, game);
                *((&raw mut game.g_ram as *mut u8).offset(0x53) as *mut u16 as *mut u8)
                    .offset(1isize) = 0x80u8;
                WriteReg(OAMDATA as u16, 0u8, game);
                WriteReg(OAMDATA as u16, 0u8, game);
                crate::rtl::write_register_and_mirror(BGMODE as u16, 1, 0x55, false, game);
                crate::rtl::write_register_and_mirror(MOSAIC as u16, 0, 0x57, false, game);
                crate::rtl::write_register_and_mirror(BG1SC as u16, 0, 0x58, false, game);
                crate::rtl::write_register_and_mirror(BG2SC as u16, 0, 0x59, false, game);
                crate::rtl::write_register_and_mirror(BG3SC as u16, 0, 0x5a, false, game);
                crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5c, false, game);
                crate::rtl::write_register_and_mirror(BG12NBA as u16, 0, 0x5d, false, game);
                crate::rtl::write_register_and_mirror(BG34NBA as u16, 0, 0x5e, false, game);
                WriteReg(BG1HOFS as u16, 0u8, game);
                WriteReg(BG1HOFS as u16, 0u8, game);
                WriteReg(BG1VOFS as u16, 0u8, game);
                WriteReg(BG1VOFS as u16, 0u8, game);
                WriteReg(BG2HOFS as u16, 0u8, game);
                WriteReg(BG2HOFS as u16, 0u8, game);
                WriteReg(BG2VOFS as u16, 0u8, game);
                WriteReg(BG2VOFS as u16, 0u8, game);
                WriteReg(BG3HOFS as u16, 0u8, game);
                WriteReg(BG3HOFS as u16, 0u8, game);
                WriteReg(BG3VOFS as u16, 0u8, game);
                WriteReg(BG3VOFS as u16, 0u8, game);
                WriteReg(BG4HOFS as u16, 0u8, game);
                WriteReg(BG4HOFS as u16, 0u8, game);
                WriteReg(BG4VOFS as u16, 0u8, game);
                WriteReg(BG4VOFS as u16, 0u8, game);
                WriteReg(VMAIN as u16, 0u8, game);
                crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
                WriteReg(M7A as u16, 0u8, game);
                WriteReg(M7B as u16, 0u8, game);
                WriteReg(M7C as u16, 0u8, game);
                WriteReg(M7D as u16, 0u8, game);
                WriteReg(M7X as u16, 0u8, game);
                WriteReg(M7Y as u16, 0u8, game);
                crate::rtl::write_register_and_mirror(W12SEL as u16, 0, 0x60, false, game);
                crate::rtl::write_register_and_mirror(W34SEL as u16, 0, 0x61, false, game);
                crate::rtl::write_register_and_mirror(WOBJSEL as u16, 0, 0x62, false, game);
                crate::rtl::write_register_and_mirror(WH0 as u16, 0, 0x63, false, game);
                crate::rtl::write_register_and_mirror(WH1 as u16, 0, 0x64, false, game);
                crate::rtl::write_register_and_mirror(WH2 as u16, 0, 0x65, false, game);
                crate::rtl::write_register_and_mirror(WH3 as u16, 0, 0x66, false, game);
                crate::rtl::write_register_and_mirror(WBGLOG as u16, 0, 0x67, false, game);
                crate::rtl::write_register_and_mirror(WOBJLOG as u16, 0, 0x68, false, game);
                crate::rtl::write_register_and_mirror(TM as u16, 0x10, 0x69, false, game);
                crate::rtl::write_register_and_mirror(TMW as u16, 0, 0x6c, false, game);
                crate::rtl::write_register_and_mirror(TS as u16, 0, 0x6b, false, game);
                crate::rtl::write_register_and_mirror(TSW as u16, 0, 0x6d, false, game);
                crate::rtl::write_register_and_mirror(CGWSEL as u16, 0, 0x6e, false, game);
                crate::rtl::write_register_and_mirror(CGADSUB as u16, 0, 0x71, false, game);
                WriteReg(COLDATA as u16, 0xe0u8, game);
                *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
                *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
                *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
                crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
                CinematicAsset::Decompress(0x9580d8, 0x15000).load(game);
                CinematicAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
                *(&raw mut game.g_ram as *mut u8).offset(0x51) = 0x80u8;
                let mut v0: u16 = 0u16;
                loop {
                    set_ram_word!(
                        game.palette_buffer[(v0 as i32 >> 1i32)] = crate::rtl::rom_table_word!(
                            game,
                            0x8ce1e9,
                            (v0 as i32 >> 1i32) as isize
                        )
                    );
                    v0 = (v0).wrapping_add(2u16);
                    if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
                        break;
                    }
                }
                EnableNMI(game);
                set_ram_word!(game.screen_fade_delay = 1u16);
                set_ram_word!(game.screen_fade_counter = 1u16);
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
                    let mut _coret__2: u8 = WaitForNMI_Async(game);
                    if _coret__2 != 0 {
                        *_state_variable_ = 4u8;
                        return _coret__2;
                    }
                    set_ram_word!(game.screen_fade_delay = 1u16);
                    set_ram_word!(game.screen_fade_counter = 1u16);
                    current_block_182 = 7019009297990327870;
                }
                6110428972393212497 => {
                    let mut _coret__0: u8 = WaitForNMI_Async(game);
                    if _coret__0 != 0 {
                        *_state_variable_ = 2u8;
                        return _coret__0;
                    }
                    set_ram_word!(game.game_options_screen_index = 120u16);
                    current_block_182 = 18425699056680496821;
                }
                17485376261910781866 => {
                    ClearOamExt(game);
                    set_ram_word!(game.oam_next_ptr = 0u16);
                    AddNintentoLogoToOam(game);
                    if AdvanceFastScreenFadeIn(game) as i32 & 1i32 != 0 {
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
                    let mut _coret_: u8 = WaitForNMI_Async(game);
                    if _coret_ != 0 {
                        *_state_variable_ = 1u8;
                        return _coret_;
                    }
                    current_block_182 = 17485376261910781866;
                    continue;
                }
                2371591747199196472 => {
                    let mut _coret__1: u8 = WaitForNMI_Async(game);
                    if _coret__1 != 0 {
                        *_state_variable_ = 3u8;
                        return _coret__1;
                    }
                    current_block_182 = 18425699056680496821;
                }
                17952583510507278934 => {
                    let mut _coret__3: u8 = WaitForNMI_Async(game);
                    if _coret__3 != 0 {
                        *_state_variable_ = 5u8;
                        return _coret__3;
                    }
                    current_block_182 = 7019009297990327870;
                }
                _ => {
                    let mut _coret__4: u8 = WaitForNMI_Async(game);
                    if _coret__4 != 0 {
                        *_state_variable_ = 6u8;
                        return _coret__4;
                    }
                    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
                    current_block_182 = 2222055338596505704;
                    continue;
                }
            }
            if current_block_182 == 18425699056680496821 {
                ClearOamExt(game);
                set_ram_word!(game.oam_next_ptr = 0u16);
                AddNintentoLogoToOam(game);
                if update_ram_word!(game.game_options_screen_index, wrapping_sub(1), reread) == 0 {
                    ClearUnusedOam(game);
                    current_block_182 = 17628988452808763652;
                } else {
                    ClearUnusedOam(game);
                    current_block_182 = 2371591747199196472;
                }
            } else {
                ClearOamExt(game);
                set_ram_word!(game.oam_next_ptr = 0u16);
                AddNintentoLogoToOam(game);
                if AdvanceFastScreenFadeOut(game) as i32 & 1i32 != 0 {
                    ClearUnusedOam(game);
                    current_block_182 = 2484129932066553195;
                } else {
                    ClearUnusedOam(game);
                    current_block_182 = 17952583510507278934;
                }
            }
        }
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn AddNintentoLogoToOam(game: &mut Game) {
    DrawSpritemap(
        0x8cu8,
        addr_kSpriteMap_NintendoLogo as u16,
        128u16,
        112u16,
        0u16,
        game,
    );
}
pub unsafe fn SpawnCinematicSpriteObject(j: u16, a: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.cinematic_spawn_param = a);
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        if ram_word!(game, cinematicspr_instr_ptr[(i >> 1i32)]) == 0 {
            return SpawnCimenaticSpriteObjectInner(j, i as u16, game);
        }
        i -= 2i32;
    }
    1u8
}
pub unsafe fn SpawnCinematicSpriteObjectAtSlot(
    j: u16,
    slot: u16,
    param: u16,
    game: &mut Game,
) -> u8 {
    game.g_ram[0x1b9d..0x1b9f].copy_from_slice(&param.to_le_bytes());
    SpawnCimenaticSpriteObjectInner(j, slot, game)
}
pub unsafe fn SpawnCinematicSpriteObjectToR18(j: u16, r18: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.cinematic_spawn_param = 0u16);
    SpawnCimenaticSpriteObjectInner(j, r18, game)
}
pub unsafe fn SpawnCimenaticSpriteObjectInner(k: u16, j: u16, game: &mut Game) -> u8 {
    let v2: *const u8 = RomPtr_8B(k, game);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_preinstr_func[v3] = *(v2.offset(2) as *mut u16));
    set_ram_word!(game.cinematicspr_instr_ptr[v3] = *(v2.offset(4) as *mut u16));
    set_ram_word!(game.cinematicspr_instr_timer[v3] = 1u16);
    set_ram_word!(game.cinematicspr_whattodraw[v3] = 0u16);
    set_ram_word!(game.cinematicspr_goto_timer[v3] = 0u16);
    set_ram_word!(game.cinematicspr_arr6[v3] = 0u16);
    set_ram_word!(game.cinematicspr_arr7[v3] = 0u16);
    CallCinematicSpriteObjectSetup(((*(v2 as *mut u16)) as u32) | (0x8b0000u32), j, game);
    0u8
}
pub unsafe fn ClearCinematicSprites(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = 0u16);
        set_ram_word!(game.cinematicspr_whattodraw[v1] = 0u16);
        i -= 2i32;
    }
}
pub unsafe fn HandleCinematicSprites(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(game.cinematic_obj_index = i as u16);
        if ram_word!(game, cinematicspr_instr_ptr[(i >> 1i32)]) != 0 {
            ProcessCinematicSpriteInstructionList(i as u16, game);
            i = ram_word!(game, cinematic_obj_index) as i32;
        }
        i -= 2i32;
    }
}
pub unsafe fn ProcessCinematicSpriteInstructionList(k: u16, game: &mut Game) {
    CallCinematicSprPreInstr(
        ((ram_word!(game, cinematicspr_preinstr_func[(k as i32 >> 1i32)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = ram_word!(game, cinematic_obj_index);
    let v2: i32 = ram_word!(game, cinematic_obj_index) as i32 >> 1i32;
    let ref mut fresh22 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1b5d) as *mut u16).offset(v2 as isize);
    let fresh23 = *fresh22;
    *fresh22 = (*fresh22).wrapping_sub(1);
    if fresh23 == 1u16 {
        let mut v4: u16 = ram_word!(game, cinematicspr_instr_ptr[v2]);
        let mut v6: u16 = 0;
        loop {
            let v5: *const u16 = RomPtr_8B(v4, game) as *const u16;
            v6 = *v5;
            if (*v5 as i32 & 0x8000i32) == 0i32 {
                break;
            }
            v4 = CallCinematicSprInstr(
                ((v6) as u32) | (0x8b0000u32),
                v1,
                (v4).wrapping_add(2u16),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let v7: i32 = v1 as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v7] = v6);
        set_ram_word!(
            game.cinematicspr_whattodraw[v7] = *(RomPtr_8B(v4, game) as *mut u16).offset(1)
        );
        set_ram_word!(game.cinematicspr_instr_ptr[v7] = (v4).wrapping_add(4u16));
    }
}
pub unsafe fn CinematicSprInstr_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_whattodraw[v2] = 0u16);
    set_ram_word!(game.cinematicspr_instr_ptr[v2] = 0u16);
    0u16
}
pub unsafe fn CinematicSprInstr_Sleep(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicspr_instr_ptr[(k as i32 >> 1i32)] = (j).wrapping_sub(2u16));
    0u16
}
pub unsafe fn CinematicSprInstr_SetPreInstr(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(
        game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = *(RomPtr_8B(j, game) as *mut u16)
    );
    (j).wrapping_add(2u16)
}
pub unsafe fn CinematicSprInstr_ClearPreInstr(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8b945fi32) as u16));
    j
}
pub unsafe fn CinematicSprInstr_GotoRel(k: u16, j: u16, game: &mut Game) -> u16 {
    (j).wrapping_add((*RomPtr_8B(j, game) as i8) as u16)
}
pub unsafe fn CinematicSprInstr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_8B(j, game) as *mut u16)
}
pub unsafe fn CinematicSprInstr_DecTimerGoto(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh26 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v2 as isize);
    let fresh27 = *fresh26;
    *fresh26 = (*fresh26).wrapping_sub(1);
    if fresh27 == 1u16 {
        (j).wrapping_add(2u16)
    } else {
        CinematicSprInstr_Goto(k, j, game)
    }
}
pub unsafe fn CinematicSprInstr_DecTimerGotoRel(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh24 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v2 as isize);
    let fresh25 = *fresh24;
    *fresh24 = (*fresh24).wrapping_sub(1);
    if fresh25 == 1u16 {
        (j).wrapping_add(1u16)
    } else {
        CinematicSprInstr_GotoRel(k, j, game)
    }
}
pub unsafe fn CinematicSprInstr_SetTimer(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(
        game.cinematicspr_goto_timer[(k as i32 >> 1i32)] = *(RomPtr_8B(j, game) as *mut u16)
    );
    (j).wrapping_add(2u16)
}
pub fn sub_8B94E1() {}
pub unsafe fn SpawnMode7Object(j: u16, a: u16, game: &mut Game) -> u8 {
    let mut Mode7ObjectDef: *mut Mode7ObjectDef = null_mut::<Mode7ObjectDef>();
    set_ram_word!(game.mode7_spawn_param = a);
    let mut v3: u16 = 2u16;
    while ram_word!(game, mode7_obj_instr_ptr[(v3 as i32 >> 1i32)]) != 0 {
        v3 = (v3).wrapping_sub(2u16);
        if (v3 as i32 & 0x8000i32) != 0i32 {
            return 1u8;
        }
    }
    Mode7ObjectDef = get_Mode7ObjectDef(j, game);
    let v6: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.mode7_obj_preinstr_func[v6] = (*Mode7ObjectDef).field_2);
    set_ram_word!(game.mode7_obj_instr_ptr[v6] = (*Mode7ObjectDef).field_4);
    set_ram_word!(game.mode7_obj_instr_timer[v6] = 1u16);
    set_ram_word!(game.mode7_obj_goto_timer[v6] = 0u16);
    CallCinematicSpriteInit(
        (((*Mode7ObjectDef).field_0) as u32) | (0x8b0000u32),
        v6 as u16,
        game,
    );
    0u8
}
pub unsafe fn HandleMode7Objects(game: &mut Game) {
    let mut i: i32 = 2i32;
    while i >= 0i32 {
        set_ram_word!(game.mode7_cur_index = i as u16);
        if ram_word!(game, mode7_obj_instr_ptr[(i >> 1i32)]) != 0 {
            ProcessMode7ObjectInstructions(i as u16, game);
            i = ram_word!(game, mode7_cur_index) as i32;
        }
        i -= 2i32;
    }
}
pub unsafe fn ProcessMode7ObjectInstructions(k: u16, game: &mut Game) {
    CallMode7PreInstr(
        ((ram_word!(game, mode7_obj_preinstr_func[(k as i32 >> 1i32)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = ram_word!(game, mode7_cur_index);
    let v2: i32 = ram_word!(game, mode7_cur_index) as i32 >> 1i32;
    let ref mut fresh19 =
        *((&raw mut game.g_ram as *mut u8).offset(0x19a9) as *mut u16).offset(v2 as isize);
    let fresh20 = *fresh19;
    *fresh19 = (*fresh19).wrapping_sub(1);
    if fresh20 == 1u16 {
        let mut v4: u16 = ram_word!(game, mode7_obj_instr_ptr[v2]);
        let mut v6: u16 = 0;
        loop {
            let v5: *const u16 = RomPtr_8B(v4, game) as *const u16;
            v6 = *v5;
            if (*v5 as i32 & 0x8000i32) == 0i32 {
                break;
            }
            v4 = CallMode7Instr(
                ((v6) as u32) | (0x8b0000u32),
                v1,
                (v4).wrapping_add(2u16),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        set_ram_word!(game.mode7_obj_instr_timer[(v1 as i32 >> 1i32)] = v6);
        let v7: *const u8 = RomPtr_8B(v4, game);
        QueueMode7Transfers(0x8bu8, *(v7.offset(2) as *mut u16), game);
        set_ram_word!(game.mode7_obj_instr_ptr[(v1 as i32 >> 1i32)] = (v4).wrapping_add(4u16));
    }
}
pub unsafe fn Mode7Instr_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.mode7_obj_instr_ptr[(k as i32 >> 1i32)] = 0u16);
    0u16
}
pub unsafe fn Mode7Instr_SetPreInstr(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(
        game.mode7_obj_preinstr_func[(k as i32 >> 1i32)] = *(RomPtr_8B(j, game) as *mut u16)
    );
    (j).wrapping_add(2u16)
}
pub unsafe fn Mode7Instr_ClearPreInstr(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.mode7_obj_preinstr_func[(k as i32 >> 1i32)] = ((0x8b9585i32) as u16));
    j
}
pub unsafe fn Mode7Instr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_8B(j, game) as *mut u16)
}
pub unsafe fn Mode7Instr_DecTimerAndGoto(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh0 =
        *((&raw mut game.g_ram as *mut u8).offset(0x19ad) as *mut u16).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 == 1u16 {
        (j).wrapping_add(2u16)
    } else {
        Mode7Instr_Goto(k, j, game)
    }
}
pub unsafe fn Mode7Instr_SetTimer(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(
        game.mode7_obj_goto_timer[(k as i32 >> 1i32)] = *(RomPtr_8B(j, game) as *mut u16)
    );
    (j).wrapping_add(2u16)
}
pub unsafe fn EnableCinematicBgObjects(game: &mut Game) {
    update_ram_word!(game.cinematic_enable_objs, |fresh13| (fresh13)
        | (0x8000u16));
}
pub unsafe fn DisableCinematicBgObjects(game: &mut Game) {
    update_ram_word!(
        game.cinematic_enable_objs,
        |fresh9| (fresh9 as i32 & !(0x8000i32)) as u16
    );
}
pub unsafe fn EnableCinematicBgTilemapUpdates(game: &mut Game) {
    update_ram_word!(game.cinematic_enable_bg_tilemap, |fresh14| (fresh14)
        | (0x8000u16));
}
pub unsafe fn DisableCinematicBgTilemapUpdates(game: &mut Game) {
    update_ram_word!(
        game.cinematic_enable_bg_tilemap,
        |fresh10| (fresh10 as i32 & !(0x8000i32)) as u16
    );
}
pub unsafe fn ClearCinematicBgObjects(a: u16, game: &mut Game) {
    let mut j: u16 = 0;
    let mut i: i32 = 2046i32;
    while i >= 0i32 {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = a;
        i -= 2i32;
    }
    j = 6u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        let v3: i32 = j as i32 >> 1i32;
        set_ram_word!(game.cinematicbg_arr1[v3] = 0u16);
        set_ram_word!(game.cinematicbg_instr_ptr[v3] = 0u16);
        j = (j).wrapping_sub(2u16);
    }
    set_ram_word!(game.cinematicbg_var1 = 0u16);
}
pub unsafe fn CallCinematicBgObjectSetup(ea: u32, j: u16, game: &mut Game) {
    match ea {
        9147353 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn SpawnCinematicBgObject(j: u16, a: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.cinematicbg_var3 = a);
    let mut v3: u16 = 6u16;
    while ram_word!(game, cinematicbg_instr_ptr[(v3 as i32 >> 1i32)]) != 0 {
        v3 = (v3).wrapping_sub(2u16);
        if (v3 as i32 & 0x8000i32) != 0i32 {
            return 1u8;
        }
    }
    let v5: *const u8 = RomPtr_8B(j, game);
    let v6: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_preinstr[v6] = *(v5.offset(2) as *mut u16));
    set_ram_word!(game.cinematicbg_instr_ptr[v6] = *(v5.offset(4) as *mut u16));
    set_ram_word!(game.cinematicbg_instr_timer[v6] = 1u16);
    set_ram_word!(game.cinematicbg_arr1[v6] = 0u16);
    set_ram_word!(game.cinematicbg_arr6[v6] = 0u16);
    CallCinematicBgObjectSetup(((*(v5 as *mut u16)) as u32) | (0x8b0000u32), v3, game);
    0u8
}
pub unsafe fn HandleCinematicBgObjects(game: &mut Game) {
    if (ram_word!(game, cinematic_enable_objs) as i32 & 0x8000i32) != 0i32 {
        let mut i: i32 = 6i32;
        while i >= 0i32 {
            set_ram_word!(game.cinematicbg_var2 = i as u16);
            if ram_word!(game, cinematicbg_instr_ptr[(i >> 1i32)]) != 0 {
                ProcessCinematicBgObjectsInstrs(i as u16, game);
                i = ram_word!(game, cinematicbg_var2) as i32;
            }
            i -= 2i32;
        }
        if (ram_word!(game, cinematic_enable_bg_tilemap) as i32 & 0x8000i32) != 0i32 {
            UpdateCinematicBgTilemap(game);
        }
        UpdateSamusEyesTilemap(game);
    }
}
pub unsafe fn ProcessCinematicBgObjectsInstrs(k: u16, game: &mut Game) {
    CallCinematicBgPreInstr(
        ((ram_word!(game, cinematicbg_preinstr[(k as i32 >> 1i32)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = ram_word!(game, cinematicbg_var2);
    let v2: i32 = ram_word!(game, cinematicbg_var2) as i32 >> 1i32;
    let ref mut fresh158 =
        *((&raw mut game.g_ram as *mut u8).offset(0x19dd) as *mut u16).offset(v2 as isize);
    let fresh159 = *fresh158;
    *fresh158 = (*fresh158).wrapping_sub(1);
    if fresh159 == 1u16 {
        let mut v4: u16 = ram_word!(game, cinematicbg_instr_ptr[v2]);
        let mut v6: u16 = 0;
        loop {
            let v5: *const u16 = RomPtr_8C(v4, game) as *const u16;
            v6 = *v5;
            if (*v5 as i32 & 0x8000i32) == 0i32 {
                break;
            }
            v4 = CallCinematicBgInstr(
                ((v6) as u32) | (0x8b0000u32),
                v1,
                (v4).wrapping_add(2u16),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let v7: i32 = v1 as i32 >> 1i32;
        set_ram_word!(game.cinematicbg_instr_timer[v7] = v6);
        let v8: *const u8 = RomPtr_8C(v4, game);
        set_ram_word!(game.cinematicbg_arr1[v7] = *(v8.offset(4) as *mut u16));
        ProcessCinematicBgObject(v1, *(v8.offset(2) as *mut u16), game);
        set_ram_word!(game.cinematicbg_instr_ptr[v7] = (v4).wrapping_add(6u16));
    }
}
pub unsafe fn CinematicBgInstr_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr1[v2] = 0u16);
    set_ram_word!(game.cinematicbg_instr_ptr[v2] = 0u16);
    0u16
}
pub unsafe fn CinematicBgInstr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_8C(j, game) as *mut u16)
}
pub fn sub_8B9743() {}
pub unsafe fn DrawCinematicSpriteObjects_Intro(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, cinematicspr_whattodraw[v1]) != 0 {
            let v2: u16 = ram_word!(game, cinematicspr_whattodraw[v1]);
            let chr: u16 = ram_word!(game, cinematicbg_arr9[v1]);
            let x: u16 =
                (ram_word!(game, cinematicbg_arr7[v1])).wrapping_sub(ram_word!(game, layer1_x_pos));
            let y: u16 =
                (ram_word!(game, cinematicbg_arr8[v1])).wrapping_sub(ram_word!(game, layer1_y_pos));
            if (y as i32 & 0xff00i32) != 0i32 {
                if (y).wrapping_add(128u16) < 0x1ffu16 {
                    DrawSpritemapOffScreen(v2, x, y, chr, game);
                }
            } else if (y).wrapping_add(128u16) < 0x1ffu16 {
                DrawSpritemap(0x8cu8, v2, x, y, chr, game);
            }
        }
        i -= 2i32;
    }
}
pub unsafe fn DrawCinematicSpriteObjects_Ending(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, cinematicspr_whattodraw[v1]) != 0 {
            let v2: u16 = ram_word!(game, cinematicspr_whattodraw[v1]);
            let chr: u16 = ram_word!(game, cinematicbg_arr9[v1]);
            let x: u16 =
                (ram_word!(game, cinematicbg_arr7[v1])).wrapping_sub(ram_word!(game, layer1_x_pos));
            let y: u16 =
                (ram_word!(game, cinematicbg_arr8[v1])).wrapping_sub(ram_word!(game, layer1_y_pos));
            if (y as i32 & 0xff00i32) != 0i32 {
                if (y).wrapping_add(128u16) < 0x1ffu16 {
                    DrawSpritemapOffScreen(v2, x, y, chr, game);
                }
            } else if (y).wrapping_add(128u16) < 0x1ffu16 {
                DrawSpritemap(0x8cu8, v2, x, y, chr, game);
            }
        }
        i -= 2i32;
    }
    if ram_word!(game, cinematic_function) as i32 - (0x8bda02i32 & 0xffffi32) & 0x8000i32 != 0 {
        MoveUnusedSpritesOffScreen(game);
    }
}
pub unsafe fn SpawnTextGlowObject(j: u16, mut r18: u16, game: &mut Game) {
    let mut v1: u16 = 14u16;
    while *(((&raw mut game.g_ram as *mut u8).offset(0x19f7) as *mut u16 as *mut u8)
        .offset(v1 as i32 as isize) as *mut u16)
        != 0
    {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    *(((&raw mut game.g_ram as *mut u8).offset(0x19f7) as *mut u16 as *mut u8)
        .offset(v1 as i32 as isize) as *mut u16) = j;
    let v2: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.eproj_pre_instr[(v2 + 2i32)] = 1u16);
    set_ram_word!(game.eproj_pre_instr[(v2 + 10i32)] = r18 as u8 as u16);
    *(((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
        .offset(v1 as i32 as isize) as *mut u16) = *(&raw mut r18 as *mut u8).offset(1isize) as u16;
    *(((&raw mut game.g_ram as *mut u8).offset(0x1a37) as *mut u16 as *mut u8)
        .offset(v1 as i32 as isize) as *mut u16) = 0u16;
}
pub unsafe fn HandleTextGlowObjects(game: &mut Game) {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        set_ram_word!(game.cinematic_var19 = i as u16);
        if *(((&raw mut game.g_ram as *mut u8).offset(0x19f7) as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16)
            != 0
        {
            ProcessTextGlowObject(game);
            i = ram_word!(game, cinematic_var19) as i32;
        }
        i -= 2i32;
    }
}
pub unsafe fn ProcessTextGlowObject(game: &mut Game) {
    let v0: u16 = ram_word!(game, cinematic_var19);
    let v1: i32 = ram_word!(game, cinematic_var19) as i32 >> 1i32;
    let ref mut fresh153 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1a03) as *mut u16).offset((v1 + 2i32) as isize);
    let fresh154 = *fresh153;
    *fresh153 = (*fresh153).wrapping_sub(1);
    if fresh154 == 1u16 {
        let R28: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x1a37) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16);
        let mut v3: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x19f7) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16);
        let mut r22: u16 = (2i32
            * *(((&raw mut game.g_ram as *mut u8).offset(0x1a03) as *mut u16)
                .offset((v1 + 10i32) as isize) as *mut u8)
                .offset(0) as i32
            + Mult8x8(
                *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
                    .offset(v0 as i32 as isize),
                0x40u8,
            ) as i32) as u16;
        let mut v4: u16 = r22;
        let v5: *const u8 = RomPtr_8C(v3, game);
        let mut n: i32 = *v5.offset(2) as i32;
        let norg: i32 = n;
        let mut m: i32 = *v5.offset(3) as i32;
        loop {
            loop {
                *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                    .pause_menu_map_tilemap as *mut u16 as *mut u8)
                    .offset(v4 as i32 as isize) as *mut u16) = (R28 as i32
                    | *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000)
                        as *mut Ram3000))
                        .pause_menu_map_tilemap as *mut u16 as *mut u8)
                        .offset(v4 as i32 as isize) as *mut u16) as i32
                        & 0xe3ffi32)
                    as u16;
                v3 = (v3).wrapping_add(2u16);
                v4 = (v4).wrapping_add(2u16);
                n -= 1;
                if (n == 0) {
                    break;
                }
            }
            n = norg;
            m -= 1;
            if m == 0 {
                break;
            }
            r22 = (r22).wrapping_add(64u16);
            v4 = r22;
        }
        let v6: u16 = ram_word!(game, cinematic_var19);
        if *(((&raw mut game.g_ram as *mut u8).offset(0x1a37) as *mut u16 as *mut u8)
            .offset(ram_word!(game, cinematic_var19) as i32 as isize) as *mut u16)
            == 3072u16
        {
            *(((&raw mut game.g_ram as *mut u8).offset(0x19f7) as *mut u16 as *mut u8)
                .offset(ram_word!(game, cinematic_var19) as i32 as isize)
                as *mut u16) = 0u16;
        } else {
            let ref mut fresh155 = *(((&raw mut game.g_ram as *mut u8).offset(0x1a37) as *mut u16
                as *mut u8)
                .offset(ram_word!(game, cinematic_var19) as i32 as isize)
                as *mut u16);
            *fresh155 = (*fresh155).wrapping_add(1024u16);
            *((&raw mut game.g_ram as *mut u8).offset(0x1a03) as *mut u16)
                .offset(((v6 as i32 >> 1i32) + 2i32) as isize) = 5u16;
        }
    }
}
pub unsafe fn EnableTextGlowObjects_(game: &mut Game) {
    update_ram_word!(game.cinematic_var25, |fresh15| (fresh15) | (0x8000u16));
}
pub unsafe fn DisableTextGlowObjects_(game: &mut Game) {
    update_ram_word!(
        game.cinematic_var25,
        |fresh7| (fresh7 as i32 & !(0x8000i32)) as u16
    );
}
pub unsafe fn CinematicFunction_Intro_Func128(a: u16, game: &mut Game) {
    let mut i: i32 = 2046i32;
    while i >= 0i32 {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = a;
        i -= 2i32;
    }
    set_ram_word!(game.cinematic_var26 = 0u16);
    set_ram_word!(game.cinematic_var21 = 0u16);
    set_ram_word!(game.cinematic_var22 = 0u16);
    set_ram_word!(game.cinematic_var23 = 0u16);
    set_ram_word!(game.cinematic_var24 = 0u16);
    set_ram_word!(game.cinematicbg_var3 = 18432u16);
    set_ram_word!(game.eproj_pre_instr[0] = 0u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
}
pub unsafe fn CreditsObject_Init(j: u16, game: &mut Game) {
    let v1: *const u8 = RomPtr_8B(j, game);
    set_ram_word!(game.cinematic_var24 = *(v1.offset(2) as *mut u16));
    set_ram_word!(game.cinematic_var21 = *(v1.offset(4) as *mut u16));
    set_ram_word!(game.cinematic_var22 = 1u16);
    set_ram_word!(game.cinematic_var23 = 0u16);
    CallCreditsObjectFunc(((*(v1 as *mut u16)) as u32) | (0x8b0000u32), 0u16, game);
}
pub unsafe fn CreditsObject_Process(game: &mut Game) {
    if (ram_word!(game, cinematic_var25) as i32 & 0x8000i32) != 0i32 {
        if ram_word!(game, cinematic_var21) != 0 {
            CreditsObject_ProcessOne(game);
        }
        CinematicUpdateSomeBg(game);
    }
}
pub unsafe fn CreditsObject_ProcessOne(game: &mut Game) {
    let mut i: i32 = 0;
    CallCreditsObjectFunc(
        ((ram_word!(game, cinematic_var24)) as u32) | (0x8b0000u32),
        0u16,
        game,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        0x8000u32,
    );
    if ram_word!(game, cinematic_var10) as i32 - ram_word!(game, eproj_pre_instr[0]) as i32 - 8i32
        & 0x8000i32
        == 0
    {
        set_ram_word!(game.eproj_pre_instr[0] = ram_word!(game, cinematic_var10));
        i = ram_word!(game, cinematic_var21) as i32;
        loop {
            let v2: *const u16 = RomPtr_8C(i as u16, game) as *const u16;
            let v3: u16 = *v2;
            if (*v2 as i32 & 0x8000i32) == 0i32 {
                break;
            }
            i = CallIntroObjectInstr(((v3) as u32) | (0x8b0000u32), 0u16, (i + 2i32) as u16, game)
                as i32;
            if i == 0 {
                return;
            }
        }
        CreditsObject_Func1(i as u16, game);
        set_ram_word!(
            game.cinematic_var26 =
                (ram_word!(game, cinematic_var26) as i32 + 1i32 & 0x1fi32) as u16
        );
        set_ram_word!(game.cinematic_var21 = (i + 4i32) as u16);
    }
}
pub unsafe fn CreditsObject_Func1(j: u16, game: &mut Game) {
    let mut RegWord: u16 = Mult8x8(ram_word!(game, cinematic_var26) as u8, 0x40u8);
    let mut n: i32 = 31i32;
    let mut src: *const u16 = (&raw mut game.g_ram as *mut u8)
        .offset(0x10000)
        .offset(*(RomPtr_8C(j, game) as *mut u16).offset(1) as i32 as isize)
        as *const u16;
    loop {
        let fresh6 = src;
        src = src.offset(1);
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut u16 as *mut u8)
            .offset(RegWord as i32 as isize) as *mut u16) = *fresh6;
        RegWord = (RegWord).wrapping_add(2u16);
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn IntroObject_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_var21 = 0u16);
    0u16
}
pub unsafe fn IntroObject_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_8C(j, game) as *mut u16)
}
pub unsafe fn IntroObject_DecTimerGoto(k: u16, j: u16, game: &mut Game) -> u16 {
    if update_ram_word!(game.cinematic_var23, wrapping_sub(1), reread) != 0 {
        IntroObject_Goto(k, j, game)
    } else {
        (j).wrapping_add(2u16)
    }
}
pub unsafe fn IntroObject_SetTimer(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_var23 = *(RomPtr_8C(j, game) as *mut u16));
    (j).wrapping_add(2u16)
}
pub unsafe fn GameState_1_OpeningCinematic_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((ram_word!(game, cinematic_function)) as u32) | (0x8b0000u32),
        game,
    );
    HandleCinematicSprites(game);
    HandleMode7Objects(game);
    PaletteFxHandler(game);
    DrawCinematicSpriteObjects_Intro(game);
    MaybeSkipCinematics(game);
    HandleCinematicsTransitions(game);
    HandleMode7TransformationNoRotation(game);
    kCoroutineNone as CoroutineRet
}
pub fn CinematicFunctionNone() {}
pub unsafe fn MaybeSkipCinematics(game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 + 0x60d8i32 & 0x8000i32 != 0
        && (ram_word!(game, joypad1_newkeys) as i32
            & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16 as i32)
            != 0i32
        && ram_word!(game, cinematic_var18) == 0
    {
        set_ram_word!(game.cinematic_var18 = 1u16);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
    }
}
pub unsafe fn HandleCinematicsTransitions_1(game: &mut Game) {
    if AdvanceFastScreenFadeOut(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_var18 = 2u16);
        set_ram_word!(game.cinematic_function = ((0x8b9f28i32) as u16));
        QueueMusic_Delayed8(6u16, game);
    }
}
pub unsafe fn HandleCinematicsTransitions_2(game: &mut Game) {
    ClearCinematicSprites(game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA119 as u16, 0u16, game);
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA125 as u16,
        0x0400, // Palette left in A by the first sprite initializer.
        game,
    );
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 2u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    set_ram_word!(game.cinematic_var18 = 3u16);
    let mut v2: u16 = 256u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v2 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce1e9, (v2 as i32 >> 1i32) as isize)
        );
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    set_ram_word!(game.palette_buffer[201] = 0x7fffu16);
    set_ram_word!(game.palette_buffer[202] = 32128u16);
    ClearPaletteFXObjects(game);
    SpawnPalfxObject(addr_kPalfx_E1A0 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as u16, game);
    SetPpuBackdropBlack(game);
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    SpawnTitleScreenGradientObjs(game);
}
pub unsafe fn CinematicSpriteInit_7(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 48u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 1024u16);
}
pub unsafe fn CinematicSpriteInit_8(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 81u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicSpriteInit_9(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 196u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn HandleCinematicsTransitions_3(game: &mut Game) {
    if AdvanceFastScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_var18 = 0u16);
        set_ram_word!(game.demo_timer = 900u16);
        set_ram_word!(game.cinematic_function = ((0x8b9f29i32) as u16));
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
    if ram_word!(game, cinematic_var18) != 0 {
        off_8B9A7B[ram_word!(game, cinematic_var18) as usize].expect("non-null function pointer")(
            game,
        );
    }
}
pub unsafe fn CinematicFunctionOpening(game: &mut Game) {
    LoadTitleSequenceGraphics(game);
    QueueMusic_Delayed8(0xff03u16, game);
    set_ram_word!(game.cinematic_function = ((0x8b9a47i32) as u16));
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA0EF as u16,
        ((0x8b9a47i32) as u16),
        game,
    );
    QueueMusic_Delayed8(5u16, game);
}
pub unsafe fn LoadTitleSequenceGraphics(game: &mut Game) {
    let mut v1: i16 = 0;
    SetupPpuForTitleSequence(game);
    set_ram_word!(game.cur_irq_handler = 0u16);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    let mut v0: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v0 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce1e9, (v0 as i32 >> 1i32) as isize)
        );
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x94e000, 0x10000).load(game);
    CinematicAsset::Decompress(0x96fc04, 0x14000).load(game);
    CinematicAsset::Decompress(0x9580d8, 0x15000).load(game);
    CinematicAsset::Decompress(0x95a5e1, 0x19000).load(game);
    CinematicAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
    WriteReg(VMAIN as u16, 0u8, game);
    WriteReg(VMADDL as u16, 0u8, game);
    WriteReg(VMADDH as u16, 0u8, game);
    v1 = 0x4000i16;
    loop {
        WriteReg(VMDATAL as u16, 0xffu8, game);
        v1 -= 1;
        if (v1 == 0) {
            break;
        }
    }
    CinematicAsset::VramLowBytes(0x7f4000, 0x0000, 0x1000).load(game);
    CinematicAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7X = 128u16);
    WriteRegWord(M7Y as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7Y = 128u16);
    EnableNMI(game);
    set_ram_word!(game.screen_fade_delay = 1u16);
    set_ram_word!(game.screen_fade_counter = 1u16);
    EnableHdmaObjects(game);
    EnablePaletteFx(game);
    set_ram_word!(game.fx_layer_blending_config_a = 0u16);
    SpawnPalfxObject(addr_kPalfx_E1A0 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as u16, game);
    SpawnMode7Object(
        addr_kMode7ObjBabyMetroidInTitle as u16,
        0x000c, // The second title palette allocation leaves its slot offset in A.
        game,
    );
}
pub unsafe fn CinematicSpriteInit_0(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 129u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
}
pub unsafe fn sub_8B9CCF(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x51) = 15u8;
    set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8b93d9i32) as u16));
}
pub fn sub_8B9CDE() {}
pub unsafe fn CinematicSprInstr_9CE1(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    set_ram_word!(game.cinematic_function = ((0x8b9d17i32) as u16));
    set_ram_word!(game.cinematic_var6 = 72u16);
    set_ram_word!(game.cinematic_var8 = 315u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var10 = 225u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.g_word_7E199B = -2i32 as u16);
    set_ram_word!(game.g_word_7E1999 = 0x8000u16);
    SetPpuBackdropSomeColor(game);
    j
}
pub unsafe fn CinematicFunc_Func2(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        (ram_word!(game, g_word_7E199B) as u32) << 16i32 | ram_word!(game, g_word_7E1999) as u32,
    );
    if ram_word!(game, cinematic_var8) as i32 + 7i32 & 0x8000i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        set_ram_word!(game.cinematic_function = ((0x8b9a47i32) as u16));
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA0F5 as u16,
            ((0x8b9a47i32) as u16),
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_1(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 129u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
}
pub unsafe fn CinematicSprInstr_9D5D(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    set_ram_word!(game.cinematic_function = ((0x8b9d90i32) as u16));
    set_ram_word!(game.cinematic_var6 = 96u16);
    set_ram_word!(game.cinematic_var8 = 44u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var10 = -155i32 as u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.g_word_7E199B = -2i32 as u16);
    set_ram_word!(game.g_word_7E1999 = 0x8000u16);
    j
}
pub unsafe fn CinematicFunc_Func4(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        (ram_word!(game, g_word_7E199B) as u32) << 16i32 | ram_word!(game, g_word_7E1999) as u32,
    );
    if ram_word!(game, cinematic_var8) as i32 + 176i32 & 0x8000i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        set_ram_word!(game.cinematic_function = ((0x8b9a47i32) as u16));
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA0FB as u16,
            ((0x8b9a47i32) as u16),
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 129u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
}
pub unsafe fn CinematicSprInstr_9DD6(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    set_ram_word!(game.cinematic_function = ((0x8b9e12i32) as u16));
    set_ram_word!(game.cinematic_var6 = 96u16);
    set_ram_word!(game.cinematic_var8 = -177i32 as u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var10 = -160i32 as u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199F = 1u16);
    set_ram_word!(game.g_word_7E199D = 0x8000u16);
    SetPpuBackdropSomeColor(game);
    j
}
pub unsafe fn CinematicFunc_Func5(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        (ram_word!(game, g_word_7E199F) as u32) << 16i32 | ram_word!(game, g_word_7E199D) as u32,
    );
    if ram_word!(game, cinematic_var10) as i32 - 163i32 & 0x8000i32 == 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        set_ram_word!(game.cinematic_function = ((0x8b9a47i32) as u16));
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA101 as u16,
            ((0x8b9a47i32) as u16),
            game,
        );
        SetPpuBackdropBlack(game);
    }
    MaybeSkipCinematics(game);
}
pub unsafe fn CinematicSpriteInit_3(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 129u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
}
pub unsafe fn CinematicSprInstr_Func8(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    set_ram_word!(game.cinematic_function = ((0x8b9e8bi32) as u16));
    set_ram_word!(game.cinematic_var6 = 67u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var7 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.g_word_7E199B = 0u16);
    set_ram_word!(game.g_word_7E1999 = 0u16);
    set_ram_word!(game.g_word_7E199F = 0u16);
    set_ram_word!(game.g_word_7E199D = 0u16);
    SpawnTitleScreenGradientObjs(game);
    j
}
pub unsafe fn CinematicFunc_Func7(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) == 0i32 {
        if ram_word!(game, cinematic_var6) as i32 - 256i32 & 0x8000i32 != 0 {
            update_ram_word!(game.cinematic_var6, wrapping_add(1));
        } else {
            set_ram_word!(game.cinematic_var6 = 256u16);
            set_ram_word!(game.cinematic_function = ((0x8b9f28i32) as u16));
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BA107 as u16,
                ((0x8b9f28i32) as u16),
                game,
            );
        }
    }
}
pub unsafe fn CinematicSpriteInit_4(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 48u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 1024u16);
    SpawnPalfxObject(addr_kPalfx_E194 as u16, game);
}
pub unsafe fn CinematicSprInstr_Func9(k: u16, j: u16, game: &mut Game) -> u16 {
    let v0: u16 = 0x9ecdu16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA113 as u16, v0, game);
    j
}
pub unsafe fn CinematicSpriteInit_5(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 81u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
    SpawnPalfxObject(addr_kPalfx_E198 as u16, game);
}
pub unsafe fn CinematicSprInstr_sub_8B9EF0(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_arr9[(k as i32 >> 1i32)] = 0u16);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA113 as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSpriteInit_6(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 196u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
    SpawnPalfxObject(addr_kPalfx_E19C as u16, game);
}
pub unsafe fn CinematicSprInstr_9F19(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.demo_timer = 900u16);
    set_ram_word!(game.cinematic_function = ((0x8b9f29i32) as u16));
    j
}
pub unsafe fn CinematicFunc_Func1(game: &mut Game) {
    let current_block: u64;
    let v0: bool =
        (update_ram_word!(game.demo_timer, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, demo_timer) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8b9faei32) as u16));
        current_block = 5176254373955358112;
    } else if (ram_word!(game, joypad1_newkeys) as i32
        & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16 as i32)
        != 0i32
    {
        set_ram_word!(game.cinematic_function = ((0x8b9f52i32) as u16));
        current_block = 5176254373955358112;
    } else {
        current_block = 4906268039856690917;
    }
    if current_block == 5176254373955358112 {
        set_ram_word!(game.screen_fade_delay = 2u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
    };
}
pub unsafe fn CinematicFunc_Func10(game: &mut Game) {
    let mut j: u16 = 0;
    if AdvanceSlowScreenFadeOut(game) as i32 & 1i32 != 0 {
        EnableNMI(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 1u8;
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.reg_BG2HOFS = 0u16);
        set_ram_word!(game.reg_BG3HOFS = 0u16);
        set_ram_word!(game.reg_BG2VOFS = 0u16);
        set_ram_word!(game.reg_BG3VOFS = 0u16);
        set_ram_word!(game.cinematic_var5 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8b9b68i32) as u16));
        DisablePaletteFx(game);
        ClearPaletteFXObjects(game);
        let mut i: i32 = 656i32;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
            i -= 2i32;
        }
        j = 510u16;
        while (j as i32 & 0x8000i32) == 0i32 {
            set_ram_word!(game.hdma_table_1[(j as i32 >> 1i32)] = 0u16);
            j = (j).wrapping_sub(2u16);
        }
        set_ram_word!(game.game_state = 4u16);
        set_ram_word!(game.game_options_screen_index = 0u16);
    }
}
pub unsafe fn CinematicFunc_Func9(game: &mut Game) {
    let mut j: u16 = 0;
    if AdvanceSlowScreenFadeOut(game) as i32 & 1i32 != 0 {
        EnableNMI(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 1u8;
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.reg_BG2HOFS = 0u16);
        set_ram_word!(game.reg_BG3HOFS = 0u16);
        set_ram_word!(game.reg_BG2VOFS = 0u16);
        set_ram_word!(game.reg_BG3VOFS = 0u16);
        set_ram_word!(game.cinematic_var5 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8b9b68i32) as u16));
        DisablePaletteFx(game);
        ClearPaletteFXObjects(game);
        let mut i: i32 = 656i32;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
            i -= 2i32;
        }
        j = 510u16;
        while (j as i32 & 0x8000i32) == 0i32 {
            set_ram_word!(game.hdma_table_1[(j as i32 >> 1i32)] = 0u16);
            j = (j).wrapping_sub(2u16);
        }
        set_ram_word!(game.game_state = 40u16);
        set_ram_word!(game.demo_scene = 0u16);
    }
}
pub unsafe fn ConfigureTitleSequenceGradientHDMA(game: &mut Game) {
    let mut v0: u16 = crate::rtl::rom_table_word!(
        game,
        0x8cbc5d,
        ((ram_word!(game, cinematic_var6) as i32 & 0xf0i32) >> 4i32) as isize
    );
    let mut i: i32 = 0i32;
    loop {
        let v2: u16 = *(RomPtr_8C(v0, game) as *mut u16);
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = v2);
        if v2 == 0 {
            break;
        }
        v0 = (v0).wrapping_add(2u16);
        i += 2i32;
    }
}
pub unsafe fn PlayBabyMetroidCry1(k: u16, j: u16, game: &mut Game) -> u16 {
    QueueSfx3_Max6(0x23u16, game);
    j
}
pub unsafe fn PlayBabyMetroidCry2(k: u16, j: u16, game: &mut Game) -> u16 {
    QueueSfx3_Max6(0x26u16, game);
    j
}
pub unsafe fn PlayBabyMetroidCry3(k: u16, j: u16, game: &mut Game) -> u16 {
    QueueSfx3_Max6(0x27u16, game);
    j
}
static kCinematicPal1: [u16; 16] = [
    0x1000, 0x6bf5, 0x2e41, 0x2da1, 0x2d01, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
static kCinematicPal2: [u16; 16] = [
    0x3800, 0x6bf5, 0x6e1, 0x641, 0x5a1, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
static kCinematicPal3: [u16; 16] = [
    0x3800, 0x77f8, 0x1344, 0x12a4, 0x1204, 0x6abf, 0x249f, 0x1c77, 0x146d, 0x1067, 0x5bff, 0x4b38,
    0x3a72, 0x7bd3, 0x7fff, 0x6b43,
];
static kCinematicPal4: [u16; 16] = [
    0x3800, 0x7ffb, 0x1fa7, 0x1f07, 0x1e67, 0x771f, 0x30ff, 0x28da, 0x20d0, 0x1cca, 0x67ff, 0x579b,
    0x46d5, 0x7bd6, 0x7fff, 0x77a6,
];
pub unsafe fn CinematicSetPal1(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 48i32)] = kCinematicPal1[(i >> 1i32) as usize]
        );
        i -= 2i32;
    }
    j
}
pub unsafe fn CinematicSetPal2(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 48i32)] = kCinematicPal2[(i >> 1i32) as usize]
        );
        i -= 2i32;
    }
    j
}
pub unsafe fn CinematicSetPal3(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 48i32)] = kCinematicPal3[(i >> 1i32) as usize]
        );
        i -= 2i32;
    }
    j
}
pub unsafe fn CinematicSetPal4(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 48i32)] = kCinematicPal4[(i >> 1i32) as usize]
        );
        i -= 2i32;
    }
    j
}
pub unsafe fn GameState_37_CeresGoesBoomWithSamus_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((ram_word!(game, cinematic_function)) as u32) | (0x8b0000u32),
        game,
    );
    if (ram_word!(game, cinematic_var11) as i32 & 0x8000i32) == 0i32 {
        update_ram_word!(game.cinematic_var11, wrapping_add(1));
    }
    update_ram_word!(game.cinematic_var12, wrapping_add(1));
    HandleSamusDuringIntro(game);
    HandleCinematicSprites(game);
    HandleMode7Objects(game);
    HandleCinematicBgObjects(game);
    HandleTextGlowObjects(game);
    PaletteFxHandler(game);
    DrawIntroSprites(game);
    HandleMode7TransformationMatrix(game);
    kCoroutineNone as CoroutineRet
}
pub fn CinematicFunc_Nothing() {}
pub unsafe fn CinematicFunction_Intro_Initial(game: &mut Game) {
    let mut j: u16 = 0;
    let mut k: u16 = 0;
    SetupPpu_Intro(game);
    set_ram_word!(game.cur_irq_handler = 0u16);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    set_ram_word!(game.area_index = 0u16);
    set_ram_word!(game.room_width_in_blocks = 16u16);
    set_ram_word!(game.room_height_in_blocks = 16u16);
    Samus_Initialize(game);
    UpdateBeamTilesAndPalette(game);
    set_ram_word!(game.samus_max_missiles = 900u16);
    set_ram_word!(game.samus_missiles = 900u16);
    set_ram_word!(game.cinematic_var15 = 0u16);
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    ResetButtonAssignmentsToDefault(game);
    set_ram_word!(game.samus_invincibility_timer = 0u16);
    set_ram_word!(game.samus_knockback_timer = 0u16);
    let mut i: i32 = 656i32;
    while i >= 0i32 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
        i -= 2i32;
    }
    let mut v1: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v1 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce3e9, (v1 as i32 >> 1i32) as isize)
        );
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x95f90e, 0x10000).load(game);
    CinematicAsset::Decompress(0x95d089, 0x18000).load(game);
    CinematicAsset::Decompress(0x9788cc, 0x19000).load(game);
    CinematicAsset::Decompress(0x96ff14, 0x19800).load(game);
    CinematicAsset::Decompress(0x95e4c2, 0x1b800).load(game);
    CinematicAsset::Decompress(0x978d12, 0x1e000).load(game);
    CinematicAsset::VramWords(0x7f0000, 0x0000, 0x8000).load(game);
    CinematicAsset::VramWords(0x7f8000, 0x4000, 0x900).load(game);
    CinematicAsset::VramWords(0x7f9000, 0x4800, 0x800).load(game);
    CinematicAsset::VramWords(0x7fe000, 0x4c00, 0x800).load(game);
    CinematicAsset::VramWords(0x7f9800, 0x5000, 0x2000).load(game);
    CinematicAsset::VramWords(0x9ad200, 0x6000, 0x2000).load(game);
    CinematicAsset::VramWords(0x7fb800, 0x6e00, 0x2400).load(game);
    CinematicAsset::Decompress(0x95d713, 0x1a000).load(game);
    EnableNMI(game);
    set_ram_word!(game.screen_fade_delay = 3u16);
    set_ram_word!(game.screen_fade_counter = 3u16);
    EnableHdmaObjects(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    set_ram_word!(game.door_def_ptr = 0u16);
    set_ram_word!(game.layer1_x_pos = 0u16);
    set_ram_word!(game.layer1_y_pos = 0u16);
    set_ram_word!(game.cinematic_var16 = 0u16);
    j = 1023u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        (*((&raw mut game.g_ram as *mut u8).offset(0x3800) as *mut Ram3800)).cinematic_bg_tilemap
            [j as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset((j as i32 * 2i32 + 11262i32) as isize)
            as *mut u16);
        j = j.wrapping_sub(1);
    }
    k = 127u16;
    while (k as i32 & 0x8000i32) == 0i32 {
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[k as usize] =
            *((game.g_rom.offset(crate::rtl::lorom_offset(0x8cd81b)) as *mut u16 as *mut u8)
                .offset((k as i32 * 2i32) as isize) as *mut u16);
        k = k.wrapping_sub(1);
    }
    set_ram_word!(game.cinematic_var11 = -1i32 as u16);
    set_ram_word!(game.cinematic_function = ((0x8ba5a7i32) as u16));
    QueueMusic_Delayed8(0u16, game);
    QueueMusic_Delayed8(0xff3fu16, game);
}
pub unsafe fn CinematicFunction_Intro_FadeIn(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba5bdi32) as u16));
        set_ram_word!(game.screen_fade_delay = 2u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
    }
}
pub unsafe fn CinematicFunction_Intro_DrawInitJpn(game: &mut Game) {
    if AdvanceSlowScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba5f8i32) as u16));
        set_ram_word!(game.cinematic_var4 = 60u16);
        if ram_word!(game, japanese_text_flag) != 0 {
            let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
            (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 256u16;
            v0 = (v0).wrapping_add(2u16);
            (*gVramWriteEntry(v0 as i32, game)).size = 12288u16;
            v0 = (v0).wrapping_add(2u16);
            let fresh146 = v0;
            v0 = v0.wrapping_add(1);
            *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                fresh146 as i32,
                game,
            ))
            .size as *mut u8)
                .offset(0) = 126u8;
            (*gVramWriteEntry(v0 as i32, game)).size = 20192u16;
            set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2u16));
        }
    }
}
pub unsafe fn CinematicFunction_Intro_LastMetroidCaptivity(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8ba613i32) as u16));
        set_ram_word!(game.cinematic_var4 = 200u16);
        QueueMusic_Delayed8(5u16, game);
    }
}
pub unsafe fn CinematicFunc_Intro_QueueGalaxyIsAtPeace(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8(0xff42u16, game);
        QueueMusic_DelayedY(5u16, 0xeu16, game);
        set_ram_word!(game.cinematic_function = ((0x8ba639i32) as u16));
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForQueueWait4secs(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba64ci32) as u16));
        set_ram_word!(game.cinematic_var4 = 240u16);
    }
}
pub unsafe fn CinematicFunc_Intro_FadeOut(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8ba663i32) as u16));
        set_ram_word!(game.screen_fade_delay = 2u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForFadeOut(game: &mut Game) {
    if AdvanceSlowScreenFadeOut(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba66fi32) as u16));
    }
}
pub unsafe fn CinematicFunc_Intro_SetupTextPage2(game: &mut Game) {
    ClearJapaneseTextTiles(game);
    CinematicAsset::VramWords(0x7e4000, 0x4180, 0x600).load(game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x16, 0x69, false, game);
    ClearCinematicBgObjects(0x2fu16, game);
    EnableCinematicBgObjects(game);
    EnableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.cinematicbg_var3 = addr_unk_604C00 as u16);
    let mut i: i32 = 127i32;
    while i >= 0i32 {
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[i as usize] = 15401u16;
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 896i32) as usize] = 15401u16;
        i -= 1;
    }
    let mut v1: u16 = 768u16;
    let mut v2: u16 = 0u16;
    loop {
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[v1 as usize] =
            crate::rtl::rom_table_word!(game, 0x8ba72b, (v2 as i32 >> 1i32) as isize);
        v2 = (v2).wrapping_add(2u16);
        v1 = v1.wrapping_add(1);
        if (((v1 as i32 * 2i32 - 1792i32) as i16) >= 0i16) {
            break;
        }
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
        .menu
        .menu_tilemap as *mut u8)
        .offset(286) as *mut u32) = 472456233u32;
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCE6D as u16,
        0x1c29u16,
        game,
    );
    set_ram_word!(game.cinematic_var4 = 127u16);
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF63 as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_var10 = 8u16);
    set_ram_word!(game.reg_BG2VOFS = 8u16);
    set_ram_word!(game.reg_BG3VOFS = 8u16);
    ClearEnglishTextTilemap(game);
    SetSomeStuffForSpriteObject_16(game);
    QueueMusic_Delayed8(0u16, game);
    QueueMusic_Delayed8(0xff36u16, game);
    QueueMusic_DelayedY(5u16, 0xeu16, game);
    set_ram_word!(game.cinematic_function = ((0x8ba82bi32) as u16));
}
pub unsafe fn CinematicFunc_Intro_WaitFadeinShowText(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba84ai32) as u16));
        set_ram_word!(game.screen_fade_delay = 2u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
        SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF3F as u16, 0x4c00u16, game);
    }
}
pub unsafe fn CinematicFunc_Intro_WaitForFadeinSleep(game: &mut Game) {
    if AdvanceSlowScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8ba391i32) as u16));
    }
}
pub unsafe fn ClearEnglishTextTilemap(game: &mut Game) {
    let mut v0: u16 = 128u16;
    loop {
        let fresh5 = v0;
        v0 = v0.wrapping_add(1);
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[fresh5 as usize] = 47u16;
        if (((v0 as i32 * 2i32 - 1536i32) as i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn ClearJapaneseTextTiles(game: &mut Game) {
    let mut v0: u16 = 0u16;
    loop {
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [v0 as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7822) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 1i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7824) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 2i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7826) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 3i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7828) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 4i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7830) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 5i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7832) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 6i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7834) as *mut u16);
        (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(v0 as i32 + 7i32) as usize] = *((&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(7836) as *mut u16);
        v0 = (v0).wrapping_add(8u16);
        if (((v0 as i32 * 2i32 - 1536i32) as i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn SetSomeStuffForSpriteObject_2(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 8u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 24u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_3(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 155u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_4_MetroidEgg(k: u16, game: &mut Game) {
    if ram_word!(game, samus_x_pos) as i32 - 169i32 & 0x8000i32 != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCB3B as u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
    }
}
pub unsafe fn CinematicSprPreInstr_A903(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var13) == 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        set_ram_word!(game.cinematic_var15 = 0u16);
    }
}
pub unsafe fn CinematicSprInstr_SpawnMetroidEggParticles(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCECD as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCED3 as u16, 1u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCED9 as u16, 2u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEDF as u16, 3u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEE5 as u16, 4u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEEB as u16, 5u16, game);
    QueueSfx2_Max6(0xbu16, game);
    j
}
static kSetSomeStuffForSpriteObject_6_Tab: [i16; 12] = [
    0x5c, 0x58, 0x63, 0x58, 0x59, 0x5d, 0x60, 0x5b, 0x66, 0x5e, 0x63, 0x60,
];
pub unsafe fn SetSomeStuffForSpriteObject_6(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = ((4u16).wrapping_mul(v1)) as i32 >> 1i32;
    set_ram_word!(
        game.cinematicbg_arr7[v2] =
            ((kSetSomeStuffForSpriteObject_6_Tab[v3 as usize]) as u16).wrapping_add(16u16)
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] =
            ((kSetSomeStuffForSpriteObject_6_Tab[(v3 + 1i32) as usize]) as u16).wrapping_add(59u16)
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 3584u16);
}
static kSetSomeStuffForSpriteObject_7_Tab0: [i16; 12] = [
    -1, 0x4000, 0, 0x4000, -1, -0x8000, -1, 0x2000, 0, -0x8000, 0, 0x2000,
];
static kSetSomeStuffForSpriteObject_7_Tab1: [u16; 82] = [
    0xfffe, 0, 0xfffe, 0xc000, 0xfffe, 0xa000, 0xfffe, 0x8000, 0xfffe, 0x6000, 0xfffe, 0x4000,
    0xfffe, 0x2000, 0xffff, 0, 0xffff, 0xc000, 0xffff, 0xa000, 0xffff, 0x8000, 0xffff, 0x6000,
    0xffff, 0x4000, 0xffff, 0x2000, 0, 0, 0, 0x2000, 0, 0x4000, 0, 0x6000, 0, 0x8000, 0, 0xa000, 0,
    0xc000, 0, 0xe000, 1, 0, 1, 0x2000, 1, 0x4000, 1, 0x6000, 1, 0x8000, 1, 0xa000, 1, 0xc000, 1,
    0xe000, 2, 0, 2, 0x2000, 2, 0x4000, 2, 0x6000, 2, 0x8000, 2, 0xa000, 2, 0xc000, 2, 0xe000,
    0x9dad, 0x991b, 0x1b7d, 0x97ad, 0x991a, 0x1a7d,
];
pub unsafe fn SetSomeStuffForSpriteObject_7(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v3: i32 = 2i32
        * *(((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize)
            as *mut u8)
            .offset(0) as i32;
    '_c2rust_label: {
        if (v3 + 1i32) < 12i32 {
        } else {
            __assert_fail(
                b"v3 + 1 < 12\0" as *const u8 as *const c_char,
                b"src/sm_8b.c\0" as *const u8 as *const c_char,
                2389u32,
                b"void SetSomeStuffForSpriteObject_7(uint16)\0" as *const u8 as *const c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        (kSetSomeStuffForSpriteObject_7_Tab0[v3 as usize] as u32) << 16i32
            | kSetSomeStuffForSpriteObject_7_Tab0[(v3 + 1i32) as usize] as u16 as u32,
    );
    let v6: i32 = 2i32
        * *(((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize)
            as *mut u8)
            .offset(1isize) as i32;
    '_c2rust_label_0: {
        if (v6 + 1i32) < 82i32 {
        } else {
            __assert_fail(
                b"v6 + 1 < 82\0" as *const u8 as *const c_char,
                b"src/sm_8b.c\0" as *const u8 as *const c_char,
                2392u32,
                b"void SetSomeStuffForSpriteObject_7(uint16)\0" as *const u8 as *const c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        (kSetSomeStuffForSpriteObject_7_Tab1[v6 as usize] as u32) << 16i32
            | kSetSomeStuffForSpriteObject_7_Tab1[(v6 + 1i32) as usize] as u32,
    );
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(256u16));
    } else {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_8(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v1] = ram_word!(game, cinematic_spawn_param));
    set_ram_word!(game.cinematicbg_arr7[v1] = ram_word!(game, cinematicbg_arr7[13]));
    set_ram_word!(game.cinematicbg_arr8[v1] = ram_word!(game, cinematicbg_arr8[13]));
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
static kSetSomeStuffForSpriteObject_9_Tab0: [i16; 10] =
    [-1, 0, -1, -0x8000, 1, 0, 0, -0x8000, -1, -0x8000];
static kSetSomeStuffForSpriteObject_9_Tab2: [i16; 124] = [
    -2, 0, -2, -0x4000, -2, -0x6000, -2, -0x8000, -2, 0x6000, -2, 0x4000, -2, 0x2000, -1, 0, -1,
    -0x4000, -1, -0x6000, -1, -0x8000, -1, 0x6000, -1, 0x4000, -1, 0x2000, 0, 0, 0, 0x2000, 0,
    0x4000, 0, 0x6000, 0, -0x8000, 0, -0x6000, 0, -0x4000, 0, -0x2000, 1, 0, 1, 0x2000, 1, 0x4000,
    1, 0x6000, 1, -0x8000, 1, -0x6000, 1, -0x4000, 1, -0x2000, 2, 0, 2, 0x2000, 2, 0x4000, 2,
    0x6000, 2, -0x8000, 2, -0x6000, 2, -0x4000, 2, -0x2000, 3, 0, 3, 0x2000, 3, 0x4000, 3, 0x6000,
    3, -0x8000, 3, -0x6000, 3, -0x4000, 3, -0x2000, 4, 0, 4, 0x2000, 4, 0x4000, 4, 0x6000, 4,
    -0x8000, 4, -0x6000, 4, -0x4000, 4, -0x2000, 5, 0, 5, 0x2000, 5, 0x4000, 5, 0x6000, 5, -0x8000,
    5, -0x6000, 5, -0x4000, 5, -0x2000,
];
static kSetSomeStuffForSpriteObject_9_Tab1: [i16; 138] = [
    -3, 0, -3, -0x4000, -3, -0x6000, -3, -0x8000, -3, 0x6000, -3, 0x4000, -3, 0x2000, -2, 0, -2,
    -0x4000, -2, -0x6000, -2, -0x8000, -2, 0x6000, -2, 0x4000, -2, 0x2000, -1, 0, -1, -0x4000, -1,
    -0x6000, -1, -0x8000, -1, 0x6000, -1, 0x4000, -1, 0x2000, 0, 0, 0, 0x2000, 0, 0x4000, 0,
    0x6000, 0, -0x8000, 0, -0x6000, 0, -0x4000, 0, -0x2000, 1, 0, 1, 0x2000, 1, 0x4000, 1, 0x6000,
    1, -0x8000, 1, -0x6000, 1, -0x4000, 1, -0x2000, 2, 0, 2, 0x2000, 2, 0x4000, 2, 0x6000, 2,
    -0x8000, 2, -0x6000, 2, -0x4000, 2, -0x2000, 3, 0, 3, 0x2000, 3, 0x4000, 3, 0x6000, 3, -0x8000,
    3, -0x6000, 3, -0x4000, 3, -0x2000, 4, 0, 4, 0x2000, 4, 0x4000, 4, 0x6000, 4, -0x8000, 4,
    -0x6000, 4, -0x4000, 4, -0x2000, 5, 0, 5, 0x2000, 5, 0x4000, 5, 0x6000, 5, -0x8000, 5, -0x6000,
    5, -0x4000, 5, -0x2000,
];
pub unsafe fn SetSomeStuffForSpriteObject_9(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v3: i32 = 2i32
        * *(((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize)
            as *mut u8)
            .offset(0) as i32;
    '_c2rust_label: {
        if (v3 + 1i32) < 10i32 {
        } else {
            __assert_fail(
                b"v3 + 1 < 10\0" as *const u8 as *const c_char,
                b"src/sm_8b.c\0" as *const u8 as *const c_char,
                2442u32,
                b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8 as *const c_char,
            );
        }
    };
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        (kSetSomeStuffForSpriteObject_9_Tab0[v3 as usize] as u32) << 16i32
            | kSetSomeStuffForSpriteObject_9_Tab0[(v3 + 1i32) as usize] as u16 as u32,
    );
    if (ram_word!(game, cinematicspr_goto_timer[v1]) as i32 & 1i32) != 0i32 {
        let v12: i32 = 2i32
            * *(((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize)
                as *mut u8)
                .offset(1isize) as i32;
        '_c2rust_label_0: {
            if (v12 + 1i32) < 124i32 {
            } else {
                __assert_fail(
                    b"v12 + 1 < 124\0" as *const u8 as *const c_char,
                    b"src/sm_8b.c\0" as *const u8 as *const c_char,
                    2446u32,
                    b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8 as *const c_char,
                );
            }
        };
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
            (kSetSomeStuffForSpriteObject_9_Tab2[v12 as usize] as u32) << 16i32
                | kSetSomeStuffForSpriteObject_9_Tab2[(v12 + 1i32) as usize] as u16 as u32,
        );
    } else {
        let v6: i32 = 2i32
            * *(((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize)
                as *mut u8)
                .offset(1isize) as i32;
        '_c2rust_label_1: {
            if (v6 + 1i32) < 138i32 {
            } else {
                __assert_fail(
                    b"v6 + 1 < 138\0" as *const u8 as *const c_char,
                    b"src/sm_8b.c\0" as *const u8 as *const c_char,
                    2450u32,
                    b"void SetSomeStuffForSpriteObject_9(uint16)\0" as *const u8 as *const c_char,
                );
            }
        };
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
            (kSetSomeStuffForSpriteObject_9_Tab1[v6 as usize] as u32) << 16i32
                | kSetSomeStuffForSpriteObject_9_Tab1[(v6 + 1i32) as usize] as u16 as u32,
        );
    }
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCD71 as u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
    } else {
        update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(256u16));
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_10(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 84u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 139u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_11(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var13) != 0 {
        if (ram_word!(game, cinematic_var4) as i32 & 3i32) == 0i32
            && ram_word!(game, cinematic_var8) != 0
        {
            update_ram_word!(game.cinematic_var8, wrapping_sub(1));
            update_ram_word!(game.cinematicbg_arr7[(k as i32 >> 1i32)], wrapping_add(1));
        }
    } else {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    };
}
pub unsafe fn SetSomeStuffForSpriteObject_12(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 111u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_13(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var13) != 0 {
        if (ram_word!(game, cinematic_var4) as i32 & 3i32) == 0i32
            && ram_word!(game, cinematic_var10) as i32 - 8i32 & 0x8000i32 != 0
        {
            update_ram_word!(game.cinematic_var10, wrapping_add(1));
            update_ram_word!(game.cinematicbg_arr8[(k as i32 >> 1i32)], wrapping_sub(1));
        }
    } else {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    };
}
pub unsafe fn CinematicBgInstr_SetSomeStuffForSpriteObject_14(
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    set_ram_word!(game.cinematicspr_instr_ptr[15] = addr_unk_8BCC03 as u16);
    set_ram_word!(game.cinematicspr_instr_timer[15] = 1u16);
    j
}
pub unsafe fn SetSomeStuffForSpriteObject_15(game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr7[15] = 8u16);
    set_ram_word!(game.cinematicbg_arr8[15] = 248u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_16(game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr7[15] = 8u16);
    set_ram_word!(game.cinematicbg_arr8[15] = 24u16);
    set_ram_word!(game.cinematicspr_instr_ptr[15] = addr_word_8BCBFB as u16);
    set_ram_word!(game.cinematicspr_instr_timer[15] = 1u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_17(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 216u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn SetSomeStuffForSpriteObject_18(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var16) == 59u16 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCC23 as u16);
    }
    if ram_word!(game, cinematic_var13) == 127u16 {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v2] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v2] = addr_off_8BCE53 as u16);
    }
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page1(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD401 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page1(
    k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    set_ram_word!(game.cinematic_function = ((0x8baeb8i32) as u16));
    j
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page2(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD407 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page2(
    k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD40D as u16, 0u16, game);
    } else {
        set_ram_word!(game.cinematic_function = ((0x8baf6ci32) as u16));
    }
    j
}
pub unsafe fn CinematicFunction_Intro_WaitInputSetupMotherBrainFight(game: &mut Game) {
    if ram_word!(game, cinematic_var16) != 0 {
        update_ram_word!(game.cinematic_var16, wrapping_sub(1));
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 80u8;
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa26) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa1e) as *mut u16);
        set_ram_word!(game.samus_x_pos = 155u16);
        set_ram_word!(game.samus_prev_x_pos = 155u16);
        set_ram_word!(game.samus_y_pos = 115u16);
        set_ram_word!(game.samus_prev_y_pos = 115u16);
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var13 = 127u16);
        SpawnCinematicSpriteObjectAtSlot(
            addr_kCinematicSpriteObjectDef_8BCE55 as u16,
            0u16,
            0x007f, // The preceding $1A4B store leaves $007F in A.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF27 as u16,
            0x0e00, // Palette left in A by the first sprite initializer.
            game,
        );
        let mut v1: u16 = 0u16;
        loop {
            set_ram_word!(
                game.level_data[(v1 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0x8cbec3, (v1 as i32 >> 1i32) as isize)
            );
            v1 = (v1).wrapping_add(2u16);
            if (((v1) as i16).wrapping_sub(448i16) >= 0i16) {
                break;
            }
        }
        let mut v2: u16 = 0u16;
        loop {
            *((&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(v2 as isize) as *mut u16) = 0u16;
            v2 = (v2).wrapping_add(2u16);
            if (((v2) as i16).wrapping_sub(512i16) >= 0i16) {
                break;
            }
        }
        set_ram_word!(game.hud_item_index = 1u16);
        set_ram_word!(game.frame_handler_alfa = ((0x90e6c9i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e833i32) as u16));
        ResetDemoData(game);
        EnableDemoInput(game);
        // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
        LoadDemoInputObject(0x8000, addr_stru_918784 as u16, game);
        set_ram_word!(game.cinematic_var15 = -1i32 as u16);
        CinematicFunction_Intro_SetupTransitionToGameplay(game);
    }
}
pub unsafe fn CinematicFunction_Intro_WaitInputSetupBabyMetroid(game: &mut Game) {
    if ram_word!(game, cinematic_var16) != 0 {
        update_ram_word!(game.cinematic_var16, wrapping_sub(1));
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 84u8;
        set_ram_word!(game.room_width_in_blocks = 32u16);
        set_ram_word!(game.room_height_in_blocks = 16u16);
        set_ram_word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa26) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa1e) as *mut u16);
        set_ram_word!(game.samus_x_pos = 376u16);
        set_ram_word!(game.samus_prev_x_pos = 376u16);
        set_ram_word!(game.samus_y_pos = 147u16);
        set_ram_word!(game.samus_prev_y_pos = 147u16);
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var13 = 127u16);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE5B as u16, 0x7fu16, game);
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE79 as u16,
            0x0e00, // Palette left in A by the first sprite initializer.
            game,
        );
        set_ram_word!(game.cinematic_var11 = 0u16);
        let mut v1: u16 = 0u16;
        loop {
            set_ram_word!(
                game.level_data[(v1 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0x8cc083, (v1 as i32 >> 1i32) as isize)
            );
            v1 = (v1).wrapping_add(2u16);
            if (((v1) as i16).wrapping_sub(768i16) >= 0i16) {
                break;
            }
        }
        set_ram_word!(game.hud_item_index = 0u16);
        set_ram_word!(game.frame_handler_alfa = ((0x90e6c9i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e833i32) as u16));
        ResetDemoData(game);
        EnableDemoInput(game);
        // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
        LoadDemoInputObject(0x8000, addr_stru_91877E as u16, game);
        set_ram_word!(game.cinematic_var15 = 1u16);
        CinematicFunction_Intro_SetupTransitionToGameplay(game);
    }
}
pub unsafe fn CinematicFunction_Intro_SetupTransitionToGameplay(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 6u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    SpawnIntroCutsceneCrossfadeHdma(game);
    SetSomeStuffForSpriteObject_15(game);
    set_ram_word!(game.cinematic_function = ((0x8bb250i32) as u16));
    let mut v0: u16 = 0u16;
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x2000) as *mut u16)
            .offset(((v0 as i32 >> 1i32) + 256i32) as isize) =
            crate::rtl::rom_table_word!(game, 0x8ce3e9, (v0 as i32 >> 1i32) as isize);
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x28u16, 3u16, game);
    ClearYColorsFromIndexX(0xe0u16, 0x10u16, game);
    ClearYColorsFromIndexX(0x180u16, 0x20u16, game);
    ClearYColorsFromIndexX(0x1e0u16, 0x10u16, game);
    ComposeFadingPalettes(game);
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page3(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD413 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page3(
    k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD419 as u16, 0u16, game);
    } else {
        set_ram_word!(game.cinematic_function = ((0x8bb0f2i32) as u16));
    }
    j
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page4(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD41F as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page4(
    k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD425 as u16, 0u16, game);
    } else {
        set_ram_word!(game.cinematic_function = ((0x8bb123i32) as u16));
    }
    j
}
pub unsafe fn CinematicFunction_Intro_Func11(game: &mut Game) {
    if ram_word!(game, cinematic_var16) != 0 {
        update_ram_word!(game.cinematic_var16, wrapping_sub(1));
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 88u8;
        set_ram_word!(game.cinematic_var8 = 32u16);
        set_ram_word!(game.cinematic_var10 = 8u16);
        set_ram_word!(game.cinematic_var13 = 127u16);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE61 as u16, 0x7fu16, game);
        CinematicFunction_Intro_Func13(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func12(game: &mut Game) {
    if ram_word!(game, cinematic_var16) != 0 {
        update_ram_word!(game.cinematic_var16, wrapping_sub(1));
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 92u8;
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var10 = -24i32 as u16);
        set_ram_word!(game.cinematic_var13 = 127u16);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE67 as u16, 0x7fu16, game);
        CinematicFunction_Intro_Func13(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func13(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 6u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    SpawnIntroCutsceneCrossfadeHdma(game);
    SetSomeStuffForSpriteObject_15(game);
    set_ram_word!(game.cinematic_function = ((0x8bb2d2i32) as u16));
    let mut v0: u16 = 0u16;
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x2000) as *mut u16)
            .offset(((v0 as i32 >> 1i32) + 256i32) as isize) =
            crate::rtl::rom_table_word!(game, 0x8ce3e9, (v0 as i32 >> 1i32) as isize);
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x40u16, 0x10u16, game);
    ClearYColorsFromIndexX(0x1c0u16, 9u16, game);
    ComposeFadingPalettes(game);
}
pub unsafe fn CinematicBgInstr_HandleCreateJpnText_Page5(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD42B as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_SpawnMarkerWaitInput_Page5(
    k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
        SpawnMode7Object(addr_kMode7ObjectDef_8BD431 as u16, 0u16, game);
    } else {
        set_ram_word!(game.cinematic_function = ((0x8bb1dai32) as u16));
    }
    j
}
pub unsafe fn CinematicFunction_Intro_Func15b(game: &mut Game) {
    EnableCinematicBgTilemapUpdates(game);
    ClearEnglishTextTilemap(game);
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF5D as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_function = ((0x8ba390i32) as u16));
    SetSomeStuffForSpriteObject_16(game);
    QueueMusic_Delayed8(0u16, game);
}
pub unsafe fn CinematicFunction_Intro_Func15(game: &mut Game) {
    if ram_word!(game, cinematic_var16) != 0 {
        update_ram_word!(game.cinematic_var16, wrapping_sub(1));
    } else if ram_word!(game, joypad1_newkeys) != 0 {
        set_ram_word!(game.cinematic_var13 = 127u16);
        if ram_word!(game, japanese_text_flag) != 0 {
            set_ram_word!(game.cinematic_function = ((0x8bb207i32) as u16));
            DisableCinematicBgTilemapUpdates(game);
            ClearJapaneseTextTiles(game);
            TransferJapaneseTextTilesToVram(game);
        } else {
            CinematicFunction_Intro_Func15b(game);
        }
    }
}
pub unsafe fn CinematicBgInstr_Func16(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_var1 = 0u16);
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnMode7Object(
            addr_kMode7ObjectDef_8BD437 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicBgInstr_Func17(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bb72fi32) as u16));
    set_ram_word!(game.screen_fade_delay = 1u16);
    set_ram_word!(game.screen_fade_counter = 1u16);
    j
}
pub unsafe fn CinematicFunction_Intro_XfadeGameplayFade(game: &mut Game) {
    if (ram_word!(game, cinematic_var4) as i32 & 3i32) == 0i32 {
        FadeOutYColorsFromIndexX(0u16, 0x14u16, game);
        FadeOutYColorsFromIndexX(0x60u16, 0x10u16, game);
        FadeOutYColorsFromIndexX(0x1d2u16, 6u16, game);
        FadeInYColorsFromIndexX(0x28u16, 3u16, game);
        FadeInYColorsFromIndexX(0xe0u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x180u16, 0x20u16, game);
        FadeInYColorsFromIndexX(0x1e0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
    }
    if (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 21u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        let mut v0: u16 = 128u16;
        loop {
            let fresh134 = v0;
            v0 = v0.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[fresh134 as usize] = 47u16;
            if (((v0 as i32 * 2i32 - 1536i32) as i16) >= 0i16) {
                break;
            }
        }
        ClearJapaneseTextTiles(game);
        set_ram_word!(game.cinematic_function = ((0x8ba391i32) as u16));
        if (ram_word!(game, cinematic_var15) as i32 & 0x8000i32) != 0i32 {
            SpawnPalfxObject(addr_kPalfx_E1BC as u16, game);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_XfadeScientistFade(game: &mut Game) {
    if (ram_word!(game, cinematic_var4) as i32 & 3i32) == 0i32 {
        FadeOutYColorsFromIndexX(0u16, 0x14u16, game);
        FadeOutYColorsFromIndexX(0x60u16, 0x10u16, game);
        FadeOutYColorsFromIndexX(0x1d2u16, 6u16, game);
        FadeInYColorsFromIndexX(0x40u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x1c0u16, 9u16, game);
        ComposeFadingPalettes(game);
    }
    if (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 21u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        let mut v0: u16 = 128u16;
        loop {
            let fresh132 = v0;
            v0 = v0.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[fresh132 as usize] = 47u16;
            if (((v0 as i32 * 2i32 - 1536i32) as i16) >= 0i16) {
                break;
            }
        }
        ClearJapaneseTextTiles(game);
        set_ram_word!(game.cinematic_function = ((0x8ba391i32) as u16));
    }
}
pub unsafe fn CinematicSprInstr_StartIntroPage2(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bb35fi32) as u16));
    Instr_StartIntroPage_Common(game);
    j
}
pub unsafe fn CinematicSprInstr_StartIntroPage3(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bb370i32) as u16));
    Instr_StartIntroPage_Common(game);
    j
}
pub unsafe fn CinematicSprInstr_StartIntroPage4(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bb381i32) as u16));
    Instr_StartIntroPage_Common(game);
    j
}
pub unsafe fn CinematicSprInstr_StartIntroPage5(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bb392i32) as u16));
    Instr_StartIntroPage_Common(game);
    j
}
pub unsafe fn Instr_StartIntroPage_Common(game: &mut Game) {
    DisableCinematicBgTilemapUpdates(game);
    ClearJapaneseTextTiles(game);
    TransferJapaneseTextTilesToVram(game);
}
pub unsafe fn CinematicFunction_Intro_SetupStuff(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 6u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
    let mut v0: u16 = 0u16;
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x2000) as *mut u16)
            .offset(((v0 as i32 >> 1i32) + 256i32) as isize) =
            crate::rtl::rom_table_word!(game, 0x8ce3e9, (v0 as i32 >> 1i32) as isize);
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0u16, 0x10u16, game);
    ClearYColorsFromIndexX(0x60u16, 0x10u16, game);
    ClearYColorsFromIndexX(0x1d2u16, 6u16, game);
    ComposeFadingPalettes(game);
    set_ram_word!(game.cinematic_var4 = 127u16);
    EnableCinematicBgTilemapUpdates(game);
    SetSomeStuffForSpriteObject_16(game);
}
pub unsafe fn CinematicFunction_Intro_Page2(game: &mut Game) {
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF45 as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bb3f4i32) as u16));
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page3(game: &mut Game) {
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF4B as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bb3f4i32) as u16));
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page4(game: &mut Game) {
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF51 as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bb458i32) as u16));
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Page5(game: &mut Game) {
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF57 as u16, 0x4c00u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bb458i32) as u16));
    CinematicFunction_Intro_SetupStuff(game);
}
pub unsafe fn CinematicFunction_Intro_Fadestuff(game: &mut Game) {
    if (ram_word!(game, cinematic_var13) as i32 & 3i32) == 0i32 {
        FadeInYColorsFromIndexX(0u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x60u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x1d2u16, 6u16, game);
        FadeOutYColorsFromIndexX(0x28u16, 3u16, game);
        FadeOutYColorsFromIndexX(0xe0u16, 0x10u16, game);
        FadeOutYColorsFromIndexX(0x180u16, 0x20u16, game);
        FadeOutYColorsFromIndexX(0x1e0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
    }
    if (update_ram_word!(game.cinematic_var13, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 22u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        set_ram_word!(game.cinematic_function = ((0x8ba391i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Fadestuff2(game: &mut Game) {
    if (ram_word!(game, cinematic_var13) as i32 & 3i32) == 0i32 {
        FadeInYColorsFromIndexX(0u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x60u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x1d2u16, 6u16, game);
        FadeOutYColorsFromIndexX(0x40u16, 0x10u16, game);
        FadeOutYColorsFromIndexX(0x1c0u16, 9u16, game);
        ComposeFadingPalettes(game);
    }
    if (update_ram_word!(game.cinematic_var13, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 22u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        let mut v0: u16 = 0u16;
        loop {
            *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                .offset(((v0 as i32 >> 1i32) + 224i32) as isize) = crate::rtl::rom_table_word!(
                game,
                0x8ce3e9,
                ((v0 as i32 >> 1i32) + 224i32) as isize
            );
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(14i16) >= 0i16) {
                break;
            }
        }
        set_ram_word!(game.cinematic_function = ((0x8ba391i32) as u16));
    }
}
pub unsafe fn CinematicBgPreInstr_SamusBlink(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bb207i32 & 0xffffi32
        || ram_word!(game, cinematic_function) as i32 == 0x8ba390i32 & 0xffffi32
    {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicbg_instr_ptr[v1] = -10733i32 as u16);
        set_ram_word!(game.cinematicbg_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicbg_preinstr[v1] = ((0x8b93d9i32) as u16));
    }
}
pub unsafe fn CallCreditsObjectFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        9147353 => (),
        9155772 => CinematicBgPreInstr_SamusBlink(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func19(k: u16, game: &mut Game) {
    if ram_word!(game, cinematicspr_instr_ptr[15]) as i32 + 0x33fdi32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicbg_instr_timer[(k as i32 >> 1i32)] = 1u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func18(game: &mut Game) {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    CinematicFunction_Intro_Func20(addr_word_8BD389 as u16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD389 as u16, game);
    TransferJapaneseTextTilesToVram(game);
}
pub unsafe fn sub_8BB51E(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BCF75 as u16, game);
    CinematicFunction_Intro_Func20(addr_word_8BCFBD as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn EnableCinematicBgTilemapUpdates__0(k: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func21(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BCFFD as u16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD055 as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func22(k: u16, game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.mode7_obj_instr_ptr[v1] = -11355i32 as u16);
        set_ram_word!(game.mode7_obj_instr_timer[v1] = 1u16);
        set_ram_word!(game.mode7_obj_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD085 as u16, game);
        CinematicFunction_Intro_Func20(addr_word_8BD0E1 as u16, game);
        TransferJapaneseTextTilesToVram(game);
        set_ram_word!(game.cinematic_var16 = 60u16);
    }
}
pub unsafe fn CinematicFunction_Intro_ThenWaitInputSetupBabyMetroid(
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.cinematic_function = ((0x8baf6ci32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func23(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD0F9 as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func24(k: u16, game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.mode7_obj_instr_ptr[v1] = -11333i32 as u16);
        set_ram_word!(game.mode7_obj_instr_timer[v1] = 1u16);
        set_ram_word!(game.mode7_obj_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD15D as u16, game);
        TransferJapaneseTextTilesToVram(game);
        set_ram_word!(game.cinematic_var16 = 60u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func25(k: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.cinematic_function = ((0x8bb0f2i32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func26(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD1B9 as u16, game);
    CinematicFunction_Intro_Func20(addr_word_8BD215 as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func27(k: u16, game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.mode7_obj_instr_ptr[v1] = -11311i32 as u16);
        set_ram_word!(game.mode7_obj_instr_timer[v1] = 1u16);
        set_ram_word!(game.mode7_obj_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD259 as u16, game);
        CinematicFunction_Intro_Func20(addr_word_8BD2A5 as u16, game);
        TransferJapaneseTextTilesToVram(game);
        set_ram_word!(game.cinematic_var16 = 60u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func28(k: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.cinematic_function = ((0x8bb123i32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func29(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
    set_ram_word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
    set_ram_word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD2D5 as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func30(k: u16, game: &mut Game) {
    if ram_word!(game, joypad1_newkeys) != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.mode7_obj_instr_ptr[v1] = addr_off_8BD3E7 as u16);
        set_ram_word!(game.mode7_obj_instr_timer[v1] = 1u16);
        set_ram_word!(game.mode7_obj_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        DisableCinematicBgTilemapUpdates(game);
        ClearJapaneseTextTiles(game);
        CinematicFunction_Intro_Func20(addr_word_8BD30D as u16, game);
        TransferJapaneseTextTilesToVram(game);
        set_ram_word!(game.cinematic_var16 = 60u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func31(k: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    set_ram_word!(game.cinematic_function = ((0x8bb1dai32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func32(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    ClearJapaneseTextTiles(game);
    CinematicFunction_Intro_Func20(addr_word_8BD371 as u16, game);
    TransferJapaneseTextTilesToVram(game);
    j
}
pub unsafe fn EnableCinematicBgTilemapUpdates__(k: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func33(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) != 0i32 {
        let ref mut fresh169 = *(&raw mut game.g_ram as *mut u8).offset(0x69);
        *fresh169 = (*fresh169 as i32 & !(2i32)) as u8;
    } else {
        let ref mut fresh170 = *(&raw mut game.g_ram as *mut u8).offset(0x69);
        *fresh170 = (*fresh170) | (2u8);
    };
}
pub unsafe fn CinematicFunction_Intro_Func34(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.reg_BG2HOFS = 0u16);
        set_ram_word!(game.reg_BG3HOFS = 0u16);
        set_ram_word!(game.reg_BG2VOFS = 0u16);
        set_ram_word!(game.reg_BG3VOFS = 0u16);
        set_ram_word!(game.cinematic_var5 = 0u16);
        let mut i: i32 = 656i32;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
            i -= 2i32;
        }
        set_ram_word!(game.cinematic_function = ((0x8bbca0i32) as u16));
        RevertButtonConfig(game);
        set_ram_word!(game.samus_max_missiles = 0u16);
        set_ram_word!(game.samus_missiles = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func35(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 56u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 111u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func36(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func37(k, game);
    let mut v1: u16 = 8u16;
    while (ram_word!(game, projectile_type[(v1 as i32 >> 1i32)]) as i32 & 0xfffi32) != 256i32 {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    if ram_word!(game, projectile_x_pos[(v1 as i32 >> 1i32)]) as i32 - 84i32 & 0x8000i32 != 0 {
        KillProjectile(v1, game);
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_arr6[v2] = 8u16);
        let v3: u16 = (ram_word!(game, cinematicspr_goto_timer[v2])).wrapping_add(1u16);
        set_ram_word!(game.cinematicspr_goto_timer[v2] = v3);
        if v3 == 4u16 {
            set_ram_word!(game.cinematicspr_goto_timer[v2] = 0u16);
            set_ram_word!(game.cinematicspr_preinstr_func[v2] = ((0x8bb80fi32) as u16));
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 1u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 2u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 1u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 2u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 3u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 4u16, game);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func39(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func37(k, game);
    CinematicFunction_Intro_Func40(game);
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = (ram_word!(game, cinematicspr_goto_timer[v1])).wrapping_add(1u16);
    set_ram_word!(game.cinematicspr_goto_timer[v1] = v2);
    if v2 as i32 - 128i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCB19 as u16);
    }
}
pub unsafe fn CinematicSprPreInstr_B82E(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func40(game);
    if ram_word!(game, cinematic_var13) == 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        set_ram_word!(game.cinematic_var15 = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func37(k: u16, game: &mut Game) {
    let mut j: u16 = 0;
    let mut v3: u16 = 0;
    let v1: u16 = ram_word!(game, cinematicspr_arr6[(k as i32 >> 1i32)]);
    if v1 != 0 {
        if (v1 as i32 & 1i32) != 0i32 {
            let mut i: i32 = 30i32;
            while i >= 0i32 {
                *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                    .offset(((i >> 1i32) + 240i32) as isize) =
                    crate::rtl::rom_table_word!(game, 0x8ce3e9, ((i >> 1i32) + 240i32) as isize);
                i -= 2i32;
            }
            v3 = k;
        } else {
            j = 30u16;
            while (j as i32 & 0x8000i32) == 0i32 {
                *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                    .offset(((j as i32 >> 1i32) + 240i32) as isize) = 0x7fffu16;
                j = (j).wrapping_sub(2u16);
            }
            v3 = k;
        }
        update_ram_word!(game.cinematicspr_arr6[(v3 as i32 >> 1i32)], wrapping_sub(1));
    }
}
pub unsafe fn CinematicFunction_Intro_Func40(game: &mut Game) {
    if (ram_word!(game, cinematic_var12) as i32 & 1i32) != 0i32 {
        update_ram_word!(game.cinematic_var10, wrapping_sub(4u16));
    } else {
        update_ram_word!(game.cinematic_var10, wrapping_add(4u16));
    };
}
static kCinematicFunction_Intro_Func42_Tab0: [u16; 4] = [0x70, 0xc0, 0x80, 0xe8];
static kCinematicFunction_Intro_Func42_Tab1: [u16; 4] = [0x50, 0x40, 0x38, 0x58];
pub unsafe fn CinematicFunction_Intro_Func42(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = v1 as i32;
    set_ram_word!(game.cinematicbg_arr7[v2] = kCinematicFunction_Intro_Func42_Tab0[v3 as usize]);
    set_ram_word!(
        game.cinematicbg_arr8[v2] =
            (kCinematicFunction_Intro_Func42_Tab1[v3 as usize]).wrapping_sub(8u16)
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 3584u16);
}
pub unsafe fn CinematicSprInstr_Func43(k: u16, j: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicspr_goto_timer[v1]) != 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bb93bi32) as u16));
    } else {
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bb8d8i32) as u16));
    }
    j
}
pub unsafe fn CinematicFunction_Intro_Func44(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        0x8000u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        0x8000u32,
    );
    if ram_word!(game, cinematicbg_arr7[v1]) as i32 + 8i32
        - (ram_word!(game, samus_x_pos) as i32 - 5i32)
        & 0x8000i32
        != 0
    {
        if ram_word!(game, cinematicspr_preinstr_func[0]) as i32 != 0x8bb80fi32 & 0xffffi32 {
            return;
        }
    } else {
        set_ram_word!(game.samus_invincibility_timer = 11u16);
        set_ram_word!(game.samus_knockback_timer = 11u16);
        set_ram_word!(game.knockback_x_dir = 1u16);
    }
    set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
    set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
}
static kCinematicFunction_Intro_Func45_Tab0: [u16; 4] = [0u16, 0xffffu16, 0u16, 0xffffu16];
pub unsafe fn CinematicFunction_Intro_Func45(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: i32 = ram_word!(game, cinematicspr_goto_timer[v1]) as i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        (kCinematicFunction_Intro_Func45_Tab0[v2 as usize] as u32) << 16i32 | 0x8000u16 as u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        0x8000u32,
    );
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 - 16i32 & 0x8000i32 != 0
        || ram_word!(game, cinematicbg_arr8[v1]) as i32 - 208i32 & 0x8000i32 == 0
        || ram_word!(game, cinematicspr_preinstr_func[0]) as i32 == 0x8bb80fi32 & 0xffffi32
    {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    }
}
static kCinematicFunction_Intro_Func46_Tab0: [u16; 5] = [0, 0x10, 0xfff0, 0xfff8, 8];
static kCinematicFunction_Intro_Func46_Tab1: [u16; 5] = [0, 0xfff0, 8, 0xfff0, 8];
static kCinematicFunction_Intro_Func46_Tab2: [u16; 5] = [1, 0x10, 0x20, 0x30, 0x40];
pub unsafe fn CinematicFunction_Intro_Func46(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = v1 as i32;
    set_ram_word!(
        game.cinematicbg_arr7[v2] =
            (kCinematicFunction_Intro_Func46_Tab0[v3 as usize]).wrapping_add(56u16)
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] =
            (kCinematicFunction_Intro_Func46_Tab1[v3 as usize]).wrapping_add(111u16)
    );
    set_ram_word!(
        game.cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func46_Tab2[v3 as usize]
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 2560u16);
}
static kCinematicFunction_Intro_Func47_Tab0: [u16; 3] = [0x10u16, 0xfff0u16, 0xfff0u16];
static kCinematicFunction_Intro_Func47_Tab1: [u16; 3] = [0u16, 4u16, 0xfff8u16];
static kCinematicFunction_Intro_Func47_Tab2: [u16; 3] = [1u16, 8u16, 0x10u16];
pub unsafe fn CinematicFunction_Intro_Func47(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = v1 as i32;
    set_ram_word!(
        game.cinematicbg_arr7[v2] =
            (kCinematicFunction_Intro_Func47_Tab0[v3 as usize]).wrapping_add(56u16)
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] =
            (kCinematicFunction_Intro_Func47_Tab1[v3 as usize]).wrapping_add(111u16)
    );
    set_ram_word!(
        game.cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func47_Tab2[v3 as usize]
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func48(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var13) == 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    }
}
pub unsafe fn CinematicSprInstr_SpawnIntroRinkas01(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 1u16, game);
    j
}
pub unsafe fn CinematicSprInstr_SpawnIntroRinkas23(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 2u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 3u16, game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func49(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 112u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 155u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func50(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicspr_instr_ptr[(v1 + 1i32)]) as i32 + 0x3487i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bba73i32) as u16));
        set_ram_word!(game.cinematic_var14 = 0u16);
        set_ram_word!(game.cinematic_var17 = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func51(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicbg_arr8[v1]) == 145u16 {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 0u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 1u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 2u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 3u16, game);
        QueueSfx3_Max6(0x23u16, game);
    }
    if ((ram_word!(game, samus_y_pos) as i32 - 32i32 - ram_word!(game, cinematicbg_arr8[v1]) as i32)
        as i16)
        < 0i16
    {
        if ram_word!(game, cinematic_var17) as i32 + 543i32 & 0x8000i32 == 0 {
            update_ram_word!(game.cinematic_var17, wrapping_sub(32u16));
        }
    } else if ram_word!(game, cinematic_var17) as i32 - 544i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var17, wrapping_add(32u16));
    }
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        ((ram_word!(game, cinematic_var17) as i16 as i32) << 8i32) as u32,
    );
    if (ram_word!(game, cinematic_var17) as i32 & 0x8000i32) == 0i32 {
        set_ram_word!(game.cinematic_var14 = 128u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbb0di32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func52(k: u16, game: &mut Game) {
    let v1: bool = (update_ram_word!(game.cinematic_var14, wrapping_sub(1), reread) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, cinematic_var14) == 0 || v1 {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_preinstr_func[v2] = ((0x8bbb24i32) as u16));
        set_ram_word!(game.cinematic_var14 = 0u16);
        set_ram_word!(game.cinematic_var17 = 0u16);
        set_ram_word!(game.cinematicspr_goto_timer[v2] = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func53(k: u16, game: &mut Game) {
    let kh: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematic_var13) != 0 {
        if ram_word!(game, cinematicspr_goto_timer[kh]) as i32 - 128i32 & 0x8000i32 != 0 {
            let v3: u16 = (ram_word!(game, cinematicspr_goto_timer[kh])).wrapping_add(1u16);
            set_ram_word!(game.cinematicspr_goto_timer[kh] = v3);
            if (v3 as i32 & 0x3fi32) == 0i32 {
                QueueSfx3_Max6(0x23u16, game);
            }
        }
        if ((ram_word!(game, samus_x_pos)) as i16)
            .wrapping_sub((ram_word!(game, cinematicbg_arr7[kh])) as i16)
            < 0i16
        {
            if ram_word!(game, cinematic_var14) as i32 + 639i32 & 0x8000i32 == 0 {
                update_ram_word!(game.cinematic_var14, wrapping_sub(32u16));
            }
        } else if ram_word!(game, cinematic_var14) as i32 - 640i32 & 0x8000i32 != 0 {
            update_ram_word!(game.cinematic_var14, wrapping_add(32u16));
        }
        set_ram_word!(
            game.cinematic_var15 = (if ram_word!(game, cinematic_var14) as i32 & 0x8000i32 != 0 {
                1i32
            } else {
                -1i32
            }) as u16
        );
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(kh as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(kh as isize),
            ((ram_word!(game, cinematic_var14) as i16 as i32) << 8i32) as u32,
        );
        if ((ram_word!(game, samus_y_pos) as i32
            - 8i32
            - ram_word!(game, cinematicbg_arr8[kh]) as i32) as i16)
            < 0i16
        {
            if ram_word!(game, cinematic_var17) as i32 + 543i32 & 0x8000i32 == 0 {
                update_ram_word!(game.cinematic_var17, wrapping_sub(32u16));
            }
        } else if ram_word!(game, cinematic_var17) as i32 - 544i32 & 0x8000i32 != 0 {
            update_ram_word!(game.cinematic_var17, wrapping_add(32u16));
        }
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(kh as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(kh as isize),
            ((ram_word!(game, cinematic_var17) as i16 as i32) << 8i32) as u32,
        );
    } else {
        set_ram_word!(game.cinematicspr_instr_timer[kh] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[kh] = addr_off_8BCE53 as u16);
        set_ram_word!(game.cinematic_var15 = 0u16);
    };
}
pub unsafe fn ResetButtonAssignmentsToDefault(game: &mut Game) {
    set_ram_word!(game.button_config_shoot_x_saved = ram_word!(game, button_config_shoot_x));
    set_ram_word!(game.button_config_jump_a_saved = ram_word!(game, button_config_jump_a));
    set_ram_word!(game.button_config_run_b_saved = ram_word!(game, button_config_run_b));
    set_ram_word!(
        game.button_config_itemcancel_y_saved = ram_word!(game, button_config_itemcancel_y)
    );
    set_ram_word!(game.button_config_itemswitch_saved = ram_word!(game, button_config_itemswitch));
    set_ram_word!(game.button_config_aim_down_L_saved = ram_word!(game, button_config_aim_down_L));
    set_ram_word!(game.button_config_aim_up_R_saved = ram_word!(game, button_config_aim_up_R));
    set_ram_word!(game.button_config_up = kButton_Up as u16);
    set_ram_word!(game.button_config_down = kButton_Down as u16);
    set_ram_word!(game.button_config_left = kButton_Left as u16);
    set_ram_word!(game.button_config_right = kButton_Right as u16);
    set_ram_word!(game.button_config_shoot_x = kButton_X as u16);
    set_ram_word!(game.button_config_jump_a = kButton_A as u16);
    set_ram_word!(game.button_config_run_b = kButton_B as u16);
    set_ram_word!(game.button_config_itemcancel_y = kButton_Y as u16);
    set_ram_word!(game.button_config_itemswitch = kButton_Select as u16);
    set_ram_word!(game.button_config_aim_down_L = kButton_L as u16);
    set_ram_word!(game.button_config_aim_up_R = kButton_R as u16);
}
pub unsafe fn RevertButtonConfig(game: &mut Game) {
    set_ram_word!(game.button_config_shoot_x = ram_word!(game, button_config_shoot_x_saved));
    set_ram_word!(game.button_config_jump_a = ram_word!(game, button_config_jump_a_saved));
    set_ram_word!(game.button_config_run_b = ram_word!(game, button_config_run_b_saved));
    set_ram_word!(
        game.button_config_itemcancel_y = ram_word!(game, button_config_itemcancel_y_saved)
    );
    set_ram_word!(game.button_config_itemswitch = ram_word!(game, button_config_itemswitch_saved));
    set_ram_word!(game.button_config_aim_down_L = ram_word!(game, button_config_aim_down_L_saved));
    set_ram_word!(game.button_config_aim_up_R = ram_word!(game, button_config_aim_up_R_saved));
}
pub unsafe fn CinematicFunction_Intro_Func54(game: &mut Game) {
    let mut v1: i16 = 0;
    SetupPpu_3_Mode7(game);
    ClearCinematicSprites(game);
    set_ram_word!(game.cur_irq_handler = 0u16);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    let mut v0: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v0 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce5e9, (v0 as i32 >> 1i32) as isize)
        );
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x95a82f, 0x10000).load(game);
    CinematicAsset::Decompress(0x96fe69, 0x14000).load(game);
    CinematicAsset::Decompress(0x96d10a, 0x15000).load(game);
    CinematicAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
    WriteReg(VMAIN as u16, 0u8, game);
    WriteRegWord(VMADDL as u16, 0u16, game);
    v1 = 0x4000i16;
    loop {
        WriteReg(VMDATAL as u16, 0x8cu8, game);
        v1 -= 1;
        if (v1 == 0) {
            break;
        }
    }
    CinematicAsset::VramLowBytes(0x7f4000, 0x0000, 0x300).load(game);
    CinematicAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x38u16, game);
    set_ram_word!(game.reg_M7X = 56u16);
    WriteRegWord(M7Y as u16, 0x18u16, game);
    set_ram_word!(game.reg_M7Y = 24u16);
    set_ram_word!(game.cinematic_var8 = -72i32 as u16);
    set_ram_word!(game.cinematic_var10 = -104i32 as u16);
    set_ram_word!(game.cinematic_var5 = 224u16);
    set_ram_word!(game.cinematic_var6 = 512u16);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    ClearCinematicBgObjects(0u16, game);
    set_ram_word!(game.cinematicbg_var3 = 23552u16);
    EnableNMI(game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 0u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bbde4i32) as u16));
    QueueMusic_Delayed8(0xff2du16, game);
    QueueMusic_DelayedY(5u16, 0xeu16, game);
}
pub unsafe fn CinematicFunction_Intro_Func55(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x51) = 15u8;
        set_ram_word!(game.cinematic_function = ((0x8bbdf9i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func56(game: &mut Game) {
    if ram_word!(game, cinematic_var6) as i32 - 32i32 & 0x8000i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 49u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 63u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 95u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = -97i32 as u8;
        ClearCinematicSprites(game);
        QueueMode7Transfers(0x8bu8, addr_kCinematicFunction_Intro_Func56_M7 as u16, game);
        set_ram_word!(game.cinematic_var8 = -32i32 as u16);
        set_ram_word!(game.cinematic_var10 = -128i32 as u16);
        set_ram_word!(game.cinematic_var5 = 32u16);
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCF39 as u16,
            0x0020, // The preceding $198D store leaves $0020 in A.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE85 as u16,
            0x0800, // Palette left in A by the preceding sprite initializer.
            game,
        );
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCE8B as u16,
            0x0c00, // Palette left in A by the preceding sprite initializer.
            game,
        );
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE91 as u16, 1u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 1u16, game);
        set_ram_word!(game.cinematic_function = ((0x8bbfdai32) as u16));
        SpawnPalfxObject(addr_kPalfx_E1A8 as u16, game);
        SpawnPalfxObject(addr_kPalfx_E1AC as u16, game);
    } else {
        update_ram_word!(game.cinematic_var6, wrapping_sub(16u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func57(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    if ram_word!(game, cinematic_spawn_param) != 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbfc6i32) as u16));
        set_ram_word!(game.cinematicbg_arr7[v1] = -32i32 as u16);
    } else {
        set_ram_word!(game.cinematicspr_goto_timer[v1] = -1024i32 as u16);
        set_ram_word!(game.cinematicbg_arr7[v1] = 112u16);
    }
    set_ram_word!(game.cinematicbg_arr8[v1] = 87u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func58(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bbdf9i32 & 0xffffi32 {
        let v1: i32 = k as i32 >> 1i32;
        update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(128u16));
        let d: i32 = (ram_word!(game, cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
            d as u32,
        );
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
            ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
            d as u32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
            d as u32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
            d as u32,
        );
    }
}
pub unsafe fn CinematicFunction_Intro_Func59(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 80u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 159u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func60(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        0x4000u32,
    );
    update_ram_word!(game.cinematicbg_arr7[v1], |fresh48| (fresh48) & (0x1ffu16));
}
pub unsafe fn CinematicFunction_Intro_Func61(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 116u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 160u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn CinematicFunction_Intro_Func62(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        4096u32,
    );
    update_ram_word!(game.cinematicbg_arr7[v1], |fresh47| (fresh47) & (0x1ffu16));
}
pub unsafe fn CinematicFunction_Intro_Func63(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 96u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func64(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        2048u32,
    );
    update_ram_word!(game.cinematicbg_arr7[v1], |fresh46| (fresh46) & (0x1ffu16));
}
pub unsafe fn CinematicFunction_Intro_Func65(j: u16, game: &mut Game) {
    if ram_word!(game, cinematic_spawn_param) != 0 {
        set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 224u16);
    } else {
        let v1: i32 = j as i32 >> 1i32;
        set_ram_word!(game.cinematicbg_arr7[v1] = 112u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbfd9i32) as u16));
    }
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr8[v2] = 87u16);
    set_ram_word!(game.cinematicbg_arr9[v2] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func66(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        -0x2000i32 as u32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func67(game: &mut Game) {
    let ref mut fresh123 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0);
    *fresh123 = (*fresh123).wrapping_sub(1);
    if *fresh123 as i32 - 32i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    }
    let ref mut fresh124 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1);
    *fresh124 = (*fresh124).wrapping_sub(1);
    if *fresh124 as i32 - 64i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    }
    let ref mut fresh125 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2);
    *fresh125 = (*fresh125).wrapping_sub(1);
    if *fresh125 as i32 + 0x80i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        -0x2000i32 as u32,
    );
    if ram_word!(game, cinematic_var6) as i32 - 3072i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var6, wrapping_add(16u16));
    } else if ram_word!(game, cinematic_var6) as i32 - 0x2000i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var6, wrapping_add(32u16));
    } else {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 1u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 92u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 6u8;
        set_ram_word!(game.cinematic_var6 = 256u16);
        set_ram_word!(game.cinematic_var5 = 0u16);
        set_ram_word!(game.cinematic_var7 = 0u16);
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var9 = 0u16);
        set_ram_word!(game.cinematic_var10 = 0u16);
        EnableCinematicBgObjects(game);
        EnableCinematicBgTilemapUpdates(game);
        SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF69 as u16, 0x5c00u16, game);
        set_ram_word!(game.cinematic_function = ((0x8ba38fi32) as u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func68(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 124u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 186u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicCommonInstr_Func69(k: u16, mut j: u16, game: &mut Game) -> u16 {
    if ram_word!(game, japanese_text_flag) == 0 {
        j = (j).wrapping_add(6u16);
    }
    j
}
pub unsafe fn CinematicCommonInstr_Func70(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_function = ((0x8bc0c5i32) as u16));
    set_ram_word!(game.screen_fade_delay = 1u16);
    set_ram_word!(game.screen_fade_counter = 1u16);
    j
}
pub unsafe fn CinematicFunction_Intro_Func71(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 124u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 204u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicFunction_Intro_Func72(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.reg_BG2HOFS = 0u16);
        set_ram_word!(game.reg_BG3HOFS = 0u16);
        set_ram_word!(game.reg_BG2VOFS = 0u16);
        set_ram_word!(game.reg_BG3VOFS = 0u16);
        set_ram_word!(game.cinematic_var5 = 0u16);
        let mut i: i32 = 656i32;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
            i -= 2i32;
        }
        set_ram_word!(game.cinematic_function = ((0x8bc100i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func73(game: &mut Game) {
    set_ram_word!(game.loading_game_state = kGameState_31_SetUpNewGame as u16);
    set_ram_word!(game.game_state = kGameState_31_SetUpNewGame as u16);
    set_ram_word!(game.area_index = 6u16);
    set_ram_word!(game.load_station_index = 0u16);
    SaveToSram(ram_word!(game, selected_save_slot), game);
}
pub unsafe fn CinematicFunctionBlackoutFromCeres(game: &mut Game) {
    let mut j: i16 = 0;
    let mut v3: i16 = 0;
    SetupPpu_3_Mode7(game);
    j = 656i16;
    while j >= 0i16 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
            .offset(j as u16 as i32 as isize) as *mut u16) = 0u16;
        j = (j).wrapping_sub(2i16);
    }
    set_ram_word!(game.door_def_ptr = 0u16);
    set_ram_word!(game.layer1_x_pos = 0u16);
    set_ram_word!(game.layer1_y_pos = 0u16);
    let mut v2: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v2 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce5e9, (v2 as i32 >> 1i32) as isize)
        );
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x95a82f, 0x10000).load(game);
    CinematicAsset::Decompress(0x96fe69, 0x14000).load(game);
    CinematicAsset::Decompress(0x96d10a, 0x15000).load(game);
    CinematicAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
    WriteReg(VMAIN as u16, 0u8, game);
    WriteReg(VMADDL as u16, 0u8, game);
    WriteReg(VMADDH as u16, 0u8, game);
    v3 = 0x4000i16;
    loop {
        WriteReg(VMDATAL as u16, 0x8cu8, game);
        v3 -= 1;
        if (v3 == 0) {
            break;
        }
    }
    CinematicAsset::VramLowBytes(0x7f4600, 0x0000, 0x600).load(game);
    CinematicAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    CinematicAsset::VramWords(0x9ad200, 0x6000, 0x1a00).load(game);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x34u16, game);
    set_ram_word!(game.reg_M7X = 52u16);
    WriteRegWord(M7Y as u16, 0x30u16, game);
    set_ram_word!(game.reg_M7Y = 48u16);
    set_ram_word!(game.cinematic_var8 = -44i32 as u16);
    set_ram_word!(game.cinematic_var10 = -112i32 as u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    ClearCinematicBgObjects(0x2fu16, game);
    set_ram_word!(game.cinematicbg_var3 = 0u16);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE7F as u16, 0u16, game);
    SpawnCinematicSpriteObjectAtSlot(
        addr_kCinematicSpriteObjectDef_8BCE8B as u16,
        2u16,
        0x0002,
        game,
    );
    SpawnCinematicSpriteObjectAtSlot(
        addr_kCinematicSpriteObjectDef_8BCE91 as u16,
        0u16,
        0x0000,
        game,
    );
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BCF33 as u16,
        0x0800, // Palette left in A by the preceding sprite initializer.
        game,
    );
    SpawnPalfxObject(addr_kPalfx_E1B8 as u16, game);
    EnableNMI(game);
    set_ram_word!(game.screen_fade_delay = 1u16);
    set_ram_word!(game.screen_fade_counter = 1u16);
    set_ram_word!(game.cinematic_function = ((0x8bc2e4i32) as u16));
    QueueMusic_Delayed8(0u16, game);
    QueueMusic_Delayed8(0xff2du16, game);
    if ram_word!(game, game_state) as i32 == kGameState_37_CeresGoesBoomWithSamus as i32 {
        QueueMusic_DelayedY(8u16, 0xeu16, game);
    } else {
        QueueMusic_DelayedY(7u16, 0xeu16, game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func74(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        set_ram_word!(game.cinematic_function = ((0x8bc2f1i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func75(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        4096u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        -0x4000i32 as u32,
    );
    update_ram_word!(game.cinematic_var6, wrapping_add(1));
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 15u8 {
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bc345i32) as u16));
        set_ram_word!(game.cinematic_var4 = 1u16);
        set_ram_word!(game.cinematic_var13 = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func76(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        4096u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        -0x4000i32 as u32,
    );
    if ram_word!(game, cinematic_var6) as i32 - 640i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var6, wrapping_add(1));
    } else {
        set_ram_word!(
            game.power_bomb_explosion_x_pos =
                (ram_word!(game, reg_M7X)).wrapping_sub(ram_word!(game, cinematic_var8))
        );
        set_ram_word!(
            game.power_bomb_explosion_y_pos =
                (ram_word!(game, reg_M7Y)).wrapping_sub(ram_word!(game, cinematic_var10))
        );
        EnableHdmaObjects(game);
        SpawnPowerBombExplosion(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 16u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 55u8;
        set_ram_word!(game.cinematic_var6 = 768u16);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF2D as u16, 0u16, game);
        if ram_word!(game, game_state) == 37u16 {
            QueueMode7Transfers(
                0x8bu8,
                addr_kCinematicFunction_Intro_Func76_M7_2 as u16,
                game,
            );
        } else {
            QueueMode7Transfers(
                0x8bu8,
                addr_kCinematicFunction_Intro_Func76_M7_0 as u16,
                game,
            );
        }
        QueueMode7Transfers(
            0x8bu8,
            addr_kCinematicFunction_Intro_Func76_M7_1 as u16,
            game,
        );
        set_ram_word!(game.cinematic_function = ((0x8bc5cai32) as u16));
    };
}
pub unsafe fn CinematicSprInstr_SpawnCeresExplosions1(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 1u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 2u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 3u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 4u16, game);
    j
}
static kCinematicFunction_Intro_Func78_Tab0: [u16; 5] = [1, 16, 32, 48, 64];
static kCinematicFunction_Intro_Func78_Tab1: [i16; 5] = [16, -16, 16, -16, 0];
static kCinematicFunction_Intro_Func78_Tab2: [i16; 5] = [-16, 16, 16, -16, 0];
pub unsafe fn CinematicFunction_Intro_Func78(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = v1 as i32;
    set_ram_word!(
        game.cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func78_Tab0[v3 as usize]
    );
    set_ram_word!(
        game.cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func78_Tab1[v3 as usize] as i32
            + ram_word!(game, reg_M7X) as i32
            - ram_word!(game, cinematic_var8) as i32) as u16
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] = (kCinematicFunction_Intro_Func78_Tab2[v3 as usize] as i32
            + ram_word!(game, reg_M7Y) as i32
            - ram_word!(game, cinematic_var10) as i32) as u16
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 2560u16);
}
pub unsafe fn CinematicSprPreInstr_C489(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bc5cai32 & 0xffffi32 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8b93d9i32) as u16));
    } else {
        let v1: bool = (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32
            & 0x8000i32)
            != 0i32;
        if ram_word!(game, cinematic_var4) == 0 || v1 {
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEC1 as u16,
                ram_word!(game, cinematic_var13),
                game,
            );
            set_ram_word!(game.cinematic_var4 = 12u16);
            set_ram_word!(
                game.cinematic_var13 =
                    (ram_word!(game, cinematic_var13) as i32 + 1i32 & 7i32) as u16
            );
        }
    };
}
static g_word_8BC4EB: [i16; 16] = [
    14, -8, 8, 12, -16, 12, -8, -14, 0, 0, 16, 14, -12, 4, -8, -16,
];
static kCinematicFunction_Intro_Func81_Tab0: [i16; 4] = [1i16, 4i16, 8i16, 16i16];
static kCinematicFunction_Intro_Func81_Tab1: [i16; 4] = [8, 12, -8, -12];
static kCinematicFunction_Intro_Func81_Tab2: [i16; 4] = [-4, 8, -10, 12];
pub unsafe fn CinematicFunction_Intro_Func80(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    let v3: i32 = 2i32 * v1 as i32;
    set_ram_word!(
        game.cinematicbg_arr7[v2] = (g_word_8BC4EB[v3 as usize] as i32
            + ram_word!(game, reg_M7X) as i32
            - ram_word!(game, cinematic_var8) as i32) as u16
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] = (g_word_8BC4EB[(v3 + 1i32) as usize] as i32
            + ram_word!(game, reg_M7Y) as i32
            - ram_word!(game, cinematic_var10) as i32) as u16
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 2560u16);
}
pub unsafe fn CinematicSprInstr_SpawnCeresExplosions3(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 1u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 2u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 3u16, game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func81(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_spawn_param);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_goto_timer[v2] = ram_word!(game, cinematic_spawn_param));
    set_ram_word!(
        game.cinematicspr_instr_timer[v2] =
            kCinematicFunction_Intro_Func81_Tab0[v1 as usize] as u16
    );
    set_ram_word!(
        game.cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func81_Tab1[v1 as usize] as i32
            + ram_word!(game, reg_M7X) as i32
            - ram_word!(game, cinematic_var8) as i32) as u16
    );
    set_ram_word!(
        game.cinematicbg_arr8[v2] = (kCinematicFunction_Intro_Func81_Tab2[v1 as usize] as i32
            + ram_word!(game, reg_M7Y) as i32
            - ram_word!(game, cinematic_var10) as i32) as u16
    );
    set_ram_word!(game.cinematicbg_arr9[v2] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func82(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        -4096i32 as u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        0x4000u32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func83(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.cinematicbg_arr7[v1] =
            (ram_word!(game, reg_M7X)).wrapping_sub(ram_word!(game, cinematic_var8))
    );
    set_ram_word!(
        game.cinematicbg_arr8[v1] =
            (ram_word!(game, reg_M7Y)).wrapping_sub(ram_word!(game, cinematic_var10))
    );
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func77(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 16u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 55u8;
    update_ram_word!(game.cinematic_var8, wrapping_add(2u16));
    set_ram_word!(
        game.cinematic_var5 = (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(1u8)) as u16
    );
    if ram_word!(game, cinematic_var6) as i32 - 16i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8bc610i32) as u16));
        set_ram_word!(game.cinematic_var4 = 192u16);
    } else {
        update_ram_word!(game.cinematic_var6, wrapping_sub(16u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func84(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.screen_fade_delay = 1u16);
        set_ram_word!(game.screen_fade_counter = 1u16);
        set_ram_word!(game.cinematic_function = ((0x8bc627i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func85(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 0x80u8 {
        EnableNMI(game);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        if ram_word!(game, game_state) as i32 == kGameState_37_CeresGoesBoomWithSamus as i32 {
            QueueMusic_Delayed8(0u16, game);
            set_ram_word!(game.hud_item_index = 0u16);
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
            set_ram_word!(game.samus_invincibility_timer = 0u16);
            set_ram_word!(game.samus_knockback_timer = 0u16);
            WaitUntilEndOfVblankAndClearHdma(game);
            DisableIrqInterrupts(game);
            set_ram_word!(game.fx_layer_blending_config_a = 0u16);
            *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x71) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x55) = 9u8;
            set_ram_word!(game.game_options_screen_index = 0u16);
            set_ram_word!(game.menu_index = 0u16);
            let mut i: i32 = 254i32;
            while i >= 0i32 {
                set_ram_word!(game.cinematicbg_arr7[((i >> 1i32) + 8i32)] = 0u16);
                i -= 2i32;
            }
            set_ram_word!(game.game_state = kGameState_25_SamusNoHealth as u16);
        } else {
            set_ram_word!(game.cinematic_function = ((0x8bc699i32) as u16));
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func86(game: &mut Game) {
    SetupPpu_4_Mode1(game);
    let mut i: i32 = 656i32;
    while i >= 0i32 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
        i -= 2i32;
    }
    CinematicAsset::Decompress(0x978adb, 0x19000).load(game);
    CinematicAsset::Decompress(0x96ec76, 0x15000).load(game);
    CinematicAsset::VramLowBytes(0x7f4300, 0x0000, 0x300).load(game);
    CinematicAsset::VramWords(0x7f9000, 0x5c00, 0x800).load(game);
    CinematicAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x38u16, game);
    set_ram_word!(game.reg_M7X = 56u16);
    WriteRegWord(M7Y as u16, 0x18u16, game);
    set_ram_word!(game.reg_M7Y = 24u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 256u16);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    *(&raw mut game.g_ram as *mut u8).offset(0x57) = -127i32 as u8;
    SpawnPalfxObject(addr_kPalfx_E1A8 as u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bc79ci32) as u16));
    EnableNMI(game);
    set_ram_word!(game.screen_fade_delay = 1u16);
    set_ram_word!(game.screen_fade_counter = 1u16);
}
pub unsafe fn CinematicFunction_Intro_Func87(game: &mut Game) {
    if (ram_word!(game, cinematic_var12) as i32 & 3i32) == 0i32 {
        let ref mut fresh116 = *(&raw mut game.g_ram as *mut u8).offset(0x57);
        *fresh116 = (*fresh116).wrapping_sub(16u8);
    }
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 15u8 {
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bc7cai32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func88(game: &mut Game) {
    if (ram_word!(game, cinematic_var12) as i32 & 3i32) == 0i32 {
        let ref mut fresh115 = *(&raw mut game.g_ram as *mut u8).offset(0x57);
        *fresh115 = (*fresh115).wrapping_sub(16u8);
        if (*(&raw mut game.g_ram as *mut u8).offset(0x57) as i32 & 0xf0i32) == 0i32 {
            *(&raw mut game.g_ram as *mut u8).offset(0x55) = 7u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0x80u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
            set_ram_word!(game.cinematic_var8 = 128u16);
            set_ram_word!(game.cinematic_var10 = -104i32 as u16);
            set_ram_word!(game.cinematic_var5 = 32u16);
            set_ram_word!(game.cinematic_var6 = 256u16);
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEA3 as u16,
                0x0100, // The preceding $198F store leaves $0100 in A.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEF7 as u16,
                0x0e00, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEFD as u16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF03 as u16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCF09 as u16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEAF as u16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            set_ram_word!(game.cinematic_function = ((0x8ba38fi32) as u16));
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func89(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 136u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 111u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func90(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bcadei32 & 0xffffi32 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bc85di32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func91(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(64u16));
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        -((ram_word!(game, cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
    );
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 + 128i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func92(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 127u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func93(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bcadei32 & 0xffffi32 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bc8b9i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func94(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(32u16));
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        -((ram_word!(game, cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
    );
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 + 128i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        set_ram_word!(game.cinematic_function = ((0x8bcadfi32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func96(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bcadei32 & 0xffffi32 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bc908i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func97(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(32u16));
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        -((ram_word!(game, cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
    );
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 + 128i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func98(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 48u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 47u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func99(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 208u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 47u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func100(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 48u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 207u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func101(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 208u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 207u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2048u16);
}
pub unsafe fn CinematicFunction_Intro_Func102(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 186u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicSprInstr_C9A5(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnPalfxObject(addr_kPalfx_E1B0 as u16, game);
    j
}
pub unsafe fn CinematicSprInstr_C9AF(k: u16, j: u16, game: &mut Game) -> u16 {
    if ram_word!(game, japanese_text_flag) != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BCEB5 as u16,
            ram_word!(game, japanese_text_flag),
            game,
        );
    }
    j
}
pub unsafe fn CinematicSprInstr_C9BD(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnPalfxObject(addr_loc_8BE1B4 as u16, game);
    j
}
pub unsafe fn CinematicSprInstr_C9C7(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematic_var8 = 62u16);
    set_ram_word!(game.cinematic_var10 = -112i32 as u16);
    set_ram_word!(game.cinematic_var5 = 32u16);
    set_ram_word!(game.cinematic_var6 = 16u16);
    set_ram_word!(game.cinematic_function = ((0x8bc9f9i32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func104(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 204u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicFunction_Intro_Func105(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        0x2000u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        -0x8000i32 as u32,
    );
    if ram_word!(game, cinematic_var6) as i32 - 1152i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var6, wrapping_add(4u16));
    } else {
        set_ram_word!(game.cinematic_function = ((0x8bca36i32) as u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func106(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        0x2000u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        -0x8000i32 as u32,
    );
    if ram_word!(game, cinematic_var8) as i32 + 128i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8bca85i32) as u16));
    } else {
        update_ram_word!(game.cinematic_var6, wrapping_add(16u16));
        if ram_word!(game, cinematic_var5) != 224u16 {
            set_ram_word!(
                game.cinematic_var5 =
                    (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(1u8)) as u16
            );
        }
    };
}
pub unsafe fn CinematicFunction_Intro_Func107(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1995) as *mut u16,
        0x2000u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        0x2000u32,
    );
    if ram_word!(game, cinematic_var6) as i32 - 0x2000i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematic_var6, wrapping_add(32u16));
    } else {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        set_ram_word!(game.cinematic_function = ((0x8bcad0i32) as u16));
        set_ram_word!(game.cinematic_var4 = 64u16);
    };
}
pub unsafe fn CinematicFunction_Intro_Func108(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8bcadei32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func95(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x51) = 0x80u8;
    let mut i: i32 = 656i32;
    while i >= 0i32 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
        i -= 2i32;
    }
    set_ram_word!(game.game_state = kGameState_6_LoadingGameData as u16);
    set_ram_word!(game.samus_health = ram_word!(game, samus_max_health));
}
pub unsafe fn GameState_39_EndingAndCredits_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((ram_word!(game, cinematic_function)) as u32) | (0x8b0000u32),
        game,
    );
    if (ram_word!(game, cinematic_var11) as i32 & 0x8000i32) == 0i32 {
        update_ram_word!(game.cinematic_var11, wrapping_add(1));
    }
    update_ram_word!(game.cinematic_var12, wrapping_add(1));
    HandleCinematicSprites(game);
    MaybeHandleCinematicBgObjects(game);
    CreditsObject_Process(game);
    PaletteFxHandler(game);
    DrawCinematicSpriteObjects_Ending(game);
    CinematicFunction_Intro_Func149(game);
    HandleMode7TransformationMatrix(game);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn MaybeHandleCinematicBgObjects(game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 + 0x1afci32 & 0x8000i32 == 0 {
        HandleCinematicBgObjects(game);
    }
}
pub unsafe fn CinematicFunctionEscapeFromCebes(game: &mut Game) {
    let mut j: i16 = 0;
    set_ram_word!(game.cur_irq_handler = 0u16);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    SetupPpu_5_Mode7(game);
    j = 656i16;
    while j >= 0i16 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x198d) as *mut u16 as *mut u8)
            .offset(j as u16 as i32 as isize) as *mut u16) = 0u16;
        j = (j).wrapping_sub(2i16);
    }
    let mut v2: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v2 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8cede9, (v2 as i32 >> 1i32) as isize)
        );
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x98bcd6, 0x10000).load(game);
    CinematicAsset::Decompress(0x99a56f, 0x14000).load(game);
    CinematicAsset::Decompress(0x99d17e, 0x18000).load(game);
    CinematicAsset::VramWords(0x7f8000, 0x0000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7f8000, 0x2000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7f4000, 0x6000, 0x4000).load(game);
    CinematicAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
    CinematicAsset::Decompress(0x988304, 0x18000).load(game);
    CinematicAsset::Decompress(0x95a82f, 0x10000).load(game);
    CinematicAsset::Decompress(0x96fe69, 0x14000).load(game);
    let mut v3: u16 = 768u16;
    loop {
        *(((&raw mut game.g_ram as *mut u8).offset(0x14000)).offset(v3 as i32 as isize)
            as *mut u16) = 0x8c8cu16;
        v3 = (v3).wrapping_add(2u16);
        if (((v3) as i16).wrapping_sub(0x4000i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x98b5c1, 0x1e000).load(game);
    CinematicAsset::Decompress(0x98b857, 0x1e800).load(game);
    CinematicAsset::Decompress(0x98baed, 0x1f000).load(game);
    CinematicAsset::Decompress(0x98bccd, 0x1f800).load(game);
    CinematicAsset::Decompress(0x97e7de, 0xa000).load(game);
    CinematicAsset::Decompress(0x99d65b, 0x2000).load(game);
    CinematicAsset::Decompress(0x99d932, 0x6000).load(game);
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7X = 128u16);
    WriteRegWord(M7Y as u16, 0x80u16, game);
    set_ram_word!(game.reg_M7Y = 128u16);
    set_ram_word!(game.cinematic_var5 = 32u16);
    set_ram_word!(game.cinematic_var6 = 64u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.door_def_ptr = 0u16);
    set_ram_word!(game.layer1_x_pos = 0u16);
    set_ram_word!(game.layer1_y_pos = 0u16);
    ClearPaletteFXObjects(game);
    EnablePaletteFx(game);
    EnableNMI(game);
    set_ram_word!(game.eproj_pre_instr[1] = 0u16);
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 1u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 17u8;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 1u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 1u16, game);
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 2u16);
    set_ram_word!(game.cinematic_function = ((0x8bd6d7i32) as u16));
    QueueMusic_Delayed8(0u16, game);
    QueueMusic_Delayed8(0xff33u16, game);
    QueueMusic_DelayedY(5u16, 0xeu16, game);
}
pub unsafe fn CinematicFunction_Intro_Func109(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        SpawnPalfxObject(addr_kPalfx_E1D8 as u16, game);
        SpawnPalfxObject(addr_kPalfx_E1DC as u16, game);
        set_ram_word!(game.cinematic_function = ((0x8bd6f2i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func110(game: &mut Game) {
    CinematicFunction_Intro_Func111(game);
    if AdvanceFastScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8bd701i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func111(game: &mut Game) {
    if (ram_word!(game, cinematic_var12) as i32 & 1i32) == 0i32 {
        set_ram_word!(
            game.cinematic_var5 =
                (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(1u8)) as u16
        );
    }
    update_ram_word!(game.cinematic_var6, wrapping_add(2u16));
    if ram_word!(game, cinematic_var6) as i32 - 384i32 & 0x8000i32 == 0
        && AdvanceFastScreenFadeOut(game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.cinematic_function = ((0x8bd731i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func112(game: &mut Game) {
    CinematicAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7e2000, 0x2000, 0x4000).load(game);
    CinematicAsset::Decompress(0x98ed4f, 0x2000).load(game);
    CinematicAsset::VramHighBytes(0x7e2000, 0x0000, 0x4000).load(game);
    ClearPaletteFXObjects(game);
    ClearCinematicSprites(game);
    SpawnPalfxObject(addr_kPalfx_E1E0 as u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEED3 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEED9 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEDF as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEE5 as u16, 0u16, game);
    set_ram_word!(game.cinematic_var5 = 32u16);
    set_ram_word!(game.cinematic_var6 = 64u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.cinematic_function = ((0x8bd7f8i32) as u16));
}
pub unsafe fn CinematicFunction_Intro_Func114(game: &mut Game) {
    if (ram_word!(game, cinematic_var12) as i32 & 1i32) == 0i32 {
        set_ram_word!(
            game.cinematic_var5 =
                (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(1u8)) as u16
        );
    }
    update_ram_word!(game.cinematic_var6, wrapping_add(3u16));
    if ram_word!(game, cinematic_var6) as i32 - 384i32 & 0x8000i32 == 0
        && AdvanceFastScreenFadeOut(game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.cinematic_function = ((0x8bd837i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func113(game: &mut Game) {
    CinematicFunction_Intro_Func114(game);
    if AdvanceFastScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8bd807i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func115(game: &mut Game) {
    CinematicAsset::VramWords(0x7e6000, 0x0000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7e6000, 0x2000, 0x4000).load(game);
    CinematicAsset::Decompress(0x999101, 0x6000).load(game);
    CinematicAsset::VramHighBytes(0x7e6000, 0x0000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7f8000, 0x4000, 0x6000).load(game);
    CinematicAsset::VramWords(0x7fe000, 0x7000, 0x800).load(game);
    CinematicAsset::VramWords(0x7fe800, 0x7400, 0x800).load(game);
    CinematicAsset::VramWords(0x7ff000, 0x7800, 0x800).load(game);
    CinematicAsset::VramWords(0x7ff800, 0x7c00, 0x800).load(game);
    CinematicAsset::VramWords(0x7ea000, 0x5000, 0x1000).load(game);
    game.g_ram[0x52] = 2;
    ClearPaletteFXObjects(game);
    ClearCinematicSprites(game);
    let mut v0: u16 = 256u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        let v2: u16 = crate::rtl::rom_table_word!(game, 0x8cebe9, v1 as isize);
        set_ram_word!(game.palette_buffer[v1] = v2);
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEE9D as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEAF as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEA3 as u16, 0u16, game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEA9 as u16, 0u16, game);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 64u16);
    set_ram_word!(game.reg_M7A = 256u16);
    set_ram_word!(game.reg_M7B = 0u16);
    set_ram_word!(game.reg_M7C = 0u16);
    set_ram_word!(game.reg_M7D = 256u16);
    set_ram_word!(game.reg_M7X = 128u16);
    set_ram_word!(game.reg_M7Y = 128u16);
    set_ram_word!(game.cinematic_var8 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 1u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 33u8;
    CopyPalettesToFadingPalettes(game);
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0x1a0u16, 0x10u16, game);
    ClearYColorsFromIndexX(0x1e0u16, 0x10u16, game);
    ComposeFadingPalettes(game);
    set_ram_word!(game.cinematic_var4 = 63u16);
    set_ram_word!(game.cinematic_function = ((0x8bda02i32) as u16));
}
pub unsafe fn CinematicFunction_Intro_Func116(game: &mut Game) {
    CinematicFunction_Intro_Func117(game);
    if AdvanceFastScreenFadeIn(game) as i32 & 1i32 != 0 {
        set_ram_word!(game.screen_fade_delay = 2u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
        set_ram_word!(game.cinematic_function = ((0x8bda1ai32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func117(game: &mut Game) {
    update_ram_word!(game.cinematic_var6, wrapping_add(4u16));
    if (ram_word!(game, cinematic_var4) as i32 & 1i32) == 0i32 {
        FadeOutYColorsFromIndexX(0xe0u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x1a0u16, 0x10u16, game);
        FadeInYColorsFromIndexX(0x1e0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
    }
    if (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        let mut v0: u16 = 0u16;
        loop {
            set_ram_word!(
                game.palette_buffer[(v0 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0x8cebe9, (v0 as i32 >> 1i32) as isize)
            );
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(256i16) >= 0i16) {
                break;
            }
        }
        *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 112u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x59) = 120u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5a) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5c) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 68u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5e) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x71) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        WriteReg(COLDATA as u16, 0xe0u8, game);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x57) = 0u8;
        set_ram_word!(game.cinematic_var7 = 0u16);
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var9 = 0u16);
        set_ram_word!(game.cinematic_var10 = 0u16);
        set_ram_word!(game.g_word_7E1999 = 0u16);
        set_ram_word!(game.g_word_7E199B = 0u16);
        set_ram_word!(game.g_word_7E199D = 0u16);
        set_ram_word!(game.g_word_7E199F = 0u16);
        set_ram_word!(game.fx_layer_blending_config_a = 0u16);
        set_ram_word!(game.reg_BG2HOFS = 0u16);
        set_ram_word!(game.reg_BG2VOFS = 0u16);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 2u16);
        set_ram_word!(game.cinematic_function = ((0x8bdad3i32) as u16));
        set_ram_word!(game.cinematic_var4 = 0u16);
    }
}
static kCinematicFunction_Intro_Func118_Tab0: [u16; 16] = [
    0, 0x800, 0x1000, 0x1800, 0x2000, 0x2800, 0x3000, 0x3800, 0x4000, 0x4800, 0x5000, 0x5800,
    0x6000, 0x6800, 0x7000, 0x7800,
];
static kCinematicFunction_Intro_Func118_Tab1: [u16; 16] = [
    0, 0x800, 0x1000, 0x1800, 0x2000, 0x2800, 0x3000, 0x3800, 0, 0x800, 0x1000, 0x1800, 0x2000,
    0x2800, 0x3000, 0x3800,
];
pub unsafe fn CinematicFunction_Intro_Func118(game: &mut Game) {
    let mut v2: u32 = 0;
    let v0: u16 = ram_word!(game, cinematic_var4);
    let v1: u16 = ram_word!(game, mode7_vram_write_queue_tail);
    v2 = ram_word!(game, mode7_vram_write_queue_tail) as u32;
    if ram_word!(game, cinematic_var4) as i32 - 8i32 & 0x8000i32 != 0 {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(ram_word!(game, mode7_vram_write_queue_tail) as i32 as isize)
            as *mut u16) = 192u16;
        let v3: i32 = v0 as i32;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut u8)
            .offset(v2 as isize) as *mut u16) = kCinematicFunction_Intro_Func118_Tab0[v3 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset(v1 as isize) as *mut u16) = 127u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v1 as i32 + 1i32) as isize) as *mut u16) = 2048u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v1 as i32 + 3i32) as isize) as *mut u16) =
            kCinematicFunction_Intro_Func118_Tab1[v3 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut u8)
            .offset(v1 as i32 as isize) as *mut u16) = 128u16;
    } else {
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0)
            .offset(ram_word!(game, mode7_vram_write_queue_tail) as i32 as isize)
            as *mut u16) = 128u16;
        let v4: i32 = v0 as i32;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_1 as *mut u8)
            .offset(v2 as isize) as *mut u16) = kCinematicFunction_Intro_Func118_Tab0[v4 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset(v1 as isize) as *mut u16) = 127u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v1 as i32 + 1i32) as isize) as *mut u16) = 2048u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(0))
        .gap3 as *mut u8)
            .offset((v1 as i32 + 3i32) as isize) as *mut u16) =
            kCinematicFunction_Intro_Func118_Tab1[v4 as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x2d0) as *mut Mode7VramWriteQueue)
            .offset(1))
        .field_1 as *mut u8)
            .offset(v1 as i32 as isize) as *mut u16) = 0u16;
    }
    set_ram_word!(game.mode7_vram_write_queue_tail = (v1).wrapping_add(9u16));
    if update_ram_word!(game.cinematic_var4, wrapping_add(1), reread) as i32 - 16i32 & 0x8000i32
        == 0
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 1u8;
        set_ram_word!(game.cinematic_var4 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bdb9di32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func119(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8(0xff3cu16, game);
        QueueMusic_DelayedY(5u16, 0xeu16, game);
        set_ram_word!(game.cinematic_function = ((0x8bdbc4i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func120(game: &mut Game) {
    if HasQueuedMusic(game) as i32 & 1i32 == 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 7u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0x80u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 48u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 63u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 95u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = -97i32 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 0u8;
        set_ram_word!(game.palette_buffer[0] = 0u16);
        set_ram_word!(game.palette_buffer[16] = 0u16);
        set_ram_word!(game.palette_buffer[128] = 0u16);
        SpawnPalfxObject(addr_kPalfx_E1D4 as u16, game);
        SpawnPalfxObject(addr_kPalfx_E1E4 as u16, game);
        set_ram_word!(game.reg_M7X = 56u16);
        set_ram_word!(game.reg_M7Y = 24u16);
        set_ram_word!(game.cinematic_var8 = -72i32 as u16);
        set_ram_word!(game.cinematic_var10 = -104i32 as u16);
        set_ram_word!(game.cinematic_var6 = 3072u16);
        set_ram_word!(game.cinematic_var5 = -112i32 as u16);
        set_ram_word!(game.cinematic_var4 = 1u16);
        set_ram_word!(game.cinematic_var13 = 192u16);
        set_ram_word!(game.cinematic_var14 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bdca5i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func122(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        let ref mut fresh97 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0);
        *fresh97 = (*fresh97).wrapping_sub(1);
        if *fresh97 as i32 - 32i32 & 0x80i32 != 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
        }
        let ref mut fresh98 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1);
        *fresh98 = (*fresh98).wrapping_sub(1);
        if *fresh98 as i32 - 64i32 & 0x80i32 != 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
        }
        let ref mut fresh99 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2);
        *fresh99 = (*fresh99).wrapping_sub(1);
        if *fresh99 as i32 + 0x80i32 & 0x80i32 != 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
        }
        if *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) == 32u8
            && *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) == 64u8
            && *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) == 0x80u8
        {
            *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        }
        set_ram_word!(game.cinematic_var4 = 8u16);
    }
}
static kCinematicFunction_Intro_Func121_Tab0: [u16; 32] = [
    0, 0x8000, 0, 0x8000, 0, 0x8000, 0, 0x8000, 0xffff, 0x8000, 0xffff, 0x8000, 0, 0x8000, 0,
    0x8000, 0, 0x8000, 0, 0x8000, 0xffff, 0x8000, 0xffff, 0x8000, 0, 0x8000, 0, 0x8000, 0xffff,
    0x8000, 0xffff, 0x8000,
];
pub unsafe fn CinematicFunction_Intro_Func121(game: &mut Game) {
    if ram_word!(game, cinematic_var13) != 0 {
        update_ram_word!(game.cinematic_var13, wrapping_sub(1));
    } else {
        CinematicFunction_Intro_Func122(game);
    }
    set_ram_word!(
        game.cinematic_var5 = (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(4u8)) as u16
    );
    let v0: i32 = 2i32 * ram_word!(game, cinematic_var14) as i32;
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        (kCinematicFunction_Intro_Func121_Tab0[v0 as usize] as u32) << 16i32
            | kCinematicFunction_Intro_Func121_Tab0[(v0 + 1i32) as usize] as u32,
    );
    set_ram_word!(
        game.cinematic_var14 = (ram_word!(game, cinematic_var14) as i32 + 1i32 & 0xfi32) as u16
    );
    update_ram_word!(game.cinematic_var6, wrapping_sub(8u16));
    if ram_word!(game, cinematic_var6) as i32 - 1456i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematic_var13 = 1u16);
        set_ram_word!(game.cinematic_var14 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bdd42i32) as u16));
    }
}
static kCinematicFunction_Intro_Func123_Tab0: [u16; 16] = [
    1, 0, 1, 0, 1, 0, 0xffff, 0, 0xffff, 0, 1, 0, 1, 0, 0xffff, 0,
];
pub unsafe fn CinematicFunction_Intro_Func123(game: &mut Game) {
    CinematicFunction_Intro_Func122(game);
    if ram_word!(game, cinematic_var5) != 224u16 {
        set_ram_word!(
            game.cinematic_var5 =
                (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(1u8)) as u16
        );
    }
    let v0: i32 = 2i32 * ram_word!(game, cinematic_var14) as i32;
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        (kCinematicFunction_Intro_Func123_Tab0[v0 as usize] as u32) << 16i32
            | kCinematicFunction_Intro_Func123_Tab0[(v0 + 1i32) as usize] as u32,
    );
    set_ram_word!(
        game.cinematic_var14 = (ram_word!(game, cinematic_var14) as i32 + 1i32 & 7i32) as u16
    );
    update_ram_word!(game.cinematic_var6, wrapping_sub(2u16));
    if ram_word!(game, cinematic_var6) as i32 - 1184i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematic_function = ((0x8bddcdi32) as u16));
        set_ram_word!(game.cinematic_var14 = 0x8000u16);
        set_ram_word!(game.cinematic_var13 = 0u16);
        if CheckEventHappened(0xfu16, game) != 0 {
            SpawnCinematicSpriteObjectToR18(0xef21u16, 4u16, game);
        }
    }
}
static g_word_8BDE43: [u16; 16] = [
    0, 0x6f7b, 0x4a52, 0x1ce7, 0, 0x5ad6, 0x4a52, 0x35ad, 0x2529, 0x7fff, 0x56b5, 0x294a, 0x4210,
    0x2d6b, 0x18c6, 0x7fff,
];
pub unsafe fn CinematicFunction_Intro_Func124(game: &mut Game) {
    CinematicFunction_Intro_Func122(game);
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1a4d) as *mut u16,
        -256i32 as u32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x1991) as *mut u16,
        (ram_word!(game, cinematic_var13) as u32) << 16i32
            | ram_word!(game, cinematic_var14) as u32,
    );
    if ram_word!(game, cinematic_var6) as i32 - 384i32 & 0x8000i32 != 0
        && (ram_word!(game, cinematic_var12) as i32 & 3i32) == 0i32
        && ram_word!(game, cinematic_var5) != 16u16
    {
        set_ram_word!(
            game.cinematic_var5 =
                (((ram_word!(game, cinematic_var5)) as u8).wrapping_add(2u8)) as u16
        );
    }
    if ram_word!(game, cinematic_var6) as i32 - 32i32 & 0x8000i32 != 0 {
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BEEC7 as u16,
            ram_word!(game, cinematic_var6),
            game,
        );
        set_ram_word!(game.cinematic_function = ((0x8bde63i32) as u16));
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            set_ram_word!(
                game.palette_buffer[((i >> 1i32) + 80i32)] = g_word_8BDE43[(i >> 1i32) as usize]
            );
            i -= 2i32;
        }
    } else {
        update_ram_word!(game.cinematic_var6, wrapping_sub(4u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func125(game: &mut Game) {
    set_ram_word!(game.screen_fade_counter = 1u16);
    if AdvanceFastScreenFadeOut(game) as i32 & 1i32 != 0 {
        EnableNMI(game);
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8bde80i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func126(game: &mut Game) {
    SetupPpu_7_Mode1(game);
    ClearCinematicSprites(game);
    ClearPaletteFXObjects(game);
    let mut v0: u16 = 0u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v0 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce9e9, (v0 as i32 >> 1i32) as isize)
        );
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    CinematicAsset::Decompress(0x979803, 0x10000).load(game);
    CinematicAsset::Decompress(0x97b957, 0x2000).load(game);
    CinematicAsset::Decompress(0x97d7fc, 0x6000).load(game);
    CinematicAsset::Decompress(0x97e7de, 0x1c000).load(game);
    CinematicAsset::Decompress(0x9796f4, 0x1e000).load(game);
    CinematicAsset::Decompress(0x97f987, 0x14000).load(game);
    CinematicAsset::Decompress(0x99da9f, 0x1e800).load(game);
    CinematicAsset::Decompress(0x99dab1, 0x1f000).load(game);
    CinematicAsset::VramWords(0x7fe800, 0x2000, 0x100).load(game);
    CinematicAsset::VramWords(0x7ff000, 0x2400, 0x800).load(game);
    CinematicAsset::VramWords(0x7fc000, 0x4000, 0x1000).load(game);
    CinematicAsset::VramWords(0x7e6000, 0x6000, 0x4000).load(game);
    CinematicAsset::VramWords(0x7f0000, 0x5000, 0x2000).load(game);
    CinematicAsset::VramWords(0x7fe000, 0x4c00, 0x800).load(game);
    CinematicAsset::Decompress(0x99e089, 0x6000).load(game);
    CinematicAsset::Decompress(0x99ecc4, 0x8000).load(game);
    if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 != 0 {
        CinematicAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
    } else {
        CinematicAsset::VramWords(0x7f0000, 0x0000, 0x4000).load(game);
    }
    WriteRegWord(M7A as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7A = 256u16);
    WriteRegWord(M7B as u16, 0u16, game);
    set_ram_word!(game.reg_M7B = 0u16);
    WriteRegWord(M7C as u16, 0u16, game);
    set_ram_word!(game.reg_M7C = 0u16);
    WriteRegWord(M7D as u16, 0x100u16, game);
    set_ram_word!(game.reg_M7D = 256u16);
    WriteRegWord(M7X as u16, 0x68u16, game);
    set_ram_word!(game.reg_M7X = 104u16);
    WriteRegWord(M7Y as u16, 0x70u16, game);
    set_ram_word!(game.reg_M7Y = 112u16);
    set_ram_word!(game.cinematic_var8 = -8i32 as u16);
    set_ram_word!(game.cinematic_var10 = -24i32 as u16);
    set_ram_word!(game.cinematic_var5 = 0u16);
    set_ram_word!(game.cinematic_var6 = 2304u16);
    EnableNMI(game);
    *(&raw mut game.g_ram as *mut u8).offset(0x51) = 15u8;
    set_ram_word!(game.layer1_x_pos = 0u16);
    set_ram_word!(game.layer1_y_pos = 0u16);
    EnableHdmaObjects(game);
    let mut i: i32 = 510i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    CinematicFunction_Intro_Func127(game);
    CinematicAsset::Decompress(0x97eeff, 0x10000).load(game);
    ClearCinematicBgObjects(0x7fu16, game);
    CinematicFunction_Intro_Func128(0x7fu16, game);
    EnableTextGlowObjects_(game);
    set_ram_word!(game.screen_fade_delay = 3u16);
    set_ram_word!(game.screen_fade_counter = 3u16);
    CreditsObject_Init(addr_stru_8BF6F8 as u16, game);
    set_ram_word!(game.cinematic_function = ((0x8bdb9di32) as u16));
}
pub unsafe fn CinematicFunction_Intro_Func129(game: &mut Game) {
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8be110i32) as u16));
        set_ram_word!(game.cinematic_var4 = 120u16);
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
    }
}
pub unsafe fn CinematicFunction_Intro_Func130(game: &mut Game) {
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut u8).offset(0x51) == 15u8 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 2u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 34u8;
        set_ram_word!(game.screen_fade_delay = 0u16);
        set_ram_word!(game.screen_fade_counter = 0u16);
        CopyPalettesToFadingPalettes(game);
        DecomposePaletteDataForFading(game);
        ClearYColorsFromIndexX(0x40u16, 0x10u16, game);
        ComposeFadingPalettes(game);
        set_ram_word!(game.cinematic_var4 = 32u16);
        set_ram_word!(game.cinematic_function = ((0x8be158i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func131(game: &mut Game) {
    FadeInYColorsFromIndexX(0x40u16, 0x10u16, game);
    ComposeFadingPalettes(game);
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 18u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 2u8;
        set_ram_word!(game.cinematic_var4 = 180u16);
        set_ram_word!(game.cinematic_function = ((0x8be190i32) as u16));
        CinematicFunction_Intro_Func133(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func132(game: &mut Game) {
    update_ram_word!(game.reg_BG3VOFS, wrapping_add(2u16));
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        let mut v1: u16 = 0u16;
        loop {
            (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v1 as i32 + 288i32) as usize] =
                crate::rtl::rom_table_word!(game, 0x8cdc9b, v1 as isize);
            v1 = v1.wrapping_add(1);
            if (((v1 as i32 * 2i32 - 576i32) as i16) >= 0i16) {
                break;
            }
        }
        CinematicUpdateSomeBg(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 1u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        set_ram_word!(game.cinematic_var4 = 180u16);
        set_ram_word!(game.cinematic_function = ((0x8be1d2i32) as u16));
        set_ram_word!(game.g_word_7E0D9C = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func134(game: &mut Game) {
    if update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) == 0 {
        set_ram_word!(game.cinematic_var4 = 127u16);
        if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 != 0 {
            CopyPalettesToFadingPalettes(game);
            DecomposePaletteDataForFading(game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF27 as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF2D as u16, 0u16, game);
        } else {
            if ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 == 0 {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF4B as u16,
                    0u16,
                    game,
                );
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF45 as u16,
                    0u16,
                    game,
                );
                set_ram_word!(game.cinematic_function = ((0x8be293i32) as u16));
                *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
                *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
                *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
                *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
                return;
            }
            CopyPalettesToFadingPalettes(game);
            DecomposePaletteDataForFading(game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF51 as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF45 as u16, 0u16, game);
            ClearYColorsFromIndexX(0x1c0u16, 0x10u16, game);
        }
        ClearYColorsFromIndexX(0x1a0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 2u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 34u8;
        set_ram_word!(game.cinematic_function = ((0x8be265i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func136(game: &mut Game) {
    if (ram_word!(game, cinematic_var4) as i32 & 3i32) != 0i32 {
        CinematicFunction_Intro_Func135(game);
    } else {
        FadeOutYColorsFromIndexX(0x40u16, 0x10u16, game);
        if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 == 0 {
            FadeInYColorsFromIndexX(0x1c0u16, 0x10u16, game);
        }
        FadeInYColorsFromIndexX(0x1a0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
        CinematicFunction_Intro_Func135(game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func135(game: &mut Game) {
    if update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 - 65i32 & 0x8000i32
        != 0
    {
        let mut i: i32 = 574i32;
        while i >= 0i32 {
            *(((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut u16)
                .offset(288) as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 127u16;
            i -= 2i32;
        }
        let mut v1: u16 = 0u16;
        loop {
            (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v1 as i32 + 384i32) as usize] =
                crate::rtl::rom_table_word!(game, 0x8cdedb, v1 as isize);
            v1 = v1.wrapping_add(1);
            if (((v1 as i32 * 2i32 - 128i32) as i16) >= 0i16) {
                break;
            }
        }
        CinematicUpdateSomeBg(game);
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 1u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        set_ram_word!(game.cinematic_var13 = 180u16);
        set_ram_word!(game.cinematic_function = ((0x8be2ddi32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func137(game: &mut Game) {
    let v0: bool = (update_ram_word!(game.cinematic_var13, wrapping_sub(1), reread) as i32
        & 0x8000i32)
        != 0i32;
    if ram_word!(game, cinematic_var13) == 0 || v0 {
        if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 != 0
            || ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 != 0
        {
            *(&raw mut game.g_ram as *mut u8).offset(0x69) = 2u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x72) = 34u8;
            set_ram_word!(game.cinematic_function = ((0x8be314i32) as u16));
        } else {
            set_ram_word!(game.cinematic_function = ((0x8be342i32) as u16));
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func138(game: &mut Game) {
    if (ram_word!(game, cinematic_var4) as i32 & 3i32) != 0i32 {
        CinematicFunction_Intro_Func145(game);
    } else {
        FadeOutYColorsFromIndexX(0x40u16, 0x10u16, game);
        if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 == 0 {
            FadeInYColorsFromIndexX(0x1c0u16, 0x10u16, game);
        }
        FadeInYColorsFromIndexX(0x1a0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
        CinematicFunction_Intro_Func145(game);
    };
}
pub unsafe fn CinematicFunction_Intro_Func145(game: &mut Game) {
    if (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 16u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 7u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0x80u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 0u8;
        set_ram_word!(game.cinematic_function = ((0x8bdb9di32) as u16));
        set_ram_word!(game.cinematic_var14 = 0u16);
        if ram_word!(game, game_time_hours) as i32 - 3i32 & 0x8000i32 != 0 {
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF33 as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF39 as u16, 0u16, game);
        } else {
            if ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 != 0 {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF69 as u16,
                    0u16,
                    game,
                );
            } else {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BEF63 as u16,
                    0u16,
                    game,
                );
            }
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF5D as u16, 0u16, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF57 as u16, 0u16, game);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func140(game: &mut Game) {
    if ram_word!(game, cinematic_var17) != 0 {
        update_ram_word!(game.cinematic_var17, wrapping_sub(1));
        FadeOutYColorsFromIndexX(0x1e0u16, 0x10u16, game);
        ComposeFadingPalettes(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func139(game: &mut Game) {
    if ram_word!(game, cinematic_var14) != 0 {
        update_ram_word!(game.cinematic_var14, wrapping_sub(1));
        FadeOutYColorsFromIndexX(0x180u16, 0x10u16, game);
        if ram_word!(game, cinematic_var4) != 0 {
            ComposeFadingPalettes(game);
        }
    }
    if ram_word!(game, cinematic_var4) != 0 {
        update_ram_word!(game.cinematic_var4, wrapping_sub(1));
    } else {
        CinematicFunction_Intro_Func140(game);
    }
    set_ram_word!(
        game.cinematic_var5 = (((ram_word!(game, cinematic_var5)) as u8).wrapping_sub(8u8)) as u16
    );
    update_ram_word!(game.cinematic_var6, wrapping_sub(64u16));
    if ram_word!(game, cinematic_var6) as i32 - 24i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematic_var6 = 24u16);
        set_ram_word!(game.cinematic_function = ((0x8be48ai32) as u16));
        set_ram_word!(game.cinematic_var4 = 180u16);
        set_ram_word!(game.cinematic_var14 = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func142(game: &mut Game) {
    if ram_word!(game, cinematic_var17) == 0
        && ram_word!(game, cinematic_var14) as i32 - 6i32 & 0x8000i32 != 0
    {
        let v0: u16 = (4u16).wrapping_mul(ram_word!(game, cinematic_var14));
        let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
        let v2: i32 = 8i32 * ram_word!(game, cinematic_var14) as i32 >> 1i32;
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, v2 as isize);
        v1 = (v1).wrapping_add(2u16);
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v2 + 1i32) as isize);
        v1 = (v1).wrapping_add(2u16);
        let fresh75 = v1;
        v1 = v1.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh75 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v0 as i32 + 2i32) as isize) as u8;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v2 + 3i32) as isize);
        set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(2u16));
        update_ram_word!(game.cinematic_var14, wrapping_add(1));
    }
}
pub unsafe fn CinematicFunction_Intro_Func141(game: &mut Game) {
    CinematicFunction_Intro_Func140(game);
    CinematicFunction_Intro_Func142(game);
    if update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) == 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x55) = 1u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x72) = 49u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 63u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 95u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = -97i32 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5f) = 0u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x58) = 78u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 84u8;
        let mut v0: u16 = 384u16;
        loop {
            set_ram_word!(
                game.palette_buffer[(v0 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0x8ce7e9, (v0 as i32 >> 1i32) as isize)
            );
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(416i16) >= 0i16) {
                break;
            }
        }
        set_ram_word!(game.cinematic_var8 = 0u16);
        set_ram_word!(game.cinematic_var10 = 0u16);
        set_ram_word!(game.cinematic_function = ((0x8be504i32) as u16));
        set_ram_word!(game.cinematic_var4 = 32u16);
        EnableCinematicBgObjects(game);
        EnableCinematicBgTilemapUpdates(game);
        set_ram_word!(game.cinematicbg_var3 = 19456u16);
        let mut i: i32 = 126i32;
        while i >= 0i32 {
            *(((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut u16)
                .offset(384) as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 127u16;
            i -= 2i32;
        }
        CinematicUpdateSomeBg(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func143(game: &mut Game) {
    let mut j: u16 = 0;
    let ref mut fresh69 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0);
    *fresh69 = (*fresh69).wrapping_sub(1);
    if *fresh69 as i32 - 32i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32u8;
    }
    let ref mut fresh70 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1);
    *fresh70 = (*fresh70).wrapping_sub(1);
    if *fresh70 as i32 - 64i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64u8;
    }
    let ref mut fresh71 = *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2);
    *fresh71 = (*fresh71).wrapping_sub(1);
    if *fresh71 as i32 + 0x80i32 & 0x80i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80u8;
    }
    let v0: bool =
        (update_ram_word!(game.cinematic_var4, wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if ram_word!(game, cinematic_var4) == 0 || v0 {
        set_ram_word!(game.cinematic_function = ((0x8be7bai32) as u16));
        set_ram_word!(game.cinematicbg_var3 = 20480u16);
        CinematicUpdateSomeBg(game);
        set_ram_word!(game.layer1_x_pos = 256u16);
        set_ram_word!(game.layer1_y_pos = 256u16);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF81 as u16, 0u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF87 as u16, 0u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF8D as u16, 0u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF93 as u16, 0u16, game);
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            set_ram_word!(game.palette_buffer[((i >> 1i32) + 16i32)] = 0u16);
            i -= 2i32;
        }
        j = 30u16;
        while (j as i32 & 0x8000i32) == 0i32 {
            *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                .offset(((j as i32 >> 1i32) + 240i32) as isize) =
                crate::rtl::rom_table_word!(game, 0x8cefe9, (j as i32 >> 1i32) as isize);
            j = (j).wrapping_sub(2u16);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func144(game: &mut Game) {
    let mut j: u16 = 0;
    let v0: i32 = ((4u16).wrapping_mul(ram_word!(game, cinematic_var4))) as i32 >> 1i32;
    let v5: u16 = crate::rtl::rom_table_word!(game, 0x8be5e7, (v0 + 1i32) as isize);
    let mut v1: u16 = crate::rtl::rom_table_word!(game, 0x8be5e7, v0 as isize);
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 16i32)] = *(RomPtr_8C(v1, game) as *mut u16)
        );
        v1 = (v1).wrapping_sub(2u16);
        i -= 2i32;
    }
    let mut v3: u16 = v5;
    j = 30u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
            .offset(((j as i32 >> 1i32) + 240i32) as isize) = *(RomPtr_8C(v3, game) as *mut u16);
        v3 = (v3).wrapping_sub(2u16);
        j = (j).wrapping_sub(2u16);
    }
    if update_ram_word!(game.cinematic_var4, wrapping_add(1), reread) as i32 - 16i32 & 0x8000i32
        == 0
    {
        set_ram_word!(game.cinematic_function = ((0x8be7bai32) as u16));
        ClearCinematicSprites(game);
        SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BF74E as u16, 0x4c00u16, game);
    }
}
pub unsafe fn CalcItemPercentageCount(k: u16, instrp: u16, game: &mut Game) -> u16 {
    let current_block: u64;
    let mut j: u16 = 0;
    let mut v0: u16 = 4u16;
    let mut r18: u16 = 0u16;
    loop {
        let v1: *const u16 = RomPtr_RAM(
            crate::rtl::rom_table_word!(game, 0x8be70d, v0 as isize),
            game,
        ) as *const u16;
        r18 = (r18).wrapping_add(SnesDivide(
            *v1,
            crate::rtl::rom_table_word!(game, 0x8be717, v0 as isize) as u8,
        ));
        v0 = v0.wrapping_sub(1);
        if ((v0 as i32 & 0x8000i32) != 0i32) {
            break;
        }
    }
    let mut i: i32 = 20i32;
    while i >= 0i32 {
        if (crate::rtl::rom_table_word!(game, 0x8be721, (i >> 1i32) as isize) as i32
            & ram_word!(game, collected_items) as i32)
            != 0i32
        {
            r18 = r18.wrapping_add(1);
        }
        i -= 2i32;
    }
    j = 8u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        if (crate::rtl::rom_table_word!(game, 0x8be737, (j as i32 >> 1i32) as isize) as i32
            & ram_word!(game, collected_beams) as i32)
            != 0i32
        {
            r18 = r18.wrapping_add(1);
        }
        j = (j).wrapping_sub(2u16);
    }
    let RegWord: u16 = (r18 as i32 / 10i32) as u16;
    let r22: u16 = (r18 as i32 % 10i32) as u16;
    r18 = (RegWord as i32 / 10i32) as u16;
    let r20: u16 = (RegWord as i32 % 10i32) as u16;
    if r18 != 0 {
        let v5: i32 = ((4u16).wrapping_mul(r18)) as i32 >> 1i32;
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[462i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, v5 as isize);
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[494i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, (v5 + 1i32) as isize);
    }
    let mut v6: u16 = r20;
    if r20 != 0 {
        current_block = 18317007320854588510;
    } else if r18 != 0 {
        v6 = r20;
        current_block = 18317007320854588510;
    } else {
        current_block = 2719512138335094285;
    }
    if current_block == 18317007320854588510 {
        let v7: i32 = ((4u16).wrapping_mul(v6)) as i32 >> 1i32;
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[463i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, v7 as isize);
        (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[495i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, (v7 + 1i32) as isize);
    }
    let v8: i32 = ((4u16).wrapping_mul(r22)) as i32 >> 1i32;
    (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000)).pause_menu_map_tilemap
        [464i32 as usize] = crate::rtl::rom_table_word!(game, 0x8be741, v8 as isize);
    (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000)).pause_menu_map_tilemap
        [496i32 as usize] = crate::rtl::rom_table_word!(game, 0x8be741, (v8 + 1i32) as isize);
    (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000)).pause_menu_map_tilemap
        [465i32 as usize] = 14442u16;
    (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000)).pause_menu_map_tilemap
        [497i32 as usize] = 14458u16;
    instrp
}
pub unsafe fn CinematicFunction_Intro_Func146(k: u16, j: u16, game: &mut Game) -> u16 {
    if ram_word!(game, japanese_text_flag) != 0 {
        let mut i: i32 = 63i32;
        while i >= 0i32 {
            (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(i + 736i32) as usize] =
                crate::rtl::rom_table_word!(game, 0x8cdf5b, i as isize);
            i -= 1;
        }
    }
    j
}
pub unsafe fn CinematicFunction_Intro_Func147(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 126i32;
    while i >= 0i32 {
        *(((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut u16)
            .offset(736) as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 127u16;
        i -= 2i32;
    }
    set_ram_word!(game.cinematic_function = ((0x8be797i32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func148(game: &mut Game) {
    update_ram_word!(game.cinematic_var10, wrapping_sub(2u16));
    if ram_word!(game, cinematic_var10) as i32 + 79i32 & 0x8000i32 != 0 {
        SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BF748 as u16, 0x4c00u16, game);
        set_ram_word!(game.cinematic_function = ((0x8be7bai32) as u16));
        CopySuperMetroidString(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func127(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v4: i16 = 0;
    let mut v0: u16 = (&raw mut game.g_ram as *mut u8)
        .offset(0xe0c)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
    v1 = 0i16;
    loop {
        v4 = v1;
        let v2: *mut u8 = RomPtr_RAM(v0, game);
        *(v2 as *mut u16) = v1 as u16;
        *(v2 as *mut u16).offset(6) = 0u16;
        *(v2 as *mut u16).offset(7) = 0u16;
        let v3: u16 = crate::rtl::rom_table_word!(
            game,
            0x8be9cf,
            ((8i32 * v1 as i32 >> 1i32) + 3i32) as isize
        );
        if v3 != 0 {
            *(v2 as *mut u16).offset(5) = v3;
            let ref mut fresh93 = *(v2 as *mut u16);
            *fresh93 = (*fresh93) | (0x8000u16);
        } else {
            *(v2 as *mut u16).offset(5) = 32u16;
        }
        *(v2 as *mut u16).offset(1) = 128u16;
        *(v2 as *mut u16).offset(3) = 128u16;
        *(v2 as *mut u16).offset(2) = 0u16;
        *(v2 as *mut u16).offset(4) = 0u16;
        v0 = (v0).wrapping_add(16u16);
        v1 += 1;
        if ((v4).wrapping_sub(39i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func149(game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v5: i16 = 0;
    let mut v8: i16 = 0;
    let mut v12: i16 = 0;
    let mut v17: i16 = 0;
    let mut v18: *mut OamEnt = null_mut::<OamEnt>();
    let mut v19: i16 = 0;
    let mut v20: i16 = 0;
    let mut v22: i16 = 0;
    if ram_word!(game, eproj_pre_instr[1]) != 0 {
        let mut n: i32 = 40i32;
        let mut v0: u16 = 0xe0cu16;
        loop {
            let v3: *mut u8 = RomPtr_RAM(v0, game);
            if *(v3 as *mut i16) < 0i16 {
                let v1: *mut u8 = v3;
                v2 = ((*(v1.offset(10) as *mut u16)) as i16).wrapping_sub(1i16);
                *(v1 as *mut u16).offset(5) = v2 as u16;
                if v2 < 0i16 {
                    *(v1 as *mut u16).offset(5) = 32u16;
                    *(v1 as *mut u16) = *v1 as u16;
                }
            } else {
                let v4: i32 = 8i32 * *(v3 as *mut u16) as u8 as i32 >> 1i32;
                if (*(v3 as *mut u16) as i32 & 0xff00i32) - 1024i32 & 0x8000i32 != 0 {
                    let ref mut fresh164 = *(v3 as *mut u16).offset(6);
                    *fresh164 = (*fresh164).wrapping_add(crate::rtl::rom_table_word!(
                        game,
                        0x8be9cf,
                        v4 as isize
                    ));
                    v5 = *(v3.offset(14) as *mut u16) as i16;
                } else {
                    let ref mut fresh165 = *(v3 as *mut u16).offset(6);
                    *fresh165 = (*fresh165 as i32
                        + (crate::rtl::rom_table_word!(game, 0x8be9cf, v4 as isize) as i32
                            + crate::rtl::rom_table_word!(game, 0x8be9cf, v4 as isize) as i32))
                        as u16;
                    v5 = ((crate::rtl::rom_table_word!(game, 0x8be9cf, (v4 + 1i32) as isize))
                        as i16)
                        .wrapping_add((*(v3 as *mut u16).offset(7)) as i16);
                }
                *(v3 as *mut u16).offset(7) =
                    (crate::rtl::rom_table_word!(game, 0x8be9cf, (v4 + 1i32) as isize))
                        .wrapping_add((v5) as u16);
                let mut v23: u16 = *(v3.offset(12) as *mut u16);
                let mut r18: u16 = *(&raw mut v23 as *mut u8).offset(1isize) as i8 as u16;
                let mut r20: u16 = ((v23 as i32) << 8i32 & 0xff00i32) as u16;
                v8 = *(v3.offset(4) as *mut u16) as i16;
                let mut v9: bool = u16::overflowing_add(r20, v8 as u16).1;
                *(v3 as *mut u16).offset(2) = (r20).wrapping_add((v8) as u16);
                let ref mut fresh166 = *(v3 as *mut u16).offset(1);
                *fresh166 = (*fresh166 as i32 + (r18 as i32 + v9 as i32)) as u16;
                let mut v24: u16 = *(v3.offset(14) as *mut u16);
                r18 = *(&raw mut v24 as *mut u8).offset(1isize) as i8 as u16;
                r20 = ((v24 as i32) << 8i32 & 0xff00i32) as u16;
                v12 = *(v3.offset(8) as *mut u16) as i16;
                v9 = u16::overflowing_add(r20, v12 as u16).1;
                *(v3 as *mut u16).offset(4) = (r20).wrapping_add((v12) as u16);
                let ref mut fresh167 = *(v3 as *mut u16).offset(3);
                *fresh167 = (*fresh167 as i32 + (r18 as i32 + v9 as i32)) as u16;
            }
            v0 = (v0).wrapping_add(16u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        let mut n2: i32 = 40i32;
        let mut v13: u16 = ram_word!(game, oam_next_ptr);
        let mut v14: u16 = 3596u16;
        loop {
            let v15: *mut u8 = RomPtr_RAM(v14, game);
            let v16: *mut u8 = v15;
            if *(v15 as *mut i16) >= 0i16 {
                v17 = *(v15.offset(2) as *mut u16) as i16;
                if *v15.offset(3) != 0u8
                    || (v17 as i32 - 4i32 & 0xff00i32) != 0i32
                    || {
                        v18 = gOamEnt(v13 as i32, game);
                        *(&raw mut (*v18).xcoord as *mut u16) = ((v17) as u16).wrapping_sub(4u16);
                        v19 = *(v16.offset(6) as *mut u16) as i16;
                        *v16.offset(7) != 0u8
                    }
                    || (v19 as i32 - 4i32 & 0xff00i32) != 0i32
                {
                    let v21: *mut u8 = v15;
                    *(v21 as *mut u16).offset(5) = 32u16;
                    *(v21 as *mut u16).offset(1) = 128u16;
                    *(v21 as *mut u16).offset(3) = 128u16;
                    *(v21 as *mut u16).offset(2) = 0u16;
                    *(v21 as *mut u16).offset(4) = 0u16;
                    *(v21 as *mut u16).offset(6) = 0u16;
                    *(v21 as *mut u16).offset(7) = 0u16;
                    *(v21 as *mut u16) = *v21 as u16;
                } else {
                    *(&raw mut (*v18).ycoord as *mut u16) = ((v19) as u16).wrapping_sub(4u16);
                    v20 = ((*(v16.offset(10) as *mut u16)) as i16).wrapping_sub(1i16);
                    *(v16 as *mut u16).offset(5) = v20 as u16;
                    if v20 <= 0i16 {
                        v22 = *(v16 as *mut u16) as i16;
                        *(v16 as *mut u16).offset(5) = crate::rtl::rom_table_word!(
                            game,
                            0x8be9cf,
                            ((8i32 * *(v16 as *mut u16) as u8 as i32 >> 1i32) + 2i32) as isize,
                        );
                        *(v16 as *mut u16) = ((v22) as u16).wrapping_add(512u16);
                    }
                    if *v16.offset(1) != 0 {
                        *(&raw mut (*(gOamEnt as unsafe fn(i32, &mut Game) -> *mut OamEnt)(
                            v13 as i32, game,
                        ))
                        .charnum as *mut u16) = crate::rtl::rom_table_word!(
                            game,
                            0x8be9a7,
                            (((*(v16 as *mut u16) as i32 & 0xff00i32) >> 8i32) as u16 as i32
                                >> 1i32) as isize,
                        );
                        v13 = (v13).wrapping_add(4u16);
                    }
                }
            }
            v14 = (v14).wrapping_add(16u16);
            n2 -= 1;
            if (n2 == 0) {
                break;
            }
        }
        set_ram_word!(game.oam_next_ptr = v13);
    }
}
pub unsafe fn CinematicFunction_Intro_Func150(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr8[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
    set_ram_word!(game.cinematicspr_goto_timer[v1] = 260u16);
}
pub unsafe fn CinematicFunction_Intro_Func151(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        128u32,
    );
    set_ram_word!(game.cinematicspr_arr6[v1] = ram_word!(game, cinematicspr_arr6[v1]));
    if update_ram_word!(game.cinematicbg_arr7[v1], wrapping_add(1), reread) as i32 - 272i32
        & 0x8000i32
        == 0
    {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func152(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func153(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = -128i32 as u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func154(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func155(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn CinematicFunction_Intro_Func156(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 96u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 1024u16);
}
pub unsafe fn CinematicFunction_Intro_Func157(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 160u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 512u16);
}
pub unsafe fn CinematicFunction_Intro_Func158(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr8[v1] = 160u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 0u16);
}
pub unsafe fn CinematicFunction_Intro_Func159(j: u16, game: &mut Game) {
    CinematicFunction_Intro_Func160(j, ram_word!(game, substate), game);
    set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 156u16);
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func161(j: u16, game: &mut Game) {
    CinematicFunction_Intro_Func160(j, ram_word!(game, suit_pickup_light_beam_pos), game);
    set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 164u16);
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func162(j: u16, game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 172u16);
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func163(j: u16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut u8).offset(0xdf0) as *mut u16),
        game,
    );
    set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 180u16);
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func164(j: u16, game: &mut Game) {
    CinematicFunction_Intro_Func160(
        j,
        *((&raw mut game.g_ram as *mut u8).offset(0xdf2) as *mut u16),
        game,
    );
    set_ram_word!(game.cinematicbg_arr7[(j as i32 >> 1i32)] = 188u16);
    CinematicFunction_Intro_Func158(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func160(j: u16, a: u16, game: &mut Game) {
    update_ram_word!(
        game.cinematicspr_instr_ptr[(j as i32 >> 1i32)],
        |fresh3| (fresh3 as i32 + 8i32 * a as i32) as u16
    );
}
pub unsafe fn CinematicFunction_Intro_Func165(j: u16, game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr8[(j as i32 >> 1i32)] = -96i32 as u16);
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_F0B8(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    if ram_word!(game, cinematic_spawn_param) != 0 {
        set_ram_word!(game.cinematicbg_arr7[v1] = 384u16);
        set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
        update_ram_word!(game.cinematicspr_instr_ptr[v1], wrapping_add(4u16));
    } else {
        set_ram_word!(game.cinematicbg_arr7[v1] = 128u16);
        set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
    };
}
pub unsafe fn CinematicFunction_Intro_Func166(j: u16, game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr8[(j as i32 >> 1i32)] = -32i32 as u16);
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func167(j: u16, game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr8[(j as i32 >> 1i32)] = 288u16);
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func168(j: u16, game: &mut Game) {
    set_ram_word!(game.cinematicbg_arr8[(j as i32 >> 1i32)] = 352u16);
    CinematicFunction_F0B8(j, game);
}
pub unsafe fn CinematicFunction_Intro_Func169(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 320u16);
    if ram_word!(game, cinematic_spawn_param) != 0 {
        set_ram_word!(game.cinematicbg_arr8[v1] = 448u16);
    } else {
        set_ram_word!(game.cinematicbg_arr8[v1] = 192u16);
    }
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func170(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = -64i32 as u16);
    if ram_word!(game, cinematic_spawn_param) != 0 {
        set_ram_word!(game.cinematicbg_arr8[v1] = -192i32 as u16);
    } else {
        set_ram_word!(game.cinematicbg_arr8[v1] = 64u16);
    }
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func171(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 120u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 136u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func172(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 120u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 152u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn CinematicFunction_Intro_Func173(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 124u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 108u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
}
pub unsafe fn CinematicFunction_Intro_Func174(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 121u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 107u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 2560u16);
}
pub unsafe fn CinematicFunction_Intro_Func175(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 530u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 231u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
    set_ram_word!(game.cinematicspr_goto_timer[v1] = 8u16);
}
pub unsafe fn CinematicFunction_Intro_Func176(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 246u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 519u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
    set_ram_word!(game.cinematicspr_goto_timer[v1] = 8u16);
}
pub unsafe fn CinematicFunction_Intro_Func177(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 385u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 366u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func178(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.cinematicbg_arr7[v1] = 391u16);
    set_ram_word!(game.cinematicbg_arr8[v1] = 384u16);
    set_ram_word!(game.cinematicbg_arr9[v1] = 3584u16);
}
pub unsafe fn CinematicFunction_Intro_Func179(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(
        game.cinematicbg_arr8[v1],
        wrapping_add(ram_word!(game, cinematicspr_goto_timer[v1]))
    );
    let v2: u16 = (ram_word!(game, cinematicbg_arr7[v1]))
        .wrapping_sub(ram_word!(game, cinematicspr_goto_timer[v1]));
    set_ram_word!(game.cinematicbg_arr7[v1] = v2);
    if v2 as i32 - 395i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicbg_arr7[v1] = 394u16);
        set_ram_word!(game.cinematicbg_arr8[v1] = 367u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        SpawnPalfxObject(addr_kPalfx_E200 as u16, game);
    } else {
        update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(2u16));
    };
}
pub unsafe fn CinematicFunction_Intro_Func180(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(
        game.cinematicbg_arr8[v1],
        wrapping_sub(ram_word!(game, cinematicspr_goto_timer[v1]))
    );
    let v2: u16 = (ram_word!(game, cinematicspr_goto_timer[v1]))
        .wrapping_add(ram_word!(game, cinematicbg_arr7[v1]));
    set_ram_word!(game.cinematicbg_arr7[v1] = v2);
    if v2 as i32 - 382i32 & 0x8000i32 != 0 {
        update_ram_word!(game.cinematicspr_goto_timer[v1], wrapping_add(2u16));
    } else {
        set_ram_word!(game.cinematicbg_arr7[v1] = 382u16);
        set_ram_word!(game.cinematicbg_arr8[v1] = 383u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
    };
}
pub unsafe fn CinematicSprInstr_Func181(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 3u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 34u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 100u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 84u8;
    set_ram_word!(game.cinematic_var4 = 0u16);
    set_ram_word!(game.cinematic_function = ((0x8be58ai32) as u16));
    j
}
pub unsafe fn CinematicSprInstr_Func182(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnPalfxObject(addr_kPalfx_E1C4 as u16, game);
    set_ram_word!(game.palette_buffer[254] = 1u16);
    j
}
pub unsafe fn CinematicSprInstr_Func183(k: u16, j: u16, game: &mut Game) -> u16 {
    let v0: u16 = 0u16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEB5 as u16, v0, game);
    set_ram_word!(game.palette_buffer[0] = 0x7fffu16);
    j
}
pub unsafe fn CinematicFunction_Intro_Func184(j: u16, game: &mut Game) {
    if ram_word!(game, cinematicspr_instr_ptr[15]) == 0 {
        let v2: i32 = j as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v2] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v2] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicSprInstr_Func185(k: u16, j: u16, game: &mut Game) -> u16 {
    let mut v0: u16 = 0;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 17u8;
    *(&raw mut v0 as *mut u8).offset(0) = 124u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 124u8;
    SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEBB as u16, 0u16, game);
    SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEF7 as u16, 2u16, game);
    SpawnPalfxObject(addr_kPalfx_E1C8 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1D0 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1E8 as u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func186(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 3u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 18u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 51u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x58) = 116u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 120u8;
    SpawnPalfxObject(addr_kPalfx_E1CC as u16, game);
    SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEC1 as u16, 6u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func187(k: u16, j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0u8;
    set_ram_word!(game.palette_buffer[0] = 0x7fffu16);
    set_ram_word!(game.palette_buffer[128] = 0x7fffu16);
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        set_ram_word!(game.palette_buffer[((i >> 1i32) + 16i32)] = 0x7fffu16);
        i -= 2i32;
    }
    set_ram_word!(game.cinematic_function = ((0x8bdb9ei32) as u16));
    set_ram_word!(game.cinematic_var4 = 120u16);
    j
}
pub unsafe fn CinematicSprPreInstr_F35A(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bdca5i32 & 0xffffi32 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bf375i32) as u16));
        set_ram_word!(game.cinematicspr_arr7[v1] = 0x4000u16);
        set_ram_word!(game.cinematicspr_goto_timer[v1] = 0u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func189(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1b7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        -32i32 as u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a7d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1add) as *mut u16).offset(v1 as isize),
        (ram_word!(game, cinematicspr_goto_timer[v1]) as u32) << 16i32
            | ram_word!(game, cinematicspr_arr7[v1]) as u32,
    );
    CinematicFunction_Intro_Func189b(k, game);
}
pub unsafe fn CinematicFunction_Intro_Func189b(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bde63i32 & 0xffffi32 {
        let v5: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v5] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v5] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicSprInstr_Func190(k: u16, j: u16, game: &mut Game) -> u16 {
    let v0: u16 = 0u16;
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEECD as u16, v0, game);
    j
}
pub unsafe fn CinematicFunction_Intro_Func191(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bde80i32 & 0xffffi32 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicSprInstr_Func192(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEFD as u16, 0u16, game);
    let hours1: i32 = ram_word!(game, game_time_hours) as i32 / 10i32;
    let hours2: i32 = ram_word!(game, game_time_hours) as i32 % 10i32;
    set_ram_word!(game.substate = hours1 as u16);
    set_ram_word!(game.suit_pickup_light_beam_pos = hours2 as u16);
    let min1: i32 = ram_word!(game, game_time_minutes) as i32 / 10i32;
    let min2: i32 = ram_word!(game, game_time_minutes) as i32 % 10i32;
    *((&raw mut game.g_ram as *mut u8).offset(0xdf0) as *mut u16) = min1 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xdf2) as *mut u16) = min2 as u16;
    j
}
pub unsafe fn CinematicSprInstr_Func193(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF03 as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func194(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF09 as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func195(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF0F as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func196(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF15 as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func197(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF1B as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func198(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 0u16);
    set_ram_word!(game.cinematic_function = ((0x8bde64i32) as u16));
    j
}
pub unsafe fn CinematicFunction_Intro_Func199(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var6) as i32 - 176i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bf464i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func200(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_arr7[v1] = ram_word!(game, cinematicspr_arr7[v1]));
    update_ram_word!(game.cinematicbg_arr8[v1], wrapping_add(1));
}
pub unsafe fn CinematicFunction_Intro_Func201(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var6) as i32 - 176i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bf487i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func202(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_arr7[v1] = ram_word!(game, cinematicspr_arr7[v1]));
    update_ram_word!(game.cinematicbg_arr8[v1], wrapping_sub(1));
}
pub unsafe fn CinematicFunction_Intro_Func203(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var6) as i32 - 96i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bf4aai32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func204(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_arr7[v1] = ram_word!(game, cinematicspr_arr7[v1]));
    update_ram_word!(game.cinematicbg_arr8[v1], wrapping_sub(2u16));
    set_ram_word!(game.cinematicspr_arr6[v1] = ram_word!(game, cinematicspr_arr6[v1]));
    update_ram_word!(game.cinematicbg_arr7[v1], wrapping_sub(1));
}
pub unsafe fn CinematicFunction_Intro_Func205(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_var6) as i32 - 96i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicspr_preinstr_func[(k as i32 >> 1i32)] = ((0x8bf4e0i32) as u16));
    }
}
pub unsafe fn CinematicFunction_Intro_Func206(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.cinematicspr_arr7[v1] = ram_word!(game, cinematicspr_arr7[v1]));
    update_ram_word!(game.cinematicbg_arr8[v1], wrapping_add(2u16));
    set_ram_word!(game.cinematicspr_arr6[v1] = ram_word!(game, cinematicspr_arr6[v1]));
    update_ram_word!(game.cinematicbg_arr7[v1], wrapping_add(1));
}
pub unsafe fn CinematicFunction_Intro_Func207(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bdb9di32 & 0xffffi32 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicSprInstr_Func208(k: u16, j: u16, game: &mut Game) -> u16 {
    SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF3F as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprPreInstr_F528(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 + 80i32 & 0x8000i32 != 0 {
        *(&raw mut game.g_ram as *mut u8).offset(0x52) = 3u8;
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BED95 as u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8bf5ddi32) as u16));
        set_ram_word!(game.cinematicbg_arr9[v1] = 3072u16);
    }
}
pub unsafe fn CinematicSprInstr_Func211(k: u16, j: u16, game: &mut Game) -> u16 {
    if ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 != 0 {
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF7B as u16, 2u16, game);
    } else {
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF75 as u16, 2u16, game);
    }
    SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF6F as u16, 0u16, game);
    j
}
pub unsafe fn CinematicSprPreInstr_F57F(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 + 80i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
    }
}
pub unsafe fn CinematicSprInstr_Func213(k: u16, j: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicbg_arr7[(v1 + 1i32)] = 120u16);
    } else {
        set_ram_word!(game.cinematicbg_arr7[(v1 + 1i32)] = 118u16);
    }
    set_ram_word!(game.cinematicbg_arr8[(v1 + 1i32)] = 120u16);
    j
}
pub unsafe fn CinematicSprInstr_Func214(k: u16, j: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, game_time_hours) as i32 - 10i32 & 0x8000i32 != 0 {
        set_ram_word!(game.cinematicbg_arr7[(v1 + 1i32)] = 121u16);
        set_ram_word!(game.cinematicbg_arr8[(v1 + 1i32)] = 116u16);
    } else {
        set_ram_word!(game.cinematicbg_arr7[(v1 + 1i32)] = 120u16);
        set_ram_word!(game.cinematicbg_arr8[(v1 + 1i32)] = 114u16);
    }
    j
}
pub unsafe fn CinematicFunction_Intro_Func215(k: u16, game: &mut Game) {
    CinematicFunction_Intro_Func216(game);
    CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, cinematicbg_arr8[v1]) as i32 - 136i32 & 0x8000i32 == 0 {
        set_ram_word!(game.cinematicbg_arr8[v1] = 136u16);
        set_ram_word!(game.cinematicspr_instr_timer[v1] = 1u16);
        set_ram_word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BED9D as u16);
        set_ram_word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
    }
}
pub unsafe fn CinematicSprInstr_Func217(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.cinematicbg_arr9[(k as i32 >> 1i32)] = 3584u16);
    set_ram_word!(game.cinematic_function = ((0x8be3aei32) as u16));
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 16u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 2u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = -127i32 as u8;
    set_ram_word!(game.cinematic_var8 = -4i32 as u16);
    set_ram_word!(game.cinematic_var10 = -8i32 as u16);
    set_ram_word!(game.cinematic_var4 = 16u16);
    set_ram_word!(game.cinematic_var14 = 32u16);
    set_ram_word!(game.cinematic_var17 = 32u16);
    CopyPalettesToFadingPalettes(game);
    DecomposePaletteDataForFading(game);
    QueueSfx1_Max15(0x22u16, game);
    j
}
pub unsafe fn CinematicSprInstr_Func218(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.samus_y_subspeed = 0u16);
    set_ram_word!(game.samus_y_speed = -16i32 as u16);
    j
}
pub unsafe fn CinematicFunction_Intro_Func210_EndingSamusFall(k: u16, game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0xb2e) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0xb2c) as *mut u16,
        0x3800u32,
    );
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a9d) as *mut u16).offset(v1 as isize),
        ((&raw mut game.g_ram as *mut u8).offset(0x1afd) as *mut u16).offset(v1 as isize),
        (ram_word!(game, samus_y_speed) as u32) << 16i32 | ram_word!(game, samus_y_subspeed) as u32,
    );
}
pub unsafe fn CinematicFunction_Intro_Func216(game: &mut Game) {
    if ram_word!(game, cinematic_var14) as i32 - 16i32 & 0x8000i32 != 0 {
        let v0: u16 = (2u16).wrapping_mul(ram_word!(game, cinematic_var14));
        let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
        (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 2048u16;
        v1 = (v1).wrapping_add(2u16);
        let v2: i32 = v0 as i32 >> 1i32;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8bf6b8, v2 as isize);
        v1 = (v1).wrapping_add(2u16);
        let fresh28 = v1;
        v1 = v1.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh28 as i32,
            game,
        ))
        .size as *mut u8)
            .offset(0) = 127u8;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8bf6d8, v2 as isize);
        set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(2u16));
        update_ram_word!(game.cinematic_var14, wrapping_add(1));
    }
}
pub unsafe fn CinematicFunction_Intro_Func219(k: u16, j: u16, game: &mut Game) -> u16 {
    DisableTextGlowObjects_(game);
    let mut v0: u16 = 8u16;
    loop {
        set_ram_word!(
            game.palette_buffer[(v0 as i32 >> 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ce7e9, (v0 as i32 >> 1i32) as isize)
        );
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 0u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x51) = 0x80u8;
    set_ram_word!(game.screen_fade_delay = 0u16);
    set_ram_word!(game.screen_fade_counter = 0u16);
    set_ram_word!(game.cinematic_var9 = 0u16);
    set_ram_word!(game.cinematic_var10 = 0u16);
    set_ram_word!(game.eproj_pre_instr[1] = 1u16);
    set_ram_word!(game.cinematic_function = ((0x8be0f4i32) as u16));
    set_ram_word!(game.cinematic_var4 = 60u16);
    j
}
pub unsafe fn CallCinematicFunction(ea: u32, game: &mut Game) {
    match ea {
        9148999 => CinematicFunctionNone(),
        9149288 => CinematicFunctionOpening(game),
        9149719 => CinematicFunc_Func2(game),
        9149840 => CinematicFunc_Func4(game),
        9149970 => CinematicFunc_Func5(game),
        9150091 => CinematicFunc_Func7(game),
        9150248 => (),
        9150249 => CinematicFunc_Func1(game),
        9150290 => CinematicFunc_Func10(game),
        9150382 => CinematicFunc_Func9(game),
        9151375 => (),
        9151376 => (),
        9151377 => CinematicFunc_Nothing(),
        9151381 => CinematicFunction_Intro_Initial(game),
        9151911 => CinematicFunction_Intro_FadeIn(game),
        9151933 => CinematicFunction_Intro_DrawInitJpn(game),
        9151992 => CinematicFunction_Intro_LastMetroidCaptivity(game),
        9152019 => CinematicFunc_Intro_QueueGalaxyIsAtPeace(game),
        9152057 => CinematicFunc_Intro_WaitForQueueWait4secs(game),
        9152076 => CinematicFunc_Intro_FadeOut(game),
        9152099 => CinematicFunc_Intro_WaitForFadeOut(game),
        9152111 => CinematicFunc_Intro_SetupTextPage2(game),
        9152555 => CinematicFunc_Intro_WaitFadeinShowText(game),
        9152586 => CinematicFunc_Intro_WaitForFadeinSleep(game),
        9154232 => CinematicFunction_Intro_WaitInputSetupMotherBrainFight(game),
        9154412 => CinematicFunction_Intro_WaitInputSetupBabyMetroid(game),
        9154802 => CinematicFunction_Intro_Func11(game),
        9154851 => CinematicFunction_Intro_Func12(game),
        9155034 => CinematicFunction_Intro_Func15(game),
        9155079 => CinematicFunction_Intro_Func15b(game),
        9155152 => CinematicFunction_Intro_XfadeGameplayFade(game),
        9155282 => CinematicFunction_Intro_XfadeScientistFade(game),
        9155423 => CinematicFunction_Intro_Page2(game),
        9155440 => CinematicFunction_Intro_Page3(game),
        9155457 => CinematicFunction_Intro_Page4(game),
        9155474 => CinematicFunction_Intro_Page5(game),
        9155572 => CinematicFunction_Intro_Fadestuff(game),
        9155672 => CinematicFunction_Intro_Fadestuff2(game),
        9156399 => CinematicFunction_Intro_Func34(game),
        9157792 => CinematicFunction_Intro_Func54(game),
        9158116 => CinematicFunction_Intro_Func55(game),
        9158137 => CinematicFunction_Intro_Func56(game),
        9158618 => CinematicFunction_Intro_Func67(game),
        9158853 => CinematicFunction_Intro_Func72(game),
        9158912 => CinematicFunction_Intro_Func73(game),
        9158939 => CinematicFunctionBlackoutFromCeres(game),
        9159396 => CinematicFunction_Intro_Func74(game),
        9159409 => CinematicFunction_Intro_Func75(game),
        9159493 => CinematicFunction_Intro_Func76(game),
        9160138 => CinematicFunction_Intro_Func77(game),
        9160208 => CinematicFunction_Intro_Func84(game),
        9160231 => CinematicFunction_Intro_Func85(game),
        9160345 => CinematicFunction_Intro_Func86(game),
        9160604 => CinematicFunction_Intro_Func87(game),
        9160650 => CinematicFunction_Intro_Func88(game),
        9161209 => CinematicFunction_Intro_Func105(game),
        9161270 => CinematicFunction_Intro_Func106(game),
        9161349 => CinematicFunction_Intro_Func107(game),
        9161424 => CinematicFunction_Intro_Func108(game),
        9161438 => (),
        9161439 => CinematicFunction_Intro_Func95(game),
        9163904 => CinematicFunctionEscapeFromCebes(game),
        9164503 => CinematicFunction_Intro_Func109(game),
        9164530 => CinematicFunction_Intro_Func110(game),
        9164545 => CinematicFunction_Intro_Func111(game),
        9164593 => CinematicFunction_Intro_Func112(game),
        9164792 => CinematicFunction_Intro_Func113(game),
        9164807 => CinematicFunction_Intro_Func114(game),
        9164855 => CinematicFunction_Intro_Func115(game),
        9165314 => CinematicFunction_Intro_Func116(game),
        9165338 => CinematicFunction_Intro_Func117(game),
        9165523 => CinematicFunction_Intro_Func118(game),
        9165725 => (),
        9165726 => CinematicFunction_Intro_Func119(game),
        9165764 => CinematicFunction_Intro_Func120(game),
        9165989 => CinematicFunction_Intro_Func121(game),
        9166146 => CinematicFunction_Intro_Func123(game),
        9166285 => CinematicFunction_Intro_Func124(game),
        9166435 => (),
        9166436 => CinematicFunction_Intro_Func125(game),
        9166464 => CinematicFunction_Intro_Func126(game),
        9167092 => CinematicFunction_Intro_Func129(game),
        9167120 => CinematicFunction_Intro_Func130(game),
        9167192 => CinematicFunction_Intro_Func131(game),
        9167248 => CinematicFunction_Intro_Func132(game),
        9167314 => CinematicFunction_Intro_Func134(game),
        9167461 => CinematicFunction_Intro_Func136(game),
        9167507 => CinematicFunction_Intro_Func135(game),
        9167581 => CinematicFunction_Intro_Func137(game),
        9167636 => CinematicFunction_Intro_Func138(game),
        9167682 => CinematicFunction_Intro_Func145(game),
        9167790 => CinematicFunction_Intro_Func139(game),
        9168010 => CinematicFunction_Intro_Func141(game),
        9168132 => CinematicFunction_Intro_Func143(game),
        9168266 => CinematicFunction_Intro_Func144(game),
        9168791 => CinematicFunction_Intro_Func148(game),
        9168826 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallCinematicSprInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    match ea {
        9147448 => CinematicSprInstr_Delete(k, j, game),
        9147458 => CinematicSprInstr_Sleep(k, j, game),
        9147468 => CinematicSprInstr_SetPreInstr(k, j, game),
        9147479 => CinematicSprInstr_ClearPreInstr(k, j, game),
        9147554 => CinematicSprInstr_GotoRel(k, j, game),
        9147580 => CinematicSprInstr_Goto(k, j, game),
        9147587 => CinematicSprInstr_DecTimerGoto(k, j, game),
        9147597 => CinematicSprInstr_DecTimerGotoRel(k, j, game),
        9147606 => CinematicSprInstr_SetTimer(k, j, game),
        9149665 => CinematicSprInstr_9CE1(k, j, game),
        9149789 => CinematicSprInstr_9D5D(k, j, game),
        9149910 => CinematicSprInstr_9DD6(k, j, game),
        9150040 => CinematicSprInstr_Func8(k, j, game),
        9150157 => CinematicSprInstr_Func9(k, j, game),
        9150192 => CinematicSprInstr_sub_8B9EF0(k, j, game),
        9150233 => CinematicSprInstr_9F19(k, j, game),
        9152792 => CinematicSprInstr_SpawnMetroidEggParticles(k, j, game),
        9155382 => CinematicSprInstr_StartIntroPage2(k, j, game),
        9155390 => CinematicSprInstr_StartIntroPage3(k, j, game),
        9155398 => CinematicSprInstr_StartIntroPage4(k, j, game),
        9155406 => CinematicSprInstr_StartIntroPage5(k, j, game),
        9156805 => CinematicSprInstr_Func43(k, j, game),
        9157153 => CinematicSprInstr_SpawnIntroRinkas01(k, j, game),
        9157174 => CinematicSprInstr_SpawnIntroRinkas23(k, j, game),
        9158806 => CinematicCommonInstr_Func69(k, j, game),
        9158818 => CinematicCommonInstr_Func70(k, j, game),
        9159684 => CinematicSprInstr_SpawnCeresExplosions1(k, j, game),
        9159948 => CinematicSprInstr_SpawnCeresExplosions3(k, j, game),
        9161125 => CinematicSprInstr_C9A5(k, j, game),
        9161135 => CinematicSprInstr_C9AF(k, j, game),
        9161149 => CinematicSprInstr_C9BD(k, j, game),
        9161159 => CinematicSprInstr_C9C7(k, j, game),
        9171550 => CinematicSprInstr_Func181(k, j, game),
        9171588 => CinematicSprInstr_Func182(k, j, game),
        9171605 => CinematicSprInstr_Func183(k, j, game),
        9171639 => CinematicSprInstr_Func185(k, j, game),
        9171706 => CinematicSprInstr_Func186(k, j, game),
        9171755 => CinematicSprInstr_Func187(k, j, game),
        9171888 => CinematicSprInstr_Func190(k, j, game),
        9171918 => CinematicSprInstr_Func192(k, j, game),
        9171995 => CinematicSprInstr_Func193(k, j, game),
        9172004 => CinematicSprInstr_Func194(k, j, game),
        9172013 => CinematicSprInstr_Func195(k, j, game),
        9172022 => CinematicSprInstr_Func196(k, j, game),
        9172031 => CinematicSprInstr_Func197(k, j, game),
        9172040 => CinematicSprInstr_Func198(k, j, game),
        9172253 => CinematicSprInstr_Func208(k, j, game),
        9172308 => CinematicSprInstr_Func211(k, j, game),
        9172375 => CinematicSprInstr_Func213(k, j, game),
        9172410 => CinematicSprInstr_Func214(k, j, game),
        9172484 => CinematicSprInstr_Func217(k, j, game),
        9172561 => CinematicSprInstr_Func218(k, j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn CallCinematicSprPreInstr(ea: u32, j: u16, game: &mut Game) {
    match ea {
        9158617 => (),
        9147353 => (),
        9147487 => (),
        9149647 => sub_8B9CCF(j, game),
        9152724 => (),
        9152744 => SetSomeStuffForSpriteObject_4_MetroidEgg(j, game),
        9152771 => CinematicSprPreInstr_A903(j, game),
        9152916 => SetSomeStuffForSpriteObject_7(j, game),
        9153203 => SetSomeStuffForSpriteObject_9(j, game),
        9153896 => SetSomeStuffForSpriteObject_11(j, game),
        9153958 => SetSomeStuffForSpriteObject_13(j, game),
        9154074 => SetSomeStuffForSpriteObject_18(j, game),
        9156486 => CinematicFunction_Intro_Func36(j, game),
        9156623 => CinematicFunction_Intro_Func39(j, game),
        9156654 => CinematicSprPreInstr_B82E(j, game),
        9156824 => CinematicFunction_Intro_Func44(j, game),
        9156923 => CinematicFunction_Intro_Func45(j, game),
        9157135 => CinematicFunction_Intro_Func48(j, game),
        9157214 => CinematicFunction_Intro_Func50(j, game),
        9157235 => CinematicFunction_Intro_Func51(j, game),
        9157389 => CinematicFunction_Intro_Func52(j, game),
        9157412 => CinematicFunction_Intro_Func53(j, game),
        9158325 => CinematicFunction_Intro_Func58(j, game),
        9158453 => CinematicFunction_Intro_Func60(j, game),
        9158495 => CinematicFunction_Intro_Func62(j, game),
        9158537 => CinematicFunction_Intro_Func64(j, game),
        9158598 => CinematicFunction_Intro_Func66(j, game),
        9159817 => CinematicSprPreInstr_C489(j, game),
        9160066 => CinematicFunction_Intro_Func82(j, game),
        9160782 => CinematicFunction_Intro_Func90(j, game),
        9160797 => CinematicFunction_Intro_Func91(j, game),
        9160874 => CinematicFunction_Intro_Func93(j, game),
        9160889 => CinematicFunction_Intro_Func94(j, game),
        9160953 => CinematicFunction_Intro_Func96(j, game),
        9160968 => CinematicFunction_Intro_Func97(j, game),
        9170866 => CinematicFunction_Intro_Func151(j, game),
        9171431 => CinematicFunction_Intro_Func179(j, game),
        9171495 => CinematicFunction_Intro_Func180(j, game),
        9171621 => CinematicFunction_Intro_Func184(j, game),
        9171802 => CinematicSprPreInstr_F35A(j, game),
        9171829 => CinematicFunction_Intro_Func189(j, game),
        9171867 => CinematicFunction_Intro_Func189b(j, game),
        9171897 => CinematicFunction_Intro_Func191(j, game),
        9172053 => CinematicFunction_Intro_Func199(j, game),
        9172068 => CinematicFunction_Intro_Func200(j, game),
        9172088 => CinematicFunction_Intro_Func201(j, game),
        9172103 => CinematicFunction_Intro_Func202(j, game),
        9172123 => CinematicFunction_Intro_Func203(j, game),
        9172138 => CinematicFunction_Intro_Func204(j, game),
        9172177 => CinematicFunction_Intro_Func205(j, game),
        9172192 => CinematicFunction_Intro_Func206(j, game),
        9172231 => CinematicFunction_Intro_Func207(j, game),
        9172252 => (),
        9172264 => CinematicSprPreInstr_F528(j, game),
        9172351 => CinematicSprPreInstr_F57F(j, game),
        9172445 => CinematicFunction_Intro_Func215(j, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallCinematicBgPreInstr(ea: u32, j: u16, game: &mut Game) {
    match ea {
        9147353 => (),
        9155772 => CinematicBgPreInstr_SamusBlink(j, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallCinematicBgInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    match ea {
        9148056 => CinematicBgInstr_Delete(k, j, game),
        9148190 => CinematicBgInstr_Goto(k, j, game),
        9154004 => CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game),
        9154115 => CinematicBgInstr_HandleCreateJpnText_Page1(k, j, game),
        9154139 => CinematicBgInstr_SpawnMarkerWaitInput_Page1(k, j, game),
        9154169 => CinematicBgInstr_HandleCreateJpnText_Page2(k, j, game),
        9154193 => CinematicBgInstr_SpawnMarkerWaitInput_Page2(k, j, game),
        9154676 => CinematicBgInstr_HandleCreateJpnText_Page3(k, j, game),
        9154700 => CinematicBgInstr_SpawnMarkerWaitInput_Page3(k, j, game),
        9154739 => CinematicBgInstr_HandleCreateJpnText_Page4(k, j, game),
        9154763 => CinematicBgInstr_SpawnMarkerWaitInput_Page4(k, j, game),
        9154971 => CinematicBgInstr_HandleCreateJpnText_Page5(k, j, game),
        9154995 => CinematicBgInstr_SpawnMarkerWaitInput_Page5(k, j, game),
        9155112 => CinematicBgInstr_Func16(k, j, game),
        9155136 => CinematicBgInstr_Func17(k, j, game),
        9158806 => CinematicCommonInstr_Func69(k, j, game),
        9158818 => CinematicCommonInstr_Func70(k, j, game),
        9168423 => CalcItemPercentageCount(k, j, game),
        9168745 => CinematicFunction_Intro_Func146(k, j, game),
        9168768 => CinematicFunction_Intro_Func147(k, j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn CallCinematicSpriteObjectSetup(ea: u32, j: u16, game: &mut Game) {
    match ea {
        9147353 => (),
        9149210 => CinematicSpriteInit_7(j, game),
        9149229 => CinematicSpriteInit_8(j, game),
        9149248 => CinematicSpriteInit_9(j, game),
        9149628 => CinematicSpriteInit_0(j, game),
        9149770 => CinematicSpriteInit_1(j, game),
        9149891 => CinematicSpriteInit_2(j, game),
        9150021 => CinematicSpriteInit_3(j, game),
        9150131 => CinematicSpriteInit_4(j, game),
        9150166 => CinematicSpriteInit_5(j, game),
        9150207 => CinematicSpriteInit_6(j, game),
        9152705 => SetSomeStuffForSpriteObject_2(j, game),
        9152725 => SetSomeStuffForSpriteObject_3(j, game),
        9152856 => SetSomeStuffForSpriteObject_6(j, game),
        9153178 => SetSomeStuffForSpriteObject_8(j, game),
        9153877 => SetSomeStuffForSpriteObject_10(j, game),
        9153939 => SetSomeStuffForSpriteObject_12(j, game),
        9154055 => SetSomeStuffForSpriteObject_17(j, game),
        9156467 => CinematicFunction_Intro_Func35(j, game),
        9156758 => CinematicFunction_Intro_Func42(j, game),
        9157005 => CinematicFunction_Intro_Func46(j, game),
        9157076 => CinematicFunction_Intro_Func47(j, game),
        9157195 => CinematicFunction_Intro_Func49(j, game),
        9158270 => CinematicFunction_Intro_Func57(j, game),
        9158434 => CinematicFunction_Intro_Func59(j, game),
        9158476 => CinematicFunction_Intro_Func61(j, game),
        9158518 => CinematicFunction_Intro_Func63(j, game),
        9158560 => CinematicFunction_Intro_Func65(j, game),
        9158787 => CinematicFunction_Intro_Func68(j, game),
        9158834 => CinematicFunction_Intro_Func71(j, game),
        9159732 => CinematicFunction_Intro_Func78(j, game),
        9159865 => CinematicFunction_Intro_Func80(j, game),
        9159987 => CinematicFunction_Intro_Func81(j, game),
        9160105 => CinematicFunction_Intro_Func83(j, game),
        9160763 => CinematicFunction_Intro_Func89(j, game),
        9160855 => CinematicFunction_Intro_Func92(j, game),
        9161026 => CinematicFunction_Intro_Func98(j, game),
        9161046 => CinematicFunction_Intro_Func99(j, game),
        9161066 => CinematicFunction_Intro_Func100(j, game),
        9161086 => CinematicFunction_Intro_Func101(j, game),
        9161106 => CinematicFunction_Intro_Func102(j, game),
        9161190 => CinematicFunction_Intro_Func104(j, game),
        9170841 => CinematicFunction_Intro_Func150(j, game),
        9170922 => CinematicFunction_Intro_Func152(j, game),
        9170941 => CinematicFunction_Intro_Func153(j, game),
        9170949 => CinematicFunction_Intro_Func154(j, game),
        9170968 => CinematicFunction_Intro_Func155(j, game),
        9170987 => CinematicFunction_Intro_Func156(j, game),
        9171006 => CinematicFunction_Intro_Func157(j, game),
        9171038 => CinematicFunction_Intro_Func159(j, game),
        9171053 => CinematicFunction_Intro_Func161(j, game),
        9171068 => CinematicFunction_Intro_Func162(j, game),
        9171077 => CinematicFunction_Intro_Func163(j, game),
        9171092 => CinematicFunction_Intro_Func164(j, game),
        9171122 => CinematicFunction_Intro_Func165(j, game),
        9171169 => CinematicFunction_Intro_Func166(j, game),
        9171177 => CinematicFunction_Intro_Func167(j, game),
        9171185 => CinematicFunction_Intro_Func168(j, game),
        9171193 => CinematicFunction_Intro_Func169(j, game),
        9171230 => CinematicFunction_Intro_Func170(j, game),
        9171267 => CinematicFunction_Intro_Func171(j, game),
        9171286 => CinematicFunction_Intro_Func172(j, game),
        9171305 => CinematicFunction_Intro_Func173(j, game),
        9171324 => CinematicFunction_Intro_Func174(j, game),
        9171343 => CinematicFunction_Intro_Func175(j, game),
        9171368 => CinematicFunction_Intro_Func176(j, game),
        9171393 => CinematicFunction_Intro_Func177(j, game),
        9171412 => CinematicFunction_Intro_Func178(j, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallIntroObjectInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    match ea {
        9148926 => IntroObject_Delete(k, j, game),
        9148934 => IntroObject_Goto(k, j, game),
        9148941 => IntroObject_DecTimerGoto(k, j, game),
        9148951 => IntroObject_SetTimer(k, j, game),
        9172734 => CinematicFunction_Intro_Func219(k, j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn CallCinematicSpriteInit(ea: u32, j: u16, game: &mut Game) {
    match ea {
        9149210 => CinematicSpriteInit_7(j, game),
        9149229 => CinematicSpriteInit_8(j, game),
        9149248 => CinematicSpriteInit_9(j, game),
        9149628 => CinematicSpriteInit_0(j, game),
        9149770 => CinematicSpriteInit_1(j, game),
        9149891 => CinematicSpriteInit_2(j, game),
        9150021 => CinematicSpriteInit_3(j, game),
        9150131 => CinematicSpriteInit_4(j, game),
        9150166 => CinematicSpriteInit_5(j, game),
        9150207 => CinematicSpriteInit_6(j, game),
        9147353 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallMode7Instr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    match ea {
        9147755 => Mode7Instr_Delete(k, j, game),
        9147762 => Mode7Instr_SetPreInstr(k, j, game),
        9147773 => Mode7Instr_ClearPreInstr(k, j, game),
        9147782 => Mode7Instr_Goto(k, j, game),
        9147789 => Mode7Instr_DecTimerAndGoto(k, j, game),
        9147799 => Mode7Instr_SetTimer(k, j, game),
        9151067 => PlayBabyMetroidCry1(k, j, game),
        9151075 => PlayBabyMetroidCry2(k, j, game),
        9151083 => PlayBabyMetroidCry3(k, j, game),
        9151091 => CinematicSetPal1(k, j, game),
        9151108 => CinematicSetPal2(k, j, game),
        9151125 => CinematicSetPal3(k, j, game),
        9151142 => CinematicSetPal4(k, j, game),
        9155870 => sub_8BB51E(k, j, game),
        9155919 => EnableCinematicBgTilemapUpdates__0(k, j, game),
        9155924 => CinematicFunction_Intro_Func21(k, j, game),
        9156024 => CinematicFunction_Intro_ThenWaitInputSetupBabyMetroid(k, j, game),
        9156035 => CinematicFunction_Intro_Func23(k, j, game),
        9156123 => CinematicFunction_Intro_Func25(k, j, game),
        9156134 => CinematicFunction_Intro_Func26(k, j, game),
        9156234 => CinematicFunction_Intro_Func28(k, j, game),
        9156245 => CinematicFunction_Intro_Func29(k, j, game),
        9156333 => CinematicFunction_Intro_Func31(k, j, game),
        9156344 => CinematicFunction_Intro_Func32(k, j, game),
        9156363 => EnableCinematicBgTilemapUpdates__(k, j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn CallMode7PreInstr(ea: u32, k: u16, game: &mut Game) {
    match ea {
        9147353 => (),
        9155973 => CinematicFunction_Intro_Func22(k, game),
        9156078 => CinematicFunction_Intro_Func24(k, game),
        9156183 => CinematicFunction_Intro_Func27(k, game),
        9156288 => CinematicFunction_Intro_Func30(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
