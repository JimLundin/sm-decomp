use super::intro_pages;
use crate::game::music::QueueMusic_DelayedY;
pub use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::sprite_drawing::Clip;
use crate::game::visual_runtime::CinematicObjects::{
    Background as CinematicBackground, Mode7 as CinematicMode7, Sprites as CinematicSprites,
};
use crate::game::visual_runtime::advance_cinematic_coordinate;
use crate::game::visual_runtime::{
    advance_ceres_station, advance_credits, advance_intro_baby_pursuit, advance_intro_rinka,
    advance_postcredits_samus_message, advance_shooting_stars, advance_zebes_destruction_zoom,
    advance_zebes_stars, draw_cinematic_sprites, draw_intro_sprites, drift_cinematic_sprite,
    launch_cinematic_sprite, move_ending_cloud, request_title_skip, wait_cinematic_scene,
    wait_ending_cloud,
};
use crate::game::visual_runtime::{
    advance_cinematic_fade, set_fixed_color, update_cinematic_transform,
};
use sm_game::visual_state::CinematicFade;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_60::*;

use crate::game::display_refresh::GraphicsAsset;
use crate::game::explosion_window::spawn_power_bomb;
use crate::game::mode7_transfers::enqueue_rom;
use crate::game::palette_effects;
use crate::game::player_projectiles::prepare;
use crate::game::session_runtime::initialize_player;
use crate::game::sprite_drawing::{clear_high_table, finish_frame};

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
/// Kept unsafe for the player transition caller checks, which wrap it in an unsafe block.
pub unsafe fn HandleSamusDuringIntro(game: &mut Game) {
    handle_samus_during_intro(game)
}
fn handle_samus_during_intro(game: &mut Game) {
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
pub fn InitializeIoDisplayLogo_Async(game: &mut Game) -> CoroutineRet {
    {
        crate::verify_probe::hit("coroutine.logo");
        let mut stage = game.coroutine_state_1();
        if stage == 0 {
            unsafe {
                crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game)
            };
            word!(game.oam_next_ptr = 0);
            clear_high_table(game);
            finish_frame(game);
            word! { game, 0;
                nmi_copy_samus_halves, nmi_copy_samus_top_half_src,
                nmi_copy_samus_bottom_half_src,
            }
            unsafe { crate::game::display_refresh::initialize_io_and_oam(None, game) };
            unsafe {
                crate::rtl::register_writes! { game;
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
            }
            set_fixed_color([32, 64, 0x80], game);
            unsafe {
                crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
                GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
                GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
            }
            game.ram().write_byte(0x51, 0x80);
            word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x8ce1e9, 0));
            unsafe { crate::game::display_refresh::enable(true, game) };
            word! { game, 1; screen_fade_delay, screen_fade_counter }
        }
        // Stages 1..=6 resume a pending NMI before drawing the next frame.
        while stage <= 6 {
            if stage != 0 {
                let yielded: u8 = crate::game::display_refresh::wait(true, game);
                if yielded != 0 {
                    game.set_coroutine_state_1(stage);
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
                        (unsafe {
                            crate::rtl::write_register_and_mirror(
                                INIDISP as u16,
                                0x8f,
                                0x51,
                                false,
                                game,
                            )
                        });
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
        game.set_coroutine_state_1(0);
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
pub fn GameState_1_OpeningCinematic_(game: &mut Game) -> CoroutineRet {
    {
        CallCinematicFunction(
            ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
            game,
        );
        CinematicSprites.advance(game);
        CinematicMode7.advance(game);
        palette_effects::advance(game);
        draw_cinematic_sprites(false, game);
        request_title_skip(game);
        HandleCinematicsTransitions(game);
        update_cinematic_transform(false, game);
        kCoroutineNone as CoroutineRet
    }
}
pub fn CinematicFunctionNone() {}
pub fn LoadTitleSequenceGraphics(game: &mut Game) {
    sm_game::title_sequence::load_graphics(game);
}
pub fn HandleCinematicsTransitions(game: &mut Game) {
    sm_game::title_sequence::handle_transitions(game);
}
pub fn GameState_37_CeresGoesBoomWithSamus_(game: &mut Game) -> CoroutineRet {
    {
        CallCinematicFunction(
            ((word!(game.cinematic_function)) as u32) | (0x8b0000u32),
            game,
        );
        if (word!(game.cinematic_var11) & 0x8000) == 0 {
            word!(game.cinematic_var11 += 1);
        }
        word!(game.cinematic_var12 += 1);
        handle_samus_during_intro(game);
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

pub fn GameState_39_EndingAndCredits_(game: &mut Game) -> CoroutineRet {
    {
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
        advance_credits(game);
        palette_effects::advance(game);
        draw_cinematic_sprites(true, game);
        advance_shooting_stars(game);
        update_cinematic_transform(true, game);
        kCoroutineNone as CoroutineRet
    }
}
pub fn MaybeHandleCinematicBgObjects(game: &mut Game) {
    {
        if word!(game.cinematic_function) as i32 + 0x1afc & 0x8000 == 0 {
            CinematicBackground.advance(game);
        }
    }
}
crate::game::callback_dispatch! {
    pub fn CallCinematicFunction(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [9149288] CinematicFunctionOpening [] { sm_game::title_sequence::setup(game); };
    [9150091] CinematicFunc_Func7 [] { sm_game::title_sequence::zooming_out(game); };
    [9150290] CinematicFunc_Func10 [] { sm_game::title_sequence::to_file_select(game); };
    [9150382] CinematicFunc_Func9 [] { sm_game::title_sequence::to_demos(game); };
    [9151381] CinematicFunction_Intro_Initial [] { sm_game::intro_sequence::initial(game); };
    [9152111] CinematicFunc_Intro_SetupTextPage2 [] { sm_game::intro_sequence::set_up_text_page_1(game); };
    [9157792] CinematicFunction_Intro_Func54 [] { sm_game::intro_sequence::ceres_initial(game); };
    [9158116] CinematicFunction_Intro_Func55 [] { sm_game::intro_sequence::ceres_wait_for_music(game); };
    [9158137] CinematicFunction_Intro_Func56 [] { sm_game::intro_sequence::ceres_flying_into_camera(game); };
    [9158618] CinematicFunction_Intro_Func67 [] { sm_game::intro_sequence::ceres_flying_into_ceres(game); };
    [9158912] CinematicFunction_Intro_Func73 [] { sm_game::intro_sequence::start_game_at_ceres(game); };
    [9158939] CinematicFunctionBlackoutFromCeres [] { sm_game::intro_sequence::boom_initial(game); };
    [9159396] CinematicFunction_Intro_Func74 [] { sm_game::intro_sequence::boom_wait_for_music(game); };
    [9159409] CinematicFunction_Intro_Func75 [] { sm_game::intro_sequence::boom_small_explosions(game); };
    [9159493] CinematicFunction_Intro_Func76 [] { sm_game::intro_sequence::boom_explosions(game); };
    [9160138] CinematicFunction_Intro_Func77 [] { sm_game::intro_sequence::boom_gunship_flying_away(game); };
    [9160231] CinematicFunction_Intro_Func85 [] { sm_game::intro_sequence::boom_fade_out(game); };
    [9160345] CinematicFunction_Intro_Func86 [] { sm_game::intro_sequence::zebes_initial(game); };
    [9161439] CinematicFunction_Intro_Func95 [] { sm_game::intro_sequence::zebes_load_game_data(game); };
    [9163904] CinematicFunctionEscapeFromCebes [] { sm_game::ending_sequence::setup(game); };
    [9164503] CinematicFunction_Intro_Func109 [] { sm_game::ending_sequence::wait_for_music(game); };
    [9164593] CinematicFunction_Intro_Func112 [] { sm_game::ending_sequence::scene_1_setup(game); };
    [9164855] CinematicFunction_Intro_Func115 [] { sm_game::ending_sequence::scene_2_setup(game); };
    [9165314] CinematicFunction_Intro_Func116 [] { sm_game::ending_sequence::scene_2_fading_in(game); };
    [9165338] CinematicFunction_Intro_Func117 [] { sm_game::ending_sequence::scene_2(game); };
    [9165764] CinematicFunction_Intro_Func120 [] { sm_game::ending_sequence::gunship_setup(game); };
    [9166146] CinematicFunction_Intro_Func123 [] { sm_game::ending_sequence::gunship_spinning_slow(game); };
    [9166285] CinematicFunction_Intro_Func124 [] { sm_game::ending_sequence::gunship_fly_into_camera(game); };
    [9166436] CinematicFunction_Intro_Func125 [] { sm_game::ending_sequence::transition_to_credits(game); };
    [9166464] CinematicFunction_Intro_Func126 [] { sm_game::ending_sequence::credits_setup(game); };
    [9167120] CinematicFunction_Intro_Func130 [] { sm_game::ending_sequence::fade_in_shooting_stars(game); };
    [9167192] CinematicFunction_Intro_Func131 [] { sm_game::ending_sequence::fade_in_samus(game); };
    [9167248] CinematicFunction_Intro_Func132 [] { sm_game::ending_sequence::wavy_samus(game); };
    [9167581] CinematicFunction_Intro_Func137 [] { sm_game::ending_sequence::nintendo_1994(game); };
    [9168132] CinematicFunction_Intro_Func143 [] { sm_game::ending_sequence::fade_from_white(game); };
    [9168266] CinematicFunction_Intro_Func144 [] { sm_game::ending_sequence::grey_out_icon(game); };
    [9168791] CinematicFunction_Intro_Func148 [] { sm_game::ending_sequence::scroll_item_percentage(game); };
    [9151992] CinematicFunction_Intro_LastMetroidCaptivity [] { sm_game::intro_sequence::last_metroid_in_captivity(game); };
    [9152019] CinematicFunc_Intro_QueueGalaxyIsAtPeace [] { sm_game::intro_sequence::galaxy_is_at_peace(game); };
    [9152076] CinematicFunc_Intro_FadeOut [] { sm_game::intro_sequence::fade_out(game); };
    [9160208] CinematicFunction_Intro_Func84 [] { sm_game::intro_sequence::boom_wait(game); };
    [9161424] CinematicFunction_Intro_Func108 [] { sm_game::intro_sequence::zebes_hold(game); };
    [9165726] CinematicFunction_Intro_Func119 [] { sm_game::ending_sequence::change_music(game); };
    [9167092] CinematicFunction_Intro_Func129 [] { sm_game::ending_sequence::blank_screen(game); };
    @external {
        9167314 => [crate::game::visual_runtime::advance_postcredits_samus_reveal(0, game)];
        9167636 => [crate::game::visual_runtime::advance_postcredits_samus_reveal(1, game)];
        9167682 => [crate::game::visual_runtime::advance_postcredits_samus_reveal(2, game)];
        9160604 => [crate::game::visual_runtime::advance_intro_landing_mosaic(true, game)];
        9160650 => [crate::game::visual_runtime::advance_intro_landing_mosaic(false, game)];
        9149719 => [crate::game::visual_runtime::advance_title_pan(0, game)];
        9149840 => [crate::game::visual_runtime::advance_title_pan(1, game)];
        9149970 => [crate::game::visual_runtime::advance_title_pan(2, game)];
        9161209 => [crate::game::visual_runtime::advance_zebes_flight(1, game)];
        9161270 => [crate::game::visual_runtime::advance_zebes_flight(2, game)];
        9161349 => [crate::game::visual_runtime::advance_zebes_flight(3, game)];
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
        9151911 => [intro_pages::wait_for_music(0xa5bd, game)];
        9151933 => [intro_pages::wait_for_fade(CinematicFade::SlowIn, 0xa5f8, game)];
        9152057 => [intro_pages::wait_for_music(0xa64c, game)];
        9152099 => [intro_pages::wait_for_fade(CinematicFade::SlowOut, 0xa66f, game)];
        9152555 => [intro_pages::wait_for_music(0xa84a, game)];
        9152586 => [intro_pages::wait_for_fade(CinematicFade::SlowIn, 0xa391, game)];
        9154232 => [intro_pages::wait_for_scene(intro_pages::Scene::MotherBrain, game)];
        9154412 => [intro_pages::wait_for_scene(intro_pages::Scene::BabyMetroid, game)];
        9154802 => [intro_pages::wait_for_scene(intro_pages::Scene::ScientistPage3, game)]; 9154851 => [intro_pages::wait_for_scene(intro_pages::Scene::ScientistPage4, game)];
        9155034 => [intro_pages::wait_for_final_page(game)]; 9155079 => [intro_pages::show_final_page(game)];
        9155152 => [intro_pages::fade_to_scene(intro_pages::Palette::Gameplay, game)];
        9155282 => [intro_pages::fade_to_scene(intro_pages::Palette::Scientist, game)];
        9155423 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF45 as u16, 0xb3f4, game)]; 9155440 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF4B as u16, 0xb3f4, game)];
        9155457 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF51 as u16, 0xb458, game)]; 9155474 => [intro_pages::show_page(addr_kCinematicBgObjectDef_8BCF57 as u16, 0xb458, game)];
        9155572 => [intro_pages::fade_to_page(intro_pages::Palette::Gameplay, game)]; 9155672 => [intro_pages::fade_to_page(intro_pages::Palette::Scientist, game)];
        9156399 => [intro_pages::fade_out_to_ceres(true, game)];
        9158853 => [intro_pages::fade_out_to_ceres(false, game)];
        9161438 => [()];
        9165725 => [()];
        9166435 => [()];
        9168826 => [()];
    }
}
pub fn CallCinematicSprInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> u16 {
    if let Some(cursor) =
        crate::game::visual_runtime::execute_cinematic_sprite_instruction(ea, k, j, game)
    {
        cursor
    } else if ea == 9150040 {
        sm_game::title_sequence::start_scene_3(j, game)
    } else {
        Unreachable(game) as u16
    }
}
macro_rules! advance_particle_axis {
    ($game:ident, $k:ident, $slot:ident, $vertical:literal, $table:ident;
        $label:lifetime, $index:ident, $limit:literal;
        $expression:literal, $line:literal, $function:literal) => {
        let $index: i32 = 2 * $game
            .ram()
            .read_byte(0x1b7d + 2 * $slot as usize + $vertical as usize)
            as i32;
        crate::native_assertion! {
            $label: ($index + 1) < $limit;
            $expression, b"src/sm_8b.c\0", $line, $function
        };
        let velocity = words!((
            $table[$index as usize],
            $table[($index + 1) as usize] as u16
        ));
        advance_cinematic_coordinate($k, $vertical, velocity, $game);
    };
}

crate::game::callback_dispatch! {
    pub fn CallCinematicSprPreInstr(ea: u32, j: u16, game: &mut Game) => ea;
    call (j, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [9149647] sub_8B9CCF [] { sm_game::title_sequence::scrolling_1994(k, game); };
    [9157389] CinematicFunction_Intro_Func52 [] { sm_game::intro_sequence::baby_idling(k, game); };
    [9159817] CinematicSprPreInstr_C489 [] { sm_game::intro_sequence::explosion_spawner(k, game); };
    [9157214] CinematicFunction_Intro_Func50 [] { sm_game::intro_sequence::baby_waiting(k, game); };
    [9157235] CinematicFunction_Intro_Func51 [] { sm_game::intro_sequence::baby_hatched(k, game); };
    [9171431] CinematicFunction_Intro_Func179 [] { sm_game::ending_sequence::icon_s_top(k, game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    @prefix { let v1: i32 = k as i32 >> 1; };
    [9152916] SetSomeStuffForSpriteObject_7 [] {
        (unsafe { advance_particle_axis! {
            game, k, v1, false, kSetSomeStuffForSpriteObject_7_Tab0;
            '_c2rust_label, v3, 12;
            b"v3 + 1 < 12\0", 2389, b"void SetSomeStuffForSpriteObject_7(uint16)\0"
        } });
        (unsafe { advance_particle_axis! {
            game, k, v1, true, kSetSomeStuffForSpriteObject_7_Tab1;
            '_c2rust_label_0, v6, 82;
            b"v6 + 1 < 82\0", 2392, b"void SetSomeStuffForSpriteObject_7(uint16)\0"
        } });
        if word!(game.cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000 != 0 {
            word!(game.cinematicspr_goto_timer[v1] += 256);
        } else {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_off_8BCE53 as u16);
        };
    };
    [9153203] SetSomeStuffForSpriteObject_9 [] {
        (unsafe { advance_particle_axis! {
            game, k, v1, false, kSetSomeStuffForSpriteObject_9_Tab0;
            '_c2rust_label, v3, 10;
            b"v3 + 1 < 10\0", 2442, b"void SetSomeStuffForSpriteObject_9(uint16)\0"
        } });
        if (word!(game.cinematicspr_goto_timer[v1]) & 1) != 0 {
            (unsafe { advance_particle_axis! {
                game, k, v1, true, kSetSomeStuffForSpriteObject_9_Tab2;
                '_c2rust_label_0, v12, 124;
                b"v12 + 1 < 124\0", 2446, b"void SetSomeStuffForSpriteObject_9(uint16)\0"
            } });
        } else {
            (unsafe { advance_particle_axis! {
                game, k, v1, true, kSetSomeStuffForSpriteObject_9_Tab1;
                '_c2rust_label_1, v6, 138;
                b"v6 + 1 < 138\0", 2450, b"void SetSomeStuffForSpriteObject_9(uint16)\0"
            } });
        }
        if word!(game.cinematicbg_arr8[v1]) as i32 - 168i32 & 0x8000 == 0 {
            word!(game.cinematicspr_instr_timer[v1] = 1);
            word!(game.cinematicspr_instr_ptr[v1] = addr_word_8BCD71 as u16);
            word!(game.cinematicspr_preinstr_func[v1] = (0x8b93d9i32) as u16);
        } else {
            word!(game.cinematicspr_goto_timer[v1] += 256);
        };
    };
    @external {
        9152744 => [intro_pages::advance_scene_sprite(j, 0, game)];
        9154074 => [intro_pages::advance_scene_sprite(j, 1, game)];
        9171621 => [intro_pages::advance_scene_sprite(j, 2, game)];
        9156486 => [crate::game::visual_runtime::advance_intro_mother_brain(j, 0, game)];
        9171495 => [{ crate::game::visual_runtime::advance_ending_sprite_pair(usize::from(j >> 1), false, game); }];
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

impl sm_game::cinematic_host::Host for Game {
    fn clear_palette_effects(&mut self) {
        palette_effects::clear(self);
    }

    fn set_palette_effects_enabled(&mut self, enabled: bool) {
        palette_effects::set_enabled(enabled, self);
    }

    fn queue_music_delayed(&mut self, track: u16, delay: u16) {
        QueueMusic_DelayedY(track, delay, self);
    }

    fn enqueue_mode7_transfers(&mut self, cursor: u16) {
        enqueue_rom(0x8b, cursor, self);
    }

    fn spawn_power_bomb(&mut self) {
        spawn_power_bomb(self);
    }

    fn initialize_player(&mut self) {
        initialize_player(self);
    }

    fn use_cinematic_bindings(&mut self) {
        crate::game::controller_input::use_cinematic_bindings(self);
    }

    fn disable_scanline_irq(&mut self) {
        crate::game::scanline_irq::disable(self);
    }
}

// The named bank $8B functions, objects and lists are the decompilation's.
const _: () = {
    use sm_game::intro_sequence as i;
    use sm_game::title_sequence as t;
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(t::function::NOTHING) == fnCinematicFunctionNone as u32);
    assert!(at(t::function::SETUP) == fnCinematicFunctionOpening as u32);
    assert!(at(t::function::ZOOMING_OUT) == 9150091 && at(t::function::TO_FILE_SELECT) == 9150290);
    assert!(at(t::function::TO_DEMOS) == 9150382 && at(t::function::WAITING) == 9150248);
    assert!(at(t::function::TITLE_SCREEN) == 9150249 && at(t::NO_PRE_INSTRUCTION) == 9147353);
    assert!(t::object::SCROLLING_1994 == addr_kCinematicSpriteObjectDef_8BA0EF as u16);
    assert!(t::object::TITLE_LOGO == addr_kCinematicSpriteObjectDef_8BA107 as u16);
    assert!(t::object::COPYRIGHT == addr_kCinematicSpriteObjectDef_8BA119 as u16);
    assert!(t::object::DEMO_LOGO == addr_kCinematicSpriteObjectDef_8BA125 as u16);
    assert!(t::object::BABY_METROID == addr_kMode7ObjBabyMetroidInTitle as u16);
    assert!(
        at(i::function::WAIT_FOR_MUSIC_AND_FADE_IN) == 0x8ba5a7
            && at(i::function::PAGE_6_DRAWN) == 0x8ba38f
    );
    assert!(
        at(i::function::GALAXY_IS_AT_PEACE) == 9152019
            && at(i::function::WAIT_240_FRAMES) == 9152057
    );
    assert!(
        at(i::function::WAIT_FOR_FADE_OUT) == 9152099
            && at(i::function::SPAWN_TEXT_PAGE_1) == 9152555
    );
    assert!(
        at(i::function::CERES_WAIT_FOR_MUSIC) == 9158116
            && at(i::function::CERES_FLYING_INTO_CAMERA) == 9158137
    );
    assert!(
        at(i::function::CERES_FLYING_INTO_CERES) == 9158618
            && at(i::function::BOOM_WAIT_FOR_MUSIC) == 9159396
    );
    assert!(
        at(i::function::BOOM_SMALL_EXPLOSIONS) == 9159409
            && at(i::function::BOOM_EXPLOSIONS) == 9159493
    );
    assert!(
        at(i::function::BOOM_GUNSHIP_FLYING_AWAY) == 9160138
            && at(i::function::BOOM_WAIT_C0H_FRAMES) == 9160208
    );
    assert!(at(i::function::BOOM_FADE_OUT) == 9160231 && at(i::function::ZEBES_INITIAL) == 9160345);
    assert!(
        at(i::function::ZEBES_FADING_IN) == 9160604
            && at(i::function::ZEBES_SLIDE_SCENE_AWAY) == 9161438
    );
    assert!(
        at(i::pre_instruction::BABY_HATCHED) == 9157235
            && at(i::pre_instruction::BABY_IDLING) == 9157389
    );
    assert!(
        at(i::pre_instruction::BABY_DANCING) == 9157412
            && at(i::pre_instruction::NOTHING) == 9147353
    );
    assert!(i::object::TEXT_CARET == addr_kCinematicSpriteObjectDef_8BCE6D as u16);
    assert!(i::object::CERES_GUNSHIP == addr_kCinematicSpriteObjectDef_8BCE7F as u16);
    assert!(i::object::CERES_UNDER_ATTACK == addr_kCinematicSpriteObjectDef_8BCE85 as u16);
    assert!(i::object::CERES_SMALL_ASTEROIDS == addr_kCinematicSpriteObjectDef_8BCE8B as u16);
    assert!(i::object::CERES_VORTEX == addr_kCinematicSpriteObjectDef_8BCE91 as u16);
    assert!(i::object::CERES_EXPLOSION_2 == addr_kCinematicSpriteObjectDef_8BCEC1 as u16);
    assert!(i::object::SLIME_DROPS == addr_kCinematicSpriteObjectDef_8BCEF1 as u16);
    assert!(i::object::CERES_STARS == addr_kCinematicSpriteObjectDef_8BCF0F as u16);
    assert!(i::object::CERES_EXPLOSION_GUNSHIP == addr_kCinematicSpriteObjectDef_8BCF2D as u16);
    assert!(i::object::CERES_EXPLOSION_SPAWNER == addr_kCinematicSpriteObjectDef_8BCF33 as u16);
    assert!(i::object::CERES_LARGE_ASTEROIDS == addr_kCinematicSpriteObjectDef_8BCF39 as u16);
    assert!(i::object::SAMUS_BLINKING == addr_kCinematicBgObjectDef_8BCF63 as u16);
    assert!(i::object::CERES_BG == addr_kCinematicBgObjectDef_8BCF69 as u16);
    assert!(i::mode7::FLYING_INTO_CERES == addr_kCinematicFunction_Intro_Func56_M7 as u16);
    assert!(i::mode7::BOOM == addr_kCinematicFunction_Intro_Func76_M7_0 as u16);
    assert!(i::mode7::BOOM_GUNSHIP == addr_kCinematicFunction_Intro_Func76_M7_1 as u16);
    assert!(i::mode7::BOOM_WITH_SAMUS == addr_kCinematicFunction_Intro_Func76_M7_2 as u16);
    assert!(i::NEXT_PAGE_ARROW == 0x3000 + core::mem::offset_of!(Ram3000, menu.menu_tilemap) + 286);
};

const _: () = {
    use sm_game::ending_sequence::{function as f, object as o};
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(f::WAIT_FOR_MUSIC) == 9164503 && at(f::SCENE_0_FADING_IN) == 9164530);
    assert!(at(f::SCENE_1_FADING_IN) == 9164792 && at(f::SCENE_2_FADING_IN) == 9165314);
    assert!(at(f::SCENE_2) == 9165338 && at(f::LOAD_GUNSHIP_BG) == 9165523);
    assert!(at(f::CREDITS) == 9165725 && at(f::GUNSHIP_SETUP) == 9165764);
    assert!(at(f::GUNSHIP_SPINNING_FAST) == 9165989 && at(f::GUNSHIP_FREEZE_FRAME) == 9166435);
    assert!(at(f::CREDITS_SETUP) == 9166464 && at(f::FADE_IN_SHOOTING_STARS) == 9167120);
    assert!(at(f::FADE_IN_SAMUS) == 9167192 && at(f::WAVY_SAMUS) == 9167248);
    assert!(at(f::DEER_FORCE) == 9167314 && at(f::IDLE_SAMUS_2_CROSS_FADE) == 9167636);
    assert!(at(f::IDLE_SAMUS_2) == 9167682 && at(f::FINISHED) == 9168826);
    assert!(o::EXPLODING_ZEBES == addr_kCinematicSpriteObjectDef_8BEE9D as u16);
    assert!(o::PURPLE_GLOW == addr_kCinematicSpriteObjectDef_8BEEA3 as u16);
    assert!(o::STARS == addr_kCinematicSpriteObjectDef_8BEEA9 as u16);
    assert!(o::LAVA == addr_kCinematicSpriteObjectDef_8BEEAF as u16);
    assert!(o::GUNSHIP_TEXT == addr_kCinematicSpriteObjectDef_8BEEC7 as u16);
    assert!(o::ITEM_PERCENTAGE == addr_kCinematicBgObjectDef_8BF748 as u16);
    assert!(
        o::ICON == addr_kCinematicBgObjectDef_8BF74E as u16
            && o::CREDITS == addr_stru_8BF6F8 as u16
    );
};
