//! The ending's cinematic functions: Zebes's destruction, the space view
//! and the gunship's emergence, the credits setup and the post-credits
//! scenes, with the Super Metroid icon's pre-instruction (bank
//! $8B:D480-$8B:F226).
//!
//! Was `CinematicFunctionEscapeFromCebes` and `CinematicFunction_Intro_Func109`
//! -`179` in the original decompilation; the names now come from the bank
//! log. The functions run from `cinematic_function`; the compat dispatch keeps
//! their ids.
use crate::actors::host::below;
use crate::cinematic::{
    capture_cinematic_palette, clear_cinematic_background, clear_cinematic_colors,
    clear_cinematic_sprites, compose_cinematic_palette, fade_cinematic_colors,
    fade_cinematic_fixed_color, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_blend, set_fixed_color,
};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::cinematic;
use crate::display_refresh::GraphicsAsset;
use crate::flow::{self, Word::CinematicFunction};
use crate::intro_sequence::countdown;
use crate::menus::prelude::{owned, scroll};
use crate::ram::RawRam as _;
use crate::ram::mirror;
use crate::ram::word;
use crate::visual_state::{CinematicDisplay, CinematicFade, CinematicTilemap};

/// Bank $8B cinematic functions, stored in `cinematic_function`.
pub mod function {
    pub const WAIT_FOR_MUSIC: u16 = 0xd6d7;
    pub const SCENE_0_FADING_IN: u16 = 0xd6f2;
    pub const SCENE_1_FADING_IN: u16 = 0xd7f8;
    pub const SCENE_2_FADING_IN: u16 = 0xda02;
    pub const SCENE_2: u16 = 0xda1a;
    pub const LOAD_GUNSHIP_BG: u16 = 0xdad3;
    pub const CREDITS: u16 = 0xdb9d;
    pub const GUNSHIP_SETUP: u16 = 0xdbc4;
    pub const GUNSHIP_SPINNING_FAST: u16 = 0xdca5;
    pub const GUNSHIP_FREEZE_FRAME: u16 = 0xde63;
    pub const CREDITS_SETUP: u16 = 0xde80;
    pub const FADE_IN_SHOOTING_STARS: u16 = 0xe110;
    pub const FADE_IN_SAMUS: u16 = 0xe158;
    pub const WAVY_SAMUS: u16 = 0xe190;
    pub const DEER_FORCE: u16 = 0xe1d2;
    pub const IDLE_SAMUS_2_CROSS_FADE: u16 = 0xe314;
    pub const IDLE_SAMUS_2: u16 = 0xe342;
    pub const FINISHED: u16 = 0xe7ba;
}

/// Bank $8B cinematic objects.
pub mod object {
    pub const EXPLODING_ZEBES: u16 = 0xee9d;
    pub const PURPLE_GLOW: u16 = 0xeea3;
    pub const STARS: u16 = 0xeea9;
    pub const LAVA: u16 = 0xeeaf;
    /// $8B:EEB5..EEC1: the Zebes explosion's silhouette, right stars and
    /// afterglow.
    pub const ZEBES_SILHOUETTE: u16 = 0xeeb5;
    pub const ZEBES_STARS_RIGHT: u16 = 0xeebb;
    pub const ZEBES_AFTERGLOW: u16 = 0xeec1;
    pub const GUNSHIP_TEXT: u16 = 0xeec7;
    /// $8B:EECD: 'COMPLETED SUCCESSFULLY'.
    pub const COMPLETED_SUCCESSFULLY: u16 = 0xeecd;
    pub const CLOUDS_TOP: u16 = 0xeed3;
    pub const CLOUDS_RIGHT: u16 = 0xeeeb;
    /// $8B:EEF7: the Zebes explosion's left stars.
    pub const ZEBES_STARS_LEFT: u16 = 0xeef7;
    /// $8B:EEFD: the clear time, and from $EF03 its hours tens, hours units,
    /// colon, minutes tens and minutes units, six bytes apart.
    pub const CLEAR_TIME: u16 = 0xeefd;
    pub const CLEAR_TIME_DIGITS: u16 = 0xef03;
    pub const CRITTERS_ESCAPE: u16 = 0xef21;
    pub const ICON_S_TOP: u16 = 0xef81;
    pub const ITEM_PERCENTAGE: u16 = 0xf748;
    pub const ICON: u16 = 0xf74e;
    pub const CREDITS: u16 = 0xf6f8;
}

/// Bank $8D palette effects of Zebes's explosion, with the bank log's names.
pub mod palette_effect {
    /// $8D:E1C4: fade out the zoomed-out exploding Zebes.
    pub const ZEBES_FADE_OUT: u16 = 0xe1c4;
    /// $8D:E1C8 and $8D:E1E8: the explosion's wide part, foreground and
    /// background.
    pub const EXPLOSION_WIDE_FOREGROUND: u16 = 0xe1c8;
    pub const EXPLOSION_WIDE_BACKGROUND: u16 = 0xe1e8;
    /// $8D:E1CC: the explosion's finale.
    pub const EXPLOSION_FINALE: u16 = 0xe1cc;
    /// $8D:E1D0: white out space during the explosion.
    pub const EXPLOSION_SPACE_WHITE_OUT: u16 = 0xe1d0;
}

/// Bank $8D palette effects.
const ZEBES_GLOW_1: u16 = 0xe1d8;
const ZEBES_GLOW_2: u16 = 0xe1dc;
const SCENE_1_GLOW: u16 = 0xe1e0;
const SPACE_GLOW_1: u16 = 0xe1d4;
const SPACE_GLOW_2: u16 = 0xe1e4;
const ICON_GLOW: u16 = 0xe200;
/// Bank $8C palettes and tiles.
const ESCAPE_PALETTES: u32 = 0x8c_ede9;
const SPACE_PALETTES: u32 = 0x8c_ebe9;
const CREDITS_PALETTES: u32 = 0x8c_e9e9;
const ICON_PALETTE: u32 = 0x8c_efe9;
const DEER_FORCE_TILES: u32 = 0x8c_dc9b;
/// The gunship's grey palette, sprite palette 5, as the original's copy at
/// $8B:DE43.
const GUNSHIP_GREY: [u16; 16] = [
    0, 0x6f7b, 0x4a52, 0x1ce7, 0, 0x5ad6, 0x4a52, 0x35ad, 0x2529, 0x7fff, 0x56b5, 0x294a, 0x4210,
    0x2d6b, 0x18c6, 0x7fff,
];
/// $7E:1C1D, the cinematic object RAM.
const CINEMATIC_OBJECTS: usize = 0x1c1d;
/// $7F:4300: the Mode 7 tilemap fill.
const MODE7_TILEMAP: usize = 0x14300;
const MODE7_FILL: u16 = 0x8c8c;
/// $7E:C1FE, the icon palette's last colour.
const ICON_PALETTE_END: usize = 0xc1fe;
/// Songs, and the queueing delay before them.
const NO_MUSIC: u16 = 0;
const ESCAPE_MUSIC_DATA: u16 = 0xff33;
const CREDITS_MUSIC_DATA: u16 = 0xff3c;
const SONG_DELAY: u16 = 0xe;
/// Game event 0Fh: the critters escaped.
const CRITTERS_ESCAPED: u16 = 0xf;
/// The post-credits Samus depends on the clear time: under 3 hours, under
/// 10 hours, or longer.
const FAST_CLEAR_HOURS: u16 = 3;
const CLEAR_HOURS: u16 = 10;
/// The item percentage stops scrolling at Y -79.
const ITEM_PERCENTAGE_STOP: u16 = -79i32 as u16;

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

fn sprites<H: Host + ?Sized>(game: &mut H, definition: u16, parameter: u16) {
    spawn(game, CinematicObjects::Sprites, definition, parameter);
}

/// Cinematic function - ending - setup ($8B:D480): Zebes's destruction,
/// with its clouds and the escape music. Was `CinematicFunctionEscapeFromCebes`.
pub fn setup<H: Host + ?Sized>(game: &mut H) {
    use crate::cinematic_states::{resuming, wait_for_nmis};
    if !resuming(game) {
        word!(game.CUR_IRQ_HANDLER = 0);
        word!(game.IRQHANDLER_NEXT_HANDLER = 0);
    }
    if wait_for_nmis(9, game) {
        return;
    }
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Escape, game);
    crate::word_transfers!(game, descending, 329; native(CINEMATIC_OBJECTS) <- value(0));
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(ESCAPE_PALETTES, 0));
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
    crate::word_transfers!(game, ascending, 7808; native(MODE7_TILEMAP) <- value(MODE7_FILL));
    GraphicsAsset::Decompress(0x98b5c1, 0x1e000).load(game);
    GraphicsAsset::Decompress(0x98b857, 0x1e800).load(game);
    GraphicsAsset::Decompress(0x98baed, 0x1f000).load(game);
    GraphicsAsset::Decompress(0x98bccd, 0x1f800).load(game);
    GraphicsAsset::Decompress(0x97e7de, 0xa000).load(game);
    GraphicsAsset::Decompress(0x99d65b, 0x2000).load(game);
    GraphicsAsset::Decompress(0x99d932, 0x6000).load(game);
    crate::cinematic_display::reset_cinematic_transform(128, 128, game);
    cinematic!(game.Mode7Angle = 32);
    cinematic!(game.Mode7Zoom = 64);
    cinematic!(game.X = 0);
    cinematic!(game.Y = 0);
    word!(game.DOOR_DEF_PTR = 0);
    word!(game.LAYER1_X_POS = 0);
    word!(game.LAYER1_Y_POS = 0);
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    crate::display_refresh::enable(true, game);
    cinematic!(game.ShootingStarsEnabled = 0);
    set_cinematic_blend(17, 1, 2, 17, game);
    for parameter in 0..2 {
        crate::cinematic_objects::spawn_cinematic_burst(
            object::CLOUDS_RIGHT,
            [parameter; 2],
            6,
            game,
        );
    }
    crate::screen_fade::set_both(game, 0, 2);
    flow::set(game, CinematicFunction, function::WAIT_FOR_MUSIC);
    crate::music_requests::queue_delayed_8(NO_MUSIC, game);
    crate::music_requests::queue_delayed_8(ESCAPE_MUSIC_DATA, game);
    crate::music_requests::queue_delayed(5, SONG_DELAY, game);
}

/// Cinematic function - ending - wait for music to change ($8B:D6D7). Was
/// `CinematicFunction_Intro_Func109`.
pub fn wait_for_music<H: Host + ?Sized>(game: &mut H) {
    if crate::music_requests::has_queued_music(game) & 1 == 0 {
        crate::palette_effects::execution::leased::spawn(ZEBES_GLOW_1, game);
        crate::palette_effects::execution::leased::spawn(ZEBES_GLOW_2, game);
        flow::set(game, CinematicFunction, function::SCENE_0_FADING_IN);
    }
}

/// Cinematic function - ending - Zebes destruction scene 1 - zooming out,
/// clouds on top/bottom - setup ($8B:D731). Was `CinematicFunction_Intro_Func112`.
pub fn scene_1_setup<H: Host + ?Sized>(game: &mut H) {
    GraphicsAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
    GraphicsAsset::VramWords(0x7e2000, 0x2000, 0x4000).load(game);
    GraphicsAsset::Decompress(0x98ed4f, 0x2000).load(game);
    GraphicsAsset::VramHighBytes(0x7e2000, 0x0000, 0x4000).load(game);
    crate::palette_effects::execution::leased::clear(game);
    clear_cinematic_sprites(game);
    crate::palette_effects::execution::leased::spawn(SCENE_1_GLOW, game);
    crate::cinematic_objects::spawn_cinematic_burst(object::CLOUDS_TOP, [0; 4], 6, game);
    cinematic!(game.Mode7Angle = 32);
    cinematic!(game.Mode7Zoom = 64);
    cinematic!(game.X = 0);
    cinematic!(game.Y = 0);
    flow::set(game, CinematicFunction, function::SCENE_1_FADING_IN);
}

/// Cinematic function - ending - Zebes destruction scene 2 - cross-fade to
/// space view - setup ($8B:D837). Was `CinematicFunction_Intro_Func115`.
pub fn scene_2_setup<H: Host + ?Sized>(game: &mut H) {
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
    game.ram().write_byte(mirror::OBSEL, 2);
    crate::palette_effects::execution::leased::clear(game);
    clear_cinematic_sprites(game);
    crate::word_transfers!(game, ascending, 128; le(PALETTE_BUFFER, 128) <- rom(SPACE_PALETTES, 128));
    // The palette copy and each initializer leave A for the next spawn.
    let palette_tail = cinematic!(game.PaletteBuffer[255]);
    sprites(game, object::EXPLODING_ZEBES, palette_tail);
    sprites(game, object::LAVA, 0x0e00);
    sprites(game, object::PURPLE_GLOW, 0x0a00);
    sprites(game, object::STARS, 0x0e00);
    cinematic!(game.Mode7Angle = 0);
    cinematic!(game.Mode7Zoom = 64);
    word!(game.REG_M7A = 256);
    word!(game.REG_M7B = 0);
    word!(game.REG_M7C = 0);
    word!(game.REG_M7D = 256);
    word!(game.REG_M7X = 128);
    word!(game.REG_M7Y = 128);
    cinematic!(game.X = 0);
    cinematic!(game.Y = 0);
    set_cinematic_blend(1, 16, 2, 33, game);
    capture_cinematic_palette(game);
    prepare_cinematic_palette_fade(game);
    clear_cinematic_colors(0x1a0, 0x10, game);
    clear_cinematic_colors(0x1e0, 0x10, game);
    compose_cinematic_palette(game);
    cinematic!(game.FunctionTimer = 63);
    flow::set(game, CinematicFunction, function::SCENE_2_FADING_IN);
}

/// Cinematic function - ending - Zebes destruction scene 2 - cross-fade to
/// space view - fading in ($8B:DA02). Was `CinematicFunction_Intro_Func116`.
pub fn scene_2_fading_in<H: Host + ?Sized>(game: &mut H) {
    scene_2(game);
    if crate::cinematic::advance_cinematic_fade(CinematicFade::FastIn, game) {
        crate::screen_fade::set_both(game, 2, 2);
        flow::set(game, CinematicFunction, function::SCENE_2);
    }
}

/// Cinematic function - ending - Zebes destruction scene 2 - cross-fade to
/// space view ($8B:DA1A): every other frame a fade step; at the end the
/// space view's display. Was `CinematicFunction_Intro_Func117`.
pub fn scene_2<H: Host + ?Sized>(game: &mut H) {
    cinematic!(game.Mode7Zoom += 4);
    if (cinematic!(game.FunctionTimer) & 1) == 0 {
        fade_cinematic_colors(0xe0, 0x10, false, game);
        fade_cinematic_colors(0x1a0, 0x10, true, game);
        fade_cinematic_colors(0x1e0, 0x10, true, game);
        compose_cinematic_palette(game);
    }
    cinematic!(game.FunctionTimer -= 1);
    if (cinematic!(game.FunctionTimer) & 0x8000) != 0 {
        crate::word_transfers!(game, ascending, 128; le(PALETTE_BUFFER, 0) <- rom(SPACE_PALETTES, 0));
        for (at, value) in [
            (mirror::M7SEL, 0),
            (mirror::BG1SC, 112),
            (mirror::BG2SC, 120),
            (mirror::BG3SC, 0),
            (mirror::BG4SC, 0),
            (mirror::BG12NBA, 68),
            (mirror::BG34NBA, 0),
            (mirror::TM, 16),
            (mirror::TS, 0),
            (mirror::TMW, 0),
            (mirror::TSW, 0),
            (0x6e, 0),
            (0x71, 0),
            (0x6f, 0),
            (0x72, 0),
        ] {
            game.ram().write_byte(at, value);
        }
        game.write_register(crate::display::registers::COLDATA as u16, 0xe0);
        set_fixed_color([32, 64, 0x80], game);
        game.ram().write_byte(mirror::MOSAIC, 0);
        cinematic!(game.XSub = 0);
        cinematic!(game.X = 0);
        cinematic!(game.YSub = 0);
        cinematic!(game.Y = 0);
        cinematic!(game.XSubspeed = 0);
        cinematic!(game.XSpeed = 0);
        cinematic!(game.YSubspeed = 0);
        cinematic!(game.YSpeed = 0);
        word!(game.FX_LAYER_BLENDING_CONFIG_A = 0);
        scroll!(game.Bg2H = 0);
        scroll!(game.Bg2V = 0);
        crate::screen_fade::set_both(game, 0, 2);
        flow::set(game, CinematicFunction, function::LOAD_GUNSHIP_BG);
        cinematic!(game.FunctionTimer = 0);
    }
}

/// Cinematic function - ending - space view - change music ($8B:DB9E). Was
/// `CinematicFunction_Intro_Func119`.
pub fn change_music<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        crate::music_requests::queue_delayed_8(NO_MUSIC, game);
        crate::music_requests::queue_delayed_8(CREDITS_MUSIC_DATA, game);
        crate::music_requests::queue_delayed(5, SONG_DELAY, game);
        flow::set(game, CinematicFunction, function::GUNSHIP_SETUP);
    }
}

/// Cinematic function - ending - space view - gunship emergence - setup
/// ($8B:DBC4). Was `CinematicFunction_Intro_Func120`.
pub fn gunship_setup<H: Host + ?Sized>(game: &mut H) {
    if crate::music_requests::has_queued_music(game) & 1 == 0 {
        game.ram().write_byte(mirror::BGMODE, 7);
        game.ram().write_byte(mirror::M7SEL, 0x80);
        set_cinematic_blend(17, 0, 0, 48, game);
        set_fixed_color([63, 95, 0x9f], game);
        game.ram().write_byte(mirror::BG1SC, 0);
        game.ram().write_byte(mirror::BG12NBA, 0);
        cinematic!(game.PaletteBuffer[0] = 0);
        cinematic!(game.PaletteBuffer[16] = 0);
        cinematic!(game.PaletteBuffer[128] = 0);
        crate::palette_effects::execution::leased::spawn(SPACE_GLOW_1, game);
        crate::palette_effects::execution::leased::spawn(SPACE_GLOW_2, game);
        word!(game.REG_M7X = 56);
        word!(game.REG_M7Y = 24);
        cinematic!(game.X = -72i32 as u16);
        cinematic!(game.Y = -104i32 as u16);
        cinematic!(game.Mode7Zoom = 3072);
        cinematic!(game.Mode7Angle = -112i32 as u16);
        cinematic!(game.FunctionTimer = 1);
        cinematic!(game.Var13 = 192);
        cinematic!(game.Var14 = 0);
        flow::set(game, CinematicFunction, function::GUNSHIP_SPINNING_FAST);
    }
}

/// Cinematic function - ending - space view - gunship emergence - spinning
/// slow ($8B:DD42): the critters escape too, if they did. Was
/// `CinematicFunction_Intro_Func123`.
pub fn gunship_spinning_slow<H: Host + ?Sized>(game: &mut H) {
    if crate::ending_screens::advance_ending_gunship_spin(true, game)
        && crate::progress::leased::event_has_happened(CRITTERS_ESCAPED, game) != 0
    {
        spawn_at(game, object::CRITTERS_ESCAPE, 4, 4);
    }
}

/// Cinematic function - ending - space view - gunship emergence - fly into
/// camera ($8B:DDCD). Was `CinematicFunction_Intro_Func124`.
pub fn gunship_fly_into_camera<H: Host + ?Sized>(game: &mut H) {
    crate::ending_screens::fade_ending_gunship_backdrop(game);
    add_pair!(game, CINEMATIC_VAR13, CINEMATIC_VAR14, -256i32 as u32);
    let speed = ((cinematic!(game.Var13) as u32) << 16) | cinematic!(game.Var14) as u32;
    add_pair!(game, CINEMATIC_X, CINEMATIC_X_SUB, speed);
    if below(cinematic!(game.Mode7Zoom), 384)
        && (cinematic!(game.FrameCounter) & 3) == 0
        && cinematic!(game.Mode7Angle) != 16
    {
        cinematic!(game.Mode7Angle = (cinematic!(game.Mode7Angle) as u8).wrapping_add(2) as u16);
    }
    if below(cinematic!(game.Mode7Zoom), 32) {
        let parameter = cinematic!(game.Mode7Zoom);
        sprites(game, object::GUNSHIP_TEXT, parameter);
        flow::set(game, CinematicFunction, function::GUNSHIP_FREEZE_FRAME);
        crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 95) <- table(GUNSHIP_GREY, 15));
    } else {
        cinematic!(game.Mode7Zoom -= 4);
    }
}

/// Cinematic function - ending - space view - transition to credits
/// ($8B:DE64). Was `CinematicFunction_Intro_Func125`.
pub fn transition_to_credits<H: Host + ?Sized>(game: &mut H) {
    crate::screen_fade::set_counter(game, 1);
    if crate::cinematic::advance_cinematic_fade(CinematicFade::FastOut, game) {
        crate::display_refresh::enable(true, game);
        crate::screen_fade::set_both(game, 0, 0);
        flow::set(game, CinematicFunction, function::CREDITS_SETUP);
    }
}

/// Cinematic function - credits - setup ($8B:DE80): the credits graphics,
/// the post-credits Samus for the clear time, the shooting stars and the
/// roll. Was `CinematicFunction_Intro_Func126`.
pub fn credits_setup<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Credits, game);
    clear_cinematic_sprites(game);
    crate::palette_effects::execution::leased::clear(game);
    crate::word_transfers!(game, ascending, 128; le(PALETTE_BUFFER, 0) <- rom(CREDITS_PALETTES, 0));
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
    if below(word!(game.GAME_TIME_HOURS), FAST_CLEAR_HOURS) {
        GraphicsAsset::VramWords(0x7e2000, 0x0000, 0x4000).load(game);
    } else {
        GraphicsAsset::VramWords(0x7f0000, 0x0000, 0x4000).load(game);
    }
    crate::cinematic_display::reset_cinematic_transform(104, 112, game);
    cinematic!(game.X = -8i32 as u16);
    cinematic!(game.Y = -24i32 as u16);
    cinematic!(game.Mode7Angle = 0);
    cinematic!(game.Mode7Zoom = 2304);
    crate::display_refresh::enable(true, game);
    game.ram().write_byte(mirror::INIDISP, 15);
    word!(game.LAYER1_X_POS = 0);
    word!(game.LAYER1_Y_POS = 0);
    crate::hdma_objects::set_hdma_enabled(true, game);
    crate::word_transfers!(game, descending, 256; le(HDMA_TABLE_1, 255) <- value(0));
    crate::shooting_star_motion::initialize_shooting_stars(game);
    GraphicsAsset::Decompress(0x97eeff, 0x10000).load(game);
    clear_cinematic_background(0x7f, game);
    let roll = crate::credits::prepare_credit_roll(object::CREDITS, game);
    crate::cinematic_objects::advance_cinematic_background_pre(roll, 0, game);
    flow::set(game, CinematicFunction, function::CREDITS);
}

/// Cinematic function - post-credits - blank screen ($8B:E0F4). Was
/// `CinematicFunction_Intro_Func129`.
pub fn blank_screen<H: Host + ?Sized>(game: &mut H) {
    if countdown(game) {
        flow::set(game, CinematicFunction, function::FADE_IN_SHOOTING_STARS);
        cinematic!(game.FunctionTimer = 120);
        game.ram().write_byte(mirror::TM, 16);
    }
}

/// Cinematic function - post-credits - fade in shooting stars ($8B:E110).
/// Was `CinematicFunction_Intro_Func130`.
pub fn fade_in_shooting_stars<H: Host + ?Sized>(game: &mut H) {
    crate::display_refresh::fade(true, game);
    if game.ram().read_byte(mirror::INIDISP) == 15 {
        set_cinematic_blend(2, 16, 2, 34, game);
        crate::screen_fade::set_both(game, 0, 0);
        capture_cinematic_palette(game);
        prepare_cinematic_palette_fade(game);
        clear_cinematic_colors(0x40, 0x10, game);
        compose_cinematic_palette(game);
        cinematic!(game.FunctionTimer = 32);
        flow::set(game, CinematicFunction, function::FADE_IN_SAMUS);
    }
}

/// Cinematic function - post-credits - fade in Samus ($8B:E158). Was
/// `CinematicFunction_Intro_Func131`.
pub fn fade_in_samus<H: Host + ?Sized>(game: &mut H) {
    fade_cinematic_colors(0x40, 0x10, true, game);
    compose_cinematic_palette(game);
    if countdown(game) {
        set_cinematic_blend(18, 4, 2, 2, game);
        cinematic!(game.FunctionTimer = 180);
        flow::set(game, CinematicFunction, function::WAVY_SAMUS);
        crate::room_visuals::initialize_intro_wave(game);
    }
}

/// Cinematic function - post-credits - wavy Samus ($8B:E190). Was
/// `CinematicFunction_Intro_Func132`.
pub fn wavy_samus<H: Host + ?Sized>(game: &mut H) {
    scroll!(game.Bg3V += 2);
    if countdown(game) {
        crate::word_transfers!(game, ascending, 288; pause_map(288) <- rom(DEER_FORCE_TILES, 0));
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
        set_cinematic_blend(1, 0, 0, 0, game);
        cinematic!(game.FunctionTimer = 180);
        flow::set(game, CinematicFunction, function::DEER_FORCE);
        word!(game.WAVY_SAMUS_ENABLED = 0);
    }
}

/// Cinematic function - post-credits - 1994 Nintendo ($8B:E2DD): then the
/// second idle Samus, cross-faded out of her suit unless the clear was slow.
/// Was `CinematicFunction_Intro_Func137`.
pub fn nintendo_1994<H: Host + ?Sized>(game: &mut H) {
    cinematic!(game.Var13 -= 1);
    let expired = (cinematic!(game.Var13) & 0x8000) != 0;
    if cinematic!(game.Var13) == 0 || expired {
        if below(word!(game.GAME_TIME_HOURS), FAST_CLEAR_HOURS)
            || below(word!(game.GAME_TIME_HOURS), CLEAR_HOURS)
        {
            set_cinematic_blend(2, 16, 2, 34, game);
            flow::set(game, CinematicFunction, function::IDLE_SAMUS_2_CROSS_FADE);
        } else {
            flow::set(game, CinematicFunction, function::IDLE_SAMUS_2);
        }
    }
}

/// Cinematic function - post-credits - fade from white ($8B:E504): then the
/// Super Metroid icon. Was `CinematicFunction_Intro_Func143`.
pub fn fade_from_white<H: Host + ?Sized>(game: &mut H) {
    fade_cinematic_fixed_color(game);
    if countdown(game) {
        flow::set(game, CinematicFunction, function::FINISHED);
        cinematic!(game.BgVramAddress = 20480);
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
        word!(game.LAYER1_X_POS = 256);
        word!(game.LAYER1_Y_POS = 256);
        // A carries into each spawn: the layer position, then what each
        // initializer loaded last ($8B:F18F..F1D4).
        let parameters = [0x0100, 8, 8, 0x0e00];
        crate::cinematic_objects::spawn_cinematic_burst(object::ICON_S_TOP, parameters, 6, game);
        crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 31) <- value(0));
        crate::word_transfers!(game, descending, 16; native(ICON_PALETTE_END) <- rom(ICON_PALETTE, 15));
    }
}

/// Cinematic function - post-credits - grey out Super Metroid icon
/// ($8B:E58A): then the icon background. Was `CinematicFunction_Intro_Func144`.
pub fn grey_out_icon<H: Host + ?Sized>(game: &mut H) {
    if crate::ending_graphics::advance_ending_logo_palette(game) {
        flow::set(game, CinematicFunction, function::FINISHED);
        clear_cinematic_sprites(game);
        spawn(game, CinematicObjects::Background, object::ICON, 0x4c00);
    }
}

/// Cinematic function - post-credits - scroll item percentage down
/// ($8B:E797): then the percentage, and the completion record. Was
/// `CinematicFunction_Intro_Func148`.
pub fn scroll_item_percentage<H: Host + ?Sized>(game: &mut H) {
    cinematic!(game.Y -= 2);
    if below(cinematic!(game.Y), ITEM_PERCENTAGE_STOP) {
        spawn(
            game,
            CinematicObjects::Background,
            object::ITEM_PERCENTAGE,
            0x4c00,
        );
        flow::set(game, CinematicFunction, function::FINISHED);
        crate::menus::save_slots::record_completion(game);
    }
}

/// Pre-instruction - cinematic sprite object $EF81 (Super Metroid icon - S -
/// top half) ($8B:F1E7). Was `CinematicFunction_Intro_Func179`.
pub fn icon_s_top<H: Host + ?Sized>(k: u16, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    if crate::ending_screens::advance_ending_sprite_pair(slot, true, game) {
        crate::palette_effects::execution::leased::spawn(ICON_GLOW, game);
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
