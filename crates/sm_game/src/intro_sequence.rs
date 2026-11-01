//! The intro's cinematic functions: the intro's setup and its first text
//! page, the flight to Ceres, Ceres going boom and the flight to Zebes, with
//! the confused baby metroid's and the Ceres explosion spawner's
//! pre-instructions (bank $8B:A395-$8B:CAEA).
//!
//! Was `CinematicFunction_Intro_*`, `CinematicFunc_Intro_*` and
//! `CinematicFunctionBlackoutFromCeres` in the original decompilation; the
//! names now come from the bank log. The functions run from
//! `cinematic_function`; the compat dispatch keeps their ids.
use crate::actors::host::below;
use crate::cinematic::{
    clear_cinematic_background, clear_cinematic_sprites, enable_cinematic_background,
    fade_cinematic_fixed_color, set_cinematic_tilemap_updates, set_fixed_color,
};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{Cinematic, Word};
use crate::display::sequence::scroll::BgScroll;
use crate::display_refresh::GraphicsAsset;
use crate::inventory;
use crate::menu_state::OptionsMenu;
use crate::menu_state::SaveSelection;
use crate::ram::RawRam as _;
use crate::ram::mirror;
use crate::ram::word;
use crate::visual_state::CinematicDisplay;

/// Bank $8B cinematic functions, stored in `cinematic_function`.
pub mod function {
    pub const PAGE_6_DRAWN: u16 = 0xa38f;
    pub const WAIT_FOR_MUSIC_AND_FADE_IN: u16 = 0xa5a7;
    pub const GALAXY_IS_AT_PEACE: u16 = 0xa613;
    pub const WAIT_240_FRAMES: u16 = 0xa639;
    pub const WAIT_FOR_FADE_OUT: u16 = 0xa663;
    pub const SPAWN_TEXT_PAGE_1: u16 = 0xa82b;
    pub const CERES_WAIT_FOR_MUSIC: u16 = 0xbde4;
    pub const CERES_FLYING_INTO_CAMERA: u16 = 0xbdf9;
    pub const CERES_FLYING_INTO_CERES: u16 = 0xbfda;
    pub const BOOM_WAIT_FOR_MUSIC: u16 = 0xc2e4;
    pub const BOOM_SMALL_EXPLOSIONS: u16 = 0xc2f1;
    pub const BOOM_EXPLOSIONS: u16 = 0xc345;
    pub const BOOM_GUNSHIP_FLYING_AWAY: u16 = 0xc5ca;
    pub const BOOM_WAIT_C0H_FRAMES: u16 = 0xc610;
    pub const BOOM_FADE_OUT: u16 = 0xc627;
    pub const ZEBES_INITIAL: u16 = 0xc699;
    pub const ZEBES_FADING_IN: u16 = 0xc79c;
    pub const ZEBES_SLIDE_SCENE_AWAY: u16 = 0xcade;
}

/// Bank $8B cinematic sprite object pre-instructions, stored in
/// `cinematicspr_preinstr_func`.
pub mod pre_instruction {
    pub const NOTHING: u16 = 0x93d9;
    pub const BABY_HATCHED: u16 = 0xba73;
    pub const BABY_IDLING: u16 = 0xbb0d;
    pub const BABY_DANCING: u16 = 0xbb24;
}

/// Bank $8B cinematic objects.
pub mod object {
    pub const TEXT_CARET: u16 = 0xce6d;
    pub const CERES_GUNSHIP: u16 = 0xce7f;
    pub const CERES_UNDER_ATTACK: u16 = 0xce85;
    pub const CERES_SMALL_ASTEROIDS: u16 = 0xce8b;
    pub const CERES_VORTEX: u16 = 0xce91;
    /// $8B:CEB5: the Japanese text under PLANET ZEBES.
    pub const PLANET_ZEBES_JAPANESE_TEXT: u16 = 0xceb5;
    pub const CERES_EXPLOSION_2: u16 = 0xcec1;
    pub const SLIME_DROPS: u16 = 0xcef1;
    pub const CERES_STARS: u16 = 0xcf0f;
    pub const CERES_EXPLOSION_GUNSHIP: u16 = 0xcf2d;
    pub const CERES_EXPLOSION_SPAWNER: u16 = 0xcf33;
    pub const CERES_LARGE_ASTEROIDS: u16 = 0xcf39;
    pub const SAMUS_BLINKING: u16 = 0xcf63;
    pub const CERES_BG: u16 = 0xcf69;
    /// The instruction lists the baby metroid hatches from.
    pub const BABY_HATCH_LIST: u16 = 0xcb79;
}

/// Bank $8B Mode 7 transfer lists: the flight into Ceres, and Ceres going
/// boom without and with Samus, then its gunship.
pub mod mode7 {
    pub const FLYING_INTO_CERES: u16 = 0xbe74;
    pub const BOOM: u16 = 0xc3e6;
    pub const BOOM_GUNSHIP: u16 = 0xc3f0;
    pub const BOOM_WITH_SAMUS: u16 = 0xc3fa;
    /// $8B:D401..D437: the Japanese intro text's pages, each page's first
    /// subpage.
    pub const JAPANESE_TEXT_PAGE_1: u16 = 0xd401;
    pub const JAPANESE_TEXT_PAGE_2: u16 = 0xd407;
    pub const JAPANESE_TEXT_PAGE_3: u16 = 0xd413;
    pub const JAPANESE_TEXT_PAGE_4: u16 = 0xd41f;
    pub const JAPANESE_TEXT_PAGE_5: u16 = 0xd42b;
    pub const JAPANESE_TEXT_PAGE_6: u16 = 0xd437;
}

/// $8D:E1B0 and $8D:E1B4: the palette effects fading the PLANET ZEBES text
/// in and out.
pub const ZEBES_TEXT_FADE_IN: u16 = 0xe1b0;
pub const ZEBES_TEXT_FADE_OUT: u16 = 0xe1b4;

/// Bank $8D palette effects.
const CERES_GLOW_1: u16 = 0xe1a8;
const CERES_GLOW_2: u16 = 0xe1ac;
const BOOM_GLOW: u16 = 0xe1b8;
/// Bank $8C palettes: the intro, and the space scenes.
const INTRO_PALETTES: u32 = 0x8c_e3e9;
const SPACE_PALETTES: u32 = 0x8c_e5e9;
/// The intro's pause map tiles at $8C:D81B and its first text page at
/// $8B:A72B.
const INTRO_PAUSE_MAP: u32 = 0x8c_d81b;
const TEXT_PAGE_1: u32 = 0x8b_a72b;
/// The blank text tile.
const BLANK_TEXT_TILE: u16 = 15401;
/// $7E:371E, in the menu tilemap: the next-page arrow's two tiles.
pub const NEXT_PAGE_ARROW: usize = 0x371e;
const NEXT_PAGE_ARROW_TILES: u32 = 472456233;
/// $7E:9800+: the BG copy the intro's background objects read.
const BG_SOURCE: usize = 0x197fe;
/// Songs, and the queueing delay before them.
const NO_MUSIC: u16 = 0;
const INTRO_MUSIC_DATA: u16 = 0xff3f;
const LAST_METROID_MUSIC: u16 = 5;
const GALAXY_AT_PEACE_DATA: u16 = 0xff42;
const TEXT_MUSIC_DATA: u16 = 0xff36;
const CERES_MUSIC_DATA: u16 = 0xff2d;
const SONG_DELAY: u16 = 0xe;
/// Samus starts the intro with 900 missiles and her default draw handler.
const INTRO_MISSILES: u16 = 900;
const DEFAULT_DRAW_HANDLER: u16 = 0xeb52;
/// Game states: a new game at Ceres, Samus out of health, loading game data,
/// and Ceres going boom with Samus aboard.
const SET_UP_NEW_GAME: u16 = 31;
const SAMUS_NO_HEALTH: u16 = 25;
const LOADING_GAME_DATA: u16 = 6;
const BOOM_WITH_SAMUS: u16 = 37;
/// Ceres is area 6.
const CERES: u16 = 6;
const HATCH_SOUND: u8 = 0x23;

/// Counts down `cinematic_function_timer`; true when it reaches zero or below.
pub fn countdown(cine: &mut Cinematic) -> bool {
    let timer = cine.function_timer();
    *timer = timer.wrapping_sub(1);
    *timer == 0 || *timer & 0x8000 != 0
}

fn load_space_graphics<H: Host + ?Sized>(game: &mut H) {
    GraphicsAsset::Decompress(0x95a82f, 0x10000).load(game);
    GraphicsAsset::Decompress(0x96fe69, 0x14000).load(game);
    GraphicsAsset::Decompress(0x96d10a, 0x15000).load(game);
}

fn clear_cinematic_objects(cine: &mut Cinematic) {
    cine.clear();
}

fn sprites<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H, definition: u16, parameter: u16) {
    spawn(cine, game, CinematicObjects::Sprites, definition, parameter);
}

/// Cinematic function - intro - initial ($8B:A395): Samus, the intro
/// graphics, the background and the intro music. Was
/// `CinematicFunction_Intro_Initial`.
pub fn initial<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Intro, cine, game);
    word!(game.CUR_IRQ_HANDLER = 0);
    word!(game.IRQHANDLER_NEXT_HANDLER = 0);
    crate::session_cohort::with(game, |_, room: &mut crate::room_view::RoomGeometry| {
        room.area = 0;
        room.width = 16;
        room.height = 16;
    });
    game.initialize_player();
    game.update_beam_tiles_and_palette();
    crate::session_cohort::with(game, |_, inventory: &mut inventory::Inventory| {
        inventory.ammunition[0] = inventory::Supply {
            current: INTRO_MISSILES,
            maximum: INTRO_MISSILES,
        };
    });
    *cine.intro_samus_display() = 0;
    word!(game.SAMUS_DRAW_HANDLER = DEFAULT_DRAW_HANDLER);
    crate::menus::controller_input::use_cinematic_bindings(game);
    game.set_invincibility_timer(0);
    game.set_knockback_timer(0);
    clear_cinematic_objects(cine);
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(INTRO_PALETTES, 0));
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
    crate::display_refresh::enable(true, game);
    crate::screen_fade::store(game, 3, 3);
    crate::hdma_objects::set_hdma_enabled(true, game);
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    crate::room_entry::clear_exit(game);
    crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
        words.layer1_x_pos = 0;
        words.layer1_y_pos = 0;
    });
    *cine.intro_japanese_text_timer() = 0;
    crate::word_transfers!(game, descending, 1024; cinematic_bg(1023) <- native(BG_SOURCE));
    crate::word_transfers!(game, descending, 128; pause_map(127) <- rom(INTRO_PAUSE_MAP, 127));
    *cine.intro_frame_counter() = 0xffff;
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::WAIT_FOR_MUSIC_AND_FADE_IN
    });
    crate::music_requests::queue_delayed_8(NO_MUSIC, game);
    crate::music_requests::queue_delayed_8(INTRO_MUSIC_DATA, game);
}

/// Cinematic function - intro - play "the last Metroid is in captivity"
/// music for 200 frames ($8B:A5F8). Was
/// `CinematicFunction_Intro_LastMetroidCaptivity`.
pub fn last_metroid_in_captivity<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if countdown(cine) {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::GALAXY_IS_AT_PEACE
        });
        *cine.function_timer() = 200;
        crate::music_requests::queue_delayed_8(LAST_METROID_MUSIC, game);
    }
}

/// Cinematic function - intro - queue "the galaxy is at peace" music
/// ($8B:A613). Was `CinematicFunc_Intro_QueueGalaxyIsAtPeace`.
pub fn galaxy_is_at_peace<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if countdown(cine) {
        crate::music_requests::queue_delayed_8(NO_MUSIC, game);
        crate::music_requests::queue_delayed_8(GALAXY_AT_PEACE_DATA, game);
        crate::music_requests::queue_delayed(5, SONG_DELAY, game);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::WAIT_240_FRAMES
        });
    }
}

/// Cinematic function - intro - fade out ($8B:A64C). Was
/// `CinematicFunc_Intro_FadeOut`.
pub fn fade_out<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if countdown(cine) {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::WAIT_FOR_FADE_OUT
        });
        crate::screen_fade::store(game, 2, 2);
    }
}

/// Cinematic function - intro - set up intro text page 1 ($8B:A66F): the
/// text background, caret and next-page arrow, Samus blinking and the text
/// music. Was `CinematicFunc_Intro_SetupTextPage2`.
pub fn set_up_text_page_1<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::intro_pages::clear_japanese_tiles(game);
    GraphicsAsset::VramWords(0x7e4000, 0x4180, 0x600).load(game);
    crate::display::sequence::write_register_and_mirror(
        game,
        crate::display::registers::TM as u16,
        0x16,
        mirror::TM,
        false,
    );
    clear_cinematic_background(0x2f, cine, game);
    enable_cinematic_background(cine);
    set_cinematic_tilemap_updates(true, cine);
    *cine.bg_vram_address() = 0x4c00;
    crate::word_transfers!(game, descending, 128;
        pause_map(127) <- value(BLANK_TEXT_TILE),
        pause_map(1023) <- value(BLANK_TEXT_TILE),
    );
    crate::word_transfers!(game, ascending, 128; pause_map(768) <- rom(TEXT_PAGE_1, 0));
    game.ram()
        .copy_in(NEXT_PAGE_ARROW, &NEXT_PAGE_ARROW_TILES.to_ne_bytes());
    sprites(cine, game, object::TEXT_CARET, 0x1c29);
    *cine.function_timer() = 127;
    spawn(
        cine,
        game,
        CinematicObjects::Background,
        object::SAMUS_BLINKING,
        0x4c00,
    );
    *cine.y() = 8;
    crate::session_cohort::with(game, |_, scroll: &mut BgScroll| {
        scroll.bg2[1] = 8;
        scroll.bg3[1] = 8;
    });
    crate::intro_pages::clear_english_text(game);
    crate::intro_pages::reset_caret(cine);
    crate::music_requests::queue_delayed_8(NO_MUSIC, game);
    crate::music_requests::queue_delayed_8(TEXT_MUSIC_DATA, game);
    crate::music_requests::queue_delayed(5, SONG_DELAY, game);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::SPAWN_TEXT_PAGE_1
    });
}

/// Cinematic function - fly to Ceres - initial ($8B:BCA0). Was
/// `CinematicFunction_Intro_Func54`.
pub fn ceres_initial<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Station, cine, game);
    clear_cinematic_sprites(cine);
    word!(game.CUR_IRQ_HANDLER = 0);
    word!(game.IRQHANDLER_NEXT_HANDLER = 0);
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(SPACE_PALETTES, 0));
    load_space_graphics(game);
    GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x300).load_planes(0x8c, true, game);
    crate::cinematic_display::reset_cinematic_transform(56, 24, game);
    *cine.x() = -72i32 as u16;
    *cine.y() = -104i32 as u16;
    *cine.mode7_angle() = 224;
    *cine.mode7_zoom() = 512;
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    clear_cinematic_background(0, cine, game);
    *cine.bg_vram_address() = 23552;
    crate::display_refresh::enable(true, game);
    sprites(cine, game, object::CERES_STARS, 0);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::CERES_WAIT_FOR_MUSIC
    });
    crate::music_requests::queue_delayed_8(CERES_MUSIC_DATA, game);
    crate::music_requests::queue_delayed(5, SONG_DELAY, game);
}

/// Cinematic function - fly to Ceres - wait for music queue and enable
/// display ($8B:BDE4). Was `CinematicFunction_Intro_Func55`.
pub fn ceres_wait_for_music<H: Host + ?Sized>(game: &mut H) {
    if crate::music_requests::has_queued_music(game) & 1 == 0 {
        game.ram().write_byte(mirror::INIDISP, 15);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::CERES_FLYING_INTO_CAMERA
        });
    }
}

/// Cinematic function - fly to Ceres - flying into camera ($8B:BDF9): zoom
/// in, then show Ceres under attack. Was `CinematicFunction_Intro_Func56`.
pub fn ceres_flying_into_camera<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if below(*cine.mode7_zoom(), 32) {
        game.ram().write_byte(0x6f, 0);
        game.ram().write_byte(0x72, 49);
        set_fixed_color([63, 95, 0x9f], game);
        clear_cinematic_sprites(cine);
        crate::display::transfer::enqueue_mode7_rom(0x8b, mode7::FLYING_INTO_CERES, game);
        *cine.x() = -32i32 as u16;
        *cine.y() = -128i32 as u16;
        *cine.mode7_angle() = 32;
        // Each parameter is what the preceding store or initializer leaves in A.
        sprites(cine, game, object::CERES_LARGE_ASTEROIDS, 0x0020);
        sprites(cine, game, object::CERES_UNDER_ATTACK, 0x0800);
        sprites(cine, game, object::CERES_SMALL_ASTEROIDS, 0x0c00);
        sprites(cine, game, object::CERES_VORTEX, 1);
        sprites(cine, game, object::CERES_STARS, 1);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::CERES_FLYING_INTO_CERES
        });
        crate::palette_effects::execution::leased::spawn(CERES_GLOW_1, game);
        crate::palette_effects::execution::leased::spawn(CERES_GLOW_2, game);
    } else {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_sub(16);
    }
}

/// Cinematic function - fly to Ceres - flying into Ceres ($8B:BFDA): zoom
/// in on Ceres, then show its background. Was `CinematicFunction_Intro_Func67`.
pub fn ceres_flying_into_ceres<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    fade_cinematic_fixed_color(game);
    cine.add_pair(Word::X, Word::XSub, -0x2000i32 as u32);
    if below(*cine.mode7_zoom(), 3072) {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(16);
    } else if below(*cine.mode7_zoom(), 0x2000) {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(32);
    } else {
        game.ram().write_byte(mirror::TM, 17);
        game.ram().write_byte(mirror::BGMODE, 1);
        game.ram().write_byte(mirror::M7SEL, 0);
        game.ram().write_byte(mirror::BG1SC, 92);
        game.ram().write_byte(mirror::BG12NBA, 6);
        *cine.mode7_zoom() = 256;
        *cine.mode7_angle() = 0;
        *cine.x_sub() = 0;
        *cine.x() = 0;
        *cine.y_sub() = 0;
        *cine.y() = 0;
        enable_cinematic_background(cine);
        set_cinematic_tilemap_updates(true, cine);
        spawn(
            cine,
            game,
            CinematicObjects::Background,
            object::CERES_BG,
            0x5c00,
        );
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::PAGE_6_DRAWN
        });
    }
}

/// Cinematic function - fly to Ceres - start game at Ceres ($8B:C100). Was
/// `CinematicFunction_Intro_Func73`.
pub fn start_game_at_ceres<H: Host + ?Sized>(game: &mut H) {
    let slot = crate::session_cohort::with(game, |game, selection: &mut SaveSelection| {
        selection.loading_game_state = SET_UP_NEW_GAME;
        word!(game.GAME_STATE = SET_UP_NEW_GAME);
        crate::session_cohort::with(game, |_, room: &mut crate::room_view::RoomGeometry| {
            room.area = CERES
        });
        selection.load_station = 0;
        selection.selected_slot
    });
    crate::menus::save_slots::save(slot, game);
}

/// Cinematic function - Ceres goes boom - initial ($8B:C11B). Was
/// `CinematicFunctionBlackoutFromCeres`.
pub fn boom_initial<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if crate::cinematic_states::wait_for_nmis(9, game) {
        return;
    }
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Station, cine, game);
    clear_cinematic_objects(cine);
    crate::room_entry::clear_exit(game);
    crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
        words.layer1_x_pos = 0;
        words.layer1_y_pos = 0;
    });
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(SPACE_PALETTES, 0));
    load_space_graphics(game);
    GraphicsAsset::VramLowBytes(0x7f4600, 0x0000, 0x600).load_planes(0x8c, false, game);
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1a00).load(game);
    crate::cinematic_display::reset_cinematic_transform(52, 48, game);
    *cine.x() = -44i32 as u16;
    *cine.y() = -112i32 as u16;
    *cine.mode7_angle() = 0;
    *cine.mode7_zoom() = 256;
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    clear_cinematic_background(0x2f, cine, game);
    *cine.bg_vram_address() = 0;
    sprites(cine, game, object::CERES_GUNSHIP, 0);
    spawn_at(cine, game, object::CERES_SMALL_ASTEROIDS, 2, 0x0002);
    spawn_at(cine, game, object::CERES_VORTEX, 0, 0x0000);
    // The palette the preceding sprite initializer leaves in A.
    sprites(cine, game, object::CERES_EXPLOSION_SPAWNER, 0x0800);
    crate::palette_effects::execution::leased::spawn(BOOM_GLOW, game);
    crate::display_refresh::enable(true, game);
    crate::screen_fade::store(game, 1, 1);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::BOOM_WAIT_FOR_MUSIC
    });
    crate::music_requests::queue_delayed_8(NO_MUSIC, game);
    crate::music_requests::queue_delayed_8(CERES_MUSIC_DATA, game);
    if word!(game.GAME_STATE) == BOOM_WITH_SAMUS {
        crate::music_requests::queue_delayed(8, SONG_DELAY, game);
    } else {
        crate::music_requests::queue_delayed(7, SONG_DELAY, game);
    }
}

/// Cinematic function - Ceres goes boom - wait for music queue ($8B:C2E4).
/// Was `CinematicFunction_Intro_Func74`.
pub fn boom_wait_for_music<H: Host + ?Sized>(game: &mut H) {
    if crate::music_requests::has_queued_music(game) & 1 == 0 {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::BOOM_SMALL_EXPLOSIONS
        });
    }
}

/// Cinematic function - Ceres goes boom - small Ceres explosions - fading in
/// ($8B:C2F1). Was `CinematicFunction_Intro_Func75`.
pub fn boom_small_explosions<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    cine.add_pair(Word::Y, Word::YSub, 4096u32);
    cine.add_pair(Word::X, Word::XSub, -0x4000i32 as u32);
    *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(1);
    crate::display_refresh::fade(true, game);
    if game.ram().read_byte(mirror::INIDISP) == 15 {
        crate::screen_fade::store(game, 0, 0);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::BOOM_EXPLOSIONS
        });
        *cine.function_timer() = 1;
        *cine.var13() = 0;
    }
}

/// Cinematic function - Ceres goes boom - Ceres explosions ($8B:C345):
/// zoom out, then the power bomb blast and the gunship. Was
/// `CinematicFunction_Intro_Func76`.
pub fn boom_explosions<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    cine.add_pair(Word::Y, Word::YSub, 4096u32);
    cine.add_pair(Word::X, Word::XSub, -0x4000i32 as u32);
    if below(*cine.mode7_zoom(), 640) {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(1);
    } else {
        let x = word!(game.REG_M7X).wrapping_sub(*cine.x());
        let y = word!(game.REG_M7Y).wrapping_sub(*cine.y());
        crate::session_cohort::with(
            game,
            |_, explosion: &mut crate::explosion_window::Explosion| {
                explosion.bomb.x = x;
                explosion.bomb.y = y;
            },
        );
        crate::hdma_objects::set_hdma_enabled(true, game);
        game.spawn_power_bomb();
        game.ram().write_byte(0x6f, 16);
        game.ram().write_byte(0x72, 55);
        *cine.mode7_zoom() = 768;
        sprites(cine, game, object::CERES_EXPLOSION_GUNSHIP, 0);
        if word!(game.GAME_STATE) == BOOM_WITH_SAMUS {
            crate::display::transfer::enqueue_mode7_rom(0x8b, mode7::BOOM_WITH_SAMUS, game);
        } else {
            crate::display::transfer::enqueue_mode7_rom(0x8b, mode7::BOOM, game);
        }
        crate::display::transfer::enqueue_mode7_rom(0x8b, mode7::BOOM_GUNSHIP, game);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::BOOM_GUNSHIP_FLYING_AWAY
        });
    }
}

/// Cinematic function - Ceres goes boom - gunship flying away ($8B:C5CA).
/// Was `CinematicFunction_Intro_Func77`.
pub fn boom_gunship_flying_away<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    game.ram().write_byte(0x6f, 16);
    game.ram().write_byte(0x72, 55);
    *cine.x() = cine.x().wrapping_add(2);
    *cine.mode7_angle() = (*cine.mode7_angle() as u8).wrapping_sub(1) as u16;
    if below(*cine.mode7_zoom(), 16) {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::BOOM_WAIT_C0H_FRAMES
        });
        *cine.function_timer() = 192;
    } else {
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_sub(16);
    }
}

/// Cinematic function - Ceres goes boom - wait C0h frames ($8B:C610). Was
/// `CinematicFunction_Intro_Func84`.
pub fn boom_wait<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if countdown(cine) {
        crate::screen_fade::store(game, 1, 1);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::BOOM_FADE_OUT
        });
    }
}

/// Cinematic function - Ceres goes boom - fade out ($8B:C627): with Samus
/// aboard, game over; else the flight to Zebes. Was
/// `CinematicFunction_Intro_Func85`.
pub fn boom_fade_out<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::display_refresh::fade(false, game);
    if game.ram().read_byte(mirror::INIDISP) == 0x80 {
        crate::display_refresh::enable(true, game);
        crate::screen_fade::store(game, 0, 0);
        if word!(game.GAME_STATE) == BOOM_WITH_SAMUS {
            crate::music_requests::queue_delayed_8(NO_MUSIC, game);
            crate::session_cohort::with(
                game,
                |_, words: &mut crate::session_words::SessionWords| words.hud_item_index = 0,
            );
            crate::session_cohort::with(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    words.samus_auto_cancel_hud_item_index = 0
                },
            );
            game.set_invincibility_timer(0);
            game.set_knockback_timer(0);
            crate::hdma_objects::clear_hdma(game);
            game.disable_scanline_irq();
            crate::session_cohort::with(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    words.fx_layer_blending_config_a = 0
                },
            );
            game.ram().write_byte(0x6e, 0);
            game.ram().write_byte(0x71, 0);
            game.ram().write_byte(mirror::TM, 16);
            game.ram().write_byte(mirror::TS, 0);
            game.ram().write_byte(mirror::TMW, 0);
            game.ram().write_byte(mirror::TSW, 0);
            game.ram().write_byte(mirror::BGMODE, 9);
            crate::session_cohort::with(game, |_, options: &mut OptionsMenu| {
                options.screen_index = 0
            });
            crate::session_cohort::with(game, |_, menu: &mut crate::menu_state::MenuState| {
                menu.menu_index = 0
            });
            for index in (8..=135).rev() {
                *cine.bg_arr7(index) = 0;
            }
            word!(game.GAME_STATE = SAMUS_NO_HEALTH);
        } else {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = function::ZEBES_INITIAL
            });
        }
    }
}

/// Cinematic function - fly to Zebes - initial ($8B:C699). Was
/// `CinematicFunction_Intro_Func86`.
pub fn zebes_initial<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Landing, cine, game);
    clear_cinematic_objects(cine);
    GraphicsAsset::Decompress(0x978adb, 0x19000).load(game);
    GraphicsAsset::Decompress(0x96ec76, 0x15000).load(game);
    GraphicsAsset::VramLowBytes(0x7f4300, 0x0000, 0x300).load(game);
    GraphicsAsset::VramWords(0x7f9000, 0x5c00, 0x800).load(game);
    GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    crate::cinematic_display::reset_cinematic_transform(56, 24, game);
    *cine.x() = 0;
    *cine.y() = 0;
    *cine.mode7_angle() = 0;
    *cine.mode7_zoom() = 256;
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    game.ram().write_byte(mirror::MOSAIC, 0x81);
    crate::palette_effects::execution::leased::spawn(CERES_GLOW_1, game);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::ZEBES_FADING_IN
    });
    crate::display_refresh::enable(true, game);
    crate::screen_fade::store(game, 1, 1);
}

/// Cinematic function - fly to Zebes - hold on scene for 40h frames
/// ($8B:CAD0). Was `CinematicFunction_Intro_Func108`.
pub fn zebes_hold<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if countdown(cine) {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::ZEBES_SLIDE_SCENE_AWAY
        });
    }
}

/// Cinematic function - fly to Zebes - load game data ($8B:CADF). Was
/// `CinematicFunction_Intro_Func95`.
pub fn zebes_load_game_data<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    game.ram().write_byte(mirror::INIDISP, 0x80);
    clear_cinematic_objects(cine);
    word!(game.GAME_STATE = LOADING_GAME_DATA);
    crate::session_cohort::with(game, |_, inventory: &mut inventory::Inventory| {
        inventory.health.current = inventory.health.maximum;
    });
}

/// Pre-instruction - the confused baby metroid waits for its egg to hatch
/// ($8B:BA5E). Was `CinematicFunction_Intro_Func50`.
pub fn baby_waiting(k: u16, cine: &mut Cinematic) {
    let slot = (k as i32 >> 1) as usize;
    if !below(
        *cine.spr_instruction_pointer(slot + 1),
        object::BABY_HATCH_LIST,
    ) {
        *cine.spr_pre_instruction(slot) = pre_instruction::BABY_HATCHED;
        *cine.var14() = 0;
        *cine.var17() = 0;
    }
}

/// Pre-instruction - confused baby metroid - hatched ($8B:BA73): the slime
/// drops, then it chases Samus vertically until it slows. Was
/// `CinematicFunction_Intro_Func51`.
pub fn baby_hatched<H: Host + ?Sized>(k: u16, cine: &mut Cinematic, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    if *cine.bg_arr8(slot) == 145 {
        crate::cinematic_objects::spawn_cinematic_burst(object::SLIME_DROPS, 0..4, 0, cine, game);
        game.queue_sound(2, HATCH_SOUND, 6);
    }
    crate::intro_objects::chase_intro_baby_y(k, 32, cine, game);
    if (*cine.var17() & 0x8000) == 0 {
        *cine.var14() = 128;
        *cine.spr_pre_instruction(slot) = pre_instruction::BABY_IDLING;
    }
}

/// Pre-instruction - confused baby metroid - idling ($8B:BB0D): then dance.
/// Was `CinematicFunction_Intro_Func52`.
pub fn baby_idling(k: u16, cine: &mut Cinematic) {
    *cine.var14() = cine.var14().wrapping_sub(1);
    let expired = (*cine.var14() & 0x8000) != 0;
    if *cine.var14() == 0 || expired {
        let slot = (k as i32 >> 1) as usize;
        *cine.spr_pre_instruction(slot) = pre_instruction::BABY_DANCING;
        *cine.var14() = 0;
        *cine.var17() = 0;
        *cine.spr_goto_timer(slot) = 0;
    }
}

/// Pre-instruction - Ceres explosion spawner - spawn Ceres explosion 2 every
/// Ch frames ($8B:C489), until the gunship flies away. Was
/// `CinematicSprPreInstr_C489`.
pub fn explosion_spawner<H: Host + ?Sized>(k: u16, cine: &mut Cinematic, game: &mut H) {
    if crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function
        == function::BOOM_GUNSHIP_FLYING_AWAY
    {
        *cine.spr_pre_instruction(k >> 1) = pre_instruction::NOTHING;
    } else if countdown(cine) {
        let pattern = *cine.var13();
        sprites(cine, game, object::CERES_EXPLOSION_2, pattern);
        *cine.function_timer() = 12;
        *cine.var13() = ((*cine.var13()).wrapping_add(1)) & 7;
    }
}

fn spawn<H: Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
    objects: CinematicObjects,
    definition: u16,
    parameter: u16,
) {
    objects.spawn(definition, parameter, cine, game);
}

fn spawn_at<H: Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
    definition: u16,
    slot: u16,
    parameter: u16,
) {
    crate::cinematic_objects::spawn_cinematic_sprite_at(definition, slot, parameter, cine, game);
}
