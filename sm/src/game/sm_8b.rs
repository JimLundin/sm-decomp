use super::intro_pages;
pub use crate::game::prelude::*;

pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_8d::*;

use crate::game::explosion_window::spawn_power_bomb;
use crate::game::projectile_animation::draw_projectiles;
use crate::game::sm_80::{
    memset7E, CheckEventHappened, ClearOamExt, ClearUnusedOam, CopySuperMetroidString,
    DisableIrqInterrupts, EnableNMI, GraphicsAsset, HandleFadeIn, HandleFadeOut, HasQueuedMusic,
    QueueMode7Transfers, QueueMusic_Delayed8, QueueMusic_DelayedY, QueueSfx1_Max15, QueueSfx2_Max6,
    QueueSfx3_Max6, WaitForNMI_Async,
};
use crate::game::sm_81::{DrawSpritemap, DrawSpritemapOffScreen, SaveToSram};
use crate::game::sm_88::{
    CinematicFunction_Intro_Func133, DisableHdmaObjects, EnableHdmaObjects,
    SpawnTitleScreenGradientObjs, WaitUntilEndOfVblankAndClearHdma,
};
use crate::game::sm_8d::{
    ClearPaletteFXObjects, DisablePaletteFx, EnablePaletteFx, PaletteFxHandler, SpawnPalfxObject,
};
use crate::game::sm_90::{
    DrawSamusAndProjectiles, HandleControllerInputForGamePhysics, HandleSamusMovementAndPause,
    KillProjectile, UpdateBeamTilesAndPalette,
};
use crate::game::sm_91::Samus_Initialize;
use crate::game::sm_94::kSinCosTable8bit_Sext;

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct Mode7ObjectDef {
        pub field_0, field_2, field_4: VoidP;
    }
}

pub use crate::types::Consts_8B;

crate::game::constants::typed_constants! { Consts_8B;
    addr_kCinematicBgObjectDef_8BF74E = 63310, addr_kCinematicBgObjectDef_8BF748 = 63304,
    addr_stru_8BF6F8 = 63224, addr_kCinematicSpriteObjectDef_8BEF93 = 61331,
    addr_kCinematicSpriteObjectDef_8BEF8D = 61325, addr_kCinematicSpriteObjectDef_8BEF87 = 61319,
    addr_kCinematicSpriteObjectDef_8BEF81 = 61313, addr_kCinematicSpriteObjectDef_8BEF7B = 61307,
    addr_kCinematicSpriteObjectDef_8BEF75 = 61301, addr_kCinematicSpriteObjectDef_8BEF6F = 61295,
    addr_kCinematicSpriteObjectDef_8BEF69 = 61289, addr_kCinematicSpriteObjectDef_8BEF63 = 61283,
    addr_kCinematicSpriteObjectDef_8BEF5D = 61277, addr_kCinematicSpriteObjectDef_8BEF57 = 61271,
    addr_kCinematicSpriteObjectDef_8BEF51 = 61265, addr_kCinematicSpriteObjectDef_8BEF4B = 61259,
    addr_kCinematicSpriteObjectDef_8BEF45 = 61253, addr_kCinematicSpriteObjectDef_8BEF3F = 61247,
    addr_kCinematicSpriteObjectDef_8BEF39 = 61241, addr_kCinematicSpriteObjectDef_8BEF33 = 61235,
    addr_kCinematicSpriteObjectDef_8BEF2D = 61229, addr_kCinematicSpriteObjectDef_8BEF27 = 61223,
    addr_kCinematicSpriteObjectDef_8BEF1B = 61211, addr_kCinematicSpriteObjectDef_8BEF15 = 61205,
    addr_kCinematicSpriteObjectDef_8BEF0F = 61199, addr_kCinematicSpriteObjectDef_8BEF09 = 61193,
    addr_kCinematicSpriteObjectDef_8BEF03 = 61187, addr_kCinematicSpriteObjectDef_8BEEFD = 61181,
    addr_kCinematicSpriteObjectDef_8BEEF7 = 61175, addr_kCinematicSpriteObjectDef_8BEEF1 = 61169,
    addr_kCinematicSpriteObjectDef_8BEEEB = 61163, addr_kCinematicSpriteObjectDef_8BEEE5 = 61157,
    addr_kCinematicSpriteObjectDef_8BEEDF = 61151, addr_kCinematicSpriteObjectDef_8BEED9 = 61145,
    addr_kCinematicSpriteObjectDef_8BEED3 = 61139, addr_kCinematicSpriteObjectDef_8BEECD = 61133,
    addr_kCinematicSpriteObjectDef_8BEEC7 = 61127, addr_kCinematicSpriteObjectDef_8BEEC1 = 61121,
    addr_kCinematicSpriteObjectDef_8BEEBB = 61115, addr_kCinematicSpriteObjectDef_8BEEB5 = 61109,
    addr_kCinematicSpriteObjectDef_8BEEAF = 61103, addr_kCinematicSpriteObjectDef_8BEEA9 = 61097,
    addr_kCinematicSpriteObjectDef_8BEEA3 = 61091, addr_kCinematicSpriteObjectDef_8BEE9D = 61085,
    addr_off_8BEE9B = 61083, addr_word_8BED9D = 60829,
    addr_word_8BED95 = 60821, addr_loc_8BE1B4 = 57780,
    addr_kMode7ObjectDef_8BD437 = 54327, addr_kMode7ObjectDef_8BD431 = 54321,
    addr_kMode7ObjectDef_8BD42B = 54315, addr_kMode7ObjectDef_8BD425 = 54309,
    addr_kMode7ObjectDef_8BD41F = 54303, addr_kMode7ObjectDef_8BD419 = 54297,
    addr_kMode7ObjectDef_8BD413 = 54291, addr_kMode7ObjectDef_8BD40D = 54285,
    addr_kMode7ObjectDef_8BD407 = 54279, addr_kMode7ObjectDef_8BD401 = 54273,
    addr_off_8BD3E7 = 54247, addr_word_8BD389 = 54153,
    addr_word_8BD371 = 54129, addr_word_8BD30D = 54029,
    addr_word_8BD2D5 = 53973, addr_word_8BD2A5 = 53925,
    addr_word_8BD259 = 53849, addr_word_8BD215 = 53781,
    addr_word_8BD1B9 = 53689, addr_word_8BD15D = 53597,
    addr_word_8BD0F9 = 53497, addr_word_8BD0E1 = 53473,
    addr_word_8BD085 = 53381, addr_word_8BD055 = 53333,
    addr_word_8BCFFD = 53245, addr_word_8BCFBD = 53181,
    addr_word_8BCF75 = 53109, addr_kCinematicBgObjectDef_8BCF69 = 53097,
    addr_kCinematicBgObjectDef_8BCF63 = 53091, addr_kCinematicBgObjectDef_8BCF5D = 53085,
    addr_kCinematicBgObjectDef_8BCF57 = 53079, addr_kCinematicBgObjectDef_8BCF51 = 53073,
    addr_kCinematicBgObjectDef_8BCF4B = 53067, addr_kCinematicBgObjectDef_8BCF45 = 53061,
    addr_kCinematicBgObjectDef_8BCF3F = 53055, addr_kCinematicSpriteObjectDef_8BCF39 = 53049,
    addr_kCinematicSpriteObjectDef_8BCF33 = 53043, addr_kCinematicSpriteObjectDef_8BCF2D = 53037,
    addr_kCinematicSpriteObjectDef_8BCF27 = 53031, addr_kCinematicSpriteObjectDef_8BCF21 = 53025,
    addr_kCinematicSpriteObjectDef_8BCF1B = 53019, addr_kCinematicSpriteObjectDef_8BCF15 = 53013,
    addr_kCinematicSpriteObjectDef_8BCF0F = 53007, addr_kCinematicSpriteObjectDef_8BCF09 = 53001,
    addr_kCinematicSpriteObjectDef_8BCF03 = 52995, addr_kCinematicSpriteObjectDef_8BCEFD = 52989,
    addr_kCinematicSpriteObjectDef_8BCEF7 = 52983, addr_kCinematicSpriteObjectDef_8BCEF1 = 52977,
    addr_kCinematicSpriteObjectDef_8BCEEB = 52971, addr_kCinematicSpriteObjectDef_8BCEE5 = 52965,
    addr_kCinematicSpriteObjectDef_8BCEDF = 52959, addr_kCinematicSpriteObjectDef_8BCED9 = 52953,
    addr_kCinematicSpriteObjectDef_8BCED3 = 52947, addr_kCinematicSpriteObjectDef_8BCECD = 52941,
    addr_kCinematicSpriteObjectDef_8BCEC7 = 52935, addr_kCinematicSpriteObjectDef_8BCEC1 = 52929,
    addr_kCinematicSpriteObjectDef_8BCEBB = 52923, addr_kCinematicSpriteObjectDef_8BCEB5 = 52917,
    addr_kCinematicSpriteObjectDef_8BCEAF = 52911, addr_kCinematicSpriteObjectDef_8BCEA3 = 52899,
    addr_kCinematicSpriteObjectDef_8BCE91 = 52881, addr_kCinematicSpriteObjectDef_8BCE8B = 52875,
    addr_kCinematicSpriteObjectDef_8BCE85 = 52869, addr_kCinematicSpriteObjectDef_8BCE7F = 52863,
    addr_kCinematicSpriteObjectDef_8BCE79 = 52857, addr_kCinematicSpriteObjectDef_8BCE73 = 52851,
    addr_kCinematicSpriteObjectDef_8BCE6D = 52845, addr_kCinematicSpriteObjectDef_8BCE67 = 52839,
    addr_kCinematicSpriteObjectDef_8BCE61 = 52833, addr_kCinematicSpriteObjectDef_8BCE5B = 52827,
    addr_kCinematicSpriteObjectDef_8BCE55 = 52821, addr_off_8BCE53 = 52819,
    addr_word_8BCD71 = 52593, addr_word_8BCC23 = 52259,
    addr_unk_8BCC03 = 52227, addr_word_8BCBFB = 52219,
    addr_word_8BCB3B = 52027, addr_off_8BCB19 = 51993,
    addr_kCinematicFunction_Intro_Func76_M7_2 = 50170, addr_kCinematicFunction_Intro_Func76_M7_1 = 50160,
    addr_kCinematicFunction_Intro_Func76_M7_0 = 50150, addr_kCinematicFunction_Intro_Func56_M7 = 48756,
    addr_kMode7ObjBabyMetroidInTitle = 41813, addr_kCinematicSpriteObjectDef_8BA125 = 41253,
    addr_kCinematicSpriteObjectDef_8BA119 = 41241, addr_kCinematicSpriteObjectDef_8BA113 = 41235,
    addr_kCinematicSpriteObjectDef_8BA107 = 41223, addr_kCinematicSpriteObjectDef_8BA101 = 41217,
    addr_kCinematicSpriteObjectDef_8BA0FB = 41211, addr_kCinematicSpriteObjectDef_8BA0F5 = 41205,
    addr_kCinematicSpriteObjectDef_8BA0EF = 41199,
}
pub use crate::types::Consts_8C;
crate::game::constants::typed_constants! { Consts_8C;
    addr_kSpriteMap_NintendoLogo = 32955,
}

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnCinematicFunctionNone = 0x8b9a47,
    fnCinematicFunctionOpening = 0x8b9b68,
    fnCinematicFunction_Intro_Page2 = 0x8bb35f,
    fnSamus_FrameHandlerAlfa_Func12 = 0x90e6c9, fnSamus_Func15 = 0x90e833,
    fnSamusDrawHandler_Default = 0x90eb52,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x2000;
        b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_8B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8b>;
const RomPtr_8C: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8c>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_Mode7ObjectDef(a: u16, game: &mut Game) -> *mut Mode7ObjectDef {
    RomPtr((0x8b0000u32) | ((a) as u32), game) as *mut Mode7ObjectDef
}
crate::game::constants::typed_constants! { u32;
    fnCinematicFunction_Intro_Initial = 9151381,
    fnCinematicFunctionBlackoutFromCeres = 9158939,
    fnCinematicFunctionEscapeFromCebes = 9163904,
}
pub unsafe fn SetupPpuForTitleSequence(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        mirror(BGMODE as u16, 7, 0x55, false); mirror(M7SEL as u16, 0x80, 0x5f, false);
    }
    *ram_ptr!(game; 0x58) = 0;
    crate::rtl::register_writes! { game;
        mirror(BG1SC as u16, 0, 0x59, false); mirror(BG2SC as u16, 0, 0x5a, false);
        mirror(BG3SC as u16, 0, 0x5c, false); mirror(BG4SC as u16, 0, 0x5d, false);
        mirror(BG12NBA as u16, 0, 0x5e, false); byte(BG34NBA as u16, 0);
        mirror(TM as u16, 0x10, 0x69, false); mirror(TS as u16, 0, 0x6b, false);
        mirror(TMW as u16, 0, 0x6c, false); byte(VMAIN as u16, 0);
        mirror(CGWSEL as u16, 0, 0x6e, false); mirror(CGADSUB as u16, 0, 0x71, false);
        byte(COLDATA as u16, 0xe0); mirror(SETINI as u16, 0, 0x77, false);
    }
    memset(ram_ptr!(game, c_void; 0x3000), 0, 0x7fe as size_t);
    word! { game, 0;
        oam_next_ptr, menu_option_index, screen_fade_delay, screen_fade_counter,
        pausemenu_start_lr_pressed_highlight_timer, pausemenu_lr_animation_timer,
        pausemenu_item_selector_animation_timer, pausemenu_reserve_tank_animation_timer,
        UNUSED_word_7E0731, UNUSED_word_7E073D, pausemenu_lr_animation_frame,
        pausemenu_item_selector_animation_frame, pausemenu_reserve_tank_animation_frame,
        UNUSED_word_7E0745, pausemenu_shoulder_button_highlight, pausemenu_button_label_mode,
        pausemenu_equipment_category_item, pausemenu_reserve_tank_delay_ctr, UNUSED_word_7E0759,
        samus_reserve_health, samus_reserve_missiles, cinematic_var5,
    }
    word!(game.cinematic_var6 = 256);
    word! { game, 0;
        cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
        g_word_7E199B, g_word_7E199D, g_word_7E199F,
    }
}
pub unsafe fn SetupPpu_Intro(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        mirror(BGMODE as u16, 9, 0x55, false); mirror(M7SEL as u16, 0, 0x5f, false);
        mirror(BG1SC as u16, 0x50, 0x58, true); mirror(BG2SC as u16, 0x48, 0x59, true);
        mirror(BG3SC as u16, 0x4c, 0x5a, true);
    }
    *ram_ptr!(game; 0x5c) = 0;
    crate::rtl::register_writes! { game;
        mirror(BG4SC as u16, 0, 0x5d, false); byte(BG12NBA as u16, 0);
        mirror(BG34NBA as u16, 4, 0x5e, true); mirror(TM as u16, 4, 0x69, false);
        mirror(TS as u16, 0, 0x6b, false); mirror(TMW as u16, 0, 0x6c, false);
        byte(VMAIN as u16, 0); mirror(CGWSEL as u16, 0, 0x6e, false);
        mirror(CGADSUB as u16, 0, 0x71, false); byte(COLDATA as u16, 0xe0);
        mirror(SETINI as u16, 0, 0x77, false);
    }
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    *ram_ptr!(game; 0x57) = 0;
    memset7E(ram_ptr!(game, u16; 0x3000), 0x2f, 0x7fe);
    word! { game, 0; oam_next_ptr, cinematic_var5 }
    word!(game.cinematic_var6 = 256);
    word! { game, 0;
        cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
        g_word_7E199B, g_word_7E199D, g_word_7E199F, fx_layer_blending_config_a, reg_BG2HOFS,
        reg_BG2VOFS, reg_BG3HOFS, reg_BG3VOFS,
    }
}
pub unsafe fn SetupPpu_3_Mode7(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        mirror(BGMODE as u16, 7, 0x55, false); mirror(M7SEL as u16, 0x80, 0x5f, false);
    }
    *ram_ptr!(game; 0x58) = 0;
    crate::rtl::register_writes! { game;
        mirror(BG1SC as u16, 0, 0x59, false); mirror(BG2SC as u16, 0, 0x5a, false);
        mirror(BG3SC as u16, 0, 0x5c, false); mirror(BG4SC as u16, 0, 0x5d, false);
        mirror(BG12NBA as u16, 0, 0x5e, false); byte(BG34NBA as u16, 0);
        mirror(TM as u16, 0x11, 0x69, false); mirror(TS as u16, 0, 0x6b, false);
        mirror(TMW as u16, 0, 0x6c, false); byte(VMAIN as u16, 0);
        mirror(CGWSEL as u16, 0, 0x6e, false); mirror(CGADSUB as u16, 0, 0x71, false);
        byte(COLDATA as u16, 0xe0);
    }
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    *ram_ptr!(game; 0x57) = 0;
    word! { game, 0;
        cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
        g_word_7E199B, g_word_7E199D, g_word_7E199F, fx_layer_blending_config_a,
    }
}
pub unsafe fn SetupPpu_4_Mode1(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
        mirror(BGMODE as u16, 1, 0x55, false); mirror(M7SEL as u16, 0, 0x5f, false);
        mirror(BG1SC as u16, 0x5c, 0x58, true);
    }
    *ram_ptr!(game; 0x59) = 0;
    crate::rtl::register_writes! { game;
        mirror(BG2SC as u16, 0, 0x5a, false); mirror(BG3SC as u16, 0, 0x5c, false);
        byte(BG4SC as u16, 0); mirror(BG12NBA as u16, 6, 0x5d, true);
        mirror(BG34NBA as u16, 0, 0x5e, true); mirror(TM as u16, 0x11, 0x69, false);
        mirror(TS as u16, 0, 0x6b, false);
    }
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 0;
    *ram_ptr!(game; 0x62) = 0;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 0;
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
}
pub unsafe fn SetupPpu_5_Mode7(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); byte(OBSEL as u16, 0xa3);
    }
    *ram_ptr!(game; 0x52) = -93i32 as u8;
    crate::rtl::register_writes! { game;
        mirror(BGMODE as u16, 7, 0x55, false); mirror(M7SEL as u16, 0, 0x5f, false);
    }
    *ram_ptr!(game; 0x58) = 0;
    crate::rtl::register_writes! { game;
        mirror(BG1SC as u16, 0, 0x59, false); mirror(BG2SC as u16, 0, 0x5a, false);
        mirror(BG3SC as u16, 0, 0x5c, false); mirror(BG4SC as u16, 0, 0x5d, false);
        mirror(BG12NBA as u16, 0, 0x5e, false); byte(BG34NBA as u16, 0);
        mirror(TM as u16, 0x11, 0x69, false); mirror(TS as u16, 0, 0x6b, false);
        mirror(TMW as u16, 0, 0x6c, false); byte(VMAIN as u16, 0);
        mirror(CGWSEL as u16, 0, 0x6e, false); mirror(CGADSUB as u16, 0, 0x71, false);
    }
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    crate::rtl::register_writes! { game;
        byte(COLDATA as u16, 0xe0); mirror(SETINI as u16, 0, 0x77, false);
    }
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    word! { game, 0;
        oam_next_ptr, screen_fade_delay, screen_fade_counter, cinematic_var5,
    }
    word!(game.cinematic_var6 = 256);
    word! { game, 0;
        cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
        g_word_7E199B, g_word_7E199D, g_word_7E199F,
    }
}

macro_rules! reset_centered_mode7_transform {
    ($game:ident) => {
        WriteRegWord(M7A as u16, 0x100, $game);
        word!($game.reg_M7A = 256);
        WriteRegWord(M7B as u16, 0, $game);
        word!($game.reg_M7B = 0);
        WriteRegWord(M7C as u16, 0, $game);
        word!($game.reg_M7C = 0);
        WriteRegWord(M7D as u16, 0x100, $game);
        word!($game.reg_M7D = 256);
        WriteRegWord(M7X as u16, 0x80, $game);
        word!($game.reg_M7X = 128);
        WriteRegWord(M7Y as u16, 0x80, $game);
        word!($game.reg_M7Y = 128);
    };
}

pub unsafe fn SetupPpu_7_Mode1(game: &mut Game) {
    crate::rtl::register_writes! { game;
        mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, 0, 0x52, false);
        mirror(BGMODE as u16, 1, 0x55, false); mirror(M7SEL as u16, 0, 0x5f, false);
        mirror(BG1SC as u16, 0x48, 0x58, true); mirror(BG2SC as u16, 0x4c, 0x59, true);
        mirror(BG3SC as u16, 0x24, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        mirror(BG12NBA as u16, 0x54, 0x5d, true); mirror(BG34NBA as u16, 2, 0x5e, true);
        mirror(TM as u16, 1, 0x69, false); mirror(TS as u16, 0, 0x6b, false);
    }
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 0;
    *ram_ptr!(game; 0x62) = 0;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 0;
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    word!(game.oam_next_ptr = 0);
    reset_centered_mode7_transform!(game);
    word! {game; cinematic_var5 = 0; cinematic_var6 = 1; }
    word! { game, 0;
        reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS, cinematic_var7, cinematic_var8,
        cinematic_var9, cinematic_var10,
    }
}
pub unsafe fn HandleMode7TransformationNoRotation(game: &mut Game) {
    word! {game; reg_M7A = word!(game.cinematic_var6); reg_M7D = word!(game.cinematic_var6); }
    word! { game, 0; reg_M7B, reg_M7C }
    word!(game.reg_BG1HOFS = word!(game.cinematic_var8));
    word!(game.reg_BG1VOFS = word!(game.cinematic_var10));
}
unsafe fn Smult16x16(r38: u16, r40: u16) -> u32 {
    let t: u32 = ((r38 as i16) as u32).wrapping_mul((r40 as i16) as u32);
    t
}
pub unsafe fn HandleMode7TransformationMatrix(game: &mut Game) {
    crate::update_place! { unaligned_word fresh152 = &raw mut (*(ram_ptr!(game, u16; 0x78))) => (Smult16x16(
        kSinCosTable8bit_Sext[((((word!(game.cinematic_var5)) as u8).wrapping_add(64))
            as i32
            + 64) as usize] as u16,
        word!(game.cinematic_var6),
    ) >> 8) as u16; }
    word!(game.reg_M7D = u16::from_le(fresh152.read_unaligned()));
    let v1: u16 = (Smult16x16(
        kSinCosTable8bit_Sext[(word!(game.cinematic_var5) as u8 as i32 + 64) as usize] as u16,
        word!(game.cinematic_var6),
    ) >> 8) as u16;
    word! {game; reg_M7B = v1; reg_M7C = -(v1 as i32) as u16;
    reg_BG1HOFS = word!(game.cinematic_var8); reg_BG1VOFS = word!(game.cinematic_var10); }
}
pub unsafe fn SetPpuBackdropSomeColor(game: &mut Game) {
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = -127i32 as u8;
    *ram_ptr!(game; 0x74, 0) = 56;
    *ram_ptr!(game; 0x74, 1) = 88;
    *ram_ptr!(game; 0x74, 2) = 0x80;
}
pub unsafe fn SetPpuBackdropBlack(game: &mut Game) {
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
}
pub unsafe fn UpdateCinematicBgTilemap(game: &mut Game) {
    if *ram_ptr!(game; 0x55) != 7 {
        let mut v0: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 1920;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, game)).size = 12288;
        v0 = (v0).wrapping_add(2);
        crate::update_place! { previous fresh157 = v0 => v0.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh157 as i32,
            game,
        ))
        .size as *mut u8) = 126;
        (*gVramWriteEntry(v0 as i32, game)).size = word!(game.cinematicbg_var3);
        word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
    }
}
pub unsafe fn UpdateSamusEyesTilemap(game: &mut Game) {
    if *ram_ptr!(game; 0x55) != 7 {
        crate::game::sm_82::queue_ram_tilemap_write!(game, 128, 832, fresh156, 18848);
    }
}
pub unsafe fn CinematicUpdateSomeBg(game: &mut Game) {
    if *ram_ptr!(game; 0x55) != 7 {
        let mut v0: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 2048;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, game)).size = (ram_ptr!(game, Ram3000; 0x3000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8)
            as c_long as u16;
        v0 = (v0).wrapping_add(2);
        crate::update_place! { previous fresh4 = v0 => v0.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh4 as i32,
            game,
        ))
        .size as *mut u8) = 126;
        (*gVramWriteEntry(v0 as i32, game)).size = word!(game.cinematicbg_var3);
        word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
    }
}

pub unsafe fn ProcessCinematicBgObject(v0: u16, r18: u16, game: &mut Game) {
    let j: u16 = word!(game.cinematicbg_arr1[(v0 >> 1)]);
    CallProcessCinematicBgObject(
        ((*(RomPtr_8C(j, game) as *mut u16)) as u32) | (0x8b0000u32),
        v0,
        j,
        r18,
        game,
    );
}
macro_rules! copy_cinematic_tilemap_row {
    (
        $game:ident, $overlay:ident, $ram_offset:literal, $tilemap_field:ident,
        $destination:ident, $source:ident, $count:ident $(,)?
    ) => {
        loop {
            *((&raw mut (*(ram_ptr!($game, $overlay; $ram_offset))).$tilemap_field
                as *mut u16 as *mut u8)
                .offset($destination as i32 as isize) as *mut u16) =
                *(RomPtr_8C($source, $game) as *mut u16).offset(2);
            $source = ($source).wrapping_add(2);
            $destination = ($destination).wrapping_add(2);
            $count = $count.wrapping_sub(1);
            if ($count == 0) {
                break;
            }
        }
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallProcessCinematicBgObject(ea: u32, k: u16, j: u16, r18: u16, game: &mut Game) => ea;
    call (k, j, r18, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, j: u16, r18: u16, game: &mut Game)];
    [9144397] ProcessCinematicBgObject_DrawChar [] {
        word!(game.cinematicbg_var1 = (word!(game.cinematicbg_var1) == 0) as u16);
        SpawnTextGlowObject(j, r18, game);
        let v2 = RomPtr_8C(word!(game.cinematicbg_instr_ptr[(k >> 1)]), game);
        let mut v3: i16 = if *(v2 as *mut i16).offset(3) < 0 {
            word!(game.cinematicbg_arr7[15] = 8);
            (8 * (*v2.offset(3) as i32 + 2)) as i16
        } else {
            word!(game.cinematicbg_arr7[15] = (8u16).wrapping_mul((*v2.offset(8)) as u16));
            (8i16).wrapping_mul((*v2.offset(9)) as i16)
        };
        word!(game.cinematicbg_arr8[15] = ((v3) as u16).wrapping_sub(8));
        if *(v2.offset(4) as *mut u16) != 0xd67d && word!(game.cinematicbg_var1) != 0 {
            QueueSfx3_Max6(0xd, game);
        }
        ProcessCinematicBgObject_DrawToTextTilemap(0x1e, j, r18, game);
    };
    [9144683] ProcessCinematicBgObject_Unk1 [] {
        let mut r22: u16 = Mult0x80Add(r18);
        let v2 = RomPtr_8C(j, game);
        let r18x: u16 = *v2.offset(2) as u16;
        let mut n: u16 = *v2.offset(3) as u16;
        let mut v3: u16 = (j).wrapping_add(4);
        let mut v4: u16 = word!(game.mode7_vram_write_queue_tail);
        do_until! { (n == 0);
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_0)
        .offset(v4 as i32 as isize) as *mut u16) = 128;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_1 as *mut u8)
        .offset(v4 as i32 as isize) as *mut u16) = v3;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset(v4 as isize) as *mut u16) = 139;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset((v4 as i32 + 1) as isize) as *mut u16) = r18x;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset((v4 as i32 + 3) as isize) as *mut u16) = r22;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0)).offset(1)).field_1
        as *mut u8)
        .offset(v4 as i32 as isize) as *mut u16) = 0;
    v4 = (v4).wrapping_add(9);
    v3 = (v3).wrapping_add(r18x);
    r22 = (r22).wrapping_add(128);
    n = n.wrapping_sub(1);
}
        word!(game.mode7_vram_write_queue_tail = v4);
    };
    [9144783] ProcessCinematicBgObject_Unk2 [] {
        let mut r22: u16 = Mult0x80Add(r18);
        let v2 = RomPtr_8C(j, game);
        let mut n: i32 = *v2.offset(2) as i32;
        let r20: u16 = *v2.offset(3) as u16;
        let mut v3: u16 = (j).wrapping_add(4);
        let mut v4: u16 = word!(game.mode7_vram_write_queue_tail);
        do_until! { (n == 0);
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_0)
        .offset(v4 as i32 as isize) as *mut u16) = 128;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_1 as *mut u8)
        .offset(v4 as i32 as isize) as *mut u16) = v3;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset(v4 as isize) as *mut u16) = 139;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset((v4 as i32 + 1) as isize) as *mut u16) = r20;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
        .offset((v4 as i32 + 3) as isize) as *mut u16) = r22;
    *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0)).offset(1)).field_1
        as *mut u8)
        .offset(v4 as i32 as isize) as *mut u16) = 2;
    v4 = (v4).wrapping_add(9);
    v3 = (v3).wrapping_add(r20);
    r22 = r22.wrapping_add(1);
    n -= 1;
}
        word!(game.mode7_vram_write_queue_tail = v4);
    };
    @functions [pub unsafe fn (
    k: u16,
    mut j: u16,
    r18: u16,
    game: &mut Game,
)];
    [9144503] ProcessCinematicBgObject_DrawToTextTilemap [] {
        let jorg: u16 = j;
        let mut offs: u16 = CinematicGetTilemapOffsetForTile(r18);
        let mut r22: u16 = offs;
        let v3 = RomPtr_8C(j, game);
        let mut m: u16 = *v3.offset(2) as u16;
        let morg: u16 = m;
        let mut n: u16 = *v3.offset(3) as u16;
        loop {
            copy_cinematic_tilemap_row!(game, Ram3000, 0x3000, pause_menu_map_tilemap, offs, j, m);
            m = morg;
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
            r22 = (r22).wrapping_add(64);
            offs = r22;
        }
    };
    [9144573] ProcessCinematicBgObject_DrawToBgTilemap [] {
        let jorg: u16 = j;
        let mut offs: u16 = CinematicGetTilemapOffsetForTile(r18);
        let mut r22: u16 = offs;
        let v3 = RomPtr_8C(j, game);
        let mut m: u16 = *v3.offset(2) as u16;
        let morg: u16 = m;
        let mut n: i32 = *v3.offset(3) as i32;
        loop {
            copy_cinematic_tilemap_row!(game, Ram3800, 0x3800, cinematic_bg_tilemap, offs, j, m);
            m = morg;
            n -= 1;
            if n == 0 {
                break;
            }
            r22 = (r22).wrapping_add(64);
            offs = r22;
        }
    };
    @external {
        9144393 => [ProcessCinematicBgObject_Nothing(k, j, r18)];
    }
}
pub fn ProcessCinematicBgObject_Nothing(k: u16, j: u16, r18: u16) {}
pub unsafe fn CinematicGetTilemapOffsetForTile(mut r18: u16) -> u16 {
    (2 * r18 as u8 as i32 + byte!(r18; 1) as i32 * 0x40) as u16
}
pub unsafe fn Mult0x80Add(mut r18: u16) -> u16 {
    (r18 as u8 as i32 + 0x80 * byte!(r18; 1) as i32) as u16
}
pub unsafe fn Samus_CalcPos_Mode7(game: &mut Game) {
    let r34: u16 = (word!(game.samus_x_pos)).wrapping_sub(word!(game.reg_M7X));
    let r36: u16 = (word!(game.reg_M7Y)).wrapping_sub(word!(game.samus_y_pos));
    let mut r26: u16 = (Smult16x16(
        (word!(game.samus_x_pos)).wrapping_sub(word!(game.reg_M7X)),
        word!(game.reg_M7A),
    ) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7B), r36) >> 8) as u16;
    word!(game.samus_x_pos = (r26).wrapping_add(word!(game.reg_M7X)));
    r26 = (Smult16x16(word!(game.reg_M7C), r34) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7A), r36) >> 8) as u16;
    word!(game.samus_y_pos = (word!(game.reg_M7Y)).wrapping_sub(r26));
}
pub unsafe fn CalcExplosion_Mode7(k: u16, game: &mut Game) -> Point16U {
    let v1: i32 = k as i32 >> 1;
    let r34: u16 = (word!(game.projectile_x_pos[v1])).wrapping_sub(word!(game.reg_M7X));
    let r36: u16 = (word!(game.reg_M7Y)).wrapping_sub(word!(game.projectile_y_pos[v1]));
    let mut r26: u16 = (Smult16x16(r34, word!(game.reg_M7A)) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7B), r36) >> 8) as u16;
    let r20: u16 =
        (r26 as i32 + word!(game.reg_M7X) as i32 - word!(game.layer1_x_pos) as i32) as u16;
    r26 = (Smult16x16(word!(game.reg_M7C), r34) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7A), r36) >> 8) as u16;
    let r18: u16 =
        (word!(game.reg_M7Y) as i32 - r26 as i32 - word!(game.layer1_y_pos) as i32) as u16;
    Point16U { x: r20, y: r18 }
}
pub unsafe fn CalcCeresSteamPos_Mode7(mut pt: Point16U, game: &mut Game) -> Point16U {
    let r34: u16 = (pt.x).wrapping_sub(word!(game.reg_M7X));
    let r36: u16 = (word!(game.reg_M7Y)).wrapping_sub(pt.y);
    let mut r26: u16 = (Smult16x16(
        (pt.x).wrapping_sub(word!(game.reg_M7X)),
        word!(game.reg_M7A),
    ) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7B), r36) >> 8) as u16;
    pt.x = (r26).wrapping_add(word!(game.reg_M7X));
    r26 = (Smult16x16(word!(game.reg_M7C), r34) >> 8) as u16;
    r26 = (r26 as u32).wrapping_add(Smult16x16(word!(game.reg_M7A), r36) >> 8) as u16;
    pt.y = (word!(game.reg_M7Y)).wrapping_sub(r26);
    pt
}
pub unsafe fn CopyPalettesToFadingPalettes(game: &mut Game) {
    word_transfers!(game, ascending, 256; native(0x2200) <- le(palette_buffer, 0));
}
pub unsafe fn DecomposePaletteDataForFading(game: &mut Game) {
    let mut v0: u16 = 0;
    for _ in 0..256 {
        let v2: i32 = v0 as i32 >> 1;
        let r18: u16 = word!(game.tilemap_stuff[(v2 + 256)]);
        let v3: i32 = r18 as i32 & 0x1f;
        word!(game.tilemap_stuff[(v2 + 512)] = (v3 << 8) as u16);
        word!(game.tilemap_stuff[(v2 + 1280)] = (v3 << 3) as u16);
        let v4: i32 = (r18 as i32 & 0x3e0) >> 5;
        word!(game.tilemap_stuff[(v2 + 768)] = (v4 << 8) as u16);
        word!(game.tilemap_stuff[(v2 + 1536)] = (v4 << 3) as u16);
        let v6: i32 = (r18 as i32 & 0x7c00) >> 10;
        word!(game.tilemap_stuff[(v2 + 1024)] = (v6 << 8) as u16);
        word!(game.tilemap_stuff[(v2 + 1792)] = (v6 << 3) as u16);
        v0 = (v0).wrapping_add(2);
    }
}
pub unsafe fn ClearYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    do_until! { (j == 0);
        let v2: i32 = k as i32 >> 1;
        word! { game, 0;
            tilemap_stuff[(v2 + 256)], tilemap_stuff[(v2 + 512)],
            tilemap_stuff[(v2 + 768)], tilemap_stuff[(v2 + 1024)],
        }
        k = (k).wrapping_add(2);
        j = j.wrapping_sub(1);
    }
}
pub unsafe fn FadeOutYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    do_until! { (j == 0);
        let v2: i32 = k as i32 >> 1;
        word!(game.tilemap_stuff[(v2 + 512)] -= word!(game.tilemap_stuff[(v2 + 1280)]));
        word!(game.tilemap_stuff[(v2 + 768)] -= word!(game.tilemap_stuff[(v2 + 1536)]));
        word!(game.tilemap_stuff[(v2 + 1024)] -= word!(game.tilemap_stuff[(v2 + 1792)]));
        k = (k).wrapping_add(2);
        j = j.wrapping_sub(1);
    }
}
pub unsafe fn FadeInYColorsFromIndexX(mut k: u16, mut j: u16, game: &mut Game) {
    do_until! { (j == 0);
        let v2: i32 = k as i32 >> 1;
        word! { game;
            tilemap_stuff[(v2 + 512)] = (word!(game.tilemap_stuff[(v2 + 1280)]) as i32
                + word!(game.tilemap_stuff[(v2 + 512)]) as i32
                & 0x1fff) as u16;
            tilemap_stuff[(v2 + 768)] = (word!(game.tilemap_stuff[(v2 + 1536)]) as i32
                + word!(game.tilemap_stuff[(v2 + 768)]) as i32
                & 0x1fff) as u16;
            tilemap_stuff[(v2 + 1024)] = (word!(game.tilemap_stuff[(v2 + 1792)]) as i32
                + word!(game.tilemap_stuff[(v2 + 1024)]) as i32
                & 0x1fff) as u16;
        }
        k = (k).wrapping_add(2);
        j = j.wrapping_sub(1);
    }
}
pub unsafe fn ComposeFadingPalettes(game: &mut Game) {
    let mut v0: u16 = 0;
    for _ in 0..256 {
        let v2: i32 = v0 as i32 >> 1;
        let mut r18: i32 = byte!(@pointer (ram_ptr!(game, u16; 0x2000)).offset((v2 + 512i32) as isize); 1)
            as i32
            & 0x1f;
        r18 |= word!(game.tilemap_stuff[(v2 + 768)]) as i32 >> 3 & 0x3e0;
        let v3: u16 = (r18 | (word!(game.tilemap_stuff[(v2 + 1024)]) as i32) << 2 & 0x7c00) as u16;
        word! {game; tilemap_stuff[v2] = v3; palette_buffer[v2] = v3; }
        v0 = (v0).wrapping_add(2);
    }
}
pub unsafe fn CinematicFunction_Intro_Func20(j: u16, game: &mut Game) {
    let mut tt = RomPtr_8B(j, game) as *mut u16;
    let mut v2: u16 = *tt;
    let mut r18: i32 = *tt.offset(1) as i32;
    let dst = &raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).intro_japanese_text_tiles as *mut u16;
    do_until! { (r18 == 0);
        memcpy(
            dst.offset((v2 >> 1) as isize) as *mut c_void,
            ram_ptr!(game, u16; (*tt.offset(2) as i32 + 0x1a000) as isize) as *const c_void,
            16 as size_t,
        );
        memcpy(
            dst.offset((v2 as i32 + 768 >> 1) as isize) as *mut c_void,
            ram_ptr!(game, u16; (*tt.offset(3) as i32 + 0x1a000) as isize) as *const c_void,
            16 as size_t,
        );
        v2 = (v2).wrapping_add(16);
        tt = tt.offset(2);
        r18 -= 1;
    }
}
pub unsafe fn TransferJapaneseTextTilesToVram(game: &mut Game) {
    let mut v0: u16 = word!(game.vram_write_queue_tail);
    (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 1536;
    v0 = (v0).wrapping_add(2);
    (*gVramWriteEntry(v0 as i32, game)).size = 0x4000;
    v0 = (v0).wrapping_add(2);
    crate::update_place! { previous fresh21 = v0 => v0.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh21 as i32,
        game,
    ))
    .size as *mut u8) = 126;
    (*gVramWriteEntry(v0 as i32, game)).size = 16768;
    word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
}
pub unsafe fn HandleSamusDuringIntro(game: &mut Game) {
    if word!(game.cinematic_var15) != 0 {
        HandleControllerInputForGamePhysics(game);
        HandleSamusMovementAndPause(game);
        if word!(game.samus_invincibility_timer) != 0 {
            word!(game.samus_invincibility_timer -= 1);
        }
        if word!(game.samus_knockback_timer) != 0 {
            word!(game.samus_knockback_timer -= 1);
        }
    }
}
pub unsafe fn DrawIntroSprites(game: &mut Game) {
    if word!(game.cinematic_var15) != 0 {
        if (word!(game.cinematic_var15) & 0x8000) != 0 {
            DrawSamusAndProjectiles(game);
            draw_projectiles(game);
            DrawCinematicSpriteObjects_Intro(game);
        } else {
            DrawCinematicSpriteObjects_Intro(game);
            DrawSamusAndProjectiles(game);
            draw_projectiles(game);
        }
    } else {
        DrawCinematicSpriteObjects_Intro(game);
    };
}
static kMoveUnusedSpritesMasks: [u16; 8] = [
    0x5555, 0x5554, 0x5550, 0x5540, 0x5500, 0x5400, 0x5000, 0x4000,
];
pub unsafe fn MoveUnusedSpritesOffScreen(game: &mut Game) {
    if word!(game.oam_next_ptr) >= 0x200 {
        return;
    }
    let j: i32 = word!(game.oam_next_ptr) as i32 >> 2;
    let mut i: i32 = word!(game.oam_next_ptr) as i32 >> 5;
    crate::update_place! { previous fresh11 = i => i + 1; }
    word!(game.oam_ext[fresh11] |= (kMoveUnusedSpritesMasks[(j & 7) as usize]));
    while i < 16 {
        word!(game.oam_ext[i] = 0x5555);
        i += 1;
    }
    let mut i_0: i32 = word!(game.oam_next_ptr) as i32 >> 2;
    while i_0 < 0x80 {
        *(&raw mut (*(ram_ptr!(game, OamEnt; 0x370)).offset(i_0 as isize)).xcoord as *mut u16) =
            0x80;
        i_0 += 1;
    }
}
pub unsafe fn AdvanceFastScreenFadeOut(game: &mut Game) -> u8 {
    if (*ram_ptr!(game; 0x51) & 0xf) != 0 {
        if (((*ram_ptr!(game; 0x51) as i32 & 0xf) - word!(game.screen_fade_counter) as i32) as i8)
            > 0
        {
            *ram_ptr!(game; 0x51) = ((*ram_ptr!(game; 0x51) as i32 & 0xf)
                - word!(game.screen_fade_counter) as i32) as u8;
            return 0;
        }
        *ram_ptr!(game; 0x51) = 0x80;
    }
    1
}
pub unsafe fn AdvanceSlowScreenFadeOut(game: &mut Game) -> u8 {
    let v0: bool = ((word!(game.screen_fade_delay)) as i8).wrapping_sub(1) < 0;
    byte!(@pointer ram_ptr!(game, u16; 0x723); 0) =
        ((word!(game.screen_fade_delay)) as u8).wrapping_sub(1);
    if (word!(game.screen_fade_delay) as u8) != 0 && !v0 {
        return 0;
    }
    if (*ram_ptr!(game; 0x51) & 0xf) != 0 {
        if (*ram_ptr!(game; 0x51) & 0xf) != 1 {
            *ram_ptr!(game; 0x51) = ((*ram_ptr!(game; 0x51) as i32 & 0xf) - 1i32) as u8;
            byte!(@pointer ram_ptr!(game, u16; 0x723); 0) = word!(game.screen_fade_counter) as u8;
            return 0;
        }
        *ram_ptr!(game; 0x51) = 0x80;
        byte!(@pointer ram_ptr!(game, u16; 0x723); 0) = 0;
        byte!(@pointer ram_ptr!(game, u16; 0x725); 0) = 0;
    }
    1
}
pub unsafe fn AdvanceFastScreenFadeIn(game: &mut Game) -> u8 {
    if (word!(game.screen_fade_counter) as i32 + *ram_ptr!(game; 0x51) as i32 & 0x1f) - 15i32 & 0x80
        != 0
    {
        *ram_ptr!(game; 0x51) =
            (word!(game.screen_fade_counter) as i32 + *ram_ptr!(game; 0x51) as i32 & 0x1f) as u8;
        0
    } else {
        *ram_ptr!(game; 0x51) = 15;
        1
    }
}
pub unsafe fn AdvanceSlowScreenFadeIn(game: &mut Game) -> u8 {
    let v0: bool = ((word!(game.screen_fade_delay)) as i8).wrapping_sub(1) < 0;
    byte!(@pointer ram_ptr!(game, u16; 0x723); 0) =
        ((word!(game.screen_fade_delay)) as u8).wrapping_sub(1);
    if (word!(game.screen_fade_delay) as u8) != 0 && !v0 {
        return 0;
    }
    if (*ram_ptr!(game; 0x51) as i32 + 1 & 0x1f) - 15i32 & 0x80 != 0 {
        *ram_ptr!(game; 0x51) = (*ram_ptr!(game; 0x51) as i32 + 1 & 0x1f) as u8;
        byte!(@pointer ram_ptr!(game, u16; 0x723); 0) = word!(game.screen_fade_counter) as u8;
        return 0;
    }
    *ram_ptr!(game; 0x51) = 15;
    byte!(@pointer ram_ptr!(game, u16; 0x723); 0) = 0;
    byte!(@pointer ram_ptr!(game, u16; 0x725); 0) = 0;
    1
}
pub unsafe fn InitializeIoDisplayLogo_Async(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.logo");
    let resume_state: *mut u8 = ram_ptr!(game; 0x612);
    let mut stage = *resume_state;
    if stage == 0 {
        crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
        word!(game.oam_next_ptr = 0);
        ClearOamExt(game);
        ClearUnusedOam(game);
        word! { game, 0;
            nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
            nmi_copy_samus_bottom_half_src,
        }
        crate::rtl::register_writes! { game;
            mirror(NMITIMEN as u16, 1, 0x84, false); byte(WRIO as u16, 0);
            byte(WRMPYA as u16, 0); byte(WRMPYB as u16, 0);
            byte(WRDIVL as u16, 0); byte(WRDIVH as u16, 0);
            byte(WRDIVB as u16, 0); byte(HTIMEL as u16, 0);
            byte(HTIMEH as u16, 0); byte(VTIMEL as u16, 0);
            byte(VTIMEH as u16, 0); byte(MDMAEN as u16, 0);
            mirror(HDMAEN as u16, 0, 0x85, false); mirror(MEMSEL as u16, 1, 0x86, false);
            mirror(OBSEL as u16, 3, 0x52, false); byte(OAMADDL as u16, 0);
        }
        *(ram_ptr!(game, u16; 0x53) as *mut u8) = 0;
        WriteReg(OAMADDH as u16, 0x80, game);
        byte!(@pointer ram_ptr!(game, u16; 0x53); 1) = 0x80;
        crate::rtl::register_writes! { game;
            byte(OAMDATA as u16, 0); byte(OAMDATA as u16, 0);
            mirror(BGMODE as u16, 1, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
            mirror(BG1SC as u16, 0, 0x58, false); mirror(BG2SC as u16, 0, 0x59, false);
            mirror(BG3SC as u16, 0, 0x5a, false); mirror(BG4SC as u16, 0, 0x5c, false);
            mirror(BG12NBA as u16, 0, 0x5d, false); mirror(BG34NBA as u16, 0, 0x5e, false);
            byte(BG1HOFS as u16, 0); byte(BG1HOFS as u16, 0);
            byte(BG1VOFS as u16, 0); byte(BG1VOFS as u16, 0);
            byte(BG2HOFS as u16, 0); byte(BG2HOFS as u16, 0);
            byte(BG2VOFS as u16, 0); byte(BG2VOFS as u16, 0);
            byte(BG3HOFS as u16, 0); byte(BG3HOFS as u16, 0);
            byte(BG3VOFS as u16, 0); byte(BG3VOFS as u16, 0);
            byte(BG4HOFS as u16, 0); byte(BG4HOFS as u16, 0);
            byte(BG4VOFS as u16, 0); byte(BG4VOFS as u16, 0);
            byte(VMAIN as u16, 0); mirror(M7SEL as u16, 0, 0x5f, false);
            byte(M7A as u16, 0); byte(M7B as u16, 0);
            byte(M7C as u16, 0); byte(M7D as u16, 0);
            byte(M7X as u16, 0); byte(M7Y as u16, 0);
            mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
            mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
            mirror(WH1 as u16, 0, 0x64, false); mirror(WH2 as u16, 0, 0x65, false);
            mirror(WH3 as u16, 0, 0x66, false); mirror(WBGLOG as u16, 0, 0x67, false);
            mirror(WOBJLOG as u16, 0, 0x68, false); mirror(TM as u16, 0x10, 0x69, false);
            mirror(TMW as u16, 0, 0x6c, false); mirror(TS as u16, 0, 0x6b, false);
            mirror(TSW as u16, 0, 0x6d, false); mirror(CGWSEL as u16, 0, 0x6e, false);
            mirror(CGADSUB as u16, 0, 0x71, false); byte(COLDATA as u16, 0xe0);
        }
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
        crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
        GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        *ram_ptr!(game; 0x51) = 0x80;
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce1e9, 0));
        EnableNMI(game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
    }
    // Stages 1..=6 resume a pending NMI before drawing the next frame.
    while stage <= 6 {
        if stage != 0 {
            let yielded: u8 = WaitForNMI_Async(game);
            if yielded != 0 {
                *resume_state = stage;
                return yielded;
            }
            match stage {
                2 => {
                    word!(game.game_options_screen_index = 120);
                }
                4 => {
                    word! { game, 1; screen_fade_delay, screen_fade_counter }
                }
                6 => {
                    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
                    break;
                }
                _ => {}
            }
        }
        ClearOamExt(game);
        word!(game.oam_next_ptr = 0);
        AddNintentoLogoToOam(game);
        stage = match stage {
            0 | 1 => {
                crate::verify_probe::hit("coroutine.logo.in");
                if AdvanceFastScreenFadeIn(game) & 1 != 0 {
                    2
                } else {
                    1
                }
            }
            2 | 3 => {
                crate::verify_probe::hit("coroutine.logo.hold");
                if word!(game.game_options_screen_index -= 1, reread) == 0 {
                    4
                } else {
                    3
                }
            }
            _ => {
                crate::verify_probe::hit("coroutine.logo.out");
                if AdvanceFastScreenFadeOut(game) & 1 != 0 {
                    6
                } else {
                    5
                }
            }
        };
        ClearUnusedOam(game);
    }
    *resume_state = 0;
    0 as CoroutineRet
}
pub unsafe fn AddNintentoLogoToOam(game: &mut Game) {
    DrawSpritemap(0x8c, addr_kSpriteMap_NintendoLogo as u16, 128, 112, 0, game);
}
pub unsafe fn SpawnCinematicSpriteObject(j: u16, a: u16, game: &mut Game) -> u8 {
    word!(game.cinematic_spawn_param = a);
    for i in (0i32..=30).rev().step_by(2) {
        if word!(game.cinematicspr_instr_ptr[(i >> 1)]) == 0 {
            return SpawnCimenaticSpriteObjectInner(j, i as u16, game);
        }
    }
    1
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
    word!(game.cinematic_spawn_param = 0);
    SpawnCimenaticSpriteObjectInner(j, r18, game)
}
pub unsafe fn SpawnCimenaticSpriteObjectInner(k: u16, j: u16, game: &mut Game) -> u8 {
    let v2 = RomPtr_8B(k, game);
    let v3: i32 = j as i32 >> 1;
    word!(game.cinematicspr_preinstr_func[v3] = *(v2.offset(2) as *mut u16));
    word!(game.cinematicspr_instr_ptr[v3] = *(v2.offset(4) as *mut u16));
    word!(game.cinematicspr_instr_timer[v3] = 1);
    word! { game, 0;
        cinematicspr_whattodraw[v3], cinematicspr_goto_timer[v3], cinematicspr_arr6[v3],
        cinematicspr_arr7[v3],
    }
    CallCinematicSpriteObjectSetup(((*(v2 as *mut u16)) as u32) | (0x8b0000u32), j, game);
    0
}
pub unsafe fn ClearCinematicSprites(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(cinematicspr_instr_ptr, 15) <- value(0),
        le(cinematicspr_whattodraw, 15) <- value(0),
    );
}
pub unsafe fn HandleCinematicSprites(game: &mut Game) {
    let mut i: i32 = 30;
    while i >= 0 {
        word!(game.cinematic_obj_index = i as u16);
        if word!(game.cinematicspr_instr_ptr[(i >> 1)]) != 0 {
            ProcessCinematicSpriteInstructionList(i as u16, game);
            i = word!(game.cinematic_obj_index) as i32;
        }
        i -= 2;
    }
}

macro_rules! execute_cinematic_commands {
    (
        $instruction:ident, $read_instruction:path, $address:ident, $game:ident,
        $dispatch:path, $object_index:ident $(,)?
    ) => {
        let mut $instruction: u16 = 0;
        loop {
            let v5 = $read_instruction($address, $game) as *const u16;
            $instruction = *v5;
            if (*v5 & 0x8000) == 0 {
                break;
            }
            $address = $dispatch(
                (($instruction) as u32) | (0x8b0000u32),
                $object_index,
                ($address).wrapping_add(2),
                $game,
            );
            if $address == 0 {
                return;
            }
        }
    };
}

pub unsafe fn ProcessCinematicSpriteInstructionList(k: u16, game: &mut Game) {
    CallCinematicSprPreInstr(
        ((word!(game.cinematicspr_preinstr_func[(k >> 1)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = word!(game.cinematic_obj_index);
    let v2: i32 = word!(game.cinematic_obj_index) as i32 >> 1;
    crate::update_place! { unaligned_word fresh22 [fresh23] = &raw mut (*(ram_ptr!(game, u16; 0x1b5d)).offset(v2 as isize)) => (u16::from_le(fresh22.read_unaligned())).wrapping_sub(1); }
    if fresh23 == 1 {
        let mut v4: u16 = word!(game.cinematicspr_instr_ptr[v2]);
        execute_cinematic_commands!(v6, RomPtr_8B, v4, game, CallCinematicSprInstr, v1);
        let v7: i32 = v1 as i32 >> 1;
        word!(game.cinematicspr_instr_timer[v7] = v6);
        word!(game.cinematicspr_whattodraw[v7] = *(RomPtr_8B(v4, game) as *mut u16).offset(1));
        word!(game.cinematicspr_instr_ptr[v7] = (v4).wrapping_add(4));
    }
}
pub unsafe fn SpawnMode7Object(j: u16, a: u16, game: &mut Game) -> u8 {
    word!(game.mode7_spawn_param = a);
    let mut v3: u16 = 2;
    while word!(game.mode7_obj_instr_ptr[(v3 >> 1)]) != 0 {
        v3 = (v3).wrapping_sub(2);
        if (v3 & 0x8000) != 0 {
            return 1;
        }
    }
    let mut Mode7ObjectDef = get_Mode7ObjectDef(j, game);
    let v6: i32 = v3 as i32 >> 1;
    word! {game; mode7_obj_preinstr_func[v6] = (*Mode7ObjectDef).field_2;
    mode7_obj_instr_ptr[v6] = (*Mode7ObjectDef).field_4; mode7_obj_instr_timer[v6] = 1;
    mode7_obj_goto_timer[v6] = 0; }
    CallCinematicSpriteInit(
        (((*Mode7ObjectDef).field_0) as u32) | (0x8b0000u32),
        v6 as u16,
        game,
    );
    0
}
pub unsafe fn HandleMode7Objects(game: &mut Game) {
    let mut i: i32 = 2;
    while i >= 0 {
        word!(game.mode7_cur_index = i as u16);
        if word!(game.mode7_obj_instr_ptr[(i >> 1)]) != 0 {
            ProcessMode7ObjectInstructions(i as u16, game);
            i = word!(game.mode7_cur_index) as i32;
        }
        i -= 2;
    }
}
pub unsafe fn ProcessMode7ObjectInstructions(k: u16, game: &mut Game) {
    CallMode7PreInstr(
        ((word!(game.mode7_obj_preinstr_func[(k >> 1)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = word!(game.mode7_cur_index);
    let v2: i32 = word!(game.mode7_cur_index) as i32 >> 1;
    crate::update_place! { unaligned_word fresh19 [fresh20] = &raw mut (*(ram_ptr!(game, u16; 0x19a9)).offset(v2 as isize)) => (u16::from_le(fresh19.read_unaligned())).wrapping_sub(1); }
    if fresh20 == 1 {
        let mut v4: u16 = word!(game.mode7_obj_instr_ptr[v2]);
        execute_cinematic_commands!(v6, RomPtr_8B, v4, game, CallMode7Instr, v1);
        word!(game.mode7_obj_instr_timer[(v1 >> 1)] = v6);
        let v7 = RomPtr_8B(v4, game);
        QueueMode7Transfers(0x8b, *(v7.offset(2) as *mut u16), game);
        word!(game.mode7_obj_instr_ptr[(v1 >> 1)] = (v4).wrapping_add(4));
    }
}
pub unsafe fn EnableCinematicBgObjects(game: &mut Game) {
    word!(game.cinematic_enable_objs |= (0x8000u16));
}
pub unsafe fn EnableCinematicBgTilemapUpdates(game: &mut Game) {
    word!(game.cinematic_enable_bg_tilemap |= (0x8000u16));
}
pub unsafe fn DisableCinematicBgTilemapUpdates(game: &mut Game) {
    word!(game.cinematic_enable_bg_tilemap &= !(0x8000));
}
pub unsafe fn ClearCinematicBgObjects(a: u16, game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(a));
    word_transfers!(game, descending, 4;
        le(cinematicbg_arr1, 3) <- value(0),
        le(cinematicbg_instr_ptr, 3) <- value(0),
    );
    word!(game.cinematicbg_var1 = 0);
}
crate::game::address_dispatch! {
    pub unsafe fn CallCinematicBgObjectSetup(ea: u32, j: u16, game: &mut Game) => ea;
        call (); default { Unreachable(game); };
    {
        9147353 => [()];
    }
}
pub unsafe fn SpawnCinematicBgObject(j: u16, a: u16, game: &mut Game) -> u8 {
    word!(game.cinematicbg_var3 = a);
    let mut v3: u16 = 6;
    while word!(game.cinematicbg_instr_ptr[(v3 >> 1)]) != 0 {
        v3 = (v3).wrapping_sub(2);
        if (v3 & 0x8000) != 0 {
            return 1;
        }
    }
    let v5 = RomPtr_8B(j, game);
    let v6: i32 = v3 as i32 >> 1;
    word! {game; cinematicbg_preinstr[v6] = *(v5.offset(2) as *mut u16);
    cinematicbg_instr_ptr[v6] = *(v5.offset(4) as *mut u16); cinematicbg_instr_timer[v6] = 1; }
    word! { game, 0; cinematicbg_arr1[v6], cinematicbg_arr6[v6] }
    CallCinematicBgObjectSetup(((*(v5 as *mut u16)) as u32) | (0x8b0000u32), v3, game);
    0
}
pub unsafe fn HandleCinematicBgObjects(game: &mut Game) {
    if (word!(game.cinematic_enable_objs) & 0x8000) != 0 {
        let mut i: i32 = 6;
        while i >= 0 {
            word!(game.cinematicbg_var2 = i as u16);
            if word!(game.cinematicbg_instr_ptr[(i >> 1)]) != 0 {
                ProcessCinematicBgObjectsInstrs(i as u16, game);
                i = word!(game.cinematicbg_var2) as i32;
            }
            i -= 2;
        }
        if (word!(game.cinematic_enable_bg_tilemap) & 0x8000) != 0 {
            UpdateCinematicBgTilemap(game);
        }
        UpdateSamusEyesTilemap(game);
    }
}
pub unsafe fn ProcessCinematicBgObjectsInstrs(k: u16, game: &mut Game) {
    CallCinematicBgPreInstr(
        ((word!(game.cinematicbg_preinstr[(k >> 1)])) as u32) | (0x8b0000u32),
        k,
        game,
    );
    let v1: u16 = word!(game.cinematicbg_var2);
    let v2: i32 = word!(game.cinematicbg_var2) as i32 >> 1;
    crate::update_place! { unaligned_word fresh158 [fresh159] = &raw mut (*(ram_ptr!(game, u16; 0x19dd)).offset(v2 as isize)) => (u16::from_le(fresh158.read_unaligned())).wrapping_sub(1); }
    if fresh159 == 1 {
        let mut v4: u16 = word!(game.cinematicbg_instr_ptr[v2]);
        execute_cinematic_commands!(v6, RomPtr_8C, v4, game, CallCinematicBgInstr, v1);
        let v7: i32 = v1 as i32 >> 1;
        word!(game.cinematicbg_instr_timer[v7] = v6);
        let v8 = RomPtr_8C(v4, game);
        word!(game.cinematicbg_arr1[v7] = *(v8.offset(4) as *mut u16));
        ProcessCinematicBgObject(v1, *(v8.offset(2) as *mut u16), game);
        word!(game.cinematicbg_instr_ptr[v7] = (v4).wrapping_add(6));
    }
}
macro_rules! draw_cinematic_sprites {
    ($object_offset:ident, $game:ident) => {
        while $object_offset >= 0 {
            let v1: i32 = $object_offset >> 1;
            if word!($game.cinematicspr_whattodraw[v1]) != 0 {
                let v2: u16 = word!($game.cinematicspr_whattodraw[v1]);
                let chr: u16 = word!($game.cinematicbg_arr9[v1]);
                let x: u16 =
                    (word!($game.cinematicbg_arr7[v1])).wrapping_sub(word!($game.layer1_x_pos));
                let y: u16 =
                    (word!($game.cinematicbg_arr8[v1])).wrapping_sub(word!($game.layer1_y_pos));
                if (y & 0xff00) != 0 {
                    if (y).wrapping_add(128) < 0x1ff {
                        DrawSpritemapOffScreen(v2, x, y, chr, $game);
                    }
                } else if (y).wrapping_add(128) < 0x1ff {
                    DrawSpritemap(0x8c, v2, x, y, chr, $game);
                }
            }
            $object_offset -= 2;
        }
    };
}

pub unsafe fn DrawCinematicSpriteObjects_Intro(game: &mut Game) {
    let mut i: i32 = 30;
    draw_cinematic_sprites!(i, game);
}
pub unsafe fn DrawCinematicSpriteObjects_Ending(game: &mut Game) {
    let mut i: i32 = 30;
    draw_cinematic_sprites!(i, game);
    if word!(game.cinematic_function) as i32 - (0x8bda02 & 0xffff) & 0x8000 != 0 {
        MoveUnusedSpritesOffScreen(game);
    }
}
pub unsafe fn SpawnTextGlowObject(j: u16, mut r18: u16, game: &mut Game) {
    let mut v1: u16 = 14;
    while *(ram_ptr!(game, u16; 0x19f7, v1 as i32 as isize)) != 0 {
        v1 = (v1).wrapping_sub(2);
        if (v1 & 0x8000) != 0 {
            return;
        }
    }
    *(ram_ptr!(game, u16; 0x19f7, v1 as i32 as isize)) = j;
    let v2: i32 = v1 as i32 >> 1;
    word! {game; eproj_pre_instr[(v2 + 2)] = 1; eproj_pre_instr[(v2 + 10)] = r18 as u8 as u16; }
    *(ram_ptr!(game, u16; 0x1a27, v1 as i32 as isize)) = byte!(r18; 1) as u16;
    *(ram_ptr!(game, u16; 0x1a37, v1 as i32 as isize)) = 0;
}
pub unsafe fn HandleTextGlowObjects(game: &mut Game) {
    let mut i: i32 = 14;
    while i >= 0 {
        word!(game.cinematic_var19 = i as u16);
        if *(ram_ptr!(game, u16; 0x19f7, i as u16 as i32 as isize)) != 0 {
            ProcessTextGlowObject(game);
            i = word!(game.cinematic_var19) as i32;
        }
        i -= 2;
    }
}
pub unsafe fn ProcessTextGlowObject(game: &mut Game) {
    let v0: u16 = word!(game.cinematic_var19);
    let v1: i32 = word!(game.cinematic_var19) as i32 >> 1;
    crate::update_place! { unaligned_word fresh153 [fresh154] = &raw mut (*(ram_ptr!(game, u16; 0x1a03)).offset((v1 + 2) as isize)) => (u16::from_le(fresh153.read_unaligned())).wrapping_sub(1); }
    if fresh154 == 1 {
        let R28: u16 = *(ram_ptr!(game, u16; 0x1a37, v0 as i32 as isize));
        let mut v3: u16 = *(ram_ptr!(game, u16; 0x19f7, v0 as i32 as isize));
        let mut r22: u16 = (2
            * byte!(@pointer (ram_ptr!(game, u16; 0x1a03)).offset((v1 + 10i32) as isize); 0) as i32
            + Mult8x8(
                byte!(@pointer ram_ptr!(game, u16; 0x1a27); v0 as i32 as isize),
                0x40,
            ) as i32) as u16;
        let mut v4: u16 = r22;
        let v5 = RomPtr_8C(v3, game);
        let mut n: i32 = *v5.offset(2) as i32;
        let norg: i32 = n;
        let mut m: i32 = *v5.offset(3) as i32;
        loop {
            do_until! { (n == 0);
                *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
                    as *mut u8)
                    .offset(v4 as i32 as isize) as *mut u16) = (R28
                    | *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                        as *mut u16 as *mut u8)
                        .offset(v4 as i32 as isize) as *mut u16)
                        & 0xe3ff);
                v3 = (v3).wrapping_add(2);
                v4 = (v4).wrapping_add(2);
                n -= 1;
            }
            n = norg;
            m -= 1;
            if m == 0 {
                break;
            }
            r22 = (r22).wrapping_add(64);
            v4 = r22;
        }
        let v6: u16 = word!(game.cinematic_var19);
        if *(ram_ptr!(game, u16; 0x1a37, word!(game.cinematic_var19) as i32 as isize)) == 3072 {
            *(ram_ptr!(game, u16; 0x19f7, word!(game.cinematic_var19) as i32 as isize)) = 0;
        } else {
            crate::update_place! { unaligned_word fresh155 = &raw mut (*(ram_ptr!(game, u16; 0x1a37, word!(game.cinematic_var19) as i32 as isize))) => (u16::from_le(fresh155.read_unaligned())).wrapping_add(1024); }
            *(ram_ptr!(game, u16; 0x1a03)).offset(((v6 as i32 >> 1) + 2) as isize) = 5;
        }
    }
}
pub unsafe fn EnableTextGlowObjects_(game: &mut Game) {
    word!(game.cinematic_var25 |= (0x8000u16));
}
pub unsafe fn DisableTextGlowObjects_(game: &mut Game) {
    word!(game.cinematic_var25 &= !(0x8000));
}
pub unsafe fn CinematicFunction_Intro_Func128(a: u16, game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(a));
    word! { game, 0;
        cinematic_var26, cinematic_var21, cinematic_var22, cinematic_var23, cinematic_var24,
    }
    word!(game.cinematicbg_var3 = 18432);
    word! { game, 0;
        eproj_pre_instr[0], cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10,
    }
}
pub unsafe fn CreditsObject_Init(j: u16, game: &mut Game) {
    let v1 = RomPtr_8B(j, game);
    word! {game; cinematic_var24 = *(v1.offset(2) as *mut u16);
    cinematic_var21 = *(v1.offset(4) as *mut u16); cinematic_var22 = 1; cinematic_var23 = 0; }
    CallCreditsObjectFunc(((*(v1 as *mut u16)) as u32) | (0x8b0000u32), 0, game);
}
pub unsafe fn CreditsObject_Process(game: &mut Game) {
    if (word!(game.cinematic_var25) & 0x8000) != 0 {
        if word!(game.cinematic_var21) != 0 {
            CreditsObject_ProcessOne(game);
        }
        CinematicUpdateSomeBg(game);
    }
}
pub unsafe fn CreditsObject_ProcessOne(game: &mut Game) {
    CallCreditsObjectFunc(
        ((word!(game.cinematic_var24)) as u32) | (0x8b0000u32),
        0,
        game,
    );
    words!(game.cinematic_var10, cinematic_var9 += 0x8000);
    if word!(game.cinematic_var10) as i32 - word!(game.eproj_pre_instr[0]) as i32 - 8i32 & 0x8000
        == 0
    {
        word!(game.eproj_pre_instr[0] = word!(game.cinematic_var10));
        let mut i: i32 = word!(game.cinematic_var21) as i32;
        loop {
            let v2 = RomPtr_8C(i as u16, game) as *const u16;
            let v3: u16 = *v2;
            if (*v2 & 0x8000) == 0 {
                break;
            }
            i = CallIntroObjectInstr(((v3) as u32) | (0x8b0000u32), 0, (i + 2) as u16, game) as i32;
            if i == 0 {
                return;
            }
        }
        CreditsObject_Func1(i as u16, game);
        word!(game.cinematic_var26 = (word!(game.cinematic_var26) as i32 + 1 & 0x1f) as u16);
        word!(game.cinematic_var21 = (i + 4) as u16);
    }
}
pub unsafe fn CreditsObject_Func1(j: u16, game: &mut Game) {
    let mut RegWord: u16 = Mult8x8(word!(game.cinematic_var26) as u8, 0x40);
    let mut src = ram_ptr!(game; 0x10000, *(RomPtr_8C(j, game) as *mut u16).offset(1) as i32 as isize)
        as *const u16;
    for _ in 0..=31 {
        crate::update_place! { previous fresh6 = src => src.offset(1); }
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
            as *mut u8)
            .offset(RegWord as i32 as isize) as *mut u16) = *fresh6;
        RegWord = (RegWord).wrapping_add(2);
    }
}
pub unsafe fn GameState_1_OpeningCinematic_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
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
    if word!(game.cinematic_function) as i32 + 0x60d8 & 0x8000 != 0
        && (word!(game.joypad1_newkeys)
            & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16)
            != 0
        && word!(game.cinematic_var18) == 0
    {
        word! {game; cinematic_var18 = 1; screen_fade_delay = 0; screen_fade_counter = 2; }
    }
}
pub unsafe fn HandleCinematicsTransitions_1(game: &mut Game) {
    if AdvanceFastScreenFadeOut(game) & 1 != 0 {
        word! {game; cinematic_var18 = 2; cinematic_function = ((0x8b9f28i32) as u16); }
        QueueMusic_Delayed8(6, game);
    }
}
pub unsafe fn HandleCinematicsTransitions_2(game: &mut Game) {
    ClearCinematicSprites(game);
    SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA119 as u16, 0, game);
    SpawnCinematicSpriteObject(
        addr_kCinematicSpriteObjectDef_8BA125 as u16,
        0x0400, // Palette left in A by the first sprite initializer.
        game,
    );
    word! {game; screen_fade_delay = 0; screen_fade_counter = 2; cinematic_var6 = 256; }
    word! { game, 0;
        cinematic_var8, cinematic_var7, cinematic_var10, cinematic_var9, g_word_7E199B,
        g_word_7E1999, g_word_7E199F, g_word_7E199D,
    }
    word!(game.cinematic_var18 = 3);
    word_transfers!(game, ascending, 128; le(palette_buffer, 128) <- rom(0x8ce1e9, 128));
    word! {game; palette_buffer[201] = 0x7fff; palette_buffer[202] = 32128; }
    ClearPaletteFXObjects(game);
    SpawnPalfxObject(addr_kPalfx_E1A0 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as u16, game);
    SetPpuBackdropBlack(game);
    *ram_ptr!(game; 0x69) = 17;
    SpawnTitleScreenGradientObjs(game);
}
pub unsafe fn HandleCinematicsTransitions_3(game: &mut Game) {
    if AdvanceFastScreenFadeIn(game) & 1 != 0 {
        word! {game; cinematic_var18 = 0; demo_timer = 900;
        cinematic_function = ((0x8b9f29i32) as u16); }
    }
}
static off_8B9A7B: [Option<Func_V>; 4] = callback_table! { unsafe Func_V;
    None, HandleCinematicsTransitions_1, HandleCinematicsTransitions_2,
    HandleCinematicsTransitions_3,
};
pub unsafe fn HandleCinematicsTransitions(game: &mut Game) {
    if word!(game.cinematic_var18) != 0 {
        off_8B9A7B[word!(game.cinematic_var18) as usize].expect("non-null function pointer")(game);
    }
}
pub unsafe fn LoadTitleSequenceGraphics(game: &mut Game) {
    SetupPpuForTitleSequence(game);
    word! { game, 0; cur_irq_handler, irqhandler_next_handler }
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce1e9, 0));
    GraphicsAsset::Decompress(0x94e000, 0x10000).load(game);
    GraphicsAsset::Decompress(0x96fc04, 0x14000).load(game);
    GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
    GraphicsAsset::Decompress(0x95a5e1, 0x19000).load(game);
    GraphicsAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
    crate::rtl::register_writes! { game;
        byte(VMAIN as u16, 0); byte(VMADDL as u16, 0);
        byte(VMADDH as u16, 0);
    }
    for _ in 0i16..0x4000i16 {
        WriteReg(VMDATAL as u16, 0xff, game);
    }
    GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x1000).load(game);
    GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    reset_centered_mode7_transform!(game);
    EnableNMI(game);
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    EnableHdmaObjects(game);
    EnablePaletteFx(game);
    word!(game.fx_layer_blending_config_a = 0);
    SpawnPalfxObject(addr_kPalfx_E1A0 as u16, game);
    SpawnPalfxObject(addr_kPalfx_E1A4 as u16, game);
    SpawnMode7Object(
        addr_kMode7ObjBabyMetroidInTitle as u16,
        0x000c, // The second title palette allocation leaves its slot offset in A.
        game,
    );
}
macro_rules! advance_cinematic_pan_to_blackout {
    ($game:ident, $cutoff:literal, $sprite_definition:ident) => {
        words!(
            $game.cinematic_var8,
            cinematic_var7 += words!($game.g_word_7E199B, g_word_7E1999)
        );
        if word!($game.cinematic_var8) as i32 + $cutoff & 0x8000 != 0 {
            *ram_ptr!($game; 0x69) = 16;
            word!($game.cinematic_function = ((0x8b9a47i32) as u16));
            SpawnCinematicSpriteObject(
                $sprite_definition as u16,
                ((0x8b9a47i32) as u16),
                $game,
            );
            SetPpuBackdropBlack($game);
        }
    };
}

pub use self::CinematicSpriteInit_0 as CinematicSpriteInit_1;
pub use self::CinematicSpriteInit_0 as CinematicSpriteInit_2;
pub use self::CinematicSpriteInit_0 as CinematicSpriteInit_3;
macro_rules! finish_cinematic_and_clear_effects {
    ($game:ident, $next_state:literal, $index_field:ident) => {
        EnableNMI($game);
        *ram_ptr!($game; 0x55) = 1;
        word! { $game, 0;
            screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
            reg_BG3VOFS, cinematic_var5,
        }
        word!($game.cinematic_function = ((0x8b9b68i32) as u16));
        DisablePaletteFx($game);
        ClearPaletteFXObjects($game);
        word_transfers!($game, descending, 329; native(0x1c1d) <- value(0));
        word_transfers!($game, descending, 256; le(hdma_table_1, 255) <- value(0));
        word!($game.game_state = $next_state);
        word!($game.$index_field = 0);
    };
}

pub unsafe fn ConfigureTitleSequenceGradientHDMA(game: &mut Game) {
    let mut v0: u16 = crate::rtl::rom_table_word!(
        game,
        0x8cbc5d,
        ((word!(game.cinematic_var6) as i32 & 0xf0) >> 4) as isize
    );
    let mut i: i32 = 0;
    loop {
        let v2: u16 = *(RomPtr_8C(v0, game) as *mut u16);
        word!(game.hdma_table_1[(i >> 1)] = v2);
        if v2 == 0 {
            break;
        }
        v0 = (v0).wrapping_add(2);
        i += 2;
    }
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
pub unsafe fn GameState_37_CeresGoesBoomWithSamus_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
        game,
    );
    if (word!(game.cinematic_var11) & 0x8000) == 0 {
        word!(game.cinematic_var11 += 1);
    }
    word!(game.cinematic_var12 += 1);
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
pub unsafe fn ClearEnglishTextTilemap(game: &mut Game) {
    word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
}
pub unsafe fn ClearJapaneseTextTiles(game: &mut Game) {
    for start in (0..768).step_by(8) {
        word_transfers!(game, ascending, 8; xray(start) <- native(0x18290));
    }
}
static kSetSomeStuffForSpriteObject_6_Tab: [i16; 12] = [
    0x5c, 0x58, 0x63, 0x58, 0x59, 0x5d, 0x60, 0x5b, 0x66, 0x5e, 0x63, 0x60,
];
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
pub unsafe fn SetSomeStuffForSpriteObject_15(game: &mut Game) {
    word! {game; cinematicbg_arr7[15] = 8; cinematicbg_arr8[15] = 248; }
}
pub unsafe fn SetSomeStuffForSpriteObject_16(game: &mut Game) {
    word! {game; cinematicbg_arr7[15] = 8; cinematicbg_arr8[15] = 24;
    cinematicspr_instr_ptr[15] = addr_word_8BCBFB as u16; cinematicspr_instr_timer[15] = 1; }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCreditsObjectFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [9155772] CinematicBgPreInstr_SamusBlink [] {
        if word!(game.cinematic_function) as i32 == 0x8bb207 & 0xffff
            || word!(game.cinematic_function) as i32 == 0x8ba390 & 0xffff
        {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicbg_instr_ptr[v1] = -10733i32 as u16);
            word!(game.cinematicbg_instr_timer[v1] = 1);
            word!(game.cinematicbg_preinstr[v1] = ((0x8b93d9i32) as u16));
        }
    };
    @external {
        9147353 => [()];
    }
}
pub unsafe fn CinematicFunction_Intro_Func37(k: u16, game: &mut Game) {
    let mut v3: u16 = 0;
    let v1: u16 = word!(game.cinematicspr_arr6[(k >> 1)]);
    if v1 != 0 {
        if (v1 & 1) != 0 {
            word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8ce3e9, 255));
            v3 = k;
        } else {
            word_transfers!(game, descending, 16; native(0xc1fe) <- value(0x7fff));
            v3 = k;
        }
        word!(game.cinematicspr_arr6[(v3 >> 1)] -= 1);
    }
}
pub unsafe fn CinematicFunction_Intro_Func40(game: &mut Game) {
    if (word!(game.cinematic_var12) & 1) != 0 {
        word!(game.cinematic_var10 -= 4);
    } else {
        word!(game.cinematic_var10 += 4);
    };
}
static kCinematicFunction_Intro_Func42_Tab0: [u16; 4] = [0x70, 0xc0, 0x80, 0xe8];
static kCinematicFunction_Intro_Func42_Tab1: [u16; 4] = [0x50, 0x40, 0x38, 0x58];
static kCinematicFunction_Intro_Func45_Tab0: [u16; 4] = [0, 0xffff, 0, 0xffff];
static kCinematicFunction_Intro_Func46_Tab0: [u16; 5] = [0, 0x10, 0xfff0, 0xfff8, 8];
static kCinematicFunction_Intro_Func46_Tab1: [u16; 5] = [0, 0xfff0, 8, 0xfff0, 8];
static kCinematicFunction_Intro_Func46_Tab2: [u16; 5] = [1, 0x10, 0x20, 0x30, 0x40];
static kCinematicFunction_Intro_Func47_Tab0: [u16; 3] = [0x10, 0xfff0, 0xfff0];
static kCinematicFunction_Intro_Func47_Tab1: [u16; 3] = [0, 4, 0xfff8];
static kCinematicFunction_Intro_Func47_Tab2: [u16; 3] = [1, 8, 0x10];
pub use self::SetSomeStuffForSpriteObject_3 as CinematicFunction_Intro_Func49;

macro_rules! chase_samus_with_cinematic_sprite {
    ($game:ident, $y_offset:literal, $object_slot:ident) => {
        if ((word!($game.samus_y_pos) as i32
            - $y_offset
            - word!($game.cinematicbg_arr8[$object_slot]) as i32) as i16)
            < 0
        {
            if word!($game.cinematic_var17) as i32 + 543 & 0x8000 == 0 {
                word!($game.cinematic_var17, wrapping_sub(32));
            }
        } else if word!($game.cinematic_var17) as i32 - 544i32 & 0x8000 != 0 {
            word!($game.cinematic_var17, wrapping_add(32));
        }
        AddToHiLo(
            (ram_ptr!($game, u16; 0x1a9d)).offset($object_slot as isize),
            (ram_ptr!($game, u16; 0x1afd)).offset($object_slot as isize),
            ((word!($game.cinematic_var17) as i16 as i32) << 8) as u32,
        );
    };
}

pub unsafe fn ResetButtonAssignmentsToDefault(game: &mut Game) {
    word! {game; button_config_shoot_x_saved = word!(game.button_config_shoot_x);
    button_config_jump_a_saved = word!(game.button_config_jump_a);
    button_config_run_b_saved = word!(game.button_config_run_b);
    button_config_itemcancel_y_saved = word!(game.button_config_itemcancel_y);
    button_config_itemswitch_saved = word!(game.button_config_itemswitch);
    button_config_aim_down_L_saved = word!(game.button_config_aim_down_L);
    button_config_aim_up_R_saved = word!(game.button_config_aim_up_R);
    button_config_up = kButton_Up as u16; button_config_down = kButton_Down as u16;
    button_config_left = kButton_Left as u16; button_config_right = kButton_Right as u16;
    button_config_shoot_x = kButton_X as u16; button_config_jump_a = kButton_A as u16;
    button_config_run_b = kButton_B as u16; button_config_itemcancel_y = kButton_Y as u16;
    button_config_itemswitch = kButton_Select as u16;
    button_config_aim_down_L = kButton_L as u16; button_config_aim_up_R = kButton_R as u16; }
}
pub unsafe fn RevertButtonConfig(game: &mut Game) {
    word!(game.button_config_shoot_x = word!(game.button_config_shoot_x_saved));
    word!(game.button_config_jump_a = word!(game.button_config_jump_a_saved));
    word!(game.button_config_run_b = word!(game.button_config_run_b_saved));
    word!(game.button_config_itemcancel_y = word!(game.button_config_itemcancel_y_saved));
    word!(game.button_config_itemswitch = word!(game.button_config_itemswitch_saved));
    word!(game.button_config_aim_down_L = word!(game.button_config_aim_down_L_saved));
    word!(game.button_config_aim_up_R = word!(game.button_config_aim_up_R_saved));
}

macro_rules! reset_cinematic_mode7_transform {
    (
        $game:ident, $x_register:literal, $x_mirror:literal, $y_register:literal,
        $y_mirror:literal $(,)?
    ) => {
        WriteRegWord(M7A as u16, 0x100, $game);
        word!($game.reg_M7A = 256);
        WriteRegWord(M7B as u16, 0, $game);
        word!($game.reg_M7B = 0);
        WriteRegWord(M7C as u16, 0, $game);
        word!($game.reg_M7C = 0);
        WriteRegWord(M7D as u16, 0x100, $game);
        word!($game.reg_M7D = 256);
        WriteRegWord(M7X as u16, $x_register, $game);
        word!($game.reg_M7X = $x_mirror);
        WriteRegWord(M7Y as u16, $y_register, $game);
        word!($game.reg_M7Y = $y_mirror);
    };
}

macro_rules! fade_cinematic_fixed_color {
    ($red_reference:ident, $game:ident, $green_reference:ident, $blue_reference:ident) => {
        crate::update_place! {
            reference $red_reference = *ram_ptr!($game; 0x74, 0)
                => (*$red_reference).wrapping_sub(1);
        }
        if *$red_reference as i32 - 32i32 & 0x80 != 0 {
            *ram_ptr!($game; 0x74, 0) = 32;
        }
        crate::update_place! {
            reference $green_reference = *ram_ptr!($game; 0x74, 1)
                => (*$green_reference).wrapping_sub(1);
        }
        if *$green_reference as i32 - 64i32 & 0x80 != 0 {
            *ram_ptr!($game; 0x74, 1) = 64;
        }
        crate::update_place! {
            reference $blue_reference = *ram_ptr!($game; 0x74, 2)
                => (*$blue_reference).wrapping_sub(1);
        }
        if *$blue_reference as i32 + 0x80 & 0x80 != 0 {
            *ram_ptr!($game; 0x74, 2) = 0x80;
        }
    };
}

static kCinematicFunction_Intro_Func78_Tab0: [u16; 5] = [1, 16, 32, 48, 64];
static kCinematicFunction_Intro_Func78_Tab1: [i16; 5] = [16, -16, 16, -16, 0];
static kCinematicFunction_Intro_Func78_Tab2: [i16; 5] = [-16, 16, 16, -16, 0];
static g_word_8BC4EB: [i16; 16] = [
    14, -8, 8, 12, -16, 12, -8, -14, 0, 0, 16, 14, -12, 4, -8, -16,
];
static kCinematicFunction_Intro_Func81_Tab0: [i16; 4] = [1, 4, 8, 16];
static kCinematicFunction_Intro_Func81_Tab1: [i16; 4] = [8, 12, -8, -12];
static kCinematicFunction_Intro_Func81_Tab2: [i16; 4] = [-4, 8, -10, 12];
pub unsafe fn GameState_39_EndingAndCredits_(game: &mut Game) -> CoroutineRet {
    CallCinematicFunction(
        ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
        game,
    );
    if (word!(game.cinematic_var11) & 0x8000) == 0 {
        word!(game.cinematic_var11 += 1);
    }
    word!(game.cinematic_var12 += 1);
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
    if word!(game.cinematic_function) as i32 + 0x1afc & 0x8000 == 0 {
        HandleCinematicBgObjects(game);
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
static kCinematicFunction_Intro_Func121_Tab0: [u16; 32] = [
    0, 0x8000, 0, 0x8000, 0, 0x8000, 0, 0x8000, 0xffff, 0x8000, 0xffff, 0x8000, 0, 0x8000, 0,
    0x8000, 0, 0x8000, 0, 0x8000, 0xffff, 0x8000, 0xffff, 0x8000, 0, 0x8000, 0, 0x8000, 0xffff,
    0x8000, 0xffff, 0x8000,
];
static kCinematicFunction_Intro_Func123_Tab0: [u16; 16] = [
    1, 0, 1, 0, 1, 0, 0xffff, 0, 0xffff, 0, 1, 0, 1, 0, 0xffff, 0,
];
static g_word_8BDE43: [u16; 16] = [
    0, 0x6f7b, 0x4a52, 0x1ce7, 0, 0x5ad6, 0x4a52, 0x35ad, 0x2529, 0x7fff, 0x56b5, 0x294a, 0x4210,
    0x2d6b, 0x18c6, 0x7fff,
];
pub unsafe fn CinematicFunction_Intro_Func140(game: &mut Game) {
    if word!(game.cinematic_var17) != 0 {
        word!(game.cinematic_var17 -= 1);
        FadeOutYColorsFromIndexX(0x1e0, 0x10, game);
        ComposeFadingPalettes(game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func142(game: &mut Game) {
    if word!(game.cinematic_var17) == 0 && word!(game.cinematic_var14) as i32 - 6i32 & 0x8000 != 0 {
        let v0: u16 = (4u16).wrapping_mul(word!(game.cinematic_var14));
        let mut v1: u16 = word!(game.vram_write_queue_tail);
        let v2: i32 = 8 * word!(game.cinematic_var14) as i32 >> 1;
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, v2 as isize);
        v1 = (v1).wrapping_add(2);
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v2 + 1) as isize);
        v1 = (v1).wrapping_add(2);
        crate::update_place! { previous fresh75 = v1 => v1.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh75 as i32,
            game,
        ))
        .size as *mut u8) =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v0 as i32 + 2) as isize) as u8;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8be45a, (v2 + 3) as isize);
        word!(game.vram_write_queue_tail = (v1).wrapping_add(2));
        word!(game.cinematic_var14 += 1);
    }
}
pub unsafe fn CinematicFunction_Intro_Func127(game: &mut Game) {
    let mut v0: u16 =
        ram_ptr!(game; 0xe0c).offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
    let mut v1: i16 = 0;
    do_until! { ((v4).wrapping_sub(39) >= 0);
        let mut v4: i16 = v1;
        let v2 = RomPtr_RAM(v0, game);
        *(v2 as *mut u16) = v1 as u16;
        *(v2 as *mut u16).offset(6) = 0;
        *(v2 as *mut u16).offset(7) = 0;
        let v3: u16 =
            crate::rtl::rom_table_word!(game, 0x8be9cf, ((8 * v1 as i32 >> 1) + 3) as isize);
        if v3 != 0 {
            *(v2 as *mut u16).offset(5) = v3;
            crate::update_place! { unaligned_word fresh93 = &raw mut (*(v2 as *mut u16)) => (u16::from_le(fresh93.read_unaligned())) | (0x8000u16); }
        } else {
            *(v2 as *mut u16).offset(5) = 32;
        }
        *(v2 as *mut u16).offset(1) = 128;
        *(v2 as *mut u16).offset(3) = 128;
        *(v2 as *mut u16).offset(2) = 0;
        *(v2 as *mut u16).offset(4) = 0;
        v0 = (v0).wrapping_add(16);
        v1 += 1;
    }
}
pub unsafe fn CinematicFunction_Intro_Func149(game: &mut Game) {
    let mut v5: i16 = 0;
    let mut v18 = null_mut::<OamEnt>();
    let mut v19: i16 = 0;
    if word!(game.eproj_pre_instr[1]) != 0 {
        let mut v0: u16 = 0xe0c;
        for _ in 0..40 {
            let v3 = RomPtr_RAM(v0, game);
            if *(v3 as *mut i16) < 0 {
                let v1 = v3;
                let mut v2: i16 = ((*(v1.offset(10) as *mut u16)) as i16).wrapping_sub(1);
                *(v1 as *mut u16).offset(5) = v2 as u16;
                if v2 < 0 {
                    *(v1 as *mut u16).offset(5) = 32;
                    *(v1 as *mut u16) = *v1 as u16;
                }
            } else {
                let v4: i32 = 8 * *(v3 as *mut u16) as u8 as i32 >> 1;
                if (*(v3 as *mut u16) as i32 & 0xff00) - 1024i32 & 0x8000 != 0 {
                    crate::update_place! { unaligned_word fresh164 = &raw mut (*(v3 as *mut u16).offset(6)) => (u16::from_le(fresh164.read_unaligned())).wrapping_add(crate::rtl::rom_table_word!(
                        game,
                        0x8be9cf,
                        v4 as isize
                    )); }
                    v5 = *(v3.offset(14) as *mut u16) as i16;
                } else {
                    crate::update_place! { unaligned_word fresh165 = &raw mut (*(v3 as *mut u16).offset(6)) => (u16::from_le(fresh165.read_unaligned()) as i32
                    + (crate::rtl::rom_table_word!(game, 0x8be9cf, v4 as isize) as i32
                        + crate::rtl::rom_table_word!(game, 0x8be9cf, v4 as isize) as i32))
                    as u16; }
                    v5 = ((crate::rtl::rom_table_word!(game, 0x8be9cf, (v4 + 1) as isize)) as i16)
                        .wrapping_add((*(v3 as *mut u16).offset(7)) as i16);
                }
                *(v3 as *mut u16).offset(7) =
                    (crate::rtl::rom_table_word!(game, 0x8be9cf, (v4 + 1) as isize))
                        .wrapping_add((v5) as u16);
                let mut v23: u16 = *(v3.offset(12) as *mut u16);
                let mut r18: u16 = byte!(v23; 1) as i8 as u16;
                let mut r20: u16 = ((v23 as i32) << 8 & 0xff00) as u16;
                let mut v8: i16 = *(v3.offset(4) as *mut u16) as i16;
                let mut v9: bool = u16::overflowing_add(r20, v8 as u16).1;
                *(v3 as *mut u16).offset(2) = (r20).wrapping_add((v8) as u16);
                crate::update_place! { unaligned_word fresh166 = &raw mut (*(v3 as *mut u16).offset(1)) => (u16::from_le(fresh166.read_unaligned()) as i32 + (r18 as i32 + v9 as i32)) as u16; }
                let mut v24: u16 = *(v3.offset(14) as *mut u16);
                r18 = byte!(v24; 1) as i8 as u16;
                r20 = ((v24 as i32) << 8 & 0xff00) as u16;
                let mut v12: i16 = *(v3.offset(8) as *mut u16) as i16;
                v9 = u16::overflowing_add(r20, v12 as u16).1;
                *(v3 as *mut u16).offset(4) = (r20).wrapping_add((v12) as u16);
                crate::update_place! { unaligned_word fresh167 = &raw mut (*(v3 as *mut u16).offset(3)) => (u16::from_le(fresh167.read_unaligned()) as i32 + (r18 as i32 + v9 as i32)) as u16; }
            }
            v0 = (v0).wrapping_add(16);
        }
        let mut v13: u16 = word!(game.oam_next_ptr);
        let mut v14: u16 = 3596;
        for _ in 0..40 {
            let v15 = RomPtr_RAM(v14, game);
            let v16 = v15;
            if *(v15 as *mut i16) >= 0 {
                let mut v17: i16 = *(v15.offset(2) as *mut u16) as i16;
                if *v15.offset(3) != 0
                    || (v17 as i32 - 4i32 & 0xff00) != 0
                    || {
                        v18 = gOamEnt(v13 as i32, game);
                        *(&raw mut (*v18).xcoord as *mut u16) = ((v17) as u16).wrapping_sub(4);
                        v19 = *(v16.offset(6) as *mut u16) as i16;
                        *v16.offset(7) != 0
                    }
                    || (v19 as i32 - 4i32 & 0xff00) != 0
                {
                    let v21 = v15;
                    *(v21 as *mut u16).offset(5) = 32;
                    *(v21 as *mut u16).offset(1) = 128;
                    *(v21 as *mut u16).offset(3) = 128;
                    *(v21 as *mut u16).offset(2) = 0;
                    *(v21 as *mut u16).offset(4) = 0;
                    *(v21 as *mut u16).offset(6) = 0;
                    *(v21 as *mut u16).offset(7) = 0;
                    *(v21 as *mut u16) = *v21 as u16;
                } else {
                    *(&raw mut (*v18).ycoord as *mut u16) = ((v19) as u16).wrapping_sub(4);
                    let mut v20: i16 = ((*(v16.offset(10) as *mut u16)) as i16).wrapping_sub(1);
                    *(v16 as *mut u16).offset(5) = v20 as u16;
                    if v20 <= 0 {
                        let mut v22: i16 = *(v16 as *mut u16) as i16;
                        *(v16 as *mut u16).offset(5) = crate::rtl::rom_table_word!(
                            game,
                            0x8be9cf,
                            ((8 * *(v16 as *mut u16) as u8 as i32 >> 1) + 2) as isize,
                        );
                        *(v16 as *mut u16) = ((v22) as u16).wrapping_add(512);
                    }
                    if *v16.offset(1) != 0 {
                        *(&raw mut (*(gOamEnt as unsafe fn(i32, &mut Game) -> *mut OamEnt)(
                            v13 as i32, game,
                        ))
                        .charnum as *mut u16) = crate::rtl::rom_table_word!(
                            game,
                            0x8be9a7,
                            (((*(v16 as *mut u16) as i32 & 0xff00) >> 8) as u16 >> 1) as isize,
                        );
                        v13 = (v13).wrapping_add(4);
                    }
                }
            }
            v14 = (v14).wrapping_add(16);
        }
        word!(game.oam_next_ptr = v13);
    }
}
pub unsafe fn CinematicFunction_Intro_Func160(j: u16, a: u16, game: &mut Game) {
    word!(
        checked_i32,
        game.cinematicspr_instr_ptr[(j >> 1)] += 8 * a as i32
    );
}
pub unsafe fn CinematicFunction_Intro_Func210_EndingSamusFall(k: u16, game: &mut Game) {
    words!(game.samus_y_speed, samus_y_subspeed += 0x3800);
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
        words!(game.samus_y_speed, samus_y_subspeed),
    );
}
pub unsafe fn CinematicFunction_Intro_Func216(game: &mut Game) {
    if word!(game.cinematic_var14) as i32 - 16i32 & 0x8000 != 0 {
        let v0: u16 = (2u16).wrapping_mul(word!(game.cinematic_var14));
        let mut v1: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 2048;
        v1 = (v1).wrapping_add(2);
        let v2: i32 = v0 as i32 >> 1;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8bf6b8, v2 as isize);
        v1 = (v1).wrapping_add(2);
        crate::update_place! { previous fresh28 = v1 => v1.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh28 as i32,
            game,
        ))
        .size as *mut u8) = 127;
        (*gVramWriteEntry(v1 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x8bf6d8, v2 as isize);
        word!(game.vram_write_queue_tail = (v1).wrapping_add(2));
        word!(game.cinematic_var14 += 1);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicFunction(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9149288] CinematicFunctionOpening [] {
        LoadTitleSequenceGraphics(game);
        QueueMusic_Delayed8(0xff03, game);
        word!(game.cinematic_function = ((0x8b9a47i32) as u16));
        SpawnCinematicSpriteObject(
            addr_kCinematicSpriteObjectDef_8BA0EF as u16,
            ((0x8b9a47i32) as u16),
            game,
        );
        QueueMusic_Delayed8(5, game);
    };
    [9149719] CinematicFunc_Func2 [] {
        advance_cinematic_pan_to_blackout!(game, 7, addr_kCinematicSpriteObjectDef_8BA0F5);
        MaybeSkipCinematics(game);
    };
    [9149840] CinematicFunc_Func4 [] {
        advance_cinematic_pan_to_blackout!(game, 176, addr_kCinematicSpriteObjectDef_8BA0FB);
        MaybeSkipCinematics(game);
    };
    [9149970] CinematicFunc_Func5 [] {
        words!(
            game.cinematic_var10,
            cinematic_var9 += words!(game.g_word_7E199F, g_word_7E199D)
        );
        if word!(game.cinematic_var10) as i32 - 163i32 & 0x8000 == 0 {
            *ram_ptr!(game; 0x69) = 16;
            word!(game.cinematic_function = ((0x8b9a47i32) as u16));
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BA101 as u16,
                ((0x8b9a47i32) as u16),
                game,
            );
            SetPpuBackdropBlack(game);
        }
        MaybeSkipCinematics(game);
    };
    [9150091] CinematicFunc_Func7 [] {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            if word!(game.cinematic_var6) as i32 - 256i32 & 0x8000 != 0 {
                word!(game.cinematic_var6 += 1);
            } else {
                word! {game; cinematic_var6 = 256; cinematic_function = ((0x8b9f28i32) as u16); }
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BA107 as u16,
                    ((0x8b9f28i32) as u16),
                    game,
                );
            }
        }
    };
    [9150249] CinematicFunc_Func1 [] {
        let v0: bool = (word!(game.demo_timer -= 1, reread) & 0x8000) != 0;
        let begin_fade: bool = if word!(game.demo_timer) == 0 || v0 {
            word!(game.cinematic_function = ((0x8b9faei32) as u16));
            true
        } else {
            ((word!(game.joypad1_newkeys)
                & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16)
                != 0)
                && ({
                    word!(game.cinematic_function = ((0x8b9f52i32) as u16));
                    true
                })
        };
        if begin_fade {
            word! { game, 2; screen_fade_delay, screen_fade_counter }
        };
    };
    [9150290] CinematicFunc_Func10 [] {
        if AdvanceSlowScreenFadeOut(game) & 1 != 0 {
            finish_cinematic_and_clear_effects!(game, 4, game_options_screen_index);
        }
    };
    [9150382] CinematicFunc_Func9 [] {
        if AdvanceSlowScreenFadeOut(game) & 1 != 0 {
            finish_cinematic_and_clear_effects!(game, 40, demo_scene);
        }
    };
    [9151381] CinematicFunction_Intro_Initial [] {
        SetupPpu_Intro(game);
        word! { game, 0; cur_irq_handler, irqhandler_next_handler, area_index }
        word! { game, 16; room_width_in_blocks, room_height_in_blocks }
        Samus_Initialize(game);
        UpdateBeamTilesAndPalette(game);
        word! { game, 900; samus_max_missiles, samus_missiles }
        word! {game; cinematic_var15 = 0; samus_draw_handler = ((0x90eb52i32) as u16); }
        ResetButtonAssignmentsToDefault(game);
        word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce3e9, 0));
        GraphicsAsset::Decompress(0x95f90e, 0x10000).load(game);
        GraphicsAsset::Decompress(0x95d089, 0x18000).load(game);
        GraphicsAsset::Decompress(0x9788cc, 0x19000).load(game);
        GraphicsAsset::Decompress(0x96ff14, 0x19800).load(game);
        GraphicsAsset::Decompress(0x95e4c2, 0x1b800).load(game);
        GraphicsAsset::Decompress(0x978d12, 0x1e000).load(game);
        GraphicsAsset::VramWords(0x7f0000, 0x0000, 0x8000).load(game);
        GraphicsAsset::VramWords(0x7f8000, 0x4000, 0x900).load(game);
        GraphicsAsset::VramWords(0x7f9000, 0x4800, 0x800).load(game);
        GraphicsAsset::VramWords(0x7fe000, 0x4c00, 0x800).load(game);
        GraphicsAsset::VramWords(0x7f9800, 0x5000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x7fb800, 0x6e00, 0x2400).load(game);
        GraphicsAsset::Decompress(0x95d713, 0x1a000).load(game);
        EnableNMI(game);
        word! { game, 3; screen_fade_delay, screen_fade_counter }
        EnableHdmaObjects(game);
        ClearPaletteFXObjects(game);
        EnablePaletteFx(game);
        word! { game, 0; door_def_ptr, layer1_x_pos, layer1_y_pos, cinematic_var16 }
        word_transfers!(game, descending, 1024; cinematic_bg(1023) <- native(0x197fe));
        word_transfers!(game, descending, 128; pause_map(127) <- rom(0x8cd81b, 127));
        word! {game; cinematic_var11 = -1i32 as u16; cinematic_function = ((0x8ba5a7i32) as u16); }
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff3f, game);
    };
    [9151911] CinematicFunction_Intro_FadeIn [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word!(game.cinematic_function = ((0x8ba5bdi32) as u16));
            word! { game, 2; screen_fade_delay, screen_fade_counter }
        }
    };
    [9151933] CinematicFunction_Intro_DrawInitJpn [] {
        if AdvanceSlowScreenFadeIn(game) & 1 != 0 {
            word! {game; cinematic_function = ((0x8ba5f8i32) as u16); cinematic_var4 = 60; }
            if word!(game.japanese_text_flag) != 0 {
                let mut v0: u16 = word!(game.vram_write_queue_tail);
                (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 256;
                v0 = (v0).wrapping_add(2);
                (*gVramWriteEntry(v0 as i32, game)).size = 12288;
                v0 = (v0).wrapping_add(2);
                crate::update_place! { previous fresh146 = v0 => v0.wrapping_add(1); }
                *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                    fresh146 as i32,
                    game,
                ))
                .size as *mut u8) = 126;
                (*gVramWriteEntry(v0 as i32, game)).size = 20192;
                word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
            }
        }
    };
    [9152057] CinematicFunc_Intro_WaitForQueueWait4secs [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word! {game; cinematic_function = ((0x8ba64ci32) as u16); cinematic_var4 = 240; }
        }
    };
    [9152099] CinematicFunc_Intro_WaitForFadeOut [] {
        if AdvanceSlowScreenFadeOut(game) & 1 != 0 {
            word!(game.cinematic_function = ((0x8ba66fi32) as u16));
        }
    };
    [9152111] CinematicFunc_Intro_SetupTextPage2 [] {
        ClearJapaneseTextTiles(game);
        GraphicsAsset::VramWords(0x7e4000, 0x4180, 0x600).load(game);
        crate::rtl::write_register_and_mirror(TM as u16, 0x16, 0x69, false, game);
        ClearCinematicBgObjects(0x2f, game);
        EnableCinematicBgObjects(game);
        EnableCinematicBgTilemapUpdates(game);
        word!(game.cinematicbg_var3 = addr_unk_604C00 as u16);
        word_transfers!(game, descending, 128;
            pause_map(127) <- value(15401),
            pause_map(1023) <- value(15401),
        );
        word_transfers!(game, ascending, 128; pause_map(768) <- rom(0x8ba72b, 0));
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8).offset(286)
            as *mut u32) = 472456233;
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE6D as u16, 0x1c29, game);
        word!(game.cinematic_var4 = 127);
        SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF63 as u16, 0x4c00, game);
        word! { game, 8; cinematic_var10, reg_BG2VOFS, reg_BG3VOFS }
        ClearEnglishTextTilemap(game);
        SetSomeStuffForSpriteObject_16(game);
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff36, game);
        QueueMusic_DelayedY(5, 0xe, game);
        word!(game.cinematic_function = ((0x8ba82bi32) as u16));
    };
    [9152555] CinematicFunc_Intro_WaitFadeinShowText [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word!(game.cinematic_function = ((0x8ba84ai32) as u16));
            word! { game, 2; screen_fade_delay, screen_fade_counter }
            SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF3F as u16, 0x4c00, game);
        }
    };
    [9152586] CinematicFunc_Intro_WaitForFadeinSleep [] {
        if AdvanceSlowScreenFadeIn(game) & 1 != 0 {
            word!(game.cinematic_function = ((0x8ba391i32) as u16));
        }
    };
    [9157792] CinematicFunction_Intro_Func54 [] {
        SetupPpu_3_Mode7(game);
        ClearCinematicSprites(game);
        word! { game, 0; cur_irq_handler, irqhandler_next_handler }
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce5e9, 0));
        GraphicsAsset::Decompress(0x95a82f, 0x10000).load(game);
        GraphicsAsset::Decompress(0x96fe69, 0x14000).load(game);
        GraphicsAsset::Decompress(0x96d10a, 0x15000).load(game);
        GraphicsAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
        crate::rtl::register_writes! { game;
            byte(VMAIN as u16, 0); word(VMADDL as u16, 0);
        }
        for _ in 0i16..0x4000i16 {
            WriteReg(VMDATAL as u16, 0x8c, game);
        }
        GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x300).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        reset_cinematic_mode7_transform!(game, 0x38, 56, 0x18, 24);
        word! {game; cinematic_var8 = -72i32 as u16; cinematic_var10 = -104i32 as u16;
            cinematic_var5 = 224; cinematic_var6 = 512; }
        ClearPaletteFXObjects(game);
        EnablePaletteFx(game);
        ClearCinematicBgObjects(0, game);
        word!(game.cinematicbg_var3 = 23552);
        EnableNMI(game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 0, game);
        word!(game.cinematic_function = ((0x8bbde4i32) as u16));
        QueueMusic_Delayed8(0xff2d, game);
        QueueMusic_DelayedY(5, 0xe, game);
    };
    [9158116] CinematicFunction_Intro_Func55 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            *ram_ptr!(game; 0x51) = 15;
            word!(game.cinematic_function = ((0x8bbdf9i32) as u16));
        }
    };
    [9158137] CinematicFunction_Intro_Func56 [] {
        if word!(game.cinematic_var6) as i32 - 32i32 & 0x8000 != 0 {
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 49;
            *ram_ptr!(game; 0x74, 0) = 63;
            *ram_ptr!(game; 0x74, 1) = 95;
            *ram_ptr!(game; 0x74, 2) = -97i32 as u8;
            ClearCinematicSprites(game);
            QueueMode7Transfers(0x8b, addr_kCinematicFunction_Intro_Func56_M7 as u16, game);
            word! {game; cinematic_var8 = -32i32 as u16; cinematic_var10 = -128i32 as u16;
                cinematic_var5 = 32; }
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
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE91 as u16, 1, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 1, game);
            word!(game.cinematic_function = ((0x8bbfdai32) as u16));
            SpawnPalfxObject(addr_kPalfx_E1A8 as u16, game);
            SpawnPalfxObject(addr_kPalfx_E1AC as u16, game);
        } else {
            word!(game.cinematic_var6 -= 16);
        };
    };
    [9158618] CinematicFunction_Intro_Func67 [] {
        fade_cinematic_fixed_color!(fresh123, game, fresh124, fresh125);
        words!(game.cinematic_var8, cinematic_var7 += -0x2000i32 as u32);
        if word!(game.cinematic_var6) as i32 - 3072i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 16);
        } else if word!(game.cinematic_var6) as i32 - 0x2000i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 32);
        } else {
            *ram_ptr!(game; 0x69) = 17;
            *ram_ptr!(game; 0x55) = 1;
            *ram_ptr!(game; 0x5f) = 0;
            *ram_ptr!(game; 0x58) = 92;
            *ram_ptr!(game; 0x5d) = 6;
            word!(game.cinematic_var6 = 256);
            word! { game, 0;
                cinematic_var5, cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10,
            }
            EnableCinematicBgObjects(game);
            EnableCinematicBgTilemapUpdates(game);
            SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF69 as u16, 0x5c00, game);
            word!(game.cinematic_function = ((0x8ba38fi32) as u16));
        };
    };
    [9158853] CinematicFunction_Intro_Func72 [] {
        HandleFadeOut(game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            EnableNMI(game);
            word! { game, 0;
                screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
                reg_BG3VOFS, cinematic_var5,
            }
            word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
            word!(game.cinematic_function = ((0x8bc100i32) as u16));
        }
    };
    [9158912] CinematicFunction_Intro_Func73 [] {
        word! {game; loading_game_state = kGameState_31_SetUpNewGame as u16;
            game_state = kGameState_31_SetUpNewGame as u16; area_index = 6;
            load_station_index = 0; }
        SaveToSram(word!(game.selected_save_slot), game);
    };
    [9158939] CinematicFunctionBlackoutFromCeres [] {
        SetupPpu_3_Mode7(game);
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word! { game, 0; door_def_ptr, layer1_x_pos, layer1_y_pos }
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce5e9, 0));
        GraphicsAsset::Decompress(0x95a82f, 0x10000).load(game);
        GraphicsAsset::Decompress(0x96fe69, 0x14000).load(game);
        GraphicsAsset::Decompress(0x96d10a, 0x15000).load(game);
        GraphicsAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
        crate::rtl::register_writes! { game;
            byte(VMAIN as u16, 0); byte(VMADDL as u16, 0);
            byte(VMADDH as u16, 0);
        }
        for _ in 0i16..0x4000i16 {
            WriteReg(VMDATAL as u16, 0x8c, game);
        }
        GraphicsAsset::VramLowBytes(0x7f4600, 0x0000, 0x600).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1a00).load(game);
        reset_cinematic_mode7_transform!(game, 0x34, 52, 0x30, 48);
        word! {game; cinematic_var8 = -44i32 as u16; cinematic_var10 = -112i32 as u16;
            cinematic_var5 = 0; cinematic_var6 = 256; }
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        ClearPaletteFXObjects(game);
        EnablePaletteFx(game);
        ClearCinematicBgObjects(0x2f, game);
        word!(game.cinematicbg_var3 = 0);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCE7F as u16, 0, game);
        SpawnCinematicSpriteObjectAtSlot(
            addr_kCinematicSpriteObjectDef_8BCE8B as u16,
            2,
            0x0002,
            game,
        );
        SpawnCinematicSpriteObjectAtSlot(
            addr_kCinematicSpriteObjectDef_8BCE91 as u16,
            0,
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
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = ((0x8bc2e4i32) as u16));
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff2d, game);
        if word!(game.game_state) as i32 == kGameState_37_CeresGoesBoomWithSamus as i32 {
            QueueMusic_DelayedY(8, 0xe, game);
        } else {
            QueueMusic_DelayedY(7, 0xe, game);
        };
    };
    [9159396] CinematicFunction_Intro_Func74 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word!(game.cinematic_function = ((0x8bc2f1i32) as u16));
        }
    };
    [9159409] CinematicFunction_Intro_Func75 [] {
        words!(game.cinematic_var10, cinematic_var9 += 4096);
        words!(game.cinematic_var8, cinematic_var7 += -0x4000i32 as u32);
        word!(game.cinematic_var6 += 1);
        HandleFadeIn(game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word! {game; cinematic_function = ((0x8bc345i32) as u16); cinematic_var4 = 1;
                cinematic_var13 = 0; }
        }
    };
    [9159493] CinematicFunction_Intro_Func76 [] {
        words!(game.cinematic_var10, cinematic_var9 += 4096);
        words!(game.cinematic_var8, cinematic_var7 += -0x4000i32 as u32);
        if word!(game.cinematic_var6) as i32 - 640i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 1);
        } else {
            word! { game;
                power_bomb_explosion_x_pos =
                    (word!(game.reg_M7X)).wrapping_sub(word!(game.cinematic_var8));
                power_bomb_explosion_y_pos =
                    (word!(game.reg_M7Y)).wrapping_sub(word!(game.cinematic_var10));
            }
            EnableHdmaObjects(game);
            spawn_power_bomb(game);
            *ram_ptr!(game; 0x6f) = 16;
            *ram_ptr!(game; 0x72) = 55;
            word!(game.cinematic_var6 = 768);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF2D as u16, 0, game);
            if word!(game.game_state) == 37 {
                QueueMode7Transfers(0x8b, addr_kCinematicFunction_Intro_Func76_M7_2 as u16, game);
            } else {
                QueueMode7Transfers(0x8b, addr_kCinematicFunction_Intro_Func76_M7_0 as u16, game);
            }
            QueueMode7Transfers(0x8b, addr_kCinematicFunction_Intro_Func76_M7_1 as u16, game);
            word!(game.cinematic_function = ((0x8bc5cai32) as u16));
        };
    };
    [9160138] CinematicFunction_Intro_Func77 [] {
        *ram_ptr!(game; 0x6f) = 16;
        *ram_ptr!(game; 0x72) = 55;
        word!(game.cinematic_var8 += 2);
        word!(game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(1)) as u16);
        if word!(game.cinematic_var6) as i32 - 16i32 & 0x8000 != 0 {
            word! {game; cinematic_function = ((0x8bc610i32) as u16); cinematic_var4 = 192; }
        } else {
            word!(game.cinematic_var6 -= 16);
        };
    };
    [9160231] CinematicFunction_Intro_Func85 [] {
        HandleFadeOut(game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            EnableNMI(game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            if word!(game.game_state) as i32 == kGameState_37_CeresGoesBoomWithSamus as i32 {
                QueueMusic_Delayed8(0, game);
                word! { game, 0;
                    hud_item_index, samus_auto_cancel_hud_item_index, samus_invincibility_timer,
                    samus_knockback_timer,
                }
                WaitUntilEndOfVblankAndClearHdma(game);
                DisableIrqInterrupts(game);
                word!(game.fx_layer_blending_config_a = 0);
                *ram_ptr!(game; 0x6e) = 0;
                *ram_ptr!(game; 0x71) = 0;
                *ram_ptr!(game; 0x69) = 16;
                *ram_ptr!(game; 0x6b) = 0;
                *ram_ptr!(game; 0x6c) = 0;
                *ram_ptr!(game; 0x6d) = 0;
                *ram_ptr!(game; 0x55) = 9;
                word! { game, 0; game_options_screen_index, menu_index }
                word_transfers!(game, descending, 128; le(cinematicbg_arr7, 135) <- value(0));
                word!(game.game_state = kGameState_25_SamusNoHealth as u16);
            } else {
                word!(game.cinematic_function = ((0x8bc699i32) as u16));
            }
        }
    };
    [9160345] CinematicFunction_Intro_Func86 [] {
        SetupPpu_4_Mode1(game);
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        GraphicsAsset::Decompress(0x978adb, 0x19000).load(game);
        GraphicsAsset::Decompress(0x96ec76, 0x15000).load(game);
        GraphicsAsset::VramLowBytes(0x7f4300, 0x0000, 0x300).load(game);
        GraphicsAsset::VramWords(0x7f9000, 0x5c00, 0x800).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        reset_cinematic_mode7_transform!(game, 0x38, 56, 0x18, 24);
        word! { game, 0; cinematic_var8, cinematic_var10, cinematic_var5 }
        word!(game.cinematic_var6 = 256);
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        ClearPaletteFXObjects(game);
        EnablePaletteFx(game);
        *ram_ptr!(game; 0x57) = -127i32 as u8;
        SpawnPalfxObject(addr_kPalfx_E1A8 as u16, game);
        word!(game.cinematic_function = ((0x8bc79ci32) as u16));
        EnableNMI(game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
    };
    [9160604] CinematicFunction_Intro_Func87 [] {
        if (word!(game.cinematic_var12) & 3) == 0 {
            crate::update_place! { reference fresh116 = *ram_ptr!(game; 0x57) => (*fresh116).wrapping_sub(16); }
        }
        HandleFadeIn(game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = ((0x8bc7cai32) as u16));
        }
    };
    [9160650] CinematicFunction_Intro_Func88 [] {
        if (word!(game.cinematic_var12) & 3) == 0 {
            crate::update_place! { reference fresh115 = *ram_ptr!(game; 0x57) => (*fresh115).wrapping_sub(16); }
            if (*ram_ptr!(game; 0x57) & 0xf0) == 0 {
                *ram_ptr!(game; 0x55) = 7;
                *ram_ptr!(game; 0x5f) = 0x80;
                *ram_ptr!(game; 0x58) = 0;
                *ram_ptr!(game; 0x5d) = 0;
                *ram_ptr!(game; 0x69) = 17;
                word! {game; cinematic_var8 = 128; cinematic_var10 = -104i32 as u16;
                    cinematic_var5 = 32; cinematic_var6 = 256; }
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
                word!(game.cinematic_function = ((0x8ba38fi32) as u16));
            }
        }
    };
    [9161209] CinematicFunction_Intro_Func105 [] {
        words!(game.cinematic_var10, cinematic_var9 += 0x2000);
        words!(game.cinematic_var8, cinematic_var7 += -0x8000i32 as u32);
        if word!(game.cinematic_var6) as i32 - 1152i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 4);
        } else {
            word!(game.cinematic_function = ((0x8bca36i32) as u16));
        };
    };
    [9161270] CinematicFunction_Intro_Func106 [] {
        words!(game.cinematic_var10, cinematic_var9 += 0x2000);
        words!(game.cinematic_var8, cinematic_var7 += -0x8000i32 as u32);
        if word!(game.cinematic_var8) as i32 + 128 & 0x8000 != 0 {
            word!(game.cinematic_function = ((0x8bca85i32) as u16));
        } else {
            word!(game.cinematic_var6 += 16);
            if word!(game.cinematic_var5) != 224 {
                word!(
                    game.cinematic_var5 =
                        (((word!(game.cinematic_var5)) as u8).wrapping_sub(1)) as u16
                );
            }
        };
    };
    [9161349] CinematicFunction_Intro_Func107 [] {
        words!(game.cinematic_var10, cinematic_var9 += 0x2000);
        words!(game.cinematic_var8, cinematic_var7 += 0x2000);
        if word!(game.cinematic_var6) as i32 - 0x2000i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 32);
        } else {
            *ram_ptr!(game; 0x69) = 16;
            word! {game; cinematic_function = ((0x8bcad0i32) as u16); cinematic_var4 = 64; }
        };
    };
    [9161439] CinematicFunction_Intro_Func95 [] {
        *ram_ptr!(game; 0x51) = 0x80;
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word!(game.game_state = kGameState_6_LoadingGameData as u16);
        word!(game.samus_health = word!(game.samus_max_health));
    };
    [9163904] CinematicFunctionEscapeFromCebes [] {
        word! { game, 0; cur_irq_handler, irqhandler_next_handler }
        SetupPpu_5_Mode7(game);
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8cede9, 0));
        GraphicsAsset::Decompress(0x98bcd6, 0x10000).load(game);
        GraphicsAsset::Decompress(0x99a56f, 0x14000).load(game);
        GraphicsAsset::Decompress(0x99d17e, 0x18000).load(game);
        GraphicsAsset::VramWords(0x7f8000, 0x0000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7f8000, 0x2000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7f4000, 0x6000, 0x4000).load(game);
        GraphicsAsset::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
        GraphicsAsset::Decompress(0x988304, 0x18000).load(game);
        GraphicsAsset::Decompress(0x95a82f, 0x10000).load(game);
        GraphicsAsset::Decompress(0x96fe69, 0x14000).load(game);
        word_transfers!(game, ascending, 7808; native(0x14300) <- value(0x8c8c));
        GraphicsAsset::Decompress(0x98b5c1, 0x1e000).load(game);
        GraphicsAsset::Decompress(0x98b857, 0x1e800).load(game);
        GraphicsAsset::Decompress(0x98baed, 0x1f000).load(game);
        GraphicsAsset::Decompress(0x98bccd, 0x1f800).load(game);
        GraphicsAsset::Decompress(0x97e7de, 0xa000).load(game);
        GraphicsAsset::Decompress(0x99d65b, 0x2000).load(game);
        GraphicsAsset::Decompress(0x99d932, 0x6000).load(game);
        reset_centered_mode7_transform!(game);
        word! {game; cinematic_var5 = 32; cinematic_var6 = 64; }
        word! { game, 0;
            cinematic_var8, cinematic_var10, door_def_ptr, layer1_x_pos, layer1_y_pos,
        }
        ClearPaletteFXObjects(game);
        EnablePaletteFx(game);
        EnableNMI(game);
        word!(game.eproj_pre_instr[1] = 0);
        *ram_ptr!(game; 0x69) = 17;
        *ram_ptr!(game; 0x6b) = 1;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 17;
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 1, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 1, game);
        word! {game; screen_fade_delay = 0; screen_fade_counter = 2;
            cinematic_function = ((0x8bd6d7i32) as u16); }
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff33, game);
        QueueMusic_DelayedY(5, 0xe, game);
    };
    [9164503] CinematicFunction_Intro_Func109 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            SpawnPalfxObject(addr_kPalfx_E1D8 as u16, game);
            SpawnPalfxObject(addr_kPalfx_E1DC as u16, game);
            word!(game.cinematic_function = ((0x8bd6f2i32) as u16));
        }
    };
    [9164530] CinematicFunction_Intro_Func110 [] {
        CinematicFunction_Intro_Func111(game);
        if AdvanceFastScreenFadeIn(game) & 1 != 0 {
            word!(game.cinematic_function = ((0x8bd701i32) as u16));
        }
    };
    [9164545] CinematicFunction_Intro_Func111 [] {
        if (word!(game.cinematic_var12) & 1) == 0 {
            word!(
                game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(1)) as u16
            );
        }
        word!(game.cinematic_var6 += 2);
        if word!(game.cinematic_var6) as i32 - 384i32 & 0x8000 == 0
            && AdvanceFastScreenFadeOut(game) & 1 != 0
        {
            word!(game.cinematic_function = ((0x8bd731i32) as u16));
        }
    };
    [9164593] CinematicFunction_Intro_Func112 [] {
        GraphicsAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7e2000, 0x2000, 0x4000).load(game);
        GraphicsAsset::Decompress(0x98ed4f, 0x2000).load(game);
        GraphicsAsset::VramHighBytes(0x7e2000, 0x0000, 0x4000).load(game);
        ClearPaletteFXObjects(game);
        ClearCinematicSprites(game);
        SpawnPalfxObject(addr_kPalfx_E1E0 as u16, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEED3 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEED9 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEDF as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEE5 as u16, 0, game);
        word! {game; cinematic_var5 = 32; cinematic_var6 = 64; }
        word! { game, 0; cinematic_var8, cinematic_var10 }
        word!(game.cinematic_function = ((0x8bd7f8i32) as u16));
    };
    [9164807] CinematicFunction_Intro_Func114 [] {
        if (word!(game.cinematic_var12) & 1) == 0 {
            word!(
                game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(1)) as u16
            );
        }
        word!(game.cinematic_var6 += 3);
        if word!(game.cinematic_var6) as i32 - 384i32 & 0x8000 == 0
            && AdvanceFastScreenFadeOut(game) & 1 != 0
        {
            word!(game.cinematic_function = ((0x8bd837i32) as u16));
        }
    };
    [9164792] CinematicFunction_Intro_Func113 [] {
        CinematicFunction_Intro_Func114(game);
        if AdvanceFastScreenFadeIn(game) & 1 != 0 {
            word!(game.cinematic_function = ((0x8bd807i32) as u16));
        }
    };
    [9164855] CinematicFunction_Intro_Func115 [] {
        GraphicsAsset::VramWords(0x7e6000, 0x0000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7e6000, 0x2000, 0x4000).load(game);
        GraphicsAsset::Decompress(0x999101, 0x6000).load(game);
        GraphicsAsset::VramHighBytes(0x7e6000, 0x0000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7f8000, 0x4000, 0x6000).load(game);
        GraphicsAsset::VramWords(0x7fe000, 0x7000, 0x800).load(game);
        GraphicsAsset::VramWords(0x7fe800, 0x7400, 0x800).load(game);
        GraphicsAsset::VramWords(0x7ff000, 0x7800, 0x800).load(game);
        GraphicsAsset::VramWords(0x7ff800, 0x7c00, 0x800).load(game);
        GraphicsAsset::VramWords(0x7ea000, 0x5000, 0x1000).load(game);
        game.g_ram[0x52] = 2;
        ClearPaletteFXObjects(game);
        ClearCinematicSprites(game);
        word_transfers!(game, ascending, 128; le(palette_buffer, 128) <- rom(0x8cebe9, 128));
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEE9D as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEAF as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEA3 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEA9 as u16, 0, game);
        word! {game; cinematic_var5 = 0; cinematic_var6 = 64; reg_M7A = 256; }
        word! { game, 0; reg_M7B, reg_M7C }
        word!(game.reg_M7D = 256);
        word! { game, 128; reg_M7X, reg_M7Y }
        word! { game, 0; cinematic_var8, cinematic_var10 }
        *ram_ptr!(game; 0x69) = 1;
        *ram_ptr!(game; 0x6b) = 16;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 33;
        CopyPalettesToFadingPalettes(game);
        DecomposePaletteDataForFading(game);
        ClearYColorsFromIndexX(0x1a0, 0x10, game);
        ClearYColorsFromIndexX(0x1e0, 0x10, game);
        ComposeFadingPalettes(game);
        word! {game; cinematic_var4 = 63; cinematic_function = ((0x8bda02i32) as u16); }
    };
    [9165314] CinematicFunction_Intro_Func116 [] {
        CinematicFunction_Intro_Func117(game);
        if AdvanceFastScreenFadeIn(game) & 1 != 0 {
            word! { game, 2; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = ((0x8bda1ai32) as u16));
        }
    };
    [9165338] CinematicFunction_Intro_Func117 [] {
        word!(game.cinematic_var6 += 4);
        if (word!(game.cinematic_var4) & 1) == 0 {
            FadeOutYColorsFromIndexX(0xe0, 0x10, game);
            FadeInYColorsFromIndexX(0x1a0, 0x10, game);
            FadeInYColorsFromIndexX(0x1e0, 0x10, game);
            ComposeFadingPalettes(game);
        }
        if (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0 {
            word_transfers!(game, ascending, 128; le(palette_buffer, 0) <- rom(0x8cebe9, 0));
            *ram_ptr!(game; 0x5f) = 0;
            *ram_ptr!(game; 0x58) = 112;
            *ram_ptr!(game; 0x59) = 120;
            *ram_ptr!(game; 0x5a) = 0;
            *ram_ptr!(game; 0x5c) = 0;
            *ram_ptr!(game; 0x5d) = 68;
            *ram_ptr!(game; 0x5e) = 0;
            *ram_ptr!(game; 0x69) = 16;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6c) = 0;
            *ram_ptr!(game; 0x6d) = 0;
            *ram_ptr!(game; 0x6e) = 0;
            *ram_ptr!(game; 0x71) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
            WriteReg(COLDATA as u16, 0xe0, game);
            *ram_ptr!(game; 0x74, 0) = 32;
            *ram_ptr!(game; 0x74, 1) = 64;
            *ram_ptr!(game; 0x74, 2) = 0x80;
            *ram_ptr!(game; 0x57) = 0;
            word! { game, 0;
                cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
                g_word_7E199B, g_word_7E199D, g_word_7E199F, fx_layer_blending_config_a, reg_BG2HOFS,
                reg_BG2VOFS, screen_fade_delay,
            }
            word! {game; screen_fade_counter = 2; cinematic_function = ((0x8bdad3i32) as u16);
                cinematic_var4 = 0; }
        }
    };
    [9165523] CinematicFunction_Intro_Func118 [] {
        let v0: u16 = word!(game.cinematic_var4);
        let v1: u16 = word!(game.mode7_vram_write_queue_tail);
        let mut v2: u32 = word!(game.mode7_vram_write_queue_tail) as u32;
        if word!(game.cinematic_var4) as i32 - 8i32 & 0x8000 != 0 {
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_0).offset(word!(
                game.mode7_vram_write_queue_tail
            )
                as i32
                as isize) as *mut u16) = 192;
            let v3: i32 = v0 as i32;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_1 as *mut u8)
                .offset(v2 as isize) as *mut u16) =
                kCinematicFunction_Intro_Func118_Tab0[v3 as usize];
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset(v1 as isize) as *mut u16) = 127;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset((v1 as i32 + 1) as isize) as *mut u16) = 2048;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset((v1 as i32 + 3) as isize) as *mut u16) =
                kCinematicFunction_Intro_Func118_Tab1[v3 as usize];
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0)).offset(1)).field_1
                as *mut u8)
                .offset(v1 as i32 as isize) as *mut u16) = 128;
        } else {
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_0).offset(word!(
                game.mode7_vram_write_queue_tail
            )
                as i32
                as isize) as *mut u16) = 128;
            let v4: i32 = v0 as i32;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).field_1 as *mut u8)
                .offset(v2 as isize) as *mut u16) =
                kCinematicFunction_Intro_Func118_Tab0[v4 as usize];
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset(v1 as isize) as *mut u16) = 127;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset((v1 as i32 + 1) as isize) as *mut u16) = 2048;
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0))).gap3 as *mut u8)
                .offset((v1 as i32 + 3) as isize) as *mut u16) =
                kCinematicFunction_Intro_Func118_Tab1[v4 as usize];
            *((&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0)).offset(1)).field_1
                as *mut u8)
                .offset(v1 as i32 as isize) as *mut u16) = 0;
        }
        word!(game.mode7_vram_write_queue_tail = (v1).wrapping_add(9));
        if word!(game.cinematic_var4 += 1, reread) as i32 - 16i32 & 0x8000 == 0 {
            *ram_ptr!(game; 0x55) = 1;
            word! {game; cinematic_var4 = 0; cinematic_function = ((0x8bdb9di32) as u16); }
        }
    };
    [9165764] CinematicFunction_Intro_Func120 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            *ram_ptr!(game; 0x55) = 7;
            *ram_ptr!(game; 0x5f) = 0x80;
            *ram_ptr!(game; 0x69) = 17;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 48;
            *ram_ptr!(game; 0x74, 0) = 63;
            *ram_ptr!(game; 0x74, 1) = 95;
            *ram_ptr!(game; 0x74, 2) = -97i32 as u8;
            *ram_ptr!(game; 0x58) = 0;
            *ram_ptr!(game; 0x5d) = 0;
            word! { game, 0; palette_buffer[0], palette_buffer[16], palette_buffer[128] }
            SpawnPalfxObject(addr_kPalfx_E1D4 as u16, game);
            SpawnPalfxObject(addr_kPalfx_E1E4 as u16, game);
            word! {game; reg_M7X = 56; reg_M7Y = 24; cinematic_var8 = -72i32 as u16;
                cinematic_var10 = -104i32 as u16; cinematic_var6 = 3072;
                cinematic_var5 = -112i32 as u16; cinematic_var4 = 1; cinematic_var13 = 192;
                cinematic_var14 = 0; cinematic_function = ((0x8bdca5i32) as u16); }
        }
    };
    [9165989] CinematicFunction_Intro_Func121 [] {
        if word!(game.cinematic_var13) != 0 {
            word!(game.cinematic_var13 -= 1);
        } else {
            CinematicFunction_Intro_Func122(game);
        }
        word!(game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(4)) as u16);
        let v0: i32 = 2 * word!(game.cinematic_var14) as i32;
        words!(
            game.cinematic_var8,
            cinematic_var7 += words!((
                kCinematicFunction_Intro_Func121_Tab0[v0 as usize],
                kCinematicFunction_Intro_Func121_Tab0[(v0 + 1) as usize]
            ))
        );
        word!(game.cinematic_var14 = (word!(game.cinematic_var14) as i32 + 1 & 0xf) as u16);
        word!(game.cinematic_var6 -= 8);
        if word!(game.cinematic_var6) as i32 - 1456i32 & 0x8000 != 0 {
            word! {game; cinematic_var13 = 1; cinematic_var14 = 0;
                cinematic_function = ((0x8bdd42i32) as u16); }
        }
    };
    [9166146] CinematicFunction_Intro_Func123 [] {
        CinematicFunction_Intro_Func122(game);
        if word!(game.cinematic_var5) != 224 {
            word!(
                game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(1)) as u16
            );
        }
        let v0: i32 = 2 * word!(game.cinematic_var14) as i32;
        words!(
            game.cinematic_var8,
            cinematic_var7 += words!((
                kCinematicFunction_Intro_Func123_Tab0[v0 as usize],
                kCinematicFunction_Intro_Func123_Tab0[(v0 + 1) as usize]
            ))
        );
        word!(game.cinematic_var14 = (word!(game.cinematic_var14) as i32 + 1 & 7) as u16);
        word!(game.cinematic_var6 -= 2);
        if word!(game.cinematic_var6) as i32 - 1184i32 & 0x8000 != 0 {
            word! {game; cinematic_function = ((0x8bddcdi32) as u16); cinematic_var14 = 0x8000;
                cinematic_var13 = 0; }
            if CheckEventHappened(0xf, game) != 0 {
                SpawnCinematicSpriteObjectToR18(0xef21, 4, game);
            }
        }
    };
    [9166285] CinematicFunction_Intro_Func124 [] {
        CinematicFunction_Intro_Func122(game);
        words!(game.cinematic_var13, cinematic_var14 += -256i32 as u32);
        words!(
            game.cinematic_var8,
            cinematic_var7 += words!(game.cinematic_var13, cinematic_var14)
        );
        if word!(game.cinematic_var6) as i32 - 384i32 & 0x8000 != 0
            && (word!(game.cinematic_var12) & 3) == 0
            && word!(game.cinematic_var5) != 16
        {
            word!(
                game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_add(2)) as u16
            );
        }
        if word!(game.cinematic_var6) as i32 - 32i32 & 0x8000 != 0 {
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BEEC7 as u16,
                word!(game.cinematic_var6),
                game,
            );
            word!(game.cinematic_function = ((0x8bde63i32) as u16));
            word_transfers!(game, descending, 16; le(palette_buffer, 95) <- table(g_word_8BDE43, 15));
        } else {
            word!(game.cinematic_var6 -= 4);
        };
    };
    [9166436] CinematicFunction_Intro_Func125 [] {
        word!(game.screen_fade_counter = 1);
        if AdvanceFastScreenFadeOut(game) & 1 != 0 {
            EnableNMI(game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = ((0x8bde80i32) as u16));
        }
    };
    [9166464] CinematicFunction_Intro_Func126 [] {
        SetupPpu_7_Mode1(game);
        ClearCinematicSprites(game);
        ClearPaletteFXObjects(game);
        word_transfers!(game, ascending, 128; le(palette_buffer, 0) <- rom(0x8ce9e9, 0));
        GraphicsAsset::Decompress(0x979803, 0x10000).load(game);
        GraphicsAsset::Decompress(0x97b957, 0x2000).load(game);
        GraphicsAsset::Decompress(0x97d7fc, 0x6000).load(game);
        GraphicsAsset::Decompress(0x97e7de, 0x1c000).load(game);
        GraphicsAsset::Decompress(0x9796f4, 0x1e000).load(game);
        GraphicsAsset::Decompress(0x97f987, 0x14000).load(game);
        GraphicsAsset::Decompress(0x99da9f, 0x1e800).load(game);
        GraphicsAsset::Decompress(0x99dab1, 0x1f000).load(game);
        GraphicsAsset::VramWords(0x7fe800, 0x2000, 0x100).load(game);
        GraphicsAsset::VramWords(0x7ff000, 0x2400, 0x800).load(game);
        GraphicsAsset::VramWords(0x7fc000, 0x4000, 0x1000).load(game);
        GraphicsAsset::VramWords(0x7e6000, 0x6000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7f0000, 0x5000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x7fe000, 0x4c00, 0x800).load(game);
        GraphicsAsset::Decompress(0x99e089, 0x6000).load(game);
        GraphicsAsset::Decompress(0x99ecc4, 0x8000).load(game);
        if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0 {
            GraphicsAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
        } else {
            GraphicsAsset::VramWords(0x7f0000, 0x0000, 0x4000).load(game);
        }
        reset_cinematic_mode7_transform!(game, 0x68, 104, 0x70, 112);
        word! {game; cinematic_var8 = -8i32 as u16; cinematic_var10 = -24i32 as u16;
            cinematic_var5 = 0; cinematic_var6 = 2304; }
        EnableNMI(game);
        *ram_ptr!(game; 0x51) = 15;
        word! { game, 0; layer1_x_pos, layer1_y_pos }
        EnableHdmaObjects(game);
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0));
        CinematicFunction_Intro_Func127(game);
        GraphicsAsset::Decompress(0x97eeff, 0x10000).load(game);
        ClearCinematicBgObjects(0x7f, game);
        CinematicFunction_Intro_Func128(0x7f, game);
        EnableTextGlowObjects_(game);
        word! { game, 3; screen_fade_delay, screen_fade_counter }
        CreditsObject_Init(addr_stru_8BF6F8 as u16, game);
        word!(game.cinematic_function = ((0x8bdb9di32) as u16));
    };
    [9167120] CinematicFunction_Intro_Func130 [] {
        HandleFadeIn(game);
        if *ram_ptr!(game; 0x51) == 15 {
            *ram_ptr!(game; 0x69) = 2;
            *ram_ptr!(game; 0x6b) = 16;
            *ram_ptr!(game; 0x6f) = 2;
            *ram_ptr!(game; 0x72) = 34;
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            CopyPalettesToFadingPalettes(game);
            DecomposePaletteDataForFading(game);
            ClearYColorsFromIndexX(0x40, 0x10, game);
            ComposeFadingPalettes(game);
            word! {game; cinematic_var4 = 32; cinematic_function = ((0x8be158i32) as u16); }
        }
    };
    [9167192] CinematicFunction_Intro_Func131 [] {
        FadeInYColorsFromIndexX(0x40, 0x10, game);
        ComposeFadingPalettes(game);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            *ram_ptr!(game; 0x69) = 18;
            *ram_ptr!(game; 0x6b) = 4;
            *ram_ptr!(game; 0x6f) = 2;
            *ram_ptr!(game; 0x72) = 2;
            word! {game; cinematic_var4 = 180; cinematic_function = ((0x8be190i32) as u16); }
            CinematicFunction_Intro_Func133(game);
        }
    };
    [9167248] CinematicFunction_Intro_Func132 [] {
        word!(game.reg_BG3VOFS += 2);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            word_transfers!(game, ascending, 288; pause_map(288) <- rom(0x8cdc9b, 0));
            CinematicUpdateSomeBg(game);
            *ram_ptr!(game; 0x69) = 1;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
            word! {game; cinematic_var4 = 180; cinematic_function = ((0x8be1d2i32) as u16);
                g_word_7E0D9C = 0; }
        }
    };
    [9167314] CinematicFunction_Intro_Func134 [] {
        if word!(game.cinematic_var4 -= 1, reread) == 0 {
            word!(game.cinematic_var4 = 127);
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0 {
                CopyPalettesToFadingPalettes(game);
                DecomposePaletteDataForFading(game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF27 as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF2D as u16, 0, game);
            } else {
                if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 == 0 {
                    SpawnCinematicSpriteObject(
                        addr_kCinematicSpriteObjectDef_8BEF4B as u16,
                        0,
                        game,
                    );
                    SpawnCinematicSpriteObject(
                        addr_kCinematicSpriteObjectDef_8BEF45 as u16,
                        0,
                        game,
                    );
                    word!(game.cinematic_function = ((0x8be293i32) as u16));
                    *ram_ptr!(game; 0x69) = 16;
                    *ram_ptr!(game; 0x6b) = 0;
                    *ram_ptr!(game; 0x6f) = 0;
                    *ram_ptr!(game; 0x72) = 0;
                    return;
                }
                CopyPalettesToFadingPalettes(game);
                DecomposePaletteDataForFading(game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF51 as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF45 as u16, 0, game);
                ClearYColorsFromIndexX(0x1c0, 0x10, game);
            }
            ClearYColorsFromIndexX(0x1a0, 0x10, game);
            ComposeFadingPalettes(game);
            *ram_ptr!(game; 0x69) = 2;
            *ram_ptr!(game; 0x6b) = 16;
            *ram_ptr!(game; 0x6f) = 2;
            *ram_ptr!(game; 0x72) = 34;
            word!(game.cinematic_function = ((0x8be265i32) as u16));
        }
    };
    [9167461] CinematicFunction_Intro_Func136 [] {
        if (word!(game.cinematic_var4) & 3) != 0 {
            CinematicFunction_Intro_Func135(game);
        } else {
            FadeOutYColorsFromIndexX(0x40, 0x10, game);
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 == 0 {
                FadeInYColorsFromIndexX(0x1c0, 0x10, game);
            }
            FadeInYColorsFromIndexX(0x1a0, 0x10, game);
            ComposeFadingPalettes(game);
            CinematicFunction_Intro_Func135(game);
        };
    };
    [9167507] CinematicFunction_Intro_Func135 [] {
        if word!(game.cinematic_var4 -= 1, reread) as i32 - 65i32 & 0x8000 != 0 {
            word_transfers!(game, descending, 288; native(0x347e) <- value(127));
            word_transfers!(game, ascending, 64; pause_map(384) <- rom(0x8cdedb, 0));
            CinematicUpdateSomeBg(game);
            *ram_ptr!(game; 0x69) = 1;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
            word! {game; cinematic_var13 = 180; cinematic_function = ((0x8be2ddi32) as u16); }
        }
    };
    [9167581] CinematicFunction_Intro_Func137 [] {
        let v0: bool = (word!(game.cinematic_var13 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var13) == 0 || v0 {
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0
                || word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0
            {
                *ram_ptr!(game; 0x69) = 2;
                *ram_ptr!(game; 0x6b) = 16;
                *ram_ptr!(game; 0x6f) = 2;
                *ram_ptr!(game; 0x72) = 34;
                word!(game.cinematic_function = ((0x8be314i32) as u16));
            } else {
                word!(game.cinematic_function = ((0x8be342i32) as u16));
            }
        }
    };
    [9167636] CinematicFunction_Intro_Func138 [] {
        if (word!(game.cinematic_var4) & 3) != 0 {
            CinematicFunction_Intro_Func145(game);
        } else {
            FadeOutYColorsFromIndexX(0x40, 0x10, game);
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 == 0 {
                FadeInYColorsFromIndexX(0x1c0, 0x10, game);
            }
            FadeInYColorsFromIndexX(0x1a0, 0x10, game);
            ComposeFadingPalettes(game);
            CinematicFunction_Intro_Func145(game);
        };
    };
    [9167682] CinematicFunction_Intro_Func145 [] {
        if (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0 {
            *ram_ptr!(game; 0x69) = 16;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
            *ram_ptr!(game; 0x55) = 7;
            *ram_ptr!(game; 0x5f) = 0x80;
            *ram_ptr!(game; 0x58) = 0;
            *ram_ptr!(game; 0x5d) = 0;
            word! {game; cinematic_function = ((0x8bdb9di32) as u16); cinematic_var14 = 0; }
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0 {
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF33 as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF39 as u16, 0, game);
            } else {
                if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
                    SpawnCinematicSpriteObject(
                        addr_kCinematicSpriteObjectDef_8BEF69 as u16,
                        0,
                        game,
                    );
                } else {
                    SpawnCinematicSpriteObject(
                        addr_kCinematicSpriteObjectDef_8BEF63 as u16,
                        0,
                        game,
                    );
                }
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF5D as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF57 as u16, 0, game);
            }
        }
    };
    [9167790] CinematicFunction_Intro_Func139 [] {
        if word!(game.cinematic_var14) != 0 {
            word!(game.cinematic_var14 -= 1);
            FadeOutYColorsFromIndexX(0x180, 0x10, game);
            if word!(game.cinematic_var4) != 0 {
                ComposeFadingPalettes(game);
            }
        }
        if word!(game.cinematic_var4) != 0 {
            word!(game.cinematic_var4 -= 1);
        } else {
            CinematicFunction_Intro_Func140(game);
        }
        word!(game.cinematic_var5 = (((word!(game.cinematic_var5)) as u8).wrapping_sub(8)) as u16);
        word!(game.cinematic_var6 -= 64);
        if word!(game.cinematic_var6) as i32 - 24i32 & 0x8000 != 0 {
            word! {game; cinematic_var6 = 24; cinematic_function = ((0x8be48ai32) as u16);
                cinematic_var4 = 180; cinematic_var14 = 0; }
        }
    };
    [9168010] CinematicFunction_Intro_Func141 [] {
        CinematicFunction_Intro_Func140(game);
        CinematicFunction_Intro_Func142(game);
        if word!(game.cinematic_var4 -= 1, reread) == 0 {
            *ram_ptr!(game; 0x55) = 1;
            *ram_ptr!(game; 0x69) = 17;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 49;
            *ram_ptr!(game; 0x74, 0) = 63;
            *ram_ptr!(game; 0x74, 1) = 95;
            *ram_ptr!(game; 0x74, 2) = -97i32 as u8;
            *ram_ptr!(game; 0x5f) = 0;
            *ram_ptr!(game; 0x58) = 78;
            *ram_ptr!(game; 0x5d) = 84;
            word_transfers!(game, ascending, 16; le(palette_buffer, 192) <- rom(0x8ce7e9, 192));
            word! { game, 0; cinematic_var8, cinematic_var10 }
            word! {game; cinematic_function = ((0x8be504i32) as u16); cinematic_var4 = 32; }
            EnableCinematicBgObjects(game);
            EnableCinematicBgTilemapUpdates(game);
            word!(game.cinematicbg_var3 = 19456);
            word_transfers!(game, descending, 64; native(0x337e) <- value(127));
            CinematicUpdateSomeBg(game);
        }
    };
    [9168132] CinematicFunction_Intro_Func143 [] {
        fade_cinematic_fixed_color!(fresh69, game, fresh70, fresh71);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            word! {game; cinematic_function = ((0x8be7bai32) as u16); cinematicbg_var3 = 20480; }
            CinematicUpdateSomeBg(game);
            word! { game, 256; layer1_x_pos, layer1_y_pos }
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF81 as u16, 0, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF87 as u16, 0, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF8D as u16, 0, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF93 as u16, 0, game);
            word_transfers!(game, descending, 16; le(palette_buffer, 31) <- value(0));
            word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8cefe9, 15));
        }
    };
    [9168266] CinematicFunction_Intro_Func144 [] {
        let v0: i32 = ((4u16).wrapping_mul(word!(game.cinematic_var4))) as i32 >> 1;
        let v5: u16 = crate::rtl::rom_table_word!(game, 0x8be5e7, (v0 + 1) as isize);
        let mut v1: u16 = crate::rtl::rom_table_word!(game, 0x8be5e7, v0 as isize);
        for i in (0i32..=30).rev().step_by(2) {
            word!(game.palette_buffer[((i >> 1) + 16)] = *(RomPtr_8C(v1, game) as *mut u16));
            v1 = (v1).wrapping_sub(2);
        }
        let mut v3: u16 = v5;
        for j in (0u16..=30).rev().step_by(2) {
            *(ram_ptr!(game, u16; 0xc000)).offset(((j as i32 >> 1) + 240) as isize) =
                *(RomPtr_8C(v3, game) as *mut u16);
            v3 = (v3).wrapping_sub(2);
        }
        if word!(game.cinematic_var4 += 1, reread) as i32 - 16i32 & 0x8000 == 0 {
            word!(game.cinematic_function = ((0x8be7bai32) as u16));
            ClearCinematicSprites(game);
            SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BF74E as u16, 0x4c00, game);
        }
    };
    [9168791] CinematicFunction_Intro_Func148 [] {
        word!(game.cinematic_var10 -= 2);
        if word!(game.cinematic_var10) as i32 + 79 & 0x8000 != 0 {
            SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BF748 as u16, 0x4c00, game);
            word!(game.cinematic_function = ((0x8be7bai32) as u16));
            CopySuperMetroidString(game);
        }
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
    }
    when word!(game.cinematic_var4) == 0 || v0;
    [9151992] CinematicFunction_Intro_LastMetroidCaptivity {
        word! {game; cinematic_function = ((0x8ba613i32) as u16); cinematic_var4 = 200; }
        QueueMusic_Delayed8(5, game);
    };
    [9152019] CinematicFunc_Intro_QueueGalaxyIsAtPeace {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff42, game);
        QueueMusic_DelayedY(5, 0xe, game);
        word!(game.cinematic_function = ((0x8ba639i32) as u16));
    };
    [9152076] CinematicFunc_Intro_FadeOut {
        word!(game.cinematic_function = ((0x8ba663i32) as u16));
        word! { game, 2; screen_fade_delay, screen_fade_counter }
    };
    [9160208] CinematicFunction_Intro_Func84 {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = ((0x8bc627i32) as u16));
    };
    [9161424] CinematicFunction_Intro_Func108 {
        word!(game.cinematic_function = ((0x8bcadei32) as u16));
    };
    [9165726] CinematicFunction_Intro_Func119 {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff3c, game);
        QueueMusic_DelayedY(5, 0xe, game);
        word!(game.cinematic_function = ((0x8bdbc4i32) as u16));
    };
    CinematicFunction_Intro_Func122 {
        fade_cinematic_fixed_color!(fresh97, game, fresh98, fresh99);
        if *ram_ptr!(game; 0x74, 0) == 32
            && *ram_ptr!(game; 0x74, 1) == 64
            && *ram_ptr!(game; 0x74, 2) == 0x80
        {
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
        }
        word!(game.cinematic_var4 = 8);
    };
    [9167092] CinematicFunction_Intro_Func129 {
        word! {game; cinematic_function = ((0x8be110i32) as u16); cinematic_var4 = 120; }
        *ram_ptr!(game; 0x69) = 16;
    };
    @external {
        9148999 => [CinematicFunctionNone()];
        9150248 => [()];
        9151375 => [()]; 9151376 => [()];
        9151377 => [CinematicFunc_Nothing()];
        9154232 => [intro_pages::wait_for_scene(intro_pages::Scene::MotherBrain, game)];
        9154412 => [intro_pages::wait_for_scene(intro_pages::Scene::BabyMetroid, game)];
        9154802 => [intro_pages::wait_for_scene(intro_pages::Scene::ScientistPage3, game)]; 9154851 => [intro_pages::wait_for_scene(intro_pages::Scene::ScientistPage4, game)];
        9155034 => [intro_pages::wait_for_final_page(game)]; 9155079 => [intro_pages::show_final_page(game)];
        9155152 => [intro_pages::fade_to_scene(intro_pages::Palette::Gameplay, game)];
        9155282 => [intro_pages::fade_to_scene(intro_pages::Palette::Scientist, game)];
        9155423 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF45 as u16, 0xb3f4, game)]; 9155440 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF4B as u16, 0xb3f4, game)];
        9155457 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF51 as u16, 0xb458, game)]; 9155474 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF57 as u16, 0xb458, game)];
        9155572 => [intro_pages::fade_to_page(intro_pages::Palette::Gameplay, game)]; 9155672 => [intro_pages::fade_to_page(intro_pages::Palette::Scientist, game)];
        9156399 => [intro_pages::fade_out_to_ceres(game)];
        9161438 => [()];
        9165725 => [()];
        9166435 => [()];
        9168826 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicSprInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9147448] CinematicSprInstr_Delete [0] {
        let v2: i32 = k as i32 >> 1;
        word! { game, 0; cinematicspr_whattodraw[v2], cinematicspr_instr_ptr[v2] }
    };
    [9147458] CinematicSprInstr_Sleep [0] {
        word!(game.cinematicspr_instr_ptr[(k >> 1)] = (j).wrapping_sub(2));
    };
    [9147468] CinematicSprInstr_SetPreInstr [(j).wrapping_add(2)] {
        word!(game.cinematicspr_preinstr_func[(k >> 1)] = *(RomPtr_8B(j, game) as *mut u16));
    };
    [9147479] CinematicSprInstr_ClearPreInstr [j] {
        word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8b945fi32) as u16));
    };
    [9147554] CinematicSprInstr_GotoRel [(j).wrapping_add((*RomPtr_8B(j, game) as i8) as u16)] { };
    [9147580] CinematicSprInstr_Goto [*(RomPtr_8B(j, game) as *mut u16)] { };
    [9147587] CinematicSprInstr_DecTimerGoto [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh26 [fresh27] = &raw mut (*(ram_ptr!(game, u16; 0x1b7d)).offset(v2 as isize)) => (u16::from_le(fresh26.read_unaligned())).wrapping_sub(1); }
        if fresh27 == 1 {
            (j).wrapping_add(2)
        } else {
            CinematicSprInstr_Goto(k, j, game)
        }
    };
    [9147597] CinematicSprInstr_DecTimerGotoRel [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh24 [fresh25] = &raw mut (*(ram_ptr!(game, u16; 0x1b7d)).offset(v2 as isize)) => (u16::from_le(fresh24.read_unaligned())).wrapping_sub(1); }
        if fresh25 == 1 {
            (j).wrapping_add(1)
        } else {
            CinematicSprInstr_GotoRel(k, j, game)
        }
    };
    [9147606] CinematicSprInstr_SetTimer [(j).wrapping_add(2)] {
        word!(game.cinematicspr_goto_timer[(k >> 1)] = *(RomPtr_8B(j, game) as *mut u16));
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9149665] CinematicSprInstr_9CE1 [] {
        *ram_ptr!(game; 0x69) = 17;
        word! {game; cinematic_function = ((0x8b9d17i32) as u16); cinematic_var6 = 72;
            cinematic_var8 = 315; cinematic_var7 = 0; cinematic_var10 = 225; cinematic_var9 = 0;
            g_word_7E199B = -2i32 as u16; g_word_7E1999 = 0x8000; }
        SetPpuBackdropSomeColor(game);
        j
    };
    [9149789] CinematicSprInstr_9D5D [] {
        *ram_ptr!(game; 0x69) = 17;
        word! {game; cinematic_function = ((0x8b9d90i32) as u16); cinematic_var6 = 96;
            cinematic_var8 = 44; cinematic_var7 = 0; cinematic_var10 = -155i32 as u16;
            cinematic_var9 = 0; g_word_7E199B = -2i32 as u16; g_word_7E1999 = 0x8000; }
        j
    };
    [9149910] CinematicSprInstr_9DD6 [] {
        *ram_ptr!(game; 0x69) = 17;
        word! {game; cinematic_function = ((0x8b9e12i32) as u16); cinematic_var6 = 96;
            cinematic_var8 = -177i32 as u16; cinematic_var7 = 0;
            cinematic_var10 = -160i32 as u16; }
        word! { game, 0; cinematic_var9, g_word_7E199B, g_word_7E1999 }
        word! {game; g_word_7E199F = 1; g_word_7E199D = 0x8000; }
        SetPpuBackdropSomeColor(game);
        j
    };
    [9150040] CinematicSprInstr_Func8 [] {
        *ram_ptr!(game; 0x69) = 17;
        word! {game; cinematic_function = ((0x8b9e8bi32) as u16); cinematic_var6 = 67; }
        word! { game, 0;
            cinematic_var8, cinematic_var7, cinematic_var10, cinematic_var9, g_word_7E199B,
            g_word_7E1999, g_word_7E199F, g_word_7E199D,
        }
        SpawnTitleScreenGradientObjs(game);
        j
    };
    [9150157] CinematicSprInstr_Func9 [] {
        let v0: u16 = 0x9ecd;
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA113 as u16, v0, game);
        j
    };
    [9150192] CinematicSprInstr_sub_8B9EF0 [] {
        word!(game.cinematicbg_arr9[(k >> 1)] = 0);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BA113 as u16, 0, game);
        j
    };
    [9150233] CinematicSprInstr_9F19 [] {
        word! {game; demo_timer = 900; cinematic_function = ((0x8b9f29i32) as u16); }
        j
    };
    [9152792] CinematicSprInstr_SpawnMetroidEggParticles [] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCECD as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCED3 as u16, 1, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCED9 as u16, 2, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEDF as u16, 3, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEE5 as u16, 4, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEEB as u16, 5, game);
        QueueSfx2_Max6(0xb, game);
        j
    };
    [9156805] CinematicSprInstr_Func43 [] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.cinematicspr_goto_timer[v1]) != 0 {
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bb93bi32) as u16));
        } else {
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bb8d8i32) as u16));
        }
        j
    };
    [9158818] CinematicCommonInstr_Func70 [] {
        word!(game.cinematic_function = ((0x8bc0c5i32) as u16));
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        j
    };
    [9159684] CinematicSprInstr_SpawnCeresExplosions1 [] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 1, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 2, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 3, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 4, game);
        j
    };
    [9159948] CinematicSprInstr_SpawnCeresExplosions3 [] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 1, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 2, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 3, game);
        j
    };
    [9171888] CinematicSprInstr_Func190 [] {
        let v0: u16 = 0;
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEECD as u16, v0, game);
        j
    };
    [9172253] CinematicSprInstr_Func208 [] {
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF3F as u16, 0, game);
        j
    };
    [9172308] CinematicSprInstr_Func211 [] {
        if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
            SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF7B as u16, 2, game);
        } else {
            SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF75 as u16, 2, game);
        }
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEF6F as u16, 0, game);
        j
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9157153] CinematicSprInstr_SpawnIntroRinkas01 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 0, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 1, game);
    };
    [9157174] CinematicSprInstr_SpawnIntroRinkas23 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 2, game);
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 3, game);
    };
    @functions [pub unsafe fn (k: u16, mut j: u16, game: &mut Game) -> u16];
    [9158806] CinematicCommonInstr_Func69 [] {
        if word!(game.japanese_text_flag) == 0 {
            j = (j).wrapping_add(6);
        }
        j
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9161125] CinematicSprInstr_C9A5 [j] { SpawnPalfxObject(addr_kPalfx_E1B0 as u16, game); };
    [9161135] CinematicSprInstr_C9AF [j] {
        if word!(game.japanese_text_flag) != 0 {
            SpawnCinematicSpriteObject(
                addr_kCinematicSpriteObjectDef_8BCEB5 as u16,
                word!(game.japanese_text_flag),
                game,
            );
        }
    };
    [9161149] CinematicSprInstr_C9BD [j] { SpawnPalfxObject(addr_loc_8BE1B4 as u16, game); };
    [9161159] CinematicSprInstr_C9C7 [j] {
        word! {game; cinematic_var8 = 62; cinematic_var10 = -112i32 as u16; cinematic_var5 = 32;
            cinematic_var6 = 16; cinematic_function = ((0x8bc9f9i32) as u16); }
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9171550] CinematicSprInstr_Func181 [j] {
        *ram_ptr!(game; 0x69) = 3;
        *ram_ptr!(game; 0x6b) = 16;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 34;
        *ram_ptr!(game; 0x5d) = 100;
        *ram_ptr!(game; 0x59) = 84;
        word! {game; cinematic_var4 = 0; cinematic_function = ((0x8be58ai32) as u16); }
    };
    [9171588] CinematicSprInstr_Func182 [j] {
        SpawnPalfxObject(addr_kPalfx_E1C4 as u16, game);
        word!(game.palette_buffer[254] = 1);
    };
    [9171605] CinematicSprInstr_Func183 [j] {
        let v0: u16 = 0;
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEB5 as u16, v0, game);
        word!(game.palette_buffer[0] = 0x7fff);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9171639] CinematicSprInstr_Func185 [j] {
        let mut v0: u16 = 0;
        *ram_ptr!(game; 0x69) = 17;
        *ram_ptr!(game; 0x6b) = 2;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 17;
        *(&raw mut v0 as *mut u8) = 124;
        *ram_ptr!(game; 0x59) = 124;
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEBB as u16, 0, game);
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEF7 as u16, 2, game);
        SpawnPalfxObject(addr_kPalfx_E1C8 as u16, game);
        SpawnPalfxObject(addr_kPalfx_E1D0 as u16, game);
        SpawnPalfxObject(addr_kPalfx_E1E8 as u16, game);
    };
    [9171706] CinematicSprInstr_Func186 [j] {
        *ram_ptr!(game; 0x69) = 3;
        *ram_ptr!(game; 0x6b) = 18;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 51;
        *ram_ptr!(game; 0x58) = 116;
        *ram_ptr!(game; 0x59) = 120;
        SpawnPalfxObject(addr_kPalfx_E1CC as u16, game);
        SpawnCinematicSpriteObjectToR18(addr_kCinematicSpriteObjectDef_8BEEC1 as u16, 6, game);
    };
    [9171755] CinematicSprInstr_Func187 [j] {
        *ram_ptr!(game; 0x69) = 0;
        *ram_ptr!(game; 0x6b) = 0;
        word! { game, 0x7fff; palette_buffer[0], palette_buffer[128] }
        word_transfers!(game, descending, 16; le(palette_buffer, 31) <- value(0x7fff));
        word! {game; cinematic_function = ((0x8bdb9ei32) as u16); cinematic_var4 = 120; }
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9171918] CinematicSprInstr_Func192 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEEFD as u16, 0, game);
        let hours1: i32 = word!(game.game_time_hours) as i32 / 10;
        let hours2: i32 = word!(game.game_time_hours) as i32 % 10;
        word! {game; substate = hours1 as u16; suit_pickup_light_beam_pos = hours2 as u16; }
        let min1: i32 = word!(game.game_time_minutes) as i32 / 10;
        let min2: i32 = word!(game.game_time_minutes) as i32 % 10;
        *(ram_ptr!(game, u16; 0xdf0)) = min1 as u16;
        *(ram_ptr!(game, u16; 0xdf2)) = min2 as u16;
    };
    [9171995] CinematicSprInstr_Func193 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF03 as u16, 0, game);
    };
    [9172004] CinematicSprInstr_Func194 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF09 as u16, 0, game);
    };
    [9172013] CinematicSprInstr_Func195 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF0F as u16, 0, game);
    };
    [9172022] CinematicSprInstr_Func196 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF15 as u16, 0, game);
    };
    [9172031] CinematicSprInstr_Func197 [j] {
        SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BEF1B as u16, 0, game);
    };
    [9172040] CinematicSprInstr_Func198 [j] {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = ((0x8bde64i32) as u16));
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9172375] CinematicSprInstr_Func213 [j] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
            word!(game.cinematicbg_arr7[(v1 + 1)] = 120);
        } else {
            word!(game.cinematicbg_arr7[(v1 + 1)] = 118);
        }
        word!(game.cinematicbg_arr8[(v1 + 1)] = 120);
    };
    [9172410] CinematicSprInstr_Func214 [j] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
            word! {game; cinematicbg_arr7[(v1 + 1)] = 121; cinematicbg_arr8[(v1 + 1)] = 116; }
        } else {
            word! {game; cinematicbg_arr7[(v1 + 1)] = 120; cinematicbg_arr8[(v1 + 1)] = 114; }
        }
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9172484] CinematicSprInstr_Func217 [j] {
        word!(game.cinematicbg_arr9[(k >> 1)] = 3584);
        word!(game.cinematic_function = ((0x8be3aei32) as u16));
        *ram_ptr!(game; 0x69) = 17;
        *ram_ptr!(game; 0x6b) = 16;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = -127i32 as u8;
        word! {game; cinematic_var8 = -4i32 as u16; cinematic_var10 = -8i32 as u16;
            cinematic_var4 = 16; }
        word! { game, 32; cinematic_var14, cinematic_var17 }
        CopyPalettesToFadingPalettes(game);
        DecomposePaletteDataForFading(game);
        QueueSfx1_Max15(0x22, game);
    };
    [9172561] CinematicSprInstr_Func218 [j] {
        word! {game; samus_y_subspeed = 0; samus_y_speed = -16i32 as u16; }
    };
    @external {
        9155382 => [intro_pages::start_page(0xb35f, j, game)]; 9155390 => [intro_pages::start_page(0xb370, j, game)];
        9155398 => [intro_pages::start_page(0xb381, j, game)]; 9155406 => [intro_pages::start_page(0xb392, j, game)];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicSprPreInstr(ea: u32, j: u16, game: &mut Game) => ea;
    call (j, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [9149647] sub_8B9CCF [] {
        *ram_ptr!(game; 0x51) = 15;
        word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8b93d9i32) as u16));
    };
    [9152744] SetSomeStuffForSpriteObject_4_MetroidEgg [] {
        if word!(game.samus_x_pos) as i32 - 169i32 & 0x8000 != 0 {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCB3B as u16);
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        }
    };
    [9152771] CinematicSprPreInstr_A903 [] {
        if word!(game.cinematic_var13) == 0 {
            let v1: i32 = k as i32 >> 1;
            word! {game; cinematicspr_instr_timer[v1] = 1;
                cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16; cinematic_var15 = 0; }
        }
    };
    [9153896] SetSomeStuffForSpriteObject_11 [] {
        if word!(game.cinematic_var13) != 0 {
            if (word!(game.cinematic_var4) & 3) == 0 && word!(game.cinematic_var8) != 0 {
                word!(game.cinematic_var8 -= 1);
                word!(game.cinematicbg_arr7[(k >> 1)] += 1);
            }
        } else {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        };
    };
    [9153958] SetSomeStuffForSpriteObject_13 [] {
        if word!(game.cinematic_var13) != 0 {
            if (word!(game.cinematic_var4) & 3) == 0
                && word!(game.cinematic_var10) as i32 - 8i32 & 0x8000 != 0
            {
                word!(game.cinematic_var10 += 1);
                word!(game.cinematicbg_arr8[(k >> 1)] -= 1);
            }
        } else {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        };
    };
    [9154074] SetSomeStuffForSpriteObject_18 [] {
        if word!(game.cinematic_var16) == 59 {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCC23 as u16);
        }
        if word!(game.cinematic_var13) == 127 {
            let v2: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v2] = 1);
            word!(game.cinematicspr_instr_ptr[v2] = addr_off_8BCE53 as u16);
        }
    };
    [9156486] CinematicFunction_Intro_Func36 [] {
        CinematicFunction_Intro_Func37(k, game);
        let mut v1: u16 = 8;
        while (word!(game.projectile_type[(v1 >> 1)]) & 0xfff) != 256 {
            v1 = (v1).wrapping_sub(2);
            if (v1 & 0x8000) != 0 {
                return;
            }
        }
        if word!(game.projectile_x_pos[(v1 >> 1)]) as i32 - 84i32 & 0x8000 != 0 {
            KillProjectile(v1, game);
            let v2: i32 = k as i32 >> 1;
            word!(game.cinematicspr_arr6[v2] = 8);
            let v3: u16 = (word!(game.cinematicspr_goto_timer[v2])).wrapping_add(1);
            word!(game.cinematicspr_goto_timer[v2] = v3);
            if v3 == 4 {
                word!(game.cinematicspr_goto_timer[v2] = 0);
                word!(game.cinematicspr_preinstr_func[v2] = ((0x8bb80fi32) as u16));
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 1, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 2, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 0, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 1, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 2, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 3, game);
                SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 4, game);
            }
        }
    };
    [9156623] CinematicFunction_Intro_Func39 [] {
        CinematicFunction_Intro_Func37(k, game);
        CinematicFunction_Intro_Func40(game);
        let v1: i32 = k as i32 >> 1;
        let v2: u16 = (word!(game.cinematicspr_goto_timer[v1])).wrapping_add(1);
        word!(game.cinematicspr_goto_timer[v1] = v2);
        if v2 as i32 - 128i32 & 0x8000 == 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCB19 as u16);
        }
    };
    [9156654] CinematicSprPreInstr_B82E [] {
        CinematicFunction_Intro_Func40(game);
        if word!(game.cinematic_var13) == 0 {
            let v1: i32 = k as i32 >> 1;
            word! {game; cinematicspr_instr_timer[v1] = 1;
                cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16; cinematic_var15 = 0; }
        }
    };
    [9157135] CinematicFunction_Intro_Func48 [] {
        if word!(game.cinematic_var13) == 0 {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        }
    };
    [9157389] CinematicFunction_Intro_Func52 [] {
        let v1: bool = (word!(game.cinematic_var14 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var14) == 0 || v1 {
            let v2: i32 = k as i32 >> 1;
            word!(game.cinematicspr_preinstr_func[v2] = ((0x8bbb24i32) as u16));
            word! { game, 0; cinematic_var14, cinematic_var17, cinematicspr_goto_timer[v2] }
        }
    };
    [9157412] CinematicFunction_Intro_Func53 [] {
        let kh: i32 = k as i32 >> 1;
        if word!(game.cinematic_var13) != 0 {
            if word!(game.cinematicspr_goto_timer[kh]) as i32 - 128i32 & 0x8000 != 0 {
                let v3: u16 = (word!(game.cinematicspr_goto_timer[kh])).wrapping_add(1);
                word!(game.cinematicspr_goto_timer[kh] = v3);
                if (v3 & 0x3f) == 0 {
                    QueueSfx3_Max6(0x23, game);
                }
            }
            if ((word!(game.samus_x_pos)) as i16)
                .wrapping_sub((word!(game.cinematicbg_arr7[kh])) as i16)
                < 0
            {
                if word!(game.cinematic_var14) as i32 + 639 & 0x8000 == 0 {
                    word!(game.cinematic_var14 -= 32);
                }
            } else if word!(game.cinematic_var14) as i32 - 640i32 & 0x8000 != 0 {
                word!(game.cinematic_var14 += 32);
            }
            word!(
                game.cinematic_var15 = (if word!(game.cinematic_var14) & 0x8000 != 0 {
                    1
                } else {
                    -1i32
                }) as u16
            );
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a7d)).offset(kh as isize),
                (ram_ptr!(game, u16; 0x1add)).offset(kh as isize),
                ((word!(game.cinematic_var14) as i16 as i32) << 8) as u32,
            );
            chase_samus_with_cinematic_sprite!(game, 8i32, kh);
        } else {
            word! {game; cinematicspr_instr_timer[kh] = 1;
                cinematicspr_instr_ptr[kh] = addr_off_8BCE53 as u16; cinematic_var15 = 0; }
        };
    };
    [9158325] CinematicFunction_Intro_Func58 [] {
        if word!(game.cinematic_function) as i32 == 0x8bbdf9 & 0xffff {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_goto_timer[v1] += 128);
            let d: i32 = (word!(game.cinematicspr_goto_timer[v1]) as i16 as i32) << 8;
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
                d as u32,
            );
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
                d as u32,
            );
            words!(game.cinematic_var8, cinematic_var7 += d as u32);
            words!(game.cinematic_var10, cinematic_var9 += d as u32);
        }
    };
    [9159817] CinematicSprPreInstr_C489 [] {
        if word!(game.cinematic_function) as i32 == 0x8bc5ca & 0xffff {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8b93d9i32) as u16));
        } else {
            let v1: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
            if word!(game.cinematic_var4) == 0 || v1 {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BCEC1 as u16,
                    word!(game.cinematic_var13),
                    game,
                );
                word!(game.cinematic_var4 = 12);
                word!(game.cinematic_var13 = (word!(game.cinematic_var13) as i32 + 1 & 7) as u16);
            }
        };
    };
    [9160782] CinematicFunction_Intro_Func90 [] {
        if word!(game.cinematic_function) as i32 == 0x8bcade & 0xffff {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bc85di32) as u16));
        }
    };
    [9160874] CinematicFunction_Intro_Func93 [] {
        if word!(game.cinematic_function) as i32 == 0x8bcade & 0xffff {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bc8b9i32) as u16));
        }
    };
    [9160953] CinematicFunction_Intro_Func96 [] {
        if word!(game.cinematic_function) as i32 == 0x8bcade & 0xffff {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bc908i32) as u16));
        }
    };
    [9171802] CinematicSprPreInstr_F35A [] {
        if word!(game.cinematic_function) as i32 == 0x8bdca5 & 0xffff {
            let v1: i32 = k as i32 >> 1;
            word! {game; cinematicspr_preinstr_func[v1] = ((0x8bf375i32) as u16);
                cinematicspr_arr7[v1] = 0x4000; cinematicspr_goto_timer[v1] = 0; }
        }
    };
    [9171867] CinematicFunction_Intro_Func189b [] {
        if word!(game.cinematic_function) as i32 == 0x8bde63 & 0xffff {
            let v5: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v5] = 1);
            word!(game.cinematicspr_instr_ptr[v5] = addr_off_8BEE9B as u16);
        }
    };
    [9171897] CinematicFunction_Intro_Func191 [] {
        if word!(game.cinematic_function) as i32 == 0x8bde80 & 0xffff {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
        }
    };
    [9172053] CinematicFunction_Intro_Func199 [] {
        if word!(game.cinematic_var6) as i32 - 176i32 & 0x8000 != 0 {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bf464i32) as u16));
        }
    };
    [9172088] CinematicFunction_Intro_Func201 [] {
        if word!(game.cinematic_var6) as i32 - 176i32 & 0x8000 != 0 {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bf487i32) as u16));
        }
    };
    [9172123] CinematicFunction_Intro_Func203 [] {
        if word!(game.cinematic_var6) as i32 - 96i32 & 0x8000 == 0 {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bf4aai32) as u16));
        }
    };
    [9172177] CinematicFunction_Intro_Func205 [] {
        if word!(game.cinematic_var6) as i32 - 96i32 & 0x8000 == 0 {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = ((0x8bf4e0i32) as u16));
        }
    };
    [9172231] CinematicFunction_Intro_Func207 [] {
        if word!(game.cinematic_function) as i32 == 0x8bdb9d & 0xffff {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
        }
    };
    [9172264] CinematicSprPreInstr_F528 [] {
        CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
        let v1: i32 = k as i32 >> 1;
        if word!(game.cinematicbg_arr8[v1]) as i32 + 80 & 0x8000 != 0 {
            *ram_ptr!(game; 0x52) = 3;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BED95 as u16);
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bf5ddi32) as u16));
            word!(game.cinematicbg_arr9[v1] = 3072);
        }
    };
    [9172351] CinematicSprPreInstr_F57F [] {
        CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
        let v1: i32 = k as i32 >> 1;
        if word!(game.cinematicbg_arr8[v1]) as i32 + 80 & 0x8000 != 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
        }
    };
    [9172445] CinematicFunction_Intro_Func215 [] {
        CinematicFunction_Intro_Func216(game);
        CinematicFunction_Intro_Func210_EndingSamusFall(k, game);
        let v1: i32 = k as i32 >> 1;
        if word!(game.cinematicbg_arr8[v1]) as i32 - 136i32 & 0x8000 == 0 {
            word! {game; cinematicbg_arr8[v1] = 136; cinematicspr_instr_timer[v1] = 1;
                cinematicspr_instr_ptr[v1] = addr_word_8BED9D as u16;
                cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16); }
        }
    };
    @functions [pub unsafe fn (j: u16, game: &mut Game)];
    [9171621] CinematicFunction_Intro_Func184 [] {
        if word!(game.cinematicspr_instr_ptr[15]) == 0 {
            let v2: i32 = j as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v2] = 1);
            word!(game.cinematicspr_instr_ptr[v2] = addr_off_8BEE9B as u16);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let v1: i32 = k as i32 >> 1; };
    [9152916] SetSomeStuffForSpriteObject_7 [] {
        let v3: i32 = 2 * byte!(@pointer (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize); 0) as i32;
        crate::native_assertion! {
            '_c2rust_label: (v3 + 1) < 12;
            b"v3 + 1 < 12\0", b"src/sm_8b.c\0", 2389, b"void SetSomeStuffForSpriteObject_7(uint16)\0"
        };
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            words!((
                kSetSomeStuffForSpriteObject_7_Tab0[v3 as usize],
                kSetSomeStuffForSpriteObject_7_Tab0[(v3 + 1) as usize] as u16
            )),
        );
        let v6: i32 = 2 * byte!(@pointer (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize); 1) as i32;
        crate::native_assertion! {
            '_c2rust_label_0: (v6 + 1) < 82;
            b"v6 + 1 < 82\0", b"src/sm_8b.c\0", 2392, b"void SetSomeStuffForSpriteObject_7(uint16)\0"
        };
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            words!((
                kSetSomeStuffForSpriteObject_7_Tab1[v6 as usize],
                kSetSomeStuffForSpriteObject_7_Tab1[(v6 + 1) as usize]
            )),
        );
        if word!(game.cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000 != 0 {
            word!(game.cinematicspr_goto_timer[v1] += 256);
        } else {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        };
    };
    [9153203] SetSomeStuffForSpriteObject_9 [] {
        let v3: i32 = 2 * byte!(@pointer (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize); 0) as i32;
        crate::native_assertion! {
            '_c2rust_label: (v3 + 1) < 10;
            b"v3 + 1 < 10\0", b"src/sm_8b.c\0", 2442, b"void SetSomeStuffForSpriteObject_9(uint16)\0"
        };
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            words!((
                kSetSomeStuffForSpriteObject_9_Tab0[v3 as usize],
                kSetSomeStuffForSpriteObject_9_Tab0[(v3 + 1) as usize] as u16
            )),
        );
        if (word!(game.cinematicspr_goto_timer[v1]) & 1) != 0 {
            let v12: i32 =
                2 * byte!(@pointer (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize); 1) as i32;
            crate::native_assertion! {
                '_c2rust_label_0: (v12 + 1) < 124;
                b"v12 + 1 < 124\0", b"src/sm_8b.c\0", 2446,
                b"void SetSomeStuffForSpriteObject_9(uint16)\0"
            };
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
                words!((
                    kSetSomeStuffForSpriteObject_9_Tab2[v12 as usize],
                    kSetSomeStuffForSpriteObject_9_Tab2[(v12 + 1) as usize] as u16
                )),
            );
        } else {
            let v6: i32 =
                2 * byte!(@pointer (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize); 1) as i32;
            crate::native_assertion! {
                '_c2rust_label_1: (v6 + 1) < 138;
                b"v6 + 1 < 138\0", b"src/sm_8b.c\0", 2450,
                b"void SetSomeStuffForSpriteObject_9(uint16)\0"
            };
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
                words!((
                    kSetSomeStuffForSpriteObject_9_Tab1[v6 as usize],
                    kSetSomeStuffForSpriteObject_9_Tab1[(v6 + 1) as usize] as u16
                )),
            );
        }
        if word!(game.cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000 == 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCD71 as u16);
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16));
        } else {
            word!(game.cinematicspr_goto_timer[v1] += 256);
        };
    };
    [9156824] CinematicFunction_Intro_Func44 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            0x8000,
        );
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            0x8000,
        );
        if word!(game.cinematicbg_arr7[v1]) as i32 + 8 - (word!(game.samus_x_pos) as i32 - 5i32)
            & 0x8000
            != 0
        {
            if word!(game.cinematicspr_preinstr_func[0]) as i32 != 0x8bb80f & 0xffff {
                return;
            }
        } else {
            word! { game, 11; samus_invincibility_timer, samus_knockback_timer }
            word!(game.knockback_x_dir = 1);
        }
        word!(game.cinematicspr_instr_timer[v1] = 1);
        word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
    };
    [9156923] CinematicFunction_Intro_Func45 [] {
        let v2: i32 = word!(game.cinematicspr_goto_timer[v1]) as i32;
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            words!((kCinematicFunction_Intro_Func45_Tab0[v2 as usize], 0x8000u16)),
        );
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            0x8000,
        );
        if word!(game.cinematicbg_arr8[v1]) as i32 - 16i32 & 0x8000 != 0
            || word!(game.cinematicbg_arr8[v1]) as i32 - 208i32 & 0x8000 == 0
            || word!(game.cinematicspr_preinstr_func[0]) as i32 == 0x8bb80f & 0xffff
        {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        }
    };
    [9157214] CinematicFunction_Intro_Func50 [] {
        if word!(game.cinematicspr_instr_ptr[(v1 + 1)]) as i32 + 0x3487 & 0x8000 == 0 {
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bba73i32) as u16));
            word! { game, 0; cinematic_var14, cinematic_var17 }
        }
    };
    [9157235] CinematicFunction_Intro_Func51 [] {
        if word!(game.cinematicbg_arr8[v1]) == 145 {
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 0, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 1, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 2, game);
            SpawnCinematicSpriteObject(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 3, game);
            QueueSfx3_Max6(0x23, game);
        }
        chase_samus_with_cinematic_sprite!(game, 32i32, v1);
        if (word!(game.cinematic_var17) & 0x8000) == 0 {
            word!(game.cinematic_var14 = 128);
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbb0di32) as u16));
        }
    };
    [9158453] CinematicFunction_Intro_Func60 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            0x4000,
        );
        word!(game.cinematicbg_arr7[v1] &= (0x1ffu16));
    };
    [9158495] CinematicFunction_Intro_Func62 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            4096,
        );
        word!(game.cinematicbg_arr7[v1] &= (0x1ffu16));
    };
    [9158537] CinematicFunction_Intro_Func64 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            2048,
        );
        word!(game.cinematicbg_arr7[v1] &= (0x1ffu16));
    };
    [9158598] CinematicFunction_Intro_Func66 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            -0x2000i32 as u32,
        );
    };
    [9160066] CinematicFunction_Intro_Func82 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            -4096i32 as u32,
        );
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            0x4000,
        );
    };
    [9160797] CinematicFunction_Intro_Func91 [] {
        word!(game.cinematicspr_goto_timer[v1] += 64);
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            -((word!(game.cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
        );
        if word!(game.cinematicbg_arr8[v1]) as i32 + 128 & 0x8000 != 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        }
    };
    [9160889] CinematicFunction_Intro_Func94 [] {
        word!(game.cinematicspr_goto_timer[v1] += 32);
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            -((word!(game.cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
        );
        if word!(game.cinematicbg_arr8[v1]) as i32 + 128 & 0x8000 != 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
            word!(game.cinematic_function = ((0x8bcadfi32) as u16));
        }
    };
    [9160968] CinematicFunction_Intro_Func97 [] {
        word!(game.cinematicspr_goto_timer[v1] += 32);
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            -((word!(game.cinematicspr_goto_timer[v1]) as i16 as i32) << 8i32) as u32,
        );
        if word!(game.cinematicbg_arr8[v1]) as i32 + 128 & 0x8000 != 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        }
    };
    [9170866] CinematicFunction_Intro_Func151 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a9d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            128,
        );
        word!(game.cinematicspr_arr6[v1] = word!(game.cinematicspr_arr6[v1]));
        if word!(game.cinematicbg_arr7[v1] += 1, reread) as i32 - 272i32 & 0x8000 == 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BEE9B as u16);
        }
    };
    [9171431] CinematicFunction_Intro_Func179 [] {
        word!(game.cinematicbg_arr8[v1] += word!(game.cinematicspr_goto_timer[v1]));
        let v2: u16 =
            (word!(game.cinematicbg_arr7[v1])).wrapping_sub(word!(game.cinematicspr_goto_timer[v1]));
        word!(game.cinematicbg_arr7[v1] = v2);
        if v2 as i32 - 395i32 & 0x8000 != 0 {
            word! {game; cinematicbg_arr7[v1] = 394; cinematicbg_arr8[v1] = 367;
                cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16); }
            SpawnPalfxObject(addr_kPalfx_E200 as u16, game);
        } else {
            word!(game.cinematicspr_goto_timer[v1] += 2);
        };
    };
    [9171495] CinematicFunction_Intro_Func180 [] {
        word!(game.cinematicbg_arr8[v1] -= word!(game.cinematicspr_goto_timer[v1]));
        let v2: u16 =
            (word!(game.cinematicspr_goto_timer[v1])).wrapping_add(word!(game.cinematicbg_arr7[v1]));
        word!(game.cinematicbg_arr7[v1] = v2);
        if v2 as i32 - 382i32 & 0x8000 != 0 {
            word!(game.cinematicspr_goto_timer[v1] += 2);
        } else {
            word! {game; cinematicbg_arr7[v1] = 382; cinematicbg_arr8[v1] = 383;
                cinematicspr_preinstr_func[v1] = ((0x8b93d9i32) as u16); }
        };
    };
    [9171829] CinematicFunction_Intro_Func189 [] {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1b7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1afd)).offset(v1 as isize),
            -32i32 as u32,
        );
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a7d)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1add)).offset(v1 as isize),
            words!((
                word!(game.cinematicspr_goto_timer[v1]),
                word!(game.cinematicspr_arr7[v1])
            )),
        );
        CinematicFunction_Intro_Func189b(k, game);
    };
    [9172068] CinematicFunction_Intro_Func200 [] {
        word!(game.cinematicspr_arr7[v1] = word!(game.cinematicspr_arr7[v1]));
        word!(game.cinematicbg_arr8[v1] += 1);
    };
    [9172103] CinematicFunction_Intro_Func202 [] {
        word!(game.cinematicspr_arr7[v1] = word!(game.cinematicspr_arr7[v1]));
        word!(game.cinematicbg_arr8[v1] -= 1);
    };
    [9172138] CinematicFunction_Intro_Func204 [] {
        word!(game.cinematicspr_arr7[v1] = word!(game.cinematicspr_arr7[v1]));
        word!(game.cinematicbg_arr8[v1] -= 2);
        word!(game.cinematicspr_arr6[v1] = word!(game.cinematicspr_arr6[v1]));
        word!(game.cinematicbg_arr7[v1] -= 1);
    };
    [9172192] CinematicFunction_Intro_Func206 [] {
        word!(game.cinematicspr_arr7[v1] = word!(game.cinematicspr_arr7[v1]));
        word!(game.cinematicbg_arr8[v1] += 2);
        word!(game.cinematicspr_arr6[v1] = word!(game.cinematicspr_arr6[v1]));
        word!(game.cinematicbg_arr7[v1] += 1);
    };
    @external {
        9158617 => [()]; 9147353 => [()]; 9147487 => [()]; 9152724 => [()];
        9172252 => [()];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallCinematicBgPreInstr(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        9147353 => [()]; 9155772 => CinematicBgPreInstr_SamusBlink;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicBgInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9148056] CinematicBgInstr_Delete [0] {
        let v2: i32 = k as i32 >> 1;
        word! { game, 0; cinematicbg_arr1[v2], cinematicbg_instr_ptr[v2] }
    };
    [9148190] CinematicBgInstr_Goto [*(RomPtr_8C(j, game) as *mut u16)] { };
    @functions [pub unsafe fn (
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16];
    [9154004] CinematicBgInstr_SetSomeStuffForSpriteObject_14 [] {
        word!(game.cinematicspr_instr_ptr[15] = addr_unk_8BCC03 as u16);
        word!(game.cinematicspr_instr_timer[15] = 1);
        j
    };
    @functions [pub unsafe fn (k: u16, instrp: u16, game: &mut Game) -> u16];
    [9168423] CalcItemPercentageCount [] {
        let mut r18: u16 = 0;
        for v0 in (0u16..=4).rev() {
            let v1 = RomPtr_RAM(
                crate::rtl::rom_table_word!(game, 0x8be70d, v0 as isize),
                game,
            ) as *const u16;
            r18 = (r18).wrapping_add(SnesDivide(
                *v1,
                crate::rtl::rom_table_word!(game, 0x8be717, v0 as isize) as u8,
            ));
        }
        for i in (0i32..=20).rev().step_by(2) {
            if (crate::rtl::rom_table_word!(game, 0x8be721, (i >> 1) as isize)
                & word!(game.collected_items))
                != 0
            {
                r18 = r18.wrapping_add(1);
            }
        }
        for j in (0u16..=8).rev().step_by(2) {
            if (crate::rtl::rom_table_word!(game, 0x8be737, (j >> 1) as isize)
                & word!(game.collected_beams))
                != 0
            {
                r18 = r18.wrapping_add(1);
            }
        }
        let RegWord: u16 = (r18 as i32 / 10) as u16;
        let r22: u16 = (r18 as i32 % 10) as u16;
        r18 = (RegWord as i32 / 10) as u16;
        let r20: u16 = (RegWord as i32 % 10) as u16;
        if r18 != 0 {
            let v5: i32 = ((4u16).wrapping_mul(r18)) as i32 >> 1;
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[462i32 as usize] =
                crate::rtl::rom_table_word!(game, 0x8be741, v5 as isize);
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[494i32 as usize] =
                crate::rtl::rom_table_word!(game, 0x8be741, (v5 + 1) as isize);
        }
        let mut v6: u16 = r20;
        let draw_percentage_digit: bool = (r20 != 0)
            || ((r18 != 0)
                && ({
                    v6 = r20;
                    true
                }));
        if draw_percentage_digit {
            let v7: i32 = ((4u16).wrapping_mul(v6)) as i32 >> 1;
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[463i32 as usize] =
                crate::rtl::rom_table_word!(game, 0x8be741, v7 as isize);
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[495i32 as usize] =
                crate::rtl::rom_table_word!(game, 0x8be741, (v7 + 1) as isize);
        }
        let v8: i32 = ((4u16).wrapping_mul(r22)) as i32 >> 1;
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[464i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, v8 as isize);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[496i32 as usize] =
            crate::rtl::rom_table_word!(game, 0x8be741, (v8 + 1) as isize);
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[465i32 as usize] = 14442;
        (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[497i32 as usize] = 14458;
        instrp
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9168745] CinematicFunction_Intro_Func146 [] {
        if word!(game.japanese_text_flag) != 0 {
            word_transfers!(game, descending, 64; pause_map(799) <- rom(0x8cdf5b, 63));
        }
        j
    };
    [9168768] CinematicFunction_Intro_Func147 [] {
        word_transfers!(game, descending, 64; native(0x363e) <- value(127));
        word!(game.cinematic_function = ((0x8be797i32) as u16));
        j
    };
    @external {
        9154115 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD401 as u16, j, game)];
        9154139 => [intro_pages::spawn_marker(0xaeb8, None, k, j, game)];
        9154169 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD407 as u16, j, game)];
        9154193 => [intro_pages::spawn_marker(0xaf6c, Some(addr_kMode7ObjectDef_8BD40D as u16), k, j, game)];
        9154676 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD413 as u16, j, game)];
        9154700 => [intro_pages::spawn_marker(0xb0f2, Some(addr_kMode7ObjectDef_8BD419 as u16), k, j, game)];
        9154739 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD41F as u16, j, game)];
        9154763 => [intro_pages::spawn_marker(0xb123, Some(addr_kMode7ObjectDef_8BD425 as u16), k, j, game)];
        9154971 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD42B as u16, j, game)];
        9154995 => [intro_pages::spawn_marker(0xb1da, Some(addr_kMode7ObjectDef_8BD431 as u16), k, j, game)]; 9155112 => [intro_pages::create_japanese_text(addr_kMode7ObjectDef_8BD437 as u16, j, game)];
        9155136 => [intro_pages::finish_pages(j, game)]; 9158806 => CinematicCommonInstr_Func69;
        9158818 => CinematicCommonInstr_Func70;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicSpriteObjectSetup(ea: u32, j: u16, game: &mut Game) => ea;
    call (j, game); default { Unreachable(game); };
    @functions [pub unsafe fn (v0: u16, game: &mut Game)];
    [9152705] SetSomeStuffForSpriteObject_2 [] {
        let v1: i32 = v0 as i32 >> 1;
        word! {game; cinematicbg_arr7[v1] = 8; cinematicbg_arr8[v1] = 24;
            cinematicbg_arr9[v1] = 3072; }
    };
    @functions [pub unsafe fn (j: u16, game: &mut Game)];
    [9152856] SetSomeStuffForSpriteObject_6 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = ((4u16).wrapping_mul(v1)) as i32 >> 1;
        word! { game;
            cinematicbg_arr7[v2] =
                ((kSetSomeStuffForSpriteObject_6_Tab[v3 as usize]) as u16).wrapping_add(16);
            cinematicbg_arr8[v2] =
                ((kSetSomeStuffForSpriteObject_6_Tab[(v3 + 1) as usize]) as u16).wrapping_add(59);
            cinematicbg_arr9[v2] = 3584;
        }
    };
    [9156758] CinematicFunction_Intro_Func42 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = v1 as i32;
        word! { game;
            cinematicbg_arr7[v2] = kCinematicFunction_Intro_Func42_Tab0[v3 as usize];
            cinematicbg_arr8[v2] = (kCinematicFunction_Intro_Func42_Tab1[v3 as usize]).wrapping_sub(8);
            cinematicbg_arr9[v2] = 3584;
        }
    };
    [9157005] CinematicFunction_Intro_Func46 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = v1 as i32;
        word! { game;
            cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func46_Tab0[v3 as usize]).wrapping_add(56);
            cinematicbg_arr8[v2] =
                (kCinematicFunction_Intro_Func46_Tab1[v3 as usize]).wrapping_add(111);
            cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func46_Tab2[v3 as usize];
            cinematicbg_arr9[v2] = 2560;
        }
    };
    [9157076] CinematicFunction_Intro_Func47 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = v1 as i32;
        word! { game;
            cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func47_Tab0[v3 as usize]).wrapping_add(56);
            cinematicbg_arr8[v2] =
                (kCinematicFunction_Intro_Func47_Tab1[v3 as usize]).wrapping_add(111);
            cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func47_Tab2[v3 as usize];
            cinematicbg_arr9[v2] = 2560;
        }
    };
    [9158560] CinematicFunction_Intro_Func65 [] {
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicbg_arr7[(j >> 1)] = 224);
        } else {
            let v1: i32 = j as i32 >> 1;
            word!(game.cinematicbg_arr7[v1] = 112);
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbfd9i32) as u16));
        }
        let v2: i32 = j as i32 >> 1;
        word! {game; cinematicbg_arr8[v2] = 87; cinematicbg_arr9[v2] = 2048; }
    };
    [9159732] CinematicFunction_Intro_Func78 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = v1 as i32;
        word! { game;
            cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func78_Tab0[v3 as usize];
            cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func78_Tab1[v3 as usize] as i32
                + word!(game.reg_M7X) as i32
                - word!(game.cinematic_var8) as i32) as u16;
            cinematicbg_arr8[v2] = (kCinematicFunction_Intro_Func78_Tab2[v3 as usize] as i32
                + word!(game.reg_M7Y) as i32
                - word!(game.cinematic_var10) as i32) as u16;
            cinematicbg_arr9[v2] = 2560;
        }
    };
    [9159865] CinematicFunction_Intro_Func80 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word!(game.cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param));
        let v3: i32 = 2 * v1 as i32;
        word! { game;
            cinematicbg_arr7[v2] = (g_word_8BC4EB[v3 as usize] as i32 + word!(game.reg_M7X) as i32
                - word!(game.cinematic_var8) as i32) as u16;
            cinematicbg_arr8[v2] = (g_word_8BC4EB[(v3 + 1) as usize] as i32
                + word!(game.reg_M7Y) as i32
                - word!(game.cinematic_var10) as i32) as u16;
            cinematicbg_arr9[v2] = 2560;
        }
    };
    [9159987] CinematicFunction_Intro_Func81 [] {
        let v1: u16 = word!(game.cinematic_spawn_param);
        let v2: i32 = j as i32 >> 1;
        word! { game;
            cinematicspr_goto_timer[v2] = word!(game.cinematic_spawn_param);
            cinematicspr_instr_timer[v2] = kCinematicFunction_Intro_Func81_Tab0[v1 as usize] as u16;
            cinematicbg_arr7[v2] = (kCinematicFunction_Intro_Func81_Tab1[v1 as usize] as i32
                + word!(game.reg_M7X) as i32
                - word!(game.cinematic_var8) as i32) as u16;
            cinematicbg_arr8[v2] = (kCinematicFunction_Intro_Func81_Tab2[v1 as usize] as i32
                + word!(game.reg_M7Y) as i32
                - word!(game.cinematic_var10) as i32) as u16;
            cinematicbg_arr9[v2] = 2560;
        }
    };
    [9171038] CinematicFunction_Intro_Func159 [] {
        CinematicFunction_Intro_Func160(j, word!(game.substate), game);
        word!(game.cinematicbg_arr7[(j >> 1)] = 156);
        CinematicFunction_Intro_Func158(j, game);
    };
    [9171053] CinematicFunction_Intro_Func161 [] {
        CinematicFunction_Intro_Func160(j, word!(game.suit_pickup_light_beam_pos), game);
        word!(game.cinematicbg_arr7[(j >> 1)] = 164);
        CinematicFunction_Intro_Func158(j, game);
    };
    [9171068] CinematicFunction_Intro_Func162 [] {
        word!(game.cinematicbg_arr7[(j >> 1)] = 172);
        CinematicFunction_Intro_Func158(j, game);
    };
    [9171077] CinematicFunction_Intro_Func163 [] {
        CinematicFunction_Intro_Func160(j, *(ram_ptr!(game, u16; 0xdf0)), game);
        word!(game.cinematicbg_arr7[(j >> 1)] = 180);
        CinematicFunction_Intro_Func158(j, game);
    };
    [9171092] CinematicFunction_Intro_Func164 [] {
        CinematicFunction_Intro_Func160(j, *(ram_ptr!(game, u16; 0xdf2)), game);
        word!(game.cinematicbg_arr7[(j >> 1)] = 188);
        CinematicFunction_Intro_Func158(j, game);
    };
    [9171122] CinematicFunction_Intro_Func165 [] {
        word!(game.cinematicbg_arr8[(j >> 1)] = -96i32 as u16);
        CinematicFunction_F0B8(j, game);
    };
    [9171169] CinematicFunction_Intro_Func166 [] {
        word!(game.cinematicbg_arr8[(j >> 1)] = -32i32 as u16);
        CinematicFunction_F0B8(j, game);
    };
    [9171177] CinematicFunction_Intro_Func167 [] {
        word!(game.cinematicbg_arr8[(j >> 1)] = 288);
        CinematicFunction_F0B8(j, game);
    };
    [9171185] CinematicFunction_Intro_Func168 [] {
        word!(game.cinematicbg_arr8[(j >> 1)] = 352);
        CinematicFunction_F0B8(j, game);
    };
    @functions [pub unsafe fn (j: u16, game: &mut Game)];
    @prefix { let v1: i32 = j as i32 >> 1; };
    [9149210] CinematicSpriteInit_7 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 48;
            cinematicbg_arr9[v1] = 1024; }
    };
    [9149229] CinematicSpriteInit_8 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 81;
            cinematicbg_arr9[v1] = 0; }
    };
    [9149248] CinematicSpriteInit_9 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 196;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9149628] CinematicSpriteInit_0 [] {
        word! {game; cinematicbg_arr7[v1] = 129; cinematicbg_arr8[v1] = 112;
            cinematicbg_arr9[v1] = 512; }
    };
    [9150131] CinematicSpriteInit_4 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 48;
            cinematicbg_arr9[v1] = 1024; }
        SpawnPalfxObject(addr_kPalfx_E194 as u16, game);
    };
    [9150166] CinematicSpriteInit_5 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 81;
            cinematicbg_arr9[v1] = 512; }
        SpawnPalfxObject(addr_kPalfx_E198 as u16, game);
    };
    [9150207] CinematicSpriteInit_6 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 196;
            cinematicbg_arr9[v1] = 2048; }
        SpawnPalfxObject(addr_kPalfx_E19C as u16, game);
    };
    [9152725] SetSomeStuffForSpriteObject_3 [] {
        word! {game; cinematicbg_arr7[v1] = 112; cinematicbg_arr8[v1] = 155;
            cinematicbg_arr9[v1] = 3584; }
    };
    [9153178] SetSomeStuffForSpriteObject_8 [] {
        word! {game; cinematicspr_goto_timer[v1] = word!(game.cinematic_spawn_param);
            cinematicbg_arr7[v1] = word!(game.cinematicbg_arr7[13]);
            cinematicbg_arr8[v1] = word!(game.cinematicbg_arr8[13]); cinematicbg_arr9[v1] = 3584; }
    };
    [9153877] SetSomeStuffForSpriteObject_10 [] {
        word! {game; cinematicbg_arr7[v1] = 84; cinematicbg_arr8[v1] = 139;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9153939] SetSomeStuffForSpriteObject_12 [] {
        word! {game; cinematicbg_arr7[v1] = 112; cinematicbg_arr8[v1] = 111;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9154055] SetSomeStuffForSpriteObject_17 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 216;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9156467] CinematicFunction_Intro_Func35 [] {
        word! {game; cinematicbg_arr7[v1] = 56; cinematicbg_arr8[v1] = 111;
            cinematicbg_arr9[v1] = 3584; }
    };
    [9158270] CinematicFunction_Intro_Func57 [] {
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicspr_preinstr_func[v1] = ((0x8bbfc6i32) as u16));
            word!(game.cinematicbg_arr7[v1] = -32i32 as u16);
        } else {
            word!(game.cinematicspr_goto_timer[v1] = -1024i32 as u16);
            word!(game.cinematicbg_arr7[v1] = 112);
        }
        word! {game; cinematicbg_arr8[v1] = 87; cinematicbg_arr9[v1] = 2048; }
    };
    [9158434] CinematicFunction_Intro_Func59 [] {
        word! {game; cinematicbg_arr7[v1] = 80; cinematicbg_arr8[v1] = 159;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9158476] CinematicFunction_Intro_Func61 [] {
        word! {game; cinematicbg_arr7[v1] = 116; cinematicbg_arr8[v1] = 160;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9158518] CinematicFunction_Intro_Func63 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 96;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9158787] CinematicFunction_Intro_Func68 [] {
        word! {game; cinematicbg_arr7[v1] = 124; cinematicbg_arr8[v1] = 186;
            cinematicbg_arr9[v1] = 0; }
    };
    [9158834] CinematicFunction_Intro_Func71 [] {
        word! {game; cinematicbg_arr7[v1] = 124; cinematicbg_arr8[v1] = 204;
            cinematicbg_arr9[v1] = 0; }
    };
    [9160105] CinematicFunction_Intro_Func83 [] {
        word! { game;
            cinematicbg_arr7[v1] = (word!(game.reg_M7X)).wrapping_sub(word!(game.cinematic_var8));
            cinematicbg_arr8[v1] = (word!(game.reg_M7Y)).wrapping_sub(word!(game.cinematic_var10));
            cinematicbg_arr9[v1] = 2560;
        }
    };
    [9160763] CinematicFunction_Intro_Func89 [] {
        word! {game; cinematicbg_arr7[v1] = 136; cinematicbg_arr8[v1] = 111;
            cinematicbg_arr9[v1] = 3584; }
    };
    [9160855] CinematicFunction_Intro_Func92 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 127;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9161026] CinematicFunction_Intro_Func98 [] {
        word! {game; cinematicbg_arr7[v1] = 48; cinematicbg_arr8[v1] = 47;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9161046] CinematicFunction_Intro_Func99 [] {
        word! {game; cinematicbg_arr7[v1] = 208; cinematicbg_arr8[v1] = 47;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9161066] CinematicFunction_Intro_Func100 [] {
        word! {game; cinematicbg_arr7[v1] = 48; cinematicbg_arr8[v1] = 207;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9161086] CinematicFunction_Intro_Func101 [] {
        word! {game; cinematicbg_arr7[v1] = 208; cinematicbg_arr8[v1] = 207;
            cinematicbg_arr9[v1] = 2048; }
    };
    [9161106] CinematicFunction_Intro_Func102 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 186;
            cinematicbg_arr9[v1] = 0; }
    };
    [9161190] CinematicFunction_Intro_Func104 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 204;
            cinematicbg_arr9[v1] = 0; }
    };
    [9170841] CinematicFunction_Intro_Func150 [] {
        word! { game, 128; cinematicbg_arr8[v1], cinematicbg_arr7[v1] }
        word! {game; cinematicbg_arr9[v1] = 3584; cinematicspr_goto_timer[v1] = 260; }
    };
    [9170922] CinematicFunction_Intro_Func152 [] {
        word! { game, 128; cinematicbg_arr7[v1], cinematicbg_arr8[v1] }
        word!(game.cinematicbg_arr9[v1] = 3584);
    };
    [9170941] CinematicFunction_Intro_Func153 [] {
        word! {game; cinematicbg_arr7[v1] = -128i32 as u16; cinematicbg_arr8[v1] = 128;
            cinematicbg_arr9[v1] = 3584; }
    };
    [9170949] CinematicFunction_Intro_Func154 [] {
        word! { game, 128; cinematicbg_arr7[v1], cinematicbg_arr8[v1] }
        word!(game.cinematicbg_arr9[v1] = 2560);
    };
    [9170968] CinematicFunction_Intro_Func155 [] {
        word! { game, 128; cinematicbg_arr7[v1], cinematicbg_arr8[v1] }
        word!(game.cinematicbg_arr9[v1] = 3072);
    };
    [9170987] CinematicFunction_Intro_Func156 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 96;
            cinematicbg_arr9[v1] = 1024; }
    };
    [9171006] CinematicFunction_Intro_Func157 [] {
        word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr8[v1] = 160;
            cinematicbg_arr9[v1] = 512; }
    };
    CinematicFunction_Intro_Func158 [] {
        word! {game; cinematicbg_arr8[v1] = 160; cinematicbg_arr9[v1] = 0; }
    };
    CinematicFunction_F0B8 [] {
        if word!(game.cinematic_spawn_param) != 0 {
            word! {game; cinematicbg_arr7[v1] = 384; cinematicbg_arr9[v1] = 2560; }
            word!(game.cinematicspr_instr_ptr[v1] += 4);
        } else {
            word! {game; cinematicbg_arr7[v1] = 128; cinematicbg_arr9[v1] = 2560; }
        };
    };
    [9171193] CinematicFunction_Intro_Func169 [] {
        word!(game.cinematicbg_arr7[v1] = 320);
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicbg_arr8[v1] = 448);
        } else {
            word!(game.cinematicbg_arr8[v1] = 192);
        }
        word!(game.cinematicbg_arr9[v1] = 2560);
    };
    [9171230] CinematicFunction_Intro_Func170 [] {
        word!(game.cinematicbg_arr7[v1] = -64i32 as u16);
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicbg_arr8[v1] = -192i32 as u16);
        } else {
            word!(game.cinematicbg_arr8[v1] = 64);
        }
        word!(game.cinematicbg_arr9[v1] = 2560);
    };
    [9171267] CinematicFunction_Intro_Func171 [] {
        word! {game; cinematicbg_arr7[v1] = 120; cinematicbg_arr8[v1] = 136;
            cinematicbg_arr9[v1] = 2560; }
    };
    [9171286] CinematicFunction_Intro_Func172 [] {
        word! {game; cinematicbg_arr7[v1] = 120; cinematicbg_arr8[v1] = 152;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9171305] CinematicFunction_Intro_Func173 [] {
        word! {game; cinematicbg_arr7[v1] = 124; cinematicbg_arr8[v1] = 108;
            cinematicbg_arr9[v1] = 3072; }
    };
    [9171324] CinematicFunction_Intro_Func174 [] {
        word! {game; cinematicbg_arr7[v1] = 121; cinematicbg_arr8[v1] = 107;
            cinematicbg_arr9[v1] = 2560; }
    };
    [9171343] CinematicFunction_Intro_Func175 [] {
        word! {game; cinematicbg_arr7[v1] = 530; cinematicbg_arr8[v1] = 231;
            cinematicbg_arr9[v1] = 3584; cinematicspr_goto_timer[v1] = 8; }
    };
    [9171368] CinematicFunction_Intro_Func176 [] {
        word! {game; cinematicbg_arr7[v1] = 246; cinematicbg_arr8[v1] = 519;
            cinematicbg_arr9[v1] = 3584; cinematicspr_goto_timer[v1] = 8; }
    };
    [9171393] CinematicFunction_Intro_Func177 [] {
        word! {game; cinematicbg_arr7[v1] = 385; cinematicbg_arr8[v1] = 366;
            cinematicbg_arr9[v1] = 3584; }
    };
    [9171412] CinematicFunction_Intro_Func178 [] {
        word! {game; cinematicbg_arr7[v1] = 391; cinematicbg_arr8[v1] = 384;
            cinematicbg_arr9[v1] = 3584; }
    };
    @external {
        9147353 => [()];
        9149770 => CinematicSpriteInit_1; 9149891 => CinematicSpriteInit_2;
        9150021 => CinematicSpriteInit_3;
        9157195 => CinematicFunction_Intro_Func49;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallIntroObjectInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9148926] IntroObject_Delete [] {
        word!(game.cinematic_var21 = 0);
        0
    };
    [9148934] IntroObject_Goto [] {
        *(RomPtr_8C(j, game) as *mut u16)
    };
    [9148941] IntroObject_DecTimerGoto [] {
        if word!(game.cinematic_var23 -= 1, reread) != 0 {
            IntroObject_Goto(k, j, game)
        } else {
            (j).wrapping_add(2)
        }
    };
    [9148951] IntroObject_SetTimer [] {
        word!(game.cinematic_var23 = *(RomPtr_8C(j, game) as *mut u16));
        (j).wrapping_add(2)
    };
    [9172734] CinematicFunction_Intro_Func219 [] {
        DisableTextGlowObjects_(game);
        word_transfers!(game, ascending, 252; le(palette_buffer, 4) <- rom(0x8ce7e9, 4));
        *ram_ptr!(game; 0x69) = 0;
        *ram_ptr!(game; 0x51) = 0x80;
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, cinematic_var9, cinematic_var10,
        }
        word! {game; eproj_pre_instr[1] = 1; cinematic_function = ((0x8be0f4i32) as u16);
            cinematic_var4 = 60; }
        j
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallCinematicSpriteInit(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        9149210 => CinematicSpriteInit_7; 9149229 => CinematicSpriteInit_8;
        9149248 => CinematicSpriteInit_9; 9149628 => CinematicSpriteInit_0;
        9149770 => CinematicSpriteInit_1; 9149891 => CinematicSpriteInit_2;
        9150021 => CinematicSpriteInit_3; 9150131 => CinematicSpriteInit_4;
        9150166 => CinematicSpriteInit_5; 9150207 => CinematicSpriteInit_6; 9147353 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMode7Instr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (k, j, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9147755] Mode7Instr_Delete [0] { word!(game.mode7_obj_instr_ptr[(k >> 1)] = 0); };
    [9147762] Mode7Instr_SetPreInstr [(j).wrapping_add(2)] {
        word!(game.mode7_obj_preinstr_func[(k >> 1)] = *(RomPtr_8B(j, game) as *mut u16));
    };
    [9147773] Mode7Instr_ClearPreInstr [j] {
        word!(game.mode7_obj_preinstr_func[(k >> 1)] = ((0x8b9585i32) as u16));
    };
    [9147782] Mode7Instr_Goto [*(RomPtr_8B(j, game) as *mut u16)] { };
    [9147789] Mode7Instr_DecTimerAndGoto [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh0 [fresh1] = &raw mut (*(ram_ptr!(game, u16; 0x19ad)).offset(v2 as isize)) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
        if fresh1 == 1 {
            (j).wrapping_add(2)
        } else {
            Mode7Instr_Goto(k, j, game)
        }
    };
    [9147799] Mode7Instr_SetTimer [(j).wrapping_add(2)] {
        word!(game.mode7_obj_goto_timer[(k >> 1)] = *(RomPtr_8B(j, game) as *mut u16));
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9151067] PlayBabyMetroidCry1 [] {
        QueueSfx3_Max6(0x23, game);
        j
    };
    [9151075] PlayBabyMetroidCry2 [] {
        QueueSfx3_Max6(0x26, game);
        j
    };
    [9151083] PlayBabyMetroidCry3 [] {
        QueueSfx3_Max6(0x27, game);
        j
    };
    [9151091] CinematicSetPal1 [] {
        word_transfers!(game, descending, 16; le(palette_buffer, 63) <- table(kCinematicPal1, 15));
        j
    };
    [9151108] CinematicSetPal2 [] {
        word_transfers!(game, descending, 16; le(palette_buffer, 63) <- table(kCinematicPal2, 15));
        j
    };
    [9151125] CinematicSetPal3 [] {
        word_transfers!(game, descending, 16; le(palette_buffer, 63) <- table(kCinematicPal3, 15));
        j
    };
    [9151142] CinematicSetPal4 [] {
        word_transfers!(game, descending, 16; le(palette_buffer, 63) <- table(kCinematicPal4, 15));
        j
    };
    @external {
        9155870 => [intro_pages::draw_japanese_text(&[addr_word_8BCF75 as u16, addr_word_8BCFBD as u16], true, j, game)];
        9155919 => [intro_pages::enable_text(j, game)]; 9155924 => [intro_pages::draw_japanese_text(&[addr_word_8BCFFD as u16, addr_word_8BD055 as u16], true, j, game)];
        9156024 => [intro_pages::wait_after_japanese_text(0xaf6c, j, game)];
        9156035 => [intro_pages::draw_japanese_text(&[addr_word_8BD0F9 as u16], true, j, game)]; 9156123 => [intro_pages::wait_after_japanese_text(0xb0f2, j, game)];
        9156134 => [intro_pages::draw_japanese_text(&[addr_word_8BD1B9 as u16, addr_word_8BD215 as u16], true, j, game)]; 9156234 => [intro_pages::wait_after_japanese_text(0xb123, j, game)];
        9156245 => [intro_pages::draw_japanese_text(&[addr_word_8BD2D5 as u16], true, j, game)]; 9156333 => [intro_pages::wait_after_japanese_text(0xb1da, j, game)];
        9156344 => [intro_pages::draw_japanese_text(&[addr_word_8BD371 as u16], false, j, game)]; 9156363 => [intro_pages::enable_text(j, game)];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMode7PreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        9147353 => [()]; 9155973 => [intro_pages::continue_japanese_text(54181, &[addr_word_8BD085 as u16, addr_word_8BD0E1 as u16], k, game)];
        9156078 => [intro_pages::continue_japanese_text(54203, &[addr_word_8BD15D as u16], k, game)]; 9156183 => [intro_pages::continue_japanese_text(54225, &[addr_word_8BD259 as u16, addr_word_8BD2A5 as u16], k, game)];
        9156288 => [intro_pages::continue_japanese_text(addr_off_8BD3E7 as u16, &[addr_word_8BD30D as u16], k, game)];
    }
}
