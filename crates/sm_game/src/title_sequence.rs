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
use crate::cinematic_state::Cinematic;
use crate::display::sequence::scroll::BgScroll;
use crate::display_refresh::GraphicsAsset;
use crate::menu_state::OptionsMenu;
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

/// A transition between the title and the demos.
type Transition<H> = fn(&mut Cinematic, &mut H);

/// The cinematic transition states in `skip_to_title_index`.
fn run_transition<H: Host + ?Sized>(state: u16, cine: &mut Cinematic, game: &mut H) {
    let transitions: [Option<Transition<H>>; 4] = [
        None,
        Some(fade_out_to_demo),
        Some(start_demo_logo),
        Some(fade_in_title),
    ];
    transitions[state as usize].expect("non-null function pointer")(cine, game);
}

/// Steps the current transition between the title and the demos. Was
/// `HandleCinematicsTransitions`.
pub fn handle_transitions<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if *cine.skip_to_title_index() != 0 {
        let state = *cine.skip_to_title_index();
        run_transition(state, cine, game);
    }
}

/// Was `HandleCinematicsTransitions_1`.
fn fade_out_to_demo<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if advance_cinematic_fade(CinematicFade::FastOut, game) {
        *cine.skip_to_title_index() = 2;
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::WAITING
        });
        crate::music_requests::queue_delayed_8(DEMO_MUSIC, game);
    }
}

/// Was `HandleCinematicsTransitions_2`.
fn start_demo_logo<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    clear_cinematic_sprites(cine);
    spawn(cine, game, CinematicObjects::Sprites, object::COPYRIGHT, 0);
    // The palette the first sprite initializer leaves in A.
    spawn(
        cine,
        game,
        CinematicObjects::Sprites,
        object::DEMO_LOGO,
        0x0400,
    );
    crate::screen_fade::set_both(game, 0, 2);
    *cine.mode7_zoom() = 256;
    *cine.x() = 0;
    *cine.x_sub() = 0;
    *cine.y() = 0;
    *cine.y_sub() = 0;
    *cine.x_speed() = 0;
    *cine.x_subspeed() = 0;
    *cine.y_speed() = 0;
    *cine.y_subspeed() = 0;
    *cine.skip_to_title_index() = 3;
    crate::word_transfers!(game, ascending, 128; le(PALETTE_BUFFER, 128) <- rom(TITLE_PALETTES, 128));
    crate::cinematic::set_cinematic_colors(&[201], 0x7fff, game);
    crate::cinematic::set_cinematic_colors(&[202], 32128, game);
    crate::palette_effects::execution::leased::clear(game);
    crate::palette_effects::execution::leased::spawn(TITLE_GLOW_1, game);
    crate::palette_effects::execution::leased::spawn(TITLE_GLOW_2, game);
    set_cinematic_backdrop(false, game);
    game.ram().write_byte(crate::ram::mirror::TM, 17);
    crate::room_visuals::initialize_title_gradient(game);
}

/// Was `HandleCinematicsTransitions_3`.
fn fade_in_title<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if advance_cinematic_fade(CinematicFade::FastIn, game) {
        *cine.skip_to_title_index() = 0;
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.demo_timer = DEMO_DELAY
        });
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::TITLE_SCREEN
        });
    }
}

/// Load title sequence graphics ($8B:9B87). Was `LoadTitleSequenceGraphics`.
pub fn load_graphics<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    crate::cinematic_display::configure_cinematic_display(CinematicDisplay::Title, cine, game);
    word!(game.CUR_IRQ_HANDLER = 0);
    word!(game.IRQHANDLER_NEXT_HANDLER = 0);
    crate::word_transfers!(game, ascending, 256; le(PALETTE_BUFFER, 0) <- rom(TITLE_PALETTES, 0));
    GraphicsAsset::Decompress(0x94e000, 0x10000).load(game);
    GraphicsAsset::Decompress(0x96fc04, 0x14000).load(game);
    GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
    GraphicsAsset::Decompress(0x95a5e1, 0x19000).load(game);
    GraphicsAsset::VramLowBytes(0x7f4000, 0x0000, 0x1000).load_planes(0xff, false, game);
    crate::cinematic_display::reset_cinematic_transform(128, 128, game);
    crate::display_refresh::enable(true, game);
    crate::screen_fade::set_both(game, 1, 1);
    crate::hdma_objects::set_hdma_enabled(true, game);
    crate::palette_effects::execution::leased::set_enabled(true, game);
    crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
        words.fx_layer_blending_config_a = 0
    });
    crate::palette_effects::execution::leased::spawn(TITLE_GLOW_1, game);
    crate::palette_effects::execution::leased::spawn(TITLE_GLOW_2, game);
    // The second title palette allocation leaves its slot offset in A.
    spawn(
        cine,
        game,
        CinematicObjects::Mode7,
        object::BABY_METROID,
        0x000c,
    );
}

/// Cinematic function - title sequence - setup ($8B:9B68). Was
/// `CinematicFunctionOpening`.
pub fn setup<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    load_graphics(cine, game);
    crate::music_requests::queue_delayed_8(TITLE_MUSIC_DATA, game);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::NOTHING
    });
    spawn(
        cine,
        game,
        CinematicObjects::Sprites,
        object::SCROLLING_1994,
        function::NOTHING,
    );
    crate::music_requests::queue_delayed_8(TITLE_THEME, game);
}

/// Cinematic function - title sequence - scene 3 - zooming out ($8B:9E8B):
/// every other frame, zoom out to full size, then show the title logo. Was
/// `CinematicFunc_Func7`.
pub fn zooming_out<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if (word!(game.NMI_FRAME_COUNTER_WORD) & 1) == 0 {
        if ((*cine.mode7_zoom()).wrapping_sub(256) as i16) < 0 {
            *cine.mode7_zoom() = cine.mode7_zoom().wrapping_add(1);
        } else {
            *cine.mode7_zoom() = 256;
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = function::WAITING
            });
            spawn(
                cine,
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
fn finish<H: Host + ?Sized>(
    next: u16,
    cine: &mut Cinematic,
    game: &mut H,
    clear_index: impl FnOnce(&mut H),
) {
    crate::display_refresh::enable(true, game);
    game.ram().write_byte(crate::ram::mirror::BGMODE, 1);
    crate::screen_fade::set_both(game, 0, 0);
    crate::session_cohort::with(game, |_, scroll: &mut BgScroll| {
        scroll.bg2 = [0, 0];
        scroll.bg3 = [0, 0];
    });
    *cine.mode7_angle() = 0;
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = function::SETUP
    });
    crate::palette_effects::execution::leased::set_enabled(false, game);
    crate::palette_effects::execution::leased::clear(game);
    cine.clear();
    crate::word_transfers!(game, descending, 256; le(HDMA_TABLE_1, 255) <- value(0));
    word!(game.GAME_STATE = next);
    clear_index(game);
}

/// Cinematic function - title sequence - transition to file select menu
/// ($8B:9F52). Was `CinematicFunc_Func10`.
pub fn to_file_select<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if advance_cinematic_fade(CinematicFade::SlowOut, game) {
        finish(FILE_SELECT_MENUS, cine, game, |game| {
            crate::session_cohort::with(game, |_, options: &mut OptionsMenu| {
                options.screen_index = 0
            })
        });
    }
}

/// Cinematic function - title sequence - transition to demos ($8B:9FAE). Was
/// `CinematicFunc_Func9`.
pub fn to_demos<H: Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if advance_cinematic_fade(CinematicFade::SlowOut, game) {
        finish(TRANSITION_TO_DEMO, cine, game, |game| {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.demo_scene = 0)
        });
    }
}

/// Pre-instruction - cinematic sprite object $A0EF ('1994' scrolling text)
/// ($8B:9CCF): full brightness, once. Was `sub_8B9CCF`.
pub fn scrolling_1994<H: Host + ?Sized>(k: u16, cine: &mut Cinematic, game: &mut H) {
    game.ram().write_byte(crate::ram::mirror::INIDISP, 15);
    *cine.spr_pre_instruction(k >> 1) = NO_PRE_INSTRUCTION;
}

/// Instruction - start the title's scene 3 pan ($8B:9E58), with its
/// gradient; returns `cursor`.
pub fn start_scene_3<H: Host + ?Sized>(cursor: u16, cine: &mut Cinematic, game: &mut H) -> u16 {
    crate::title_screen::start_title_pan(3, cursor, cine, game);
    crate::room_visuals::initialize_title_gradient(game);
    cursor
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
