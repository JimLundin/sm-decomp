use super::intro_pages;
use crate::game::music::{HasQueuedMusic, QueueMusic_Delayed8, QueueMusic_DelayedY};
pub use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::save_slots;
use crate::game::sprite_drawing::Clip;
use crate::game::visual_runtime::{
    METROID_PAL_1, METROID_PAL_2, METROID_PAL_3, METROID_PAL_4, advance_ceres_station,
    advance_credit_roll, advance_intro_baby_pursuit, advance_intro_rinka,
    advance_postcredits_samus_message, advance_shooting_stars, advance_zebes_destruction_zoom,
    advance_zebes_stars, chase_intro_baby_y, draw_cinematic_sprites, draw_intro_sprites,
    drift_cinematic_sprite, fade_postcredits_samus_suit, initialize_ceres_explosion,
    initialize_ending_cloud_band, initialize_ending_cloud_side, initialize_ending_time_glyph,
    initialize_intro_brain_particle, initialize_intro_rinka, initialize_shooting_stars,
    initialize_title_sprite, launch_cinematic_sprite, move_ending_cloud, place_cinematic_sprite,
    prepare_credit_roll, prepare_ending_time, start_intro_rinka, title_metroid_cry,
    title_metroid_palette, wait_cinematic_scene, wait_ending_cloud,
};
use crate::game::visual_runtime::{advance_cinematic_background_pre, fade_cinematic_fixed_color};
use crate::game::visual_runtime::{
    advance_cinematic_fade, capture_cinematic_palette, clear_cinematic_background,
    clear_cinematic_colors, clear_cinematic_sprites, clear_hdma, compose_cinematic_palette,
    configure_cinematic_display, enable_cinematic_background, fade_cinematic_colors,
    prepare_cinematic_palette_fade, queue_cinematic_tilemap, reset_cinematic_transform,
    set_cinematic_backdrop, set_cinematic_blend, set_cinematic_tilemap_updates, set_fixed_color,
    set_hdma_enabled, spawn_cinematic_sprite_at, update_cinematic_transform,
};
use crate::game::visual_state::CinematicObjects::{
    Background as CinematicBackground, Mode7 as CinematicMode7, Sprites as CinematicSprites,
};
use crate::game::visual_state::{
    CeresExplosion, CinematicDisplay, CinematicFade, CinematicTilemap,
};

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_60::*;

use crate::game::display_refresh::GraphicsAsset;
use crate::game::explosion_window::spawn_power_bomb;
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::mode7_transfers::enqueue_rom;
use crate::game::music::{QueueSfx1_Max15, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::palette_effects;
use crate::game::player_projectiles::prepare;
use crate::game::session_runtime::initialize_player;
use crate::game::sprite_drawing::{clear_high_table, finish_frame};
use crate::game::visual_runtime::{initialize_intro_wave, initialize_title_gradient};

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
    addr_off_8BEE9B = 61083,
    addr_loc_8BE1B4 = 57780,
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
    addr_word_8BCB3B = 52027,
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
crate::game::constants::typed_constants! { u32;
    fnCinematicFunction_Intro_Initial = 9151381,
    fnCinematicFunctionBlackoutFromCeres = 9158939,
    fnCinematicFunctionEscapeFromCebes = 9163904,
}
pub unsafe fn HandleSamusDuringIntro(game: &mut Game) {
    {
        if word!(game.cinematic_var15) != 0 {
            prepare(game);
            crate::game::session_runtime::player_update_runtime::advance_frame(game);
            if word!(game.samus_invincibility_timer) != 0 {
                word!(game.samus_invincibility_timer -= 1);
            }
            if word!(game.samus_knockback_timer) != 0 {
                word!(game.samus_knockback_timer -= 1);
            }
        }
    }
}
pub unsafe fn InitializeIoDisplayLogo_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::verify_probe::hit("coroutine.logo");
        let resume_state: *mut u8 = ram_ptr!(game; 0x612);
        let mut stage = *resume_state;
        if stage == 0 {
            crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
            word!(game.oam_next_ptr = 0);
            clear_high_table(game);
            finish_frame(game);
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
            set_fixed_color([32, 64, 0x80], game);
            crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
            GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
            GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
            *ram_ptr!(game; 0x51) = 0x80;
            word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce1e9, 0));
            crate::game::display_refresh::enable(true, game);
            word! { game, 1; screen_fade_delay, screen_fade_counter }
        }
        // Stages 1..=6 resume a pending NMI before drawing the next frame.
        while stage <= 6 {
            if stage != 0 {
                let yielded: u8 = crate::game::display_refresh::wait(true, game);
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
                        crate::rtl::write_register_and_mirror(
                            INIDISP as u16,
                            0x8f,
                            0x51,
                            false,
                            game,
                        );
                        break;
                    }
                    _ => {}
                }
            }
            clear_high_table(game);
            word!(game.oam_next_ptr = 0);
            AddNintentoLogoToOam(game);
            stage = match stage {
                0 | 1 => {
                    crate::verify_probe::hit("coroutine.logo.in");
                    if advance_cinematic_fade(CinematicFade::FastIn, game) {
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
                    if advance_cinematic_fade(CinematicFade::FastOut, game) {
                        6
                    } else {
                        5
                    }
                }
            };
            finish_frame(game);
        }
        *resume_state = 0;
        0 as CoroutineRet
    }
}
pub fn AddNintentoLogoToOam(game: &mut Game) {
    sprite_output::paletted(
        0x8c,
        addr_kSpriteMap_NintendoLogo as u16,
        (128, 112),
        0,
        Clip::Bounded,
        game,
    );
}
// Legacy pre-instruction and failure contracts remain in the bank.
pub unsafe fn CreditsObject_Process(game: &mut Game) {
    unsafe {
        if word!(game.cinematic_var25) & 0x8000 != 0 {
            if word!(game.cinematic_var21) != 0 {
                advance_cinematic_background_pre(
                    0x8b0000 | u32::from(word!(game.cinematic_var24)),
                    0,
                    game,
                );
                if !advance_credit_roll(game) {
                    Unreachable(game);
                }
            }
            // Delete and disable still publish this frame's tilemap.
            queue_cinematic_tilemap(CinematicTilemap::FullText, game);
        }
    }
}
pub unsafe fn GameState_1_OpeningCinematic_(game: &mut Game) -> CoroutineRet {
    unsafe {
        CallCinematicFunction(
            ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
            game,
        );
        CinematicSprites.advance(game);
        CinematicMode7.advance(game);
        palette_effects::advance(game);
        draw_cinematic_sprites(false, game);
        MaybeSkipCinematics(game);
        HandleCinematicsTransitions(game);
        update_cinematic_transform(false, game);
        kCoroutineNone as CoroutineRet
    }
}
pub fn CinematicFunctionNone() {}
pub unsafe fn MaybeSkipCinematics(game: &mut Game) {
    {
        if word!(game.cinematic_function) as i32 + 0x60d8 & 0x8000 != 0
            && (word!(game.joypad1_newkeys)
                & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16)
                != 0
            && word!(game.cinematic_var18) == 0
        {
            word! {game; cinematic_var18 = 1; screen_fade_delay = 0; screen_fade_counter = 2; }
        }
    }
}
pub fn HandleCinematicsTransitions_1(game: &mut Game) {
    if advance_cinematic_fade(CinematicFade::FastOut, game) {
        word! {game; cinematic_var18 = 2; cinematic_function = ((0x8b9f28i32) as u16); }
        QueueMusic_Delayed8(6, game);
    }
}
pub unsafe fn HandleCinematicsTransitions_2(game: &mut Game) {
    unsafe {
        clear_cinematic_sprites(game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA119 as u16, 0, game);
        CinematicSprites.spawn(
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
        palette_effects::clear(game);
        palette_effects::spawn(addr_kPalfx_E1A0 as u16, game);
        palette_effects::spawn(addr_kPalfx_E1A4 as u16, game);
        set_cinematic_backdrop(false, game);
        *ram_ptr!(game; 0x69) = 17;
        initialize_title_gradient(game);
    }
}
pub fn HandleCinematicsTransitions_3(game: &mut Game) {
    if advance_cinematic_fade(CinematicFade::FastIn, game) {
        word! {game; cinematic_var18 = 0; demo_timer = 900;
        cinematic_function = ((0x8b9f29i32) as u16); }
    }
}
static off_8B9A7B: [Option<Func_V>; 4] = callback_table! { Func_V;
    None, HandleCinematicsTransitions_1, HandleCinematicsTransitions_2,
    HandleCinematicsTransitions_3,
};
pub unsafe fn HandleCinematicsTransitions(game: &mut Game) {
    unsafe {
        if word!(game.cinematic_var18) != 0 {
            off_8B9A7B[word!(game.cinematic_var18) as usize].expect("non-null function pointer")(
                game,
            );
        }
    }
}
pub unsafe fn LoadTitleSequenceGraphics(game: &mut Game) {
    unsafe {
        configure_cinematic_display(CinematicDisplay::Title, game);
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
        reset_cinematic_transform(128, 128, game);
        crate::game::display_refresh::enable(true, game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        set_hdma_enabled(true, game);
        palette_effects::set_enabled(true, game);
        word!(game.fx_layer_blending_config_a = 0);
        palette_effects::spawn(addr_kPalfx_E1A0 as u16, game);
        palette_effects::spawn(addr_kPalfx_E1A4 as u16, game);
        CinematicMode7.spawn(
            addr_kMode7ObjBabyMetroidInTitle as u16,
            0x000c, // The second title palette allocation leaves its slot offset in A.
            game,
        );
    }
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
            CinematicSprites.spawn($sprite_definition as u16,
                ((0x8b9a47i32) as u16),
                $game,
            );
            set_cinematic_backdrop(false, $game);
        }
    };
}

macro_rules! finish_cinematic_and_clear_effects {
    ($game:ident, $next_state:literal, $index_field:ident) => {
        crate::game::display_refresh::enable(true, $game);
        *ram_ptr!($game; 0x55) = 1;
        word! { $game, 0;
            screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
            reg_BG3VOFS, cinematic_var5,
        }
        word!($game.cinematic_function = ((0x8b9b68i32) as u16));
        palette_effects::set_enabled(false, $game);
        palette_effects::clear($game);
        word_transfers!($game, descending, 329; native(0x1c1d) <- value(0));
        word_transfers!($game, descending, 256; le(hdma_table_1, 255) <- value(0));
        word!($game.game_state = $next_state);
        word!($game.$index_field = 0);
    };
}

pub unsafe fn GameState_37_CeresGoesBoomWithSamus_(game: &mut Game) -> CoroutineRet {
    unsafe {
        CallCinematicFunction(
            ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
            game,
        );
        if (word!(game.cinematic_var11) & 0x8000) == 0 {
            word!(game.cinematic_var11 += 1);
        }
        word!(game.cinematic_var12 += 1);
        HandleSamusDuringIntro(game);
        CinematicSprites.advance(game);
        CinematicMode7.advance(game);
        CinematicBackground.advance(game);
        intro_pages::advance_text_glow(game);
        palette_effects::advance(game);
        draw_intro_sprites(game);
        update_cinematic_transform(true, game);
        kCoroutineNone as CoroutineRet
    }
}
pub fn CinematicFunc_Nothing() {}
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

pub unsafe fn GameState_39_EndingAndCredits_(game: &mut Game) -> CoroutineRet {
    unsafe {
        CallCinematicFunction(
            ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
            game,
        );
        if (word!(game.cinematic_var11) & 0x8000) == 0 {
            word!(game.cinematic_var11 += 1);
        }
        word!(game.cinematic_var12 += 1);
        CinematicSprites.advance(game);
        MaybeHandleCinematicBgObjects(game);
        CreditsObject_Process(game);
        palette_effects::advance(game);
        draw_cinematic_sprites(true, game);
        advance_shooting_stars(game);
        update_cinematic_transform(true, game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn MaybeHandleCinematicBgObjects(game: &mut Game) {
    unsafe {
        if word!(game.cinematic_function) as i32 + 0x1afc & 0x8000 == 0 {
            CinematicBackground.advance(game);
        }
    }
}
static g_word_8BDE43: [u16; 16] = [
    0, 0x6f7b, 0x4a52, 0x1ce7, 0, 0x5ad6, 0x4a52, 0x35ad, 0x2529, 0x7fff, 0x56b5, 0x294a, 0x4210,
    0x2d6b, 0x18c6, 0x7fff,
];
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicFunction(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9149288] CinematicFunctionOpening [] {
        LoadTitleSequenceGraphics(game);
        QueueMusic_Delayed8(0xff03, game);
        word!(game.cinematic_function = (0x8b9a47i32) as u16);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA0EF as u16,
            (0x8b9a47i32) as u16,
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
            word!(game.cinematic_function = (0x8b9a47i32) as u16);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA101 as u16,
                (0x8b9a47i32) as u16,
                game,
            );
            set_cinematic_backdrop(false, game);
        }
        MaybeSkipCinematics(game);
    };
    [9150091] CinematicFunc_Func7 [] {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            if word!(game.cinematic_var6) as i32 - 256i32 & 0x8000 != 0 {
                word!(game.cinematic_var6 += 1);
            } else {
                word! {game; cinematic_var6 = 256; cinematic_function = ((0x8b9f28i32) as u16); }
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA107 as u16,
                    (0x8b9f28i32) as u16,
                    game,
                );
            }
        }
    };
    [9150290] CinematicFunc_Func10 [] {
        if advance_cinematic_fade(CinematicFade::SlowOut, game) {
            finish_cinematic_and_clear_effects!(game, 4, game_options_screen_index);
        }
    };
    [9150382] CinematicFunc_Func9 [] {
        if advance_cinematic_fade(CinematicFade::SlowOut, game) {
            finish_cinematic_and_clear_effects!(game, 40, demo_scene);
        }
    };
    [9151381] CinematicFunction_Intro_Initial [] {
        configure_cinematic_display(CinematicDisplay::Intro, game);
        word! { game, 0; cur_irq_handler, irqhandler_next_handler, area_index }
        word! { game, 16; room_width_in_blocks, room_height_in_blocks }
        initialize_player(game);
        crate::game::visual_runtime::update_beam_tiles_and_palette(game);
        word! { game, 900; samus_max_missiles, samus_missiles }
        word! {game; cinematic_var15 = 0; samus_draw_handler = ((0x90eb52i32) as u16); }
        crate::game::controller_input::use_cinematic_bindings(game);
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
        crate::game::display_refresh::enable(true, game);
        word! { game, 3; screen_fade_delay, screen_fade_counter }
        set_hdma_enabled(true, game);
        palette_effects::clear(game);
        palette_effects::set_enabled(true, game);
        word! { game, 0; door_def_ptr, layer1_x_pos, layer1_y_pos, cinematic_var16 }
        word_transfers!(game, descending, 1024; cinematic_bg(1023) <- native(0x197fe));
        word_transfers!(game, descending, 128; pause_map(127) <- rom(0x8cd81b, 127));
        word! {game; cinematic_var11 = -1i32 as u16; cinematic_function = ((0x8ba5a7i32) as u16); }
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff3f, game);
    };
    [9151911] CinematicFunction_Intro_FadeIn [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word!(game.cinematic_function = (0x8ba5bdi32) as u16);
            word! { game, 2; screen_fade_delay, screen_fade_counter }
        }
    };
    [9151933] CinematicFunction_Intro_DrawInitJpn [] {
        if advance_cinematic_fade(CinematicFade::SlowIn, game) {
            word! {game; cinematic_function = ((0x8ba5f8i32) as u16); cinematic_var4 = 60; }
            if word!(game.japanese_text_flag) != 0 {
                queue_cinematic_tilemap(CinematicTilemap::JapaneseIntro, game);
            }
        }
    };
    [9152057] CinematicFunc_Intro_WaitForQueueWait4secs [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word! {game; cinematic_function = ((0x8ba64ci32) as u16); cinematic_var4 = 240; }
        }
    };
    [9152099] CinematicFunc_Intro_WaitForFadeOut [] {
        if advance_cinematic_fade(CinematicFade::SlowOut, game) {
            word!(game.cinematic_function = (0x8ba66fi32) as u16);
        }
    };
    [9152111] CinematicFunc_Intro_SetupTextPage2 [] {
        intro_pages::clear_japanese_tiles(game);
        GraphicsAsset::VramWords(0x7e4000, 0x4180, 0x600).load(game);
        crate::rtl::write_register_and_mirror(TM as u16, 0x16, 0x69, false, game);
        clear_cinematic_background(0x2f, game);
        enable_cinematic_background(game);
        set_cinematic_tilemap_updates(true, game);
        word!(game.cinematicbg_var3 = addr_unk_604C00 as u16);
        word_transfers!(game, descending, 128;
            pause_map(127) <- value(15401),
            pause_map(1023) <- value(15401),
        );
        word_transfers!(game, ascending, 128; pause_map(768) <- rom(0x8ba72b, 0));
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8).offset(286)
            as *mut u32) = 472456233;
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCE6D as u16, 0x1c29, game);
        word!(game.cinematic_var4 = 127);
        CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BCF63 as u16, 0x4c00, game);
        word! { game, 8; cinematic_var10, reg_BG2VOFS, reg_BG3VOFS }
        intro_pages::clear_english_text(game);
        intro_pages::reset_caret(game);
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff36, game);
        QueueMusic_DelayedY(5, 0xe, game);
        word!(game.cinematic_function = (0x8ba82bi32) as u16);
    };
    [9152555] CinematicFunc_Intro_WaitFadeinShowText [] {
        if HasQueuedMusic(game) & 1 == 0 {
            word!(game.cinematic_function = (0x8ba84ai32) as u16);
            word! { game, 2; screen_fade_delay, screen_fade_counter }
            CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BCF3F as u16, 0x4c00, game);
        }
    };
    [9152586] CinematicFunc_Intro_WaitForFadeinSleep [] {
        if advance_cinematic_fade(CinematicFade::SlowIn, game) {
            word!(game.cinematic_function = (0x8ba391i32) as u16);
        }
    };
    [9157792] CinematicFunction_Intro_Func54 [] {
        configure_cinematic_display(CinematicDisplay::Station, game);
        clear_cinematic_sprites(game);
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
        reset_cinematic_transform(56, 24, game);
        word! {game; cinematic_var8 = -72i32 as u16; cinematic_var10 = -104i32 as u16;
            cinematic_var5 = 224; cinematic_var6 = 512; }
        palette_effects::clear(game);
        palette_effects::set_enabled(true, game);
        clear_cinematic_background(0, game);
        word!(game.cinematicbg_var3 = 23552);
        crate::game::display_refresh::enable(true, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 0, game);
        word!(game.cinematic_function = (0x8bbde4i32) as u16);
        QueueMusic_Delayed8(0xff2d, game);
        QueueMusic_DelayedY(5, 0xe, game);
    };
    [9158116] CinematicFunction_Intro_Func55 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            *ram_ptr!(game; 0x51) = 15;
            word!(game.cinematic_function = (0x8bbdf9i32) as u16);
        }
    };
    [9158137] CinematicFunction_Intro_Func56 [] {
        if word!(game.cinematic_var6) as i32 - 32i32 & 0x8000 != 0 {
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 49;
            set_fixed_color([63, 95, -97i32 as u8], game);
            clear_cinematic_sprites(game);
            enqueue_rom(0x8b, addr_kCinematicFunction_Intro_Func56_M7 as u16, game);
            word! {game; cinematic_var8 = -32i32 as u16; cinematic_var10 = -128i32 as u16;
                cinematic_var5 = 32; }
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF39 as u16,
                0x0020, // The preceding $198D store leaves $0020 in A.
                game,
            );
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCE85 as u16,
                0x0800, // Palette left in A by the preceding sprite initializer.
                game,
            );
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCE8B as u16,
                0x0c00, // Palette left in A by the preceding sprite initializer.
                game,
            );
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCE91 as u16, 1, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF0F as u16, 1, game);
            word!(game.cinematic_function = (0x8bbfdai32) as u16);
            palette_effects::spawn(addr_kPalfx_E1A8 as u16, game);
            palette_effects::spawn(addr_kPalfx_E1AC as u16, game);
        } else {
            word!(game.cinematic_var6 -= 16);
        };
    };
    [9158618] CinematicFunction_Intro_Func67 [] {
        fade_cinematic_fixed_color(game);
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
            enable_cinematic_background(game);
            set_cinematic_tilemap_updates(true, game);
            CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BCF69 as u16, 0x5c00, game);
            word!(game.cinematic_function = (0x8ba38fi32) as u16);
        };
    };
    [9158853] CinematicFunction_Intro_Func72 [] {
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0;
                screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
                reg_BG3VOFS, cinematic_var5,
            }
            word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
            word!(game.cinematic_function = (0x8bc100i32) as u16);
        }
    };
    [9158912] CinematicFunction_Intro_Func73 [] {
        word! {game; loading_game_state = kGameState_31_SetUpNewGame as u16;
            game_state = kGameState_31_SetUpNewGame as u16; area_index = 6;
            load_station_index = 0; }
        save_slots::save(word!(game.selected_save_slot), game);
    };
    [9158939] CinematicFunctionBlackoutFromCeres [] {
        configure_cinematic_display(CinematicDisplay::Station, game);
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
        reset_cinematic_transform(52, 48, game);
        word! {game; cinematic_var8 = -44i32 as u16; cinematic_var10 = -112i32 as u16;
            cinematic_var5 = 0; cinematic_var6 = 256; }
        set_hdma_enabled(false, game);
        clear_hdma(game);
        palette_effects::clear(game);
        palette_effects::set_enabled(true, game);
        clear_cinematic_background(0x2f, game);
        word!(game.cinematicbg_var3 = 0);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCE7F as u16, 0, game);
        spawn_cinematic_sprite_at(
            addr_kCinematicSpriteObjectDef_8BCE8B as u16,
            2,
            0x0002,
            game,
        );
        spawn_cinematic_sprite_at(
            addr_kCinematicSpriteObjectDef_8BCE91 as u16,
            0,
            0x0000,
            game,
        );
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF33 as u16,
            0x0800, // Palette left in A by the preceding sprite initializer.
            game,
        );
        palette_effects::spawn(addr_kPalfx_E1B8 as u16, game);
        crate::game::display_refresh::enable(true, game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = (0x8bc2e4i32) as u16);
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
            word!(game.cinematic_function = (0x8bc2f1i32) as u16);
        }
    };
    [9159409] CinematicFunction_Intro_Func75 [] {
        words!(game.cinematic_var10, cinematic_var9 += 4096);
        words!(game.cinematic_var8, cinematic_var7 += -0x4000i32 as u32);
        word!(game.cinematic_var6 += 1);
        crate::game::display_refresh::fade(true, game);
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
            set_hdma_enabled(true, game);
            spawn_power_bomb(game);
            *ram_ptr!(game; 0x6f) = 16;
            *ram_ptr!(game; 0x72) = 55;
            word!(game.cinematic_var6 = 768);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF2D as u16, 0, game);
            if word!(game.game_state) == 37 {
                enqueue_rom(0x8b, addr_kCinematicFunction_Intro_Func76_M7_2 as u16, game);
            } else {
                enqueue_rom(0x8b, addr_kCinematicFunction_Intro_Func76_M7_0 as u16, game);
            }
            enqueue_rom(0x8b, addr_kCinematicFunction_Intro_Func76_M7_1 as u16, game);
            word!(game.cinematic_function = (0x8bc5cai32) as u16);
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
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            if word!(game.game_state) as i32 == kGameState_37_CeresGoesBoomWithSamus as i32 {
                QueueMusic_Delayed8(0, game);
                word! { game, 0;
                    hud_item_index, samus_auto_cancel_hud_item_index, samus_invincibility_timer,
                    samus_knockback_timer,
                }
                clear_hdma(game);
                crate::game::scanline_irq::disable(game);
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
                word!(game.cinematic_function = (0x8bc699i32) as u16);
            }
        }
    };
    [9160345] CinematicFunction_Intro_Func86 [] {
        configure_cinematic_display(CinematicDisplay::Landing, game);
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        GraphicsAsset::Decompress(0x978adb, 0x19000).load(game);
        GraphicsAsset::Decompress(0x96ec76, 0x15000).load(game);
        GraphicsAsset::VramLowBytes(0x7f4300, 0x0000, 0x300).load(game);
        GraphicsAsset::VramWords(0x7f9000, 0x5c00, 0x800).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        reset_cinematic_transform(56, 24, game);
        word! { game, 0; cinematic_var8, cinematic_var10, cinematic_var5 }
        word!(game.cinematic_var6 = 256);
        set_hdma_enabled(false, game);
        clear_hdma(game);
        palette_effects::clear(game);
        palette_effects::set_enabled(true, game);
        *ram_ptr!(game; 0x57) = -127i32 as u8;
        palette_effects::spawn(addr_kPalfx_E1A8 as u16, game);
        word!(game.cinematic_function = (0x8bc79ci32) as u16);
        crate::game::display_refresh::enable(true, game);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
    };
    [9160604] CinematicFunction_Intro_Func87 [] {
        if (word!(game.cinematic_var12) & 3) == 0 {
            crate::update_place! { reference fresh116 = *ram_ptr!(game; 0x57) => (*fresh116).wrapping_sub(16); }
        }
        crate::game::display_refresh::fade(true, game);
        if *ram_ptr!(game; 0x51) == 15 {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = (0x8bc7cai32) as u16);
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
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEA3 as u16,
                    0x0100, // The preceding $198F store leaves $0100 in A.
                    game,
                );
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEF7 as u16,
                    0x0e00, // Palette left in A by the preceding sprite initializer.
                    game,
                );
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEFD as u16,
                    0x0800, // Palette left in A by the preceding sprite initializer.
                    game,
                );
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF03 as u16,
                    0x0800, // Palette left in A by the preceding sprite initializer.
                    game,
                );
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF09 as u16,
                    0x0800, // Palette left in A by the preceding sprite initializer.
                    game,
                );
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEAF as u16,
                    0x0800, // Palette left in A by the preceding sprite initializer.
                    game,
                );
                word!(game.cinematic_function = (0x8ba38fi32) as u16);
            }
        }
    };
    [9161209] CinematicFunction_Intro_Func105 [] {
        words!(game.cinematic_var10, cinematic_var9 += 0x2000);
        words!(game.cinematic_var8, cinematic_var7 += -0x8000i32 as u32);
        if word!(game.cinematic_var6) as i32 - 1152i32 & 0x8000 != 0 {
            word!(game.cinematic_var6 += 4);
        } else {
            word!(game.cinematic_function = (0x8bca36i32) as u16);
        };
    };
    [9161270] CinematicFunction_Intro_Func106 [] {
        words!(game.cinematic_var10, cinematic_var9 += 0x2000);
        words!(game.cinematic_var8, cinematic_var7 += -0x8000i32 as u32);
        if word!(game.cinematic_var8) as i32 + 128 & 0x8000 != 0 {
            word!(game.cinematic_function = (0x8bca85i32) as u16);
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
        configure_cinematic_display(CinematicDisplay::Escape, game);
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
        reset_cinematic_transform(128, 128, game);
        word! {game; cinematic_var5 = 32; cinematic_var6 = 64; }
        word! { game, 0;
            cinematic_var8, cinematic_var10, door_def_ptr, layer1_x_pos, layer1_y_pos,
        }
        palette_effects::clear(game);
        palette_effects::set_enabled(true, game);
        crate::game::display_refresh::enable(true, game);
        word!(game.eproj_pre_instr[1] = 0);
        set_cinematic_blend(17, 1, 2, 17, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEEB as u16, 1, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEF1 as u16, 1, game);
        word! {game; screen_fade_delay = 0; screen_fade_counter = 2;
            cinematic_function = ((0x8bd6d7i32) as u16); }
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff33, game);
        QueueMusic_DelayedY(5, 0xe, game);
    };
    [9164503] CinematicFunction_Intro_Func109 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            palette_effects::spawn(addr_kPalfx_E1D8 as u16, game);
            palette_effects::spawn(addr_kPalfx_E1DC as u16, game);
            word!(game.cinematic_function = (0x8bd6f2i32) as u16);
        }
    };
    [9164593] CinematicFunction_Intro_Func112 [] {
        GraphicsAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
        GraphicsAsset::VramWords(0x7e2000, 0x2000, 0x4000).load(game);
        GraphicsAsset::Decompress(0x98ed4f, 0x2000).load(game);
        GraphicsAsset::VramHighBytes(0x7e2000, 0x0000, 0x4000).load(game);
        palette_effects::clear(game);
        clear_cinematic_sprites(game);
        palette_effects::spawn(addr_kPalfx_E1E0 as u16, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEED3 as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEED9 as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEDF as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEE5 as u16, 0, game);
        word! {game; cinematic_var5 = 32; cinematic_var6 = 64; }
        word! { game, 0; cinematic_var8, cinematic_var10 }
        word!(game.cinematic_function = (0x8bd7f8i32) as u16);
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
        palette_effects::clear(game);
        clear_cinematic_sprites(game);
        word_transfers!(game, ascending, 128; le(palette_buffer, 128) <- rom(0x8cebe9, 128));
        // The palette copy and each initializer leave A for the next spawn.
        let palette_tail = word!(game.palette_buffer[255]);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEE9D as u16, palette_tail, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEAF as u16, 0x0e00, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEA3 as u16, 0x0a00, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEA9 as u16, 0x0e00, game);
        word! {game; cinematic_var5 = 0; cinematic_var6 = 64; reg_M7A = 256; }
        word! { game, 0; reg_M7B, reg_M7C }
        word!(game.reg_M7D = 256);
        word! { game, 128; reg_M7X, reg_M7Y }
        word! { game, 0; cinematic_var8, cinematic_var10 }
        set_cinematic_blend(1, 16, 2, 33, game);
        capture_cinematic_palette(game);
        prepare_cinematic_palette_fade(game);
        clear_cinematic_colors(0x1a0, 0x10, game);
        clear_cinematic_colors(0x1e0, 0x10, game);
        compose_cinematic_palette(game);
        word! {game; cinematic_var4 = 63; cinematic_function = ((0x8bda02i32) as u16); }
    };
    [9165314] CinematicFunction_Intro_Func116 [] {
        CinematicFunction_Intro_Func117(game);
        if advance_cinematic_fade(CinematicFade::FastIn, game) {
            word! { game, 2; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = (0x8bda1ai32) as u16);
        }
    };
    [9165338] CinematicFunction_Intro_Func117 [] {
        word!(game.cinematic_var6 += 4);
        if (word!(game.cinematic_var4) & 1) == 0 {
            fade_cinematic_colors(0xe0, 0x10, false, game);
            fade_cinematic_colors(0x1a0, 0x10, true, game);
            fade_cinematic_colors(0x1e0, 0x10, true, game);
            compose_cinematic_palette(game);
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
            set_fixed_color([32, 64, 0x80], game);
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
    [9165764] CinematicFunction_Intro_Func120 [] {
        if HasQueuedMusic(game) & 1 == 0 {
            *ram_ptr!(game; 0x55) = 7;
            *ram_ptr!(game; 0x5f) = 0x80;
            set_cinematic_blend(17, 0, 0, 48, game);
            set_fixed_color([63, 95, -97i32 as u8], game);
            *ram_ptr!(game; 0x58) = 0;
            *ram_ptr!(game; 0x5d) = 0;
            word! { game, 0; palette_buffer[0], palette_buffer[16], palette_buffer[128] }
            palette_effects::spawn(addr_kPalfx_E1D4 as u16, game);
            palette_effects::spawn(addr_kPalfx_E1E4 as u16, game);
            word! {game; reg_M7X = 56; reg_M7Y = 24; cinematic_var8 = -72i32 as u16;
                cinematic_var10 = -104i32 as u16; cinematic_var6 = 3072;
                cinematic_var5 = -112i32 as u16; cinematic_var4 = 1; cinematic_var13 = 192;
                cinematic_var14 = 0; cinematic_function = ((0x8bdca5i32) as u16); }
        }
    };
    [9166146] CinematicFunction_Intro_Func123 [] {
        if crate::game::visual_runtime::advance_ending_gunship_spin(true, game)
            && CheckEventHappened(0xf, game) != 0
        {
            spawn_cinematic_sprite_at(0xef21, 4, 4, game);
        }
    };
    [9166285] CinematicFunction_Intro_Func124 [] {
        crate::game::visual_runtime::fade_ending_gunship_backdrop(game);
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
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEC7 as u16,
                word!(game.cinematic_var6),
                game,
            );
            word!(game.cinematic_function = (0x8bde63i32) as u16);
            word_transfers!(game, descending, 16; le(palette_buffer, 95) <- table(g_word_8BDE43, 15));
        } else {
            word!(game.cinematic_var6 -= 4);
        };
    };
    [9166436] CinematicFunction_Intro_Func125 [] {
        word!(game.screen_fade_counter = 1);
        if advance_cinematic_fade(CinematicFade::FastOut, game) {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.cinematic_function = (0x8bde80i32) as u16);
        }
    };
    [9166464] CinematicFunction_Intro_Func126 [] {
        configure_cinematic_display(CinematicDisplay::Credits, game);
        clear_cinematic_sprites(game);
        palette_effects::clear(game);
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
        reset_cinematic_transform(104, 112, game);
        word! {game; cinematic_var8 = -8i32 as u16; cinematic_var10 = -24i32 as u16;
            cinematic_var5 = 0; cinematic_var6 = 2304; }
        crate::game::display_refresh::enable(true, game);
        *ram_ptr!(game; 0x51) = 15;
        word! { game, 0; layer1_x_pos, layer1_y_pos }
        set_hdma_enabled(true, game);
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0));
        initialize_shooting_stars(game);
        GraphicsAsset::Decompress(0x97eeff, 0x10000).load(game);
        clear_cinematic_background(0x7f, game);
        let pre = prepare_credit_roll(addr_stru_8BF6F8 as u16, game);
        advance_cinematic_background_pre(pre, 0, game);
        word!(game.cinematic_function = (0x8bdb9di32) as u16);
    };
    [9167120] CinematicFunction_Intro_Func130 [] {
        crate::game::display_refresh::fade(true, game);
        if *ram_ptr!(game; 0x51) == 15 {
            set_cinematic_blend(2, 16, 2, 34, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            capture_cinematic_palette(game);
            prepare_cinematic_palette_fade(game);
            clear_cinematic_colors(0x40, 0x10, game);
            compose_cinematic_palette(game);
            word! {game; cinematic_var4 = 32; cinematic_function = ((0x8be158i32) as u16); }
        }
    };
    [9167192] CinematicFunction_Intro_Func131 [] {
        fade_cinematic_colors(0x40, 0x10, true, game);
        compose_cinematic_palette(game);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            set_cinematic_blend(18, 4, 2, 2, game);
            word! {game; cinematic_var4 = 180; cinematic_function = ((0x8be190i32) as u16); }
            initialize_intro_wave(game);
        }
    };
    [9167248] CinematicFunction_Intro_Func132 [] {
        word!(game.reg_BG3VOFS += 2);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            word_transfers!(game, ascending, 288; pause_map(288) <- rom(0x8cdc9b, 0));
            queue_cinematic_tilemap(CinematicTilemap::FullText, game);
            set_cinematic_blend(1, 0, 0, 0, game);
            word! {game; cinematic_var4 = 180; cinematic_function = ((0x8be1d2i32) as u16);
                g_word_7E0D9C = 0; }
        }
    };
    [9167314] CinematicFunction_Intro_Func134 [] {
        if word!(game.cinematic_var4 -= 1, reread) == 0 {
            word!(game.cinematic_var4 = 127);
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0 {
                capture_cinematic_palette(game);
                prepare_cinematic_palette_fade(game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF27 as u16, 0, game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF2D as u16, 0, game);
            } else {
                if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 == 0 {
                    CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF4B as u16,
                        0,
                        game,
                    );
                    CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF45 as u16,
                        0,
                        game,
                    );
                    word!(game.cinematic_function = (0x8be293i32) as u16);
                    set_cinematic_blend(16, 0, 0, 0, game);
                    return;
                }
                capture_cinematic_palette(game);
                prepare_cinematic_palette_fade(game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF51 as u16, 0, game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF45 as u16, 0, game);
                clear_cinematic_colors(0x1c0, 0x10, game);
            }
            clear_cinematic_colors(0x1a0, 0x10, game);
            compose_cinematic_palette(game);
            set_cinematic_blend(2, 16, 2, 34, game);
            word!(game.cinematic_function = (0x8be265i32) as u16);
        }
    };
    [9167581] CinematicFunction_Intro_Func137 [] {
        let v0: bool = (word!(game.cinematic_var13 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var13) == 0 || v0 {
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0
                || word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0
            {
                set_cinematic_blend(2, 16, 2, 34, game);
                word!(game.cinematic_function = (0x8be314i32) as u16);
            } else {
                word!(game.cinematic_function = (0x8be342i32) as u16);
            }
        }
    };
    [9167636] CinematicFunction_Intro_Func138 [] {
        fade_postcredits_samus_suit(game);
        CinematicFunction_Intro_Func145(game);
    };
    [9167682] CinematicFunction_Intro_Func145 [] {
        if (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0 {
            set_cinematic_blend(16, 0, 0, 0, game);
            *ram_ptr!(game; 0x55) = 7;
            *ram_ptr!(game; 0x5f) = 0x80;
            *ram_ptr!(game; 0x58) = 0;
            *ram_ptr!(game; 0x5d) = 0;
            word! {game; cinematic_function = ((0x8bdb9di32) as u16); cinematic_var14 = 0; }
            if word!(game.game_time_hours) as i32 - 3i32 & 0x8000 != 0 {
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF33 as u16, 0, game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF39 as u16, 0, game);
            } else {
                if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
                    CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF69 as u16,
                        0,
                        game,
                    );
                } else {
                    CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF63 as u16,
                        0,
                        game,
                    );
                }
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF5D as u16, 0, game);
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF57 as u16, 0, game);
            }
        }
    };
    [9168132] CinematicFunction_Intro_Func143 [] {
        fade_cinematic_fixed_color(game);
        let v0: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var4) == 0 || v0 {
            word! {game; cinematic_function = ((0x8be7bai32) as u16); cinematicbg_var3 = 20480; }
            queue_cinematic_tilemap(CinematicTilemap::FullText, game);
            word! { game, 256; layer1_x_pos, layer1_y_pos }
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF81 as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF87 as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF8D as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF93 as u16, 0, game);
            word_transfers!(game, descending, 16; le(palette_buffer, 31) <- value(0));
            word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8cefe9, 15));
        }
    };
    [9168266] CinematicFunction_Intro_Func144 [] {
        if crate::game::visual_runtime::advance_ending_logo_palette(game) {
            word!(game.cinematic_function = (0x8be7bai32) as u16);
            clear_cinematic_sprites(game);
            CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BF74E as u16, 0x4c00, game);
        }
    };
    [9168791] CinematicFunction_Intro_Func148 [] {
        word!(game.cinematic_var10 -= 2);
        if word!(game.cinematic_var10) as i32 + 79 & 0x8000 != 0 {
            CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BF748 as u16, 0x4c00, game);
            word!(game.cinematic_function = (0x8be7bai32) as u16);
            crate::game::save_slots::record_completion(game);
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
        word!(game.cinematic_function = (0x8ba639i32) as u16);
    };
    [9152076] CinematicFunc_Intro_FadeOut {
        word!(game.cinematic_function = (0x8ba663i32) as u16);
        word! { game, 2; screen_fade_delay, screen_fade_counter }
    };
    [9160208] CinematicFunction_Intro_Func84 {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = (0x8bc627i32) as u16);
    };
    [9161424] CinematicFunction_Intro_Func108 {
        word!(game.cinematic_function = (0x8bcadei32) as u16);
    };
    [9165726] CinematicFunction_Intro_Func119 {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff3c, game);
        QueueMusic_DelayedY(5, 0xe, game);
        word!(game.cinematic_function = (0x8bdbc4i32) as u16);
    };
    [9167092] CinematicFunction_Intro_Func129 {
        word! {game; cinematic_function = ((0x8be110i32) as u16); cinematic_var4 = 120; }
        *ram_ptr!(game; 0x69) = 16;
    };
    @external {
        9165523 => [crate::game::mode7_transfers::advance_ending_tile_upload(game)];
        9165989 => [{ crate::game::visual_runtime::advance_ending_gunship_spin(false, game); }];
        9167790 => [crate::game::visual_runtime::advance_postcredits_zoom(true, game)];
        9168010 => [crate::game::visual_runtime::advance_postcredits_zoom(false, game)];
        9150249 => [crate::game::visual_runtime::advance_title_wait(game)];
        9167461 => [advance_postcredits_samus_message(true, game)];
        9167507 => [advance_postcredits_samus_message(false, game)];
        9164530 => [advance_zebes_destruction_zoom(2, 0xd731, Some(0xd701), game)];
        9164545 => [advance_zebes_destruction_zoom(2, 0xd731, None, game)];
        9164792 => [advance_zebes_destruction_zoom(3, 0xd837, Some(0xd807), game)];
        9164807 => [advance_zebes_destruction_zoom(3, 0xd837, None, game)];
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
    [9150040] CinematicSprInstr_Func8 [] {
        crate::game::visual_runtime::start_title_pan(3, j, game);
        initialize_title_gradient(game);
        j
    };
    [9150157] CinematicSprInstr_Func9 [] {
        let v0: u16 = 0x9ecd;
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA113 as u16, v0, game);
        j
    };
    [9150192] CinematicSprInstr_sub_8B9EF0 [] {
        word!(game.cinematicbg_arr9[(k >> 1)] = 0);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BA113 as u16, 0, game);
        j
    };
    [9150233] CinematicSprInstr_9F19 [] {
        word! {game; demo_timer = 900; cinematic_function = ((0x8b9f29i32) as u16); }
        j
    };
    [9152792] CinematicSprInstr_SpawnMetroidEggParticles [] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCECD as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCED3 as u16, 1, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCED9 as u16, 2, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEDF as u16, 3, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEE5 as u16, 4, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEEB as u16, 5, game);
        QueueSfx2_Max6(0xb, game);
        j
    };
    [9158818] CinematicCommonInstr_Func70 [] {
        word!(game.cinematic_function = (0x8bc0c5i32) as u16);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        j
    };
    [9159684] CinematicSprInstr_SpawnCeresExplosions1 [] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 1, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 2, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 3, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEBB as u16, 4, game);
        j
    };
    [9159948] CinematicSprInstr_SpawnCeresExplosions3 [] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 1, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 2, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEC7 as u16, 3, game);
        j
    };
    [9171888] CinematicSprInstr_Func190 [] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEECD as u16, 0xf3b0, game);
        j
    };
    [9172253] CinematicSprInstr_Func208 [] {
        spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEF3F as u16, 0, 0, game);
        j
    };
    [9172308] CinematicSprInstr_Func211 [] {
        if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
            spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEF7B as u16, 2, 0, game);
        } else {
            spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEF75 as u16, 2, 0, game);
        }
        spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEF6F as u16, 0, 0, game);
        j
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9157153] CinematicSprInstr_SpawnIntroRinkas01 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 0, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 1, game);
    };
    [9157174] CinematicSprInstr_SpawnIntroRinkas23 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 2, game);
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF21 as u16, 3, game);
    };
    @functions [pub unsafe fn (k: u16, mut j: u16, game: &mut Game) -> u16];
    [9158806] CinematicCommonInstr_Func69 [] {
        if word!(game.japanese_text_flag) == 0 {
            j = (j).wrapping_add(6);
        }
        j
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9161125] CinematicSprInstr_C9A5 [j] { palette_effects::spawn(addr_kPalfx_E1B0 as u16, game); };
    [9161135] CinematicSprInstr_C9AF [j] {
        if word!(game.japanese_text_flag) != 0 {
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEB5 as u16,
                word!(game.japanese_text_flag),
                game,
            );
        }
    };
    [9161149] CinematicSprInstr_C9BD [j] { palette_effects::spawn(addr_loc_8BE1B4 as u16, game); };
    [9161159] CinematicSprInstr_C9C7 [j] {
        word! {game; cinematic_var8 = 62; cinematic_var10 = -112i32 as u16; cinematic_var5 = 32;
            cinematic_var6 = 16; cinematic_function = ((0x8bc9f9i32) as u16); }
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9171550] CinematicSprInstr_Func181 [j] {
        set_cinematic_blend(3, 16, 2, 34, game);
        *ram_ptr!(game; 0x5d) = 100;
        *ram_ptr!(game; 0x59) = 84;
        word! {game; cinematic_var4 = 0; cinematic_function = ((0x8be58ai32) as u16); }
    };
    [9171588] CinematicSprInstr_Func182 [j] {
        palette_effects::spawn(addr_kPalfx_E1C4 as u16, game);
        word!(game.palette_buffer[254] = 1);
    };
    [9171605] CinematicSprInstr_Func183 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEB5 as u16, 0xf295, game);
        word!(game.palette_buffer[0] = 0x7fff);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9171639] CinematicSprInstr_Func185 [j] {
        set_cinematic_blend(17, 2, 2, 17, game);
        *ram_ptr!(game; 0x59) = 124;
        spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEEBB as u16, 0, 0xf27c, game);
        spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEEF7 as u16, 2, 2, game);
        palette_effects::spawn(addr_kPalfx_E1C8 as u16, game);
        palette_effects::spawn(addr_kPalfx_E1D0 as u16, game);
        palette_effects::spawn(addr_kPalfx_E1E8 as u16, game);
    };
    [9171706] CinematicSprInstr_Func186 [j] {
        set_cinematic_blend(3, 18, 2, 51, game);
        *ram_ptr!(game; 0x58) = 116;
        *ram_ptr!(game; 0x59) = 120;
        palette_effects::spawn(addr_kPalfx_E1CC as u16, game);
        spawn_cinematic_sprite_at(addr_kCinematicSpriteObjectDef_8BEEC1 as u16, 6, 6, game);
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
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEEFD as u16, 0xf3ce, game);
        prepare_ending_time(game);
    };
    [9171995] CinematicSprInstr_Func193 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF03 as u16, 0xf41b, game);
    };
    [9172004] CinematicSprInstr_Func194 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF09 as u16, 0xf424, game);
    };
    [9172013] CinematicSprInstr_Func195 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF0F as u16, 0xf42d, game);
    };
    [9172022] CinematicSprInstr_Func196 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF15 as u16, 0xf436, game);
    };
    [9172031] CinematicSprInstr_Func197 [j] {
        CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BEF1B as u16, 0xf43f, game);
    };
    [9172040] CinematicSprInstr_Func198 [j] {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.cinematic_function = (0x8bde64i32) as u16);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [9172484] CinematicSprInstr_Func217 [j] {
        word!(game.cinematicbg_arr9[(k >> 1)] = 3584);
        word!(game.cinematic_function = (0x8be3aei32) as u16);
        set_cinematic_blend(17, 16, 2, -127i32 as u8, game);
        word! {game; cinematic_var8 = -4i32 as u16; cinematic_var10 = -8i32 as u16;
            cinematic_var4 = 16; }
        word! { game, 32; cinematic_var14, cinematic_var17 }
        capture_cinematic_palette(game);
        prepare_cinematic_palette_fade(game);
        QueueSfx1_Max15(0x22, game);
    };
    [9172561] CinematicSprInstr_Func218 [j] {
        word! {game; samus_y_subspeed = 0; samus_y_speed = -16i32 as u16; }
    };
    @external {
        9172375 => [crate::game::visual_runtime::position_ending_samus_head(k, false, j, game)];
        9172410 => [crate::game::visual_runtime::position_ending_samus_head(k, true, j, game)];
        9149665 => [crate::game::visual_runtime::start_title_pan(0, j, game)];
        9149789 => [crate::game::visual_runtime::start_title_pan(1, j, game)];
        9149910 => [crate::game::visual_runtime::start_title_pan(2, j, game)];
        9156805 => [start_intro_rinka(k, j, game)];
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
        word!(game.cinematicspr_preinstr_func[(k >> 1)] = (0x8b93d9i32) as u16);
    };
    [9152744] SetSomeStuffForSpriteObject_4_MetroidEgg [] {
        if word!(game.samus_x_pos) as i32 - 169i32 & 0x8000 != 0 {
            let v1: i32 = k as i32 >> 1;
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCB3B as u16);
            word!(game.cinematicspr_preinstr_func[v1] = (0x8b93d9i32) as u16);
        }
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
        if crate::game::visual_runtime::advance_intro_mother_brain(k, 0, game) {
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 1, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF1B as u16, 2, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 1, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 2, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 3, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCF15 as u16, 4, game);
        }
    };
    [9157389] CinematicFunction_Intro_Func52 [] {
        let v1: bool = (word!(game.cinematic_var14 -= 1, reread) & 0x8000) != 0;
        if word!(game.cinematic_var14) == 0 || v1 {
            let v2: i32 = k as i32 >> 1;
            word!(game.cinematicspr_preinstr_func[v2] = (0x8bbb24i32) as u16);
            word! { game, 0; cinematic_var14, cinematic_var17, cinematicspr_goto_timer[v2] }
        }
    };
    [9159817] CinematicSprPreInstr_C489 [] {
        if word!(game.cinematic_function) as i32 == 0x8bc5ca & 0xffff {
            word!(game.cinematicspr_preinstr_func[(k >> 1)] = (0x8b93d9i32) as u16);
        } else {
            let v1: bool = (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0;
            if word!(game.cinematic_var4) == 0 || v1 {
                CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEC1 as u16,
                    word!(game.cinematic_var13),
                    game,
                );
                word!(game.cinematic_var4 = 12);
                word!(game.cinematic_var13 = (word!(game.cinematic_var13) as i32 + 1 & 7) as u16);
            }
        };
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
            word!(game.cinematicspr_preinstr_func[v1] = (0x8b93d9i32) as u16);
        } else {
            word!(game.cinematicspr_goto_timer[v1] += 256);
        };
    };
    [9157214] CinematicFunction_Intro_Func50 [] {
        if word!(game.cinematicspr_instr_ptr[(v1 + 1)]) as i32 + 0x3487 & 0x8000 == 0 {
            word!(game.cinematicspr_preinstr_func[v1] = (0x8bba73i32) as u16);
            word! { game, 0; cinematic_var14, cinematic_var17 }
        }
    };
    [9157235] CinematicFunction_Intro_Func51 [] {
        if word!(game.cinematicbg_arr8[v1]) == 145 {
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 0, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 1, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 2, game);
            CinematicSprites.spawn(addr_kCinematicSpriteObjectDef_8BCEF1 as u16, 3, game);
            QueueSfx3_Max6(0x23, game);
        }
        chase_intro_baby_y(k, 32, game);
        if (word!(game.cinematic_var17) & 0x8000) == 0 {
            word!(game.cinematic_var14 = 128);
            word!(game.cinematicspr_preinstr_func[v1] = (0x8bbb0di32) as u16);
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
            palette_effects::spawn(addr_kPalfx_E200 as u16, game);
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
    @external {
        9156623 => [{ crate::game::visual_runtime::advance_intro_mother_brain(j, 1, game); }];
        9156654 => [{ crate::game::visual_runtime::advance_intro_mother_brain(j, 2, game); }];
        9172264 => [crate::game::visual_runtime::advance_ending_samus_jump(j, 0, game)];
        9172351 => [crate::game::visual_runtime::advance_ending_samus_jump(j, 1, game)];
        9172445 => [crate::game::visual_runtime::advance_ending_samus_jump(j, 2, game)];
        9170866 => [crate::game::visual_runtime::update_ending_critters(j, false, game)];
        9152771 => [intro_pages::wait_for_scene_end(j, true, game)];
        9157135 => [intro_pages::wait_for_scene_end(j, false, game)];
        9153896 => [intro_pages::advance_laboratory_baby(j, false, game)];
        9153958 => [intro_pages::advance_laboratory_baby(j, true, game)];
        9157412 => [advance_intro_baby_pursuit(j, game)];
        9171802 => [advance_zebes_stars(j, false, game)];
        9171829 => [advance_zebes_stars(j, true, game)];
        9158325 => [advance_ceres_station(j, game)];
        9160782 => [wait_cinematic_scene(j, 0xcade, Some(0xc85d), game)];
        9160874 => [wait_cinematic_scene(j, 0xcade, Some(0xc8b9), game)];
        9160953 => [wait_cinematic_scene(j, 0xcade, Some(0xc908), game)];
        9171867 => [wait_cinematic_scene(j, 0xde63, None, game)];
        9171897 => [wait_cinematic_scene(j, 0xde80, None, game)];
        9172231 => [wait_cinematic_scene(j, 0xdb9d, None, game)];
        9156824 => [advance_intro_rinka(j, false, game)];
        9156923 => [advance_intro_rinka(j, true, game)];
        9172053 => [wait_ending_cloud(j, true, 0xf464, game)];
        9172088 => [wait_ending_cloud(j, true, 0xf487, game)];
        9172123 => [wait_ending_cloud(j, false, 0xf4aa, game)];
        9172177 => [wait_ending_cloud(j, false, 0xf4e0, game)];
        9172068 => [move_ending_cloud(j, (0, 1), game)];
        9172103 => [move_ending_cloud(j, (0, -1), game)];
        9172138 => [move_ending_cloud(j, (-1, -2), game)];
        9172192 => [move_ending_cloud(j, (1, 2), game)];
        9158453 => [drift_cinematic_sprite(j, 0x4000, None, true, game)];
        9158495 => [drift_cinematic_sprite(j, 4096, None, true, game)];
        9158537 => [drift_cinematic_sprite(j, 2048, None, true, game)];
        9158598 => [drift_cinematic_sprite(j, -0x2000i32 as u32, None, false, game)];
        9160066 => [drift_cinematic_sprite(j, 0x4000, Some(-4096i32 as u32), false, game)];
        9160797 => [launch_cinematic_sprite(j, 64, false, game)];
        9160889 => [launch_cinematic_sprite(j, 32, true, game)];
        9160968 => [launch_cinematic_sprite(j, 32, false, game)];
        9158617 => [()]; 9147353 => [()]; 9147487 => [()]; 9152724 => [()];
        9172252 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCinematicSpriteObjectSetup(ea: u32, j: u16, game: &mut Game) => ea;
    call (j, game); default { Unreachable(game); };
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
    [9158560] CinematicFunction_Intro_Func65 [] {
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicbg_arr7[(j >> 1)] = 224);
        } else {
            let v1: i32 = j as i32 >> 1;
            word!(game.cinematicbg_arr7[v1] = 112);
            word!(game.cinematicspr_preinstr_func[v1] = (0x8bbfd9i32) as u16);
        }
        let v2: i32 = j as i32 >> 1;
        word! {game; cinematicbg_arr8[v2] = 87; cinematicbg_arr9[v2] = 2048; }
    };
    @functions [pub unsafe fn (j: u16, game: &mut Game)];
    @prefix { let v1: i32 = j as i32 >> 1; };
    [9153178] SetSomeStuffForSpriteObject_8 [] {
        word! {game; cinematicspr_goto_timer[v1] = word!(game.cinematic_spawn_param);
            cinematicbg_arr7[v1] = word!(game.cinematicbg_arr7[13]);
            cinematicbg_arr8[v1] = word!(game.cinematicbg_arr8[13]); cinematicbg_arr9[v1] = 3584; }
    };
    [9158270] CinematicFunction_Intro_Func57 [] {
        if word!(game.cinematic_spawn_param) != 0 {
            word!(game.cinematicspr_preinstr_func[v1] = (0x8bbfc6i32) as u16);
            word!(game.cinematicbg_arr7[v1] = -32i32 as u16);
        } else {
            word!(game.cinematicspr_goto_timer[v1] = -1024i32 as u16);
            word!(game.cinematicbg_arr7[v1] = 112);
        }
        word! {game; cinematicbg_arr8[v1] = 87; cinematicbg_arr9[v1] = 2048; }
    };
    [9171343] CinematicFunction_Intro_Func175 [] {
        word! {game; cinematicbg_arr7[v1] = 530; cinematicbg_arr8[v1] = 231;
            cinematicbg_arr9[v1] = 3584; cinematicspr_goto_timer[v1] = 8; }
    };
    [9171368] CinematicFunction_Intro_Func176 [] {
        word! {game; cinematicbg_arr7[v1] = 246; cinematicbg_arr8[v1] = 519;
            cinematicbg_arr9[v1] = 3584; cinematicspr_goto_timer[v1] = 8; }
    };
    @external {
        9170841 => [crate::game::visual_runtime::update_ending_critters(j, true, game)];
        9150131 => [initialize_title_sprite(j, 0, game)];
        9150166 => [initialize_title_sprite(j, 1, game)];
        9150207 => [initialize_title_sprite(j, 2, game)];
        9157005 => [initialize_intro_brain_particle(j, true, game)];
        9157076 => [initialize_intro_brain_particle(j, false, game)];
        9159732 => [initialize_ceres_explosion(j, CeresExplosion::First, game)];
        9159865 => [initialize_ceres_explosion(j, CeresExplosion::Second, game)];
        9159987 => [initialize_ceres_explosion(j, CeresExplosion::Third, game)];
        9160105 => [initialize_ceres_explosion(j, CeresExplosion::Final, game)];
        9156758 => [initialize_intro_rinka(j, game)];
        9171038 => [initialize_ending_time_glyph(j, 0, game)];
        9171053 => [initialize_ending_time_glyph(j, 1, game)];
        9171068 => [initialize_ending_time_glyph(j, 2, game)];
        9171077 => [initialize_ending_time_glyph(j, 3, game)];
        9171092 => [initialize_ending_time_glyph(j, 4, game)];
        9171122 => [initialize_ending_cloud_band(j, -96i32 as u16, game)];
        9171169 => [initialize_ending_cloud_band(j, -32i32 as u16, game)];
        9171177 => [initialize_ending_cloud_band(j, 288, game)];
        9171185 => [initialize_ending_cloud_band(j, 352, game)];
        9171193 => [initialize_ending_cloud_side(j, true, game)];
        9171230 => [initialize_ending_cloud_side(j, false, game)];
        9147353 => [()];
        9149210 => [place_cinematic_sprite(j, (128, 48), 1024, game)];
        9149229 => [place_cinematic_sprite(j, (128, 81), 0, game)];
        9149248 => [place_cinematic_sprite(j, (128, 196), 2048, game)];
        9149628 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9149770 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9149891 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9150021 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9152705 => [place_cinematic_sprite(j, (8, 24), 3072, game)];
        9152725 => [place_cinematic_sprite(j, (112, 155), 3584, game)];
        9153877 => [place_cinematic_sprite(j, (84, 139), 3072, game)];
        9153939 => [place_cinematic_sprite(j, (112, 111), 3072, game)];
        9154055 => [place_cinematic_sprite(j, (128, 216), 3072, game)];
        9156467 => [place_cinematic_sprite(j, (56, 111), 3584, game)];
        9157195 => [place_cinematic_sprite(j, (112, 155), 3584, game)];
        9158434 => [place_cinematic_sprite(j, (80, 159), 2048, game)];
        9158476 => [place_cinematic_sprite(j, (116, 160), 3072, game)];
        9158518 => [place_cinematic_sprite(j, (128, 96), 2048, game)];
        9158787 => [place_cinematic_sprite(j, (124, 186), 0, game)];
        9158834 => [place_cinematic_sprite(j, (124, 204), 0, game)];
        9160763 => [place_cinematic_sprite(j, (136, 111), 3584, game)];
        9160855 => [place_cinematic_sprite(j, (128, 127), 2048, game)];
        9161026 => [place_cinematic_sprite(j, (48, 47), 2048, game)];
        9161046 => [place_cinematic_sprite(j, (208, 47), 2048, game)];
        9161066 => [place_cinematic_sprite(j, (48, 207), 2048, game)];
        9161086 => [place_cinematic_sprite(j, (208, 207), 2048, game)];
        9161106 => [place_cinematic_sprite(j, (128, 186), 0, game)];
        9161190 => [place_cinematic_sprite(j, (128, 204), 0, game)];
        9170922 => [place_cinematic_sprite(j, (128, 128), 3584, game)];
        9170941 => [place_cinematic_sprite(j, (-128i32 as u16, 128), 3584, game)];
        9170949 => [place_cinematic_sprite(j, (128, 128), 2560, game)];
        9170968 => [place_cinematic_sprite(j, (128, 128), 3072, game)];
        9170987 => [place_cinematic_sprite(j, (128, 96), 1024, game)];
        9171006 => [place_cinematic_sprite(j, (128, 160), 512, game)];
        9171267 => [place_cinematic_sprite(j, (120, 136), 2560, game)];
        9171286 => [place_cinematic_sprite(j, (120, 152), 3072, game)];
        9171305 => [place_cinematic_sprite(j, (124, 108), 3072, game)];
        9171324 => [place_cinematic_sprite(j, (121, 107), 2560, game)];
        9171393 => [place_cinematic_sprite(j, (385, 366), 3584, game)];
        9171412 => [place_cinematic_sprite(j, (391, 384), 3584, game)];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallCinematicSpriteInit(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        9149210 => [place_cinematic_sprite(j, (128, 48), 1024, game)];
        9149229 => [place_cinematic_sprite(j, (128, 81), 0, game)];
        9149248 => [place_cinematic_sprite(j, (128, 196), 2048, game)];
        9149628 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9149770 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9149891 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9150021 => [place_cinematic_sprite(j, (129, 112), 512, game)];
        9150131 => [initialize_title_sprite(j, 0, game)];
        9150166 => [initialize_title_sprite(j, 1, game)]; 9150207 => [initialize_title_sprite(j, 2, game)]; 9147353 => [()];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMode7Instr(ea: u32, _k: u16, j: u16, game: &mut Game) -> u16 => ea;
    call (_k, j, game); default Unreachable(game) as u16;
    {
        9151067 => [title_metroid_cry(0x23, j, game)];
        9151075 => [title_metroid_cry(0x26, j, game)];
        9151083 => [title_metroid_cry(0x27, j, game)];
        9151091 => [title_metroid_palette(METROID_PAL_1, j, game)];
        9151108 => [title_metroid_palette(METROID_PAL_2, j, game)];
        9151125 => [title_metroid_palette(METROID_PAL_3, j, game)];
        9151142 => [title_metroid_palette(METROID_PAL_4, j, game)];
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
