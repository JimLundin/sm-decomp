//! The title sequence's cinematic functions: the setup and its graphics,
//! the zoom out to the title screen, the transitions to the file select menu
//! and the demos, and the transitions between title and demo loops (bank
//! $8B:9B68-$8B:9FFF).
//!
//! Was `CinematicFunctionOpening`, `CinematicFunc_Func7`/`9`/`10`,
//! `HandleCinematicsTransitions*` and `LoadTitleSequenceGraphics` in the
//! original decompilation. The functions run from `cinematic_function`; the
//! compat dispatch keeps their ids.
use crate::cinematic::{advance_cinematic_fade, clear_cinematic_sprites, set_cinematic_backdrop};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::display_refresh::GraphicsAsset;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::{CinematicDisplay, CinematicFade};

/// Bank $8B cinematic functions, stored in `cinematic_function`.
pub mod function {
    pub const NOTHING: u16 = 0x9a47;
    pub const SETUP: u16 = 0x9b68;
    pub const ZOOMING_OUT: u16 = 0x9e8b;
    pub const WAITING: u16 = 0x9f28;
    pub const TITLE_SCREEN: u16 = 0x9f29;
    pub const TO_FILE_SELECT: u16 = 0x9f52;
    pub const TO_DEMOS: u16 = 0x9fae;
}

/// Bank $8B cinematic sprite object pre-instruction: nothing.
pub const NO_PRE_INSTRUCTION: u16 = 0x93d9;

/// Bank $8B cinematic object definitions: the '1994' scrolling text, the
/// title logo, the title's Nintendo copyright and its Super Metroid logo, and
/// the baby metroid.
pub mod object {
    pub const SCROLLING_1994: u16 = 0xa0ef;
    pub const TITLE_LOGO: u16 = 0xa107;
    pub const COPYRIGHT: u16 = 0xa119;
    /// $8B:A113: the Nintendo copyright, fading in.
    pub const COPYRIGHT_FADE_IN: u16 = 0xa113;
    pub const DEMO_LOGO: u16 = 0xa125;
    pub const BABY_METROID: u16 = 0xa355;
}

/// Bank $8D palette effects: the title's palette glows.
const TITLE_GLOW_1: u16 = 0xe1a0;
const TITLE_GLOW_2: u16 = 0xe1a4;
/// The title sequence's palettes at $8C:E1E9.
const TITLE_PALETTES: u32 = 0x8c_e1e9;
/// Songs: the title sequence (FF03h), its theme (5) and the demos' (6).
const TITLE_MUSIC_DATA: u16 = 0xff03;
const TITLE_THEME: u16 = 5;
const DEMO_MUSIC: u16 = 6;
/// Game states after the title: the file select menus and a demo.
const FILE_SELECT_MENUS: u16 = 4;
const TRANSITION_TO_DEMO: u16 = 40;
/// The title screen waits 900 frames before the demos.
const DEMO_DELAY: u16 = 900;
/// $7E:1C1D, the cinematic object RAM the finish clears.
const CINEMATIC_OBJECTS: usize = 0x1c1d;

/// The cinematic transition states in `skip_to_title_index`.
fn run_transition<H: Host + ?Sized>(state: u16, game: &mut H) {
    let transitions: [Option<fn(&mut H)>; 4] = [
        None,
        Some(fade_out_to_demo),
        Some(start_demo_logo),
        Some(fade_in_title),
    ];
    transitions[state as usize].expect("non-null function pointer")(game);
}

/// Steps the current transition between the title and the demos. Was
/// `HandleCinematicsTransitions`.
pub fn handle_transitions<H: Host + ?Sized>(game: &mut H) {
    if word!(game.SKIP_TO_TITLE_INDEX) != 0 {
        let state = word!(game.SKIP_TO_TITLE_INDEX);
        run_transition(state, game);
    }
}

/// Was `HandleCinematicsTransitions_1`.
fn fade_out_to_demo<H: Host + ?Sized>(game: &mut H) {
    if advance_cinematic_fade(CinematicFade::FastOut, game) {
        word!(game.SKIP_TO_TITLE_INDEX = 2);
        word!(game.CINEMATIC_FUNCTION = function::WAITING);
        game.queue_music_delayed8(DEMO_MUSIC);
    }
}

/// Was `HandleCinematicsTransitions_2`.
fn start_demo_logo<H: Host + ?Sized>(game: &mut H) {
    clear_cinematic_sprites(game);
    spawn(game, CinematicObjects::Sprites, object::COPYRIGHT, 0);
    // The palette the first sprite initializer leaves in A.
    spawn(game, CinematicObjects::Sprites, object::DEMO_LOGO, 0x0400);
    crate::screen_fade::set_both(game, 0, 2);
    word!(game.MODE7_ZOOM = 256);
    word!(game.CINEMATIC_X = 0);
    word!(game.CINEMATIC_X_SUB = 0);
    word!(game.CINEMATIC_Y = 0);
    word!(game.CINEMATIC_Y_SUB = 0);
    word!(game.CINEMATIC_X_SPEED = 0);
    word!(game.CINEMATIC_X_SUBSPEED = 0);
    word!(game.CINEMATIC_Y_SPEED = 0);
    word!(game.CINEMATIC_Y_SUBSPEED = 0);
    word!(game.SKIP_TO_TITLE_INDEX = 3);
    crate::word_transfers!(game, ascending, 128; le(PALETTE_BUFFER, 128) <- rom(TITLE_PALETTES, 128));
    word!(game.PALETTE_BUFFER[201] = 0x7fff);
    word!(game.PALETTE_BUFFER[202] = 32128);
    game.clear_palette_effects();
    game.spawn_palette_effect(TITLE_GLOW_1);
    game.spawn_palette_effect(TITLE_GLOW_2);
    set_cinematic_backdrop(false, game);
    game.ram().write_byte(crate::ram::mirror::TM, 17);
    crate::room_visuals::initialize_title_gradient(game);
}

/// Was `HandleCinematicsTransitions_3`.
fn fade_in_title<H: Host + ?Sized>(game: &mut H) {
    if advance_cinematic_fade(CinematicFade::FastIn, game) {
        word!(game.SKIP_TO_TITLE_INDEX = 0);
        word!(game.DEMO_TIMER = DEMO_DELAY);
        word!(game.CINEMATIC_FUNCTION = function::TITLE_SCREEN);
    }
}

/// Load title sequence graphics ($8B:9B87). Was `LoadTitleSequenceGraphics`.
pub fn load_graphics<H: Host + ?Sized>(game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Title, game);
    word!(game.CUR_IRQ_HANDLER = 0);
    word!(game.IRQHANDLER_NEXT_HANDLER = 0);
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(TITLE_PALETTES, 0));
    GraphicsAsset::Decompress(0x94e000, 0x10000).load(game);
    GraphicsAsset::Decompress(0x96fc04, 0x14000).load(game);
    GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
    GraphicsAsset::Decompress(0x95a5e1, 0x19000).load(game);
    GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x1000).load_planes(0xff, false, game);
    crate::cinematic_display::reset_cinematic_transform(128, 128, game);
    game.enable_display_nmi(true);
    crate::screen_fade::set_both(game, 1, 1);
    game.set_hdma_enabled(true);
    game.set_palette_effects_enabled(true);
    word!(game.FX_LAYER_BLENDING_CONFIG_A = 0);
    game.spawn_palette_effect(TITLE_GLOW_1);
    game.spawn_palette_effect(TITLE_GLOW_2);
    // The second title palette allocation leaves its slot offset in A.
    spawn(game, CinematicObjects::Mode7, object::BABY_METROID, 0x000c);
}

/// Cinematic function - title sequence - setup ($8B:9B68). Was
/// `CinematicFunctionOpening`.
pub fn setup<H: Host + ?Sized>(game: &mut H) {
    load_graphics(game);
    game.queue_music_delayed8(TITLE_MUSIC_DATA);
    word!(game.CINEMATIC_FUNCTION = function::NOTHING);
    spawn(
        game,
        CinematicObjects::Sprites,
        object::SCROLLING_1994,
        function::NOTHING,
    );
    game.queue_music_delayed8(TITLE_THEME);
}

/// Cinematic function - title sequence - scene 3 - zooming out ($8B:9E8B):
/// every other frame, zoom out to full size, then show the title logo. Was
/// `CinematicFunc_Func7`.
pub fn zooming_out<H: Host + ?Sized>(game: &mut H) {
    if (word!(game.NMI_FRAME_COUNTER_WORD) & 1) == 0 {
        if ((word!(game.MODE7_ZOOM)).wrapping_sub(256) as i16) < 0 {
            word!(game.MODE7_ZOOM, wrapping_add(1));
        } else {
            word!(game.MODE7_ZOOM = 256);
            word!(game.CINEMATIC_FUNCTION = function::WAITING);
            spawn(
                game,
                CinematicObjects::Sprites,
                object::TITLE_LOGO,
                function::WAITING,
            );
        }
    }
}

/// Finishes the title sequence for game state `next`: the display back on,
/// effects and cinematic objects cleared.
fn finish<H: Host + ?Sized>(next: u16, game: &mut H, clear_index: impl FnOnce(&mut H)) {
    game.enable_display_nmi(true);
    game.ram().write_byte(crate::ram::mirror::BGMODE, 1);
    crate::screen_fade::set_both(game, 0, 0);
    word!(game.REG_BG2HOFS = 0);
    word!(game.REG_BG3HOFS = 0);
    word!(game.REG_BG2VOFS = 0);
    word!(game.REG_BG3VOFS = 0);
    word!(game.MODE7_ANGLE = 0);
    word!(game.CINEMATIC_FUNCTION = function::SETUP);
    game.set_palette_effects_enabled(false);
    game.clear_palette_effects();
    crate::word_transfers!(game, descending, 329; native(CINEMATIC_OBJECTS) <- value(0));
    crate::word_transfers!(game, descending, 256; le(HDMA_TABLE_1, 255) <- value(0));
    word!(game.GAME_STATE = next);
    clear_index(game);
}

/// Cinematic function - title sequence - transition to file select menu
/// ($8B:9F52). Was `CinematicFunc_Func10`.
pub fn to_file_select<H: Host + ?Sized>(game: &mut H) {
    if advance_cinematic_fade(CinematicFade::SlowOut, game) {
        finish(FILE_SELECT_MENUS, game, |game| {
            word!(game.GAME_OPTIONS_SCREEN_INDEX = 0)
        });
    }
}

/// Cinematic function - title sequence - transition to demos ($8B:9FAE). Was
/// `CinematicFunc_Func9`.
pub fn to_demos<H: Host + ?Sized>(game: &mut H) {
    if advance_cinematic_fade(CinematicFade::SlowOut, game) {
        finish(TRANSITION_TO_DEMO, game, |game| word!(game.DEMO_SCENE = 0));
    }
}

/// Pre-instruction - cinematic sprite object $A0EF ('1994' scrolling text)
/// ($8B:9CCF): full brightness, once. Was `sub_8B9CCF`.
pub fn scrolling_1994<H: Host + ?Sized>(k: u16, game: &mut H) {
    game.ram().write_byte(crate::ram::mirror::INIDISP, 15);
    word!(game.CINEMATICSPR_PREINSTR_FUNC[(k >> 1)] = NO_PRE_INSTRUCTION);
}

/// Instruction - start the title's scene 3 pan ($8B:9E58), with its
/// gradient; returns `cursor`.
pub fn start_scene_3<H: Host + ?Sized>(cursor: u16, game: &mut H) -> u16 {
    crate::title_screen::start_title_pan(3, cursor, game);
    crate::room_visuals::initialize_title_gradient(game);
    cursor
}

fn spawn<H: Host + ?Sized>(
    game: &mut H,
    objects: CinematicObjects,
    definition: u16,
    parameter: u16,
) {
    objects.spawn(definition, parameter, game);
}
