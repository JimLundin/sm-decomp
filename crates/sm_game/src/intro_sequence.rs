//! The intro's cinematic functions: the intro's setup and its first text
//! page, the flight to Ceres, Ceres going boom and the flight to Zebes, with
//! the confused baby metroid's and the Ceres explosion spawner's
//! pre-instructions (bank $8B:A395-$8B:CAEA).
//!
//! Was `CinematicFunction_Intro_*`, `CinematicFunc_Intro_*` and
//! `CinematicFunctionBlackoutFromCeres` in the original decompilation; the
//! names now come from the bank log. The functions run from
//! `cinematic_function`; the compat dispatch keeps their ids.
#![forbid(unsafe_code)]
use crate::cinematic::{
    clear_cinematic_background, clear_cinematic_sprites, enable_cinematic_background,
    fade_cinematic_fixed_color, set_cinematic_tilemap_updates, set_fixed_color,
};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::display_refresh::GraphicsAsset;
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
}

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
/// $7E:1C1D, the cinematic object RAM.
const CINEMATIC_OBJECTS: usize = 0x1c1d;
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

/// Adds `value` to the 16.16 pair `high`:`low`, low word first.
macro_rules! add_pair {
    ($game:ident, $high:ident, $low:ident, $value:expr) => {{
        let low = word!($game.$low) as u32;
        let high = word!($game.$high) as u32;
        let sum = (low | (high << 16)).wrapping_add($value as u32);
        word!($game.$low = sum as u16);
        word!($game.$high = (sum >> 16) as u16);
    }};
}

/// Whether `value` is below `limit`, as the original's signed 16-bit difference.
fn below(value: u16, limit: u16) -> bool {
    (value.wrapping_sub(limit) as i16) < 0
}

/// Counts down `cinematic_var4`; true when it reaches zero or below.
pub fn countdown<H: Host + ?Sized>(game: &mut H) -> bool {
    word!(game.cinematic_var4 = word!(game.cinematic_var4).wrapping_sub(1));
    let expired = (word!(game.cinematic_var4) & 0x8000) != 0;
    word!(game.cinematic_var4) == 0 || expired
}

fn load_space_graphics<H: Host + ?Sized>(game: &mut H) {
    GraphicsAsset::Decompress(0x95a82f, 0x10000).load(game);
    GraphicsAsset::Decompress(0x96fe69, 0x14000).load(game);
    GraphicsAsset::Decompress(0x96d10a, 0x15000).load(game);
}

fn clear_cinematic_objects<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 329; native(CINEMATIC_OBJECTS) <- value(0));
}

fn sprites<H: Host + ?Sized>(game: &mut H, definition: u16, parameter: u16) {
    spawn(game, CinematicObjects::Sprites, definition, parameter);
}

/// Cinematic function - intro - initial ($8B:A395): Samus, the intro
/// graphics, the background and the intro music. Was
/// `CinematicFunction_Intro_Initial`.
pub fn initial<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Intro, game);
    word!(game.cur_irq_handler = 0);
    word!(game.irqhandler_next_handler = 0);
    word!(game.area_index = 0);
    word!(game.room_width_in_blocks = 16);
    word!(game.room_height_in_blocks = 16);
    game.initialize_player();
    game.update_beam_tiles_and_palette();
    word!(game.samus_max_missiles = INTRO_MISSILES);
    word!(game.samus_missiles = INTRO_MISSILES);
    word!(game.cinematic_var15 = 0);
    word!(game.samus_draw_handler = DEFAULT_DRAW_HANDLER);
    game.use_cinematic_bindings();
    word!(game.samus_invincibility_timer = 0);
    word!(game.samus_knockback_timer = 0);
    clear_cinematic_objects(game);
    crate::word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(INTRO_PALETTES, 0));
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
    game.enable_display_nmi(true);
    word!(game.screen_fade_delay = 3);
    word!(game.screen_fade_counter = 3);
    game.set_hdma_enabled(true);
    game.clear_palette_effects();
    game.set_palette_effects_enabled(true);
    word!(game.door_def_ptr = 0);
    word!(game.layer1_x_pos = 0);
    word!(game.layer1_y_pos = 0);
    word!(game.cinematic_var16 = 0);
    crate::word_transfers!(game, descending, 1024; cinematic_bg(1023) <- native(BG_SOURCE));
    crate::word_transfers!(game, descending, 128; pause_map(127) <- rom(INTRO_PAUSE_MAP, 127));
    word!(game.cinematic_var11 = 0xffff);
    word!(game.cinematic_function = function::WAIT_FOR_MUSIC_AND_FADE_IN);
    game.queue_music_delayed8(NO_MUSIC);
    game.queue_music_delayed8(INTRO_MUSIC_DATA);
}

/// Cinematic function - intro - play "the last Metroid is in captivity"
/// music for 200 frames ($8B:A5F8). Was
/// `CinematicFunction_Intro_LastMetroidCaptivity`.
pub fn last_metroid_in_captivity<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        word!(game.cinematic_function = function::GALAXY_IS_AT_PEACE);
        word!(game.cinematic_var4 = 200);
        game.queue_music_delayed8(LAST_METROID_MUSIC);
    }
}

/// Cinematic function - intro - queue "the galaxy is at peace" music
/// ($8B:A613). Was `CinematicFunc_Intro_QueueGalaxyIsAtPeace`.
pub fn galaxy_is_at_peace<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        game.queue_music_delayed8(NO_MUSIC);
        game.queue_music_delayed8(GALAXY_AT_PEACE_DATA);
        game.queue_music_delayed(5, SONG_DELAY);
        word!(game.cinematic_function = function::WAIT_240_FRAMES);
    }
}

/// Cinematic function - intro - fade out ($8B:A64C). Was
/// `CinematicFunc_Intro_FadeOut`.
pub fn fade_out<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        word!(game.cinematic_function = function::WAIT_FOR_FADE_OUT);
        word!(game.screen_fade_delay = 2);
        word!(game.screen_fade_counter = 2);
    }
}

/// Cinematic function - intro - set up intro text page 1 ($8B:A66F): the
/// text background, caret and next-page arrow, Samus blinking and the text
/// music. Was `CinematicFunc_Intro_SetupTextPage2`.
pub fn set_up_text_page_1<H: Host + ?Sized>(game: &mut H) {
    crate::intro_pages::clear_japanese_tiles(game);
    GraphicsAsset::VramWords(0x7e4000, 0x4180, 0x600).load(game);
    crate::display::sequence::write_register_and_mirror(
        game,
        crate::display::registers::TM as u16,
        0x16,
        mirror::TM,
        false,
    );
    clear_cinematic_background(0x2f, game);
    enable_cinematic_background(game);
    set_cinematic_tilemap_updates(true, game);
    word!(game.cinematicbg_var3 = 0x4c00);
    crate::word_transfers!(game, descending, 128;
        pause_map(127) <- value(BLANK_TEXT_TILE),
        pause_map(1023) <- value(BLANK_TEXT_TILE),
    );
    crate::word_transfers!(game, ascending, 128; pause_map(768) <- rom(TEXT_PAGE_1, 0));
    game.ram()
        .copy_in(NEXT_PAGE_ARROW, &NEXT_PAGE_ARROW_TILES.to_ne_bytes());
    sprites(game, object::TEXT_CARET, 0x1c29);
    word!(game.cinematic_var4 = 127);
    spawn(
        game,
        CinematicObjects::Background,
        object::SAMUS_BLINKING,
        0x4c00,
    );
    word!(game.cinematic_var10 = 8);
    word!(game.reg_BG2VOFS = 8);
    word!(game.reg_BG3VOFS = 8);
    crate::intro_pages::clear_english_text(game);
    crate::intro_pages::reset_caret(game);
    game.queue_music_delayed8(NO_MUSIC);
    game.queue_music_delayed8(TEXT_MUSIC_DATA);
    game.queue_music_delayed(5, SONG_DELAY);
    word!(game.cinematic_function = function::SPAWN_TEXT_PAGE_1);
}

/// Cinematic function - fly to Ceres - initial ($8B:BCA0). Was
/// `CinematicFunction_Intro_Func54`.
pub fn ceres_initial<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Station, game);
    clear_cinematic_sprites(game);
    word!(game.cur_irq_handler = 0);
    word!(game.irqhandler_next_handler = 0);
    crate::word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(SPACE_PALETTES, 0));
    load_space_graphics(game);
    GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x300).load_planes(0x8c, true, game);
    crate::cinematic_display::reset_cinematic_transform(56, 24, game);
    word!(game.cinematic_var8 = -72i32 as u16);
    word!(game.cinematic_var10 = -104i32 as u16);
    word!(game.cinematic_var5 = 224);
    word!(game.cinematic_var6 = 512);
    game.clear_palette_effects();
    game.set_palette_effects_enabled(true);
    clear_cinematic_background(0, game);
    word!(game.cinematicbg_var3 = 23552);
    game.enable_display_nmi(true);
    sprites(game, object::CERES_STARS, 0);
    word!(game.cinematic_function = function::CERES_WAIT_FOR_MUSIC);
    game.queue_music_delayed8(CERES_MUSIC_DATA);
    game.queue_music_delayed(5, SONG_DELAY);
}

/// Cinematic function - fly to Ceres - wait for music queue and enable
/// display ($8B:BDE4). Was `CinematicFunction_Intro_Func55`.
pub fn ceres_wait_for_music<H: Host + ?Sized>(game: &mut H) {
    if game.has_queued_music() & 1 == 0 {
        game.ram().write_byte(mirror::INIDISP, 15);
        word!(game.cinematic_function = function::CERES_FLYING_INTO_CAMERA);
    }
}

/// Cinematic function - fly to Ceres - flying into camera ($8B:BDF9): zoom
/// in, then show Ceres under attack. Was `CinematicFunction_Intro_Func56`.
pub fn ceres_flying_into_camera<H: Host + ?Sized>(game: &mut H) {
    if below(word!(game.cinematic_var6), 32) {
        game.ram().write_byte(0x6f, 0);
        game.ram().write_byte(0x72, 49);
        set_fixed_color([63, 95, 0x9f], game);
        clear_cinematic_sprites(game);
        game.enqueue_mode7_transfers(mode7::FLYING_INTO_CERES);
        word!(game.cinematic_var8 = -32i32 as u16);
        word!(game.cinematic_var10 = -128i32 as u16);
        word!(game.cinematic_var5 = 32);
        // Each parameter is what the preceding store or initializer leaves in A.
        sprites(game, object::CERES_LARGE_ASTEROIDS, 0x0020);
        sprites(game, object::CERES_UNDER_ATTACK, 0x0800);
        sprites(game, object::CERES_SMALL_ASTEROIDS, 0x0c00);
        sprites(game, object::CERES_VORTEX, 1);
        sprites(game, object::CERES_STARS, 1);
        word!(game.cinematic_function = function::CERES_FLYING_INTO_CERES);
        game.spawn_palette_effect(CERES_GLOW_1);
        game.spawn_palette_effect(CERES_GLOW_2);
    } else {
        word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_sub(16));
    }
}

/// Cinematic function - fly to Ceres - flying into Ceres ($8B:BFDA): zoom
/// in on Ceres, then show its background. Was `CinematicFunction_Intro_Func67`.
pub fn ceres_flying_into_ceres<H: Host + ?Sized>(game: &mut H) {
    fade_cinematic_fixed_color(game);
    add_pair!(game, cinematic_var8, cinematic_var7, -0x2000i32 as u32);
    if below(word!(game.cinematic_var6), 3072) {
        word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_add(16));
    } else if below(word!(game.cinematic_var6), 0x2000) {
        word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_add(32));
    } else {
        game.ram().write_byte(mirror::TM, 17);
        game.ram().write_byte(mirror::BGMODE, 1);
        game.ram().write_byte(mirror::M7SEL, 0);
        game.ram().write_byte(mirror::BG1SC, 92);
        game.ram().write_byte(mirror::BG12NBA, 6);
        word!(game.cinematic_var6 = 256);
        word!(game.cinematic_var5 = 0);
        word!(game.cinematic_var7 = 0);
        word!(game.cinematic_var8 = 0);
        word!(game.cinematic_var9 = 0);
        word!(game.cinematic_var10 = 0);
        enable_cinematic_background(game);
        set_cinematic_tilemap_updates(true, game);
        spawn(game, CinematicObjects::Background, object::CERES_BG, 0x5c00);
        word!(game.cinematic_function = function::PAGE_6_DRAWN);
    }
}

/// Cinematic function - fly to Ceres - start game at Ceres ($8B:C100). Was
/// `CinematicFunction_Intro_Func73`.
pub fn start_game_at_ceres<H: Host + ?Sized>(game: &mut H) {
    word!(game.loading_game_state = SET_UP_NEW_GAME);
    word!(game.game_state = SET_UP_NEW_GAME);
    word!(game.area_index = CERES);
    word!(game.load_station_index = 0);
    crate::menus::save_slots::save(word!(game.selected_save_slot), game);
}

/// Cinematic function - Ceres goes boom - initial ($8B:C11B). Was
/// `CinematicFunctionBlackoutFromCeres`.
pub fn boom_initial<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Station, game);
    clear_cinematic_objects(game);
    word!(game.door_def_ptr = 0);
    word!(game.layer1_x_pos = 0);
    word!(game.layer1_y_pos = 0);
    crate::word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(SPACE_PALETTES, 0));
    load_space_graphics(game);
    GraphicsAsset::VramLowBytes(0x7f4600, 0x0000, 0x600).load_planes(0x8c, false, game);
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x1a00).load(game);
    crate::cinematic_display::reset_cinematic_transform(52, 48, game);
    word!(game.cinematic_var8 = -44i32 as u16);
    word!(game.cinematic_var10 = -112i32 as u16);
    word!(game.cinematic_var5 = 0);
    word!(game.cinematic_var6 = 256);
    game.set_hdma_enabled(false);
    game.clear_hdma();
    game.clear_palette_effects();
    game.set_palette_effects_enabled(true);
    clear_cinematic_background(0x2f, game);
    word!(game.cinematicbg_var3 = 0);
    sprites(game, object::CERES_GUNSHIP, 0);
    spawn_at(game, object::CERES_SMALL_ASTEROIDS, 2, 0x0002);
    spawn_at(game, object::CERES_VORTEX, 0, 0x0000);
    // The palette the preceding sprite initializer leaves in A.
    sprites(game, object::CERES_EXPLOSION_SPAWNER, 0x0800);
    game.spawn_palette_effect(BOOM_GLOW);
    game.enable_display_nmi(true);
    word!(game.screen_fade_delay = 1);
    word!(game.screen_fade_counter = 1);
    word!(game.cinematic_function = function::BOOM_WAIT_FOR_MUSIC);
    game.queue_music_delayed8(NO_MUSIC);
    game.queue_music_delayed8(CERES_MUSIC_DATA);
    if word!(game.game_state) == BOOM_WITH_SAMUS {
        game.queue_music_delayed(8, SONG_DELAY);
    } else {
        game.queue_music_delayed(7, SONG_DELAY);
    }
}

/// Cinematic function - Ceres goes boom - wait for music queue ($8B:C2E4).
/// Was `CinematicFunction_Intro_Func74`.
pub fn boom_wait_for_music<H: Host + ?Sized>(game: &mut H) {
    if game.has_queued_music() & 1 == 0 {
        word!(game.cinematic_function = function::BOOM_SMALL_EXPLOSIONS);
    }
}

/// Cinematic function - Ceres goes boom - small Ceres explosions - fading in
/// ($8B:C2F1). Was `CinematicFunction_Intro_Func75`.
pub fn boom_small_explosions<H: Host + ?Sized>(game: &mut H) {
    add_pair!(game, cinematic_var10, cinematic_var9, 4096u32);
    add_pair!(game, cinematic_var8, cinematic_var7, -0x4000i32 as u32);
    word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_add(1));
    crate::display_refresh::fade(true, game);
    if game.ram().read_byte(mirror::INIDISP) == 15 {
        word!(game.screen_fade_delay = 0);
        word!(game.screen_fade_counter = 0);
        word!(game.cinematic_function = function::BOOM_EXPLOSIONS);
        word!(game.cinematic_var4 = 1);
        word!(game.cinematic_var13 = 0);
    }
}

/// Cinematic function - Ceres goes boom - Ceres explosions ($8B:C345):
/// zoom out, then the power bomb blast and the gunship. Was
/// `CinematicFunction_Intro_Func76`.
pub fn boom_explosions<H: Host + ?Sized>(game: &mut H) {
    add_pair!(game, cinematic_var10, cinematic_var9, 4096u32);
    add_pair!(game, cinematic_var8, cinematic_var7, -0x4000i32 as u32);
    if below(word!(game.cinematic_var6), 640) {
        word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_add(1));
    } else {
        word!(
            game.power_bomb_explosion_x_pos =
                (word!(game.reg_M7X)).wrapping_sub(word!(game.cinematic_var8))
        );
        word!(
            game.power_bomb_explosion_y_pos =
                (word!(game.reg_M7Y)).wrapping_sub(word!(game.cinematic_var10))
        );
        game.set_hdma_enabled(true);
        game.spawn_power_bomb();
        game.ram().write_byte(0x6f, 16);
        game.ram().write_byte(0x72, 55);
        word!(game.cinematic_var6 = 768);
        sprites(game, object::CERES_EXPLOSION_GUNSHIP, 0);
        if word!(game.game_state) == BOOM_WITH_SAMUS {
            game.enqueue_mode7_transfers(mode7::BOOM_WITH_SAMUS);
        } else {
            game.enqueue_mode7_transfers(mode7::BOOM);
        }
        game.enqueue_mode7_transfers(mode7::BOOM_GUNSHIP);
        word!(game.cinematic_function = function::BOOM_GUNSHIP_FLYING_AWAY);
    }
}

/// Cinematic function - Ceres goes boom - gunship flying away ($8B:C5CA).
/// Was `CinematicFunction_Intro_Func77`.
pub fn boom_gunship_flying_away<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(0x6f, 16);
    game.ram().write_byte(0x72, 55);
    word!(game.cinematic_var8 = word!(game.cinematic_var8).wrapping_add(2));
    word!(game.cinematic_var5 = ((word!(game.cinematic_var5)) as u8).wrapping_sub(1) as u16);
    if below(word!(game.cinematic_var6), 16) {
        word!(game.cinematic_function = function::BOOM_WAIT_C0H_FRAMES);
        word!(game.cinematic_var4 = 192);
    } else {
        word!(game.cinematic_var6 = word!(game.cinematic_var6).wrapping_sub(16));
    }
}

/// Cinematic function - Ceres goes boom - wait C0h frames ($8B:C610). Was
/// `CinematicFunction_Intro_Func84`.
pub fn boom_wait<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        word!(game.screen_fade_delay = 1);
        word!(game.screen_fade_counter = 1);
        word!(game.cinematic_function = function::BOOM_FADE_OUT);
    }
}

/// Cinematic function - Ceres goes boom - fade out ($8B:C627): with Samus
/// aboard, game over; else the flight to Zebes. Was
/// `CinematicFunction_Intro_Func85`.
pub fn boom_fade_out<H: Host + ?Sized>(game: &mut H) {
    crate::display_refresh::fade(false, game);
    if game.ram().read_byte(mirror::INIDISP) == 0x80 {
        game.enable_display_nmi(true);
        word!(game.screen_fade_delay = 0);
        word!(game.screen_fade_counter = 0);
        if word!(game.game_state) == BOOM_WITH_SAMUS {
            game.queue_music_delayed8(NO_MUSIC);
            word!(game.hud_item_index = 0);
            word!(game.samus_auto_cancel_hud_item_index = 0);
            word!(game.samus_invincibility_timer = 0);
            word!(game.samus_knockback_timer = 0);
            game.clear_hdma();
            game.disable_scanline_irq();
            word!(game.fx_layer_blending_config_a = 0);
            game.ram().write_byte(0x6e, 0);
            game.ram().write_byte(0x71, 0);
            game.ram().write_byte(mirror::TM, 16);
            game.ram().write_byte(mirror::TS, 0);
            game.ram().write_byte(mirror::TMW, 0);
            game.ram().write_byte(mirror::TSW, 0);
            game.ram().write_byte(mirror::BGMODE, 9);
            word!(game.game_options_screen_index = 0);
            word!(game.menu_index = 0);
            crate::word_transfers!(game, descending, 128; le(cinematicbg_arr7, 135) <- value(0));
            word!(game.game_state = SAMUS_NO_HEALTH);
        } else {
            word!(game.cinematic_function = function::ZEBES_INITIAL);
        }
    }
}

/// Cinematic function - fly to Zebes - initial ($8B:C699). Was
/// `CinematicFunction_Intro_Func86`.
pub fn zebes_initial<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Landing, game);
    clear_cinematic_objects(game);
    GraphicsAsset::Decompress(0x978adb, 0x19000).load(game);
    GraphicsAsset::Decompress(0x96ec76, 0x15000).load(game);
    GraphicsAsset::VramLowBytes(0x7f4300, 0x0000, 0x300).load(game);
    GraphicsAsset::VramWords(0x7f9000, 0x5c00, 0x800).load(game);
    GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    crate::cinematic_display::reset_cinematic_transform(56, 24, game);
    word!(game.cinematic_var8 = 0);
    word!(game.cinematic_var10 = 0);
    word!(game.cinematic_var5 = 0);
    word!(game.cinematic_var6 = 256);
    game.set_hdma_enabled(false);
    game.clear_hdma();
    game.clear_palette_effects();
    game.set_palette_effects_enabled(true);
    game.ram().write_byte(mirror::MOSAIC, 0x81);
    game.spawn_palette_effect(CERES_GLOW_1);
    word!(game.cinematic_function = function::ZEBES_FADING_IN);
    game.enable_display_nmi(true);
    word!(game.screen_fade_delay = 1);
    word!(game.screen_fade_counter = 1);
}

/// Cinematic function - fly to Zebes - hold on scene for 40h frames
/// ($8B:CAD0). Was `CinematicFunction_Intro_Func108`.
pub fn zebes_hold<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        word!(game.cinematic_function = function::ZEBES_SLIDE_SCENE_AWAY);
    }
}

/// Cinematic function - fly to Zebes - load game data ($8B:CADF). Was
/// `CinematicFunction_Intro_Func95`.
pub fn zebes_load_game_data<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(mirror::INIDISP, 0x80);
    clear_cinematic_objects(game);
    word!(game.game_state = LOADING_GAME_DATA);
    word!(game.samus_health = word!(game.samus_max_health));
}

/// Pre-instruction - the confused baby metroid waits for its egg to hatch
/// ($8B:BA5E). Was `CinematicFunction_Intro_Func50`.
pub fn baby_waiting<H: Host + ?Sized>(k: u16, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    if !below(
        word!(game.cinematicspr_instr_ptr[(slot + 1)]),
        object::BABY_HATCH_LIST,
    ) {
        word!(game.cinematicspr_preinstr_func[slot] = pre_instruction::BABY_HATCHED);
        word!(game.cinematic_var14 = 0);
        word!(game.cinematic_var17 = 0);
    }
}

/// Pre-instruction - confused baby metroid - hatched ($8B:BA73): the slime
/// drops, then it chases Samus vertically until it slows. Was
/// `CinematicFunction_Intro_Func51`.
pub fn baby_hatched<H: Host + ?Sized>(k: u16, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    if word!(game.cinematicbg_arr8[slot]) == 145 {
        crate::cinematic_objects::spawn_cinematic_burst(object::SLIME_DROPS, 0..4, 0, game);
        game.queue_sound(2, HATCH_SOUND, 6);
    }
    crate::intro_objects::chase_intro_baby_y(k, 32, game);
    if (word!(game.cinematic_var17) & 0x8000) == 0 {
        word!(game.cinematic_var14 = 128);
        word!(game.cinematicspr_preinstr_func[slot] = pre_instruction::BABY_IDLING);
    }
}

/// Pre-instruction - confused baby metroid - idling ($8B:BB0D): then dance.
/// Was `CinematicFunction_Intro_Func52`.
pub fn baby_idling<H: Host + ?Sized>(k: u16, game: &mut H) {
    word!(game.cinematic_var14 = word!(game.cinematic_var14).wrapping_sub(1));
    let expired = (word!(game.cinematic_var14) & 0x8000) != 0;
    if word!(game.cinematic_var14) == 0 || expired {
        let slot = (k as i32 >> 1) as usize;
        word!(game.cinematicspr_preinstr_func[slot] = pre_instruction::BABY_DANCING);
        word!(game.cinematic_var14 = 0);
        word!(game.cinematic_var17 = 0);
        word!(game.cinematicspr_goto_timer[slot] = 0);
    }
}

/// Pre-instruction - Ceres explosion spawner - spawn Ceres explosion 2 every
/// Ch frames ($8B:C489), until the gunship flies away. Was
/// `CinematicSprPreInstr_C489`.
pub fn explosion_spawner<H: Host + ?Sized>(k: u16, game: &mut H) {
    if word!(game.cinematic_function) == function::BOOM_GUNSHIP_FLYING_AWAY {
        word!(game.cinematicspr_preinstr_func[(k >> 1)] = pre_instruction::NOTHING);
    } else if countdown(game) {
        let pattern = word!(game.cinematic_var13);
        sprites(game, object::CERES_EXPLOSION_2, pattern);
        word!(game.cinematic_var4 = 12);
        word!(game.cinematic_var13 = ((word!(game.cinematic_var13) as i32 + 1) & 7) as u16);
    }
}

fn spawn<H: Host + ?Sized>(
    game: &mut H,
    objects: CinematicObjects,
    definition: u16,
    parameter: u16,
) {
    objects.spawn(definition, parameter, game);
}

fn spawn_at<H: Host + ?Sized>(game: &mut H, definition: u16, slot: u16, parameter: u16) {
    crate::cinematic_objects::spawn_cinematic_sprite_at(definition, slot, parameter, game);
}
