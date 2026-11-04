//! The cinematic game states and what they share: the Nintendo logo, the
//! title sequence, intro and ending frames, the final screen's background
//! objects, the cinematic sprite instructions and the metroid egg's particle
//! pre-instructions (bank $8B:8000-$8B:A39F and $8B:D443-$8B:D47F).
//!
use crate::cinematic::CinematicDestination;
use crate::cinematic::{advance_cinematic_coordinate, advance_cinematic_fade, set_fixed_color};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::Cinematic;
use crate::display::control::{Brightness, FixedColor, Layers};
use crate::display::preparation::Field;
use crate::display::video::{
    Axis, Background, Command, Control, Mirror, Mode7Parameter, ScrollOffset, apply_controls,
    apply_mirrored,
};
use crate::display_refresh::GraphicsAsset;
use crate::menu_state::OptionsMenu;
use crate::visual_state::CinematicFade;

/// The cinematic functions run in bank $8B.
const BANK: u32 = 0x8b;
/// The Nintendo logo's spritemap at $8C:80BB, centred.
const NINTENDO_LOGO: u16 = 0x80bb;
const NINTENDO_LOGO_POSITION: (u16, u16) = (128, 112);
/// The title palettes at $8C:E1E9.
const TITLE_PALETTES: u32 = 0x8c_e1e9;
/// The logo holds for 120 frames.
const LOGO_HOLD: u16 = 120;
/// The final screen's background objects run from $8B:E504 on.
const FINAL_SCREEN: u16 = 0xe504;
/// The instruction that starts the title's scene 3 pan, $8B:9E58.
pub const START_SCENE_3: u32 = 0x8b_9e58;
/// The egg particles fall away below Y 168.
const PARTICLE_FLOOR: u16 = 168;
/// What the egg particles become: the egg particle's and slime drop's
/// landing lists.
const PARTICLE_LANDED: u16 = 0xce53;
const SLIME_LANDED: u16 = 0xcd71;
const NO_PRE_INSTRUCTION: u16 = 0x93d9;

/// The egg particles' and slime drops' velocity tables, 16.16 pairs
/// ($8B:A9xx-$8B:AB9x).
const PARTICLE_X: [i16; 12] = [
    -1, 0x4000, 0, 0x4000, -1, -0x8000, -1, 0x2000, 0, -0x8000, 0, 0x2000,
];
const PARTICLE_Y: [u16; 82] = [
    0xfffe, 0, 0xfffe, 0xc000, 0xfffe, 0xa000, 0xfffe, 0x8000, 0xfffe, 0x6000, 0xfffe, 0x4000,
    0xfffe, 0x2000, 0xffff, 0, 0xffff, 0xc000, 0xffff, 0xa000, 0xffff, 0x8000, 0xffff, 0x6000,
    0xffff, 0x4000, 0xffff, 0x2000, 0, 0, 0, 0x2000, 0, 0x4000, 0, 0x6000, 0, 0x8000, 0, 0xa000, 0,
    0xc000, 0, 0xe000, 1, 0, 1, 0x2000, 1, 0x4000, 1, 0x6000, 1, 0x8000, 1, 0xa000, 1, 0xc000, 1,
    0xe000, 2, 0, 2, 0x2000, 2, 0x4000, 2, 0x6000, 2, 0x8000, 2, 0xa000, 2, 0xc000, 2, 0xe000,
    0x9dad, 0x991b, 0x1b7d, 0x97ad, 0x991a, 0x1a7d,
];
const SLIME_X: [i16; 10] = [-1, 0, -1, -0x8000, 1, 0, 0, -0x8000, -1, -0x8000];
const SLIME_Y_SLOW: [i16; 124] = [
    -2, 0, -2, -0x4000, -2, -0x6000, -2, -0x8000, -2, 0x6000, -2, 0x4000, -2, 0x2000, -1, 0, -1,
    -0x4000, -1, -0x6000, -1, -0x8000, -1, 0x6000, -1, 0x4000, -1, 0x2000, 0, 0, 0, 0x2000, 0,
    0x4000, 0, 0x6000, 0, -0x8000, 0, -0x6000, 0, -0x4000, 0, -0x2000, 1, 0, 1, 0x2000, 1, 0x4000,
    1, 0x6000, 1, -0x8000, 1, -0x6000, 1, -0x4000, 1, -0x2000, 2, 0, 2, 0x2000, 2, 0x4000, 2,
    0x6000, 2, -0x8000, 2, -0x6000, 2, -0x4000, 2, -0x2000, 3, 0, 3, 0x2000, 3, 0x4000, 3, 0x6000,
    3, -0x8000, 3, -0x6000, 3, -0x4000, 3, -0x2000, 4, 0, 4, 0x2000, 4, 0x4000, 4, 0x6000, 4,
    -0x8000, 4, -0x6000, 4, -0x4000, 4, -0x2000, 5, 0, 5, 0x2000, 5, 0x4000, 5, 0x6000, 5, -0x8000,
    5, -0x6000, 5, -0x4000, 5, -0x2000,
];
const SLIME_Y_FAST: [i16; 138] = [
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

fn run_function<H: Host>(cine: &mut Cinematic, game: &mut H) {
    let address = (BANK << 16)
        | u32::from(crate::sequence_state::Access::sequence_flow(&*game).cinematic_function);
    crate::cinematic_dispatch::run_function(address, cine, game);
}

/// Whether the cinematic function resumes after a wait for NMIs.
pub fn resuming<H: Host>(game: &H) -> bool {
    game.screen_primary() != 0
}

/// Wait for `count` NMIs inside a cinematic function (`LDX #count-1;
/// JSL $808338; DEX; BPL`): true while the function must return and the
/// game state yield; the next frame's call of the same function resumes here.
pub fn wait_for_nmis<H: Host>(count: u8, game: &mut H) -> bool {
    let mut left = game.screen_primary();
    if left == 0 {
        left = count;
    }
    while crate::display_refresh::wait(true, game) == 0 {
        left -= 1;
        if left == 0 {
            game.screen_set_primary(0);
            return false;
        }
    }
    game.screen_set_primary(left);
    true
}

/// The intro and ending's frame counters.
fn count_frames(cine: &mut Cinematic) {
    if (*cine.intro_frame_counter() & 0x8000) == 0 {
        *cine.intro_frame_counter() = cine.intro_frame_counter().wrapping_add(1);
    }
    *cine.frame_counter() = cine.frame_counter().wrapping_add(1);
}

/// Draws the Nintendo logo.
pub fn draw_nintendo_logo<H: crate::cinematic_host::Scene + ?Sized>(game: &mut H) {
    crate::sprite_output::paletted(
        0x8c,
        NINTENDO_LOGO,
        NINTENDO_LOGO_POSITION,
        0,
        crate::sprite_drawing::Clip::Bounded,
        game,
    );
}

/// The producer name of the logo's display commands.
const LOGO: &str = "nintendo logo";

/// The logo's last stage, after its fade out: wait for the NMI, then blank
/// the screen.
const LOGO_END: u8 = 6;

/// Whether the logo's next frame is its last, which only waits for the NMI
/// and blanks the screen ([`end_nintendo_logo`]). Boot runs that frame
/// without a lease, since its own work follows on the same frame.
pub fn nintendo_logo_ending(
    game: &(impl crate::presentation_workspace::ContinuationAccess + ?Sized),
) -> bool {
    game.screen_primary() == LOGO_END
}

/// The logo's last frame: resume a pending NMI, then blank the screen and end
/// the logo; nonzero when it yields to wait again.
pub fn end_nintendo_logo<
    H: crate::display::video::Video
        + crate::display::preparation::Access
        + crate::ram::CompatibilityRam
        + crate::display_refresh::DisplayOwner
        + crate::presentation_workspace::ContinuationAccess
        + ?Sized,
>(
    game: &mut H,
) -> u8 {
    let yielded: u8 = crate::display_refresh::wait(true, game);
    if yielded != 0 {
        game.screen_set_primary(LOGO_END);
        return yielded;
    }
    apply_mirrored(
        game,
        LOGO,
        Control::Brightness(Brightness::BLANK_FULL),
        Field::Brightness,
        false,
    );
    game.screen_set_primary(0);
    0
}

/// The Nintendo logo coroutine on the boot frame's lease: set up the display,
/// then fade the logo in, hold it and fade it out, yielding to the frame loop
/// at each refresh.
pub fn nintendo_logo<H: Host>(game: &mut H) -> u8 {
    crate::probe::hit("coroutine.logo");
    let mut stage = game.screen_primary();
    if stage == 0 {
        apply_mirrored(
            game,
            LOGO,
            Control::Brightness(Brightness::BLANK_FULL),
            Field::Brightness,
            false,
        );
        crate::sprite_drawing::reset_cursor(game);
        crate::sprite_drawing::clear_high_table(game);
        crate::sprite_drawing::finish_frame(game);
        game.session_mut().presentation.clear_samus_requests();
        game.words_stored();
        crate::display_refresh::initialize_io_and_oam(None, game);
        apply_controls(
            game,
            LOGO,
            &[
                (Control::BackgroundMode(1), Mirror::After(Field::Mode)),
                (Control::Mosaic(0), Mirror::After(Field::Mosaic)),
                (
                    Control::Tilemap(Background::Bg1, 0),
                    Mirror::After(Field::Screens0),
                ),
                (
                    Control::Tilemap(Background::Bg2, 0),
                    Mirror::After(Field::Screens1),
                ),
                (
                    Control::Tilemap(Background::Bg3, 0),
                    Mirror::After(Field::Screens2),
                ),
                (
                    Control::Tilemap(Background::Bg4, 0),
                    Mirror::After(Field::Screens3),
                ),
                (
                    Control::CharactersBg1Bg2(0),
                    Mirror::After(Field::Characters0),
                ),
                (
                    Control::CharactersBg3Bg4(0),
                    Mirror::After(Field::Characters1),
                ),
            ],
        );
        for background in Background::ALL {
            for axis in [Axis::Horizontal, Axis::Vertical] {
                let scroll = ScrollOffset { background, axis };
                game.apply(LOGO, Command::Scroll { scroll, offset: 0 });
            }
        }
        apply_controls(
            game,
            LOGO,
            &[
                (Control::VramStep(0), Mirror::None),
                (
                    Control::Mode7Selection(0),
                    Mirror::After(Field::AffineSelection),
                ),
            ],
        );
        // One byte into each Mode 7 latch, as the original wrote.
        for parameter in Mode7Parameter::ALL {
            game.apply(LOGO, Command::Control(Control::Mode7Latch(parameter, 0)));
        }
        apply_controls(
            game,
            LOGO,
            &[
                (
                    Control::WindowsBg1Bg2(0),
                    Mirror::After(Field::WindowSelection0),
                ),
                (
                    Control::WindowsBg3Bg4(0),
                    Mirror::After(Field::WindowSelection1),
                ),
                (
                    Control::WindowsSpritesColor(0),
                    Mirror::After(Field::WindowSelection2),
                ),
                (
                    Control::Window1Left(0),
                    Mirror::After(Field::WindowPositions0),
                ),
                (
                    Control::Window1Right(0),
                    Mirror::After(Field::WindowPositions1),
                ),
                (
                    Control::Window2Left(0),
                    Mirror::After(Field::WindowPositions2),
                ),
                (
                    Control::Window2Right(0),
                    Mirror::After(Field::WindowPositions3),
                ),
                (
                    Control::WindowLogicBackgrounds(0),
                    Mirror::After(Field::WindowLogic0),
                ),
                (
                    Control::WindowLogicSpritesColor(0),
                    Mirror::After(Field::WindowLogic1),
                ),
                (
                    Control::MainScreen(Layers::SPRITES),
                    Mirror::After(Field::MainLayers),
                ),
                (
                    Control::MainWindowMask(Layers::NONE),
                    Mirror::After(Field::MainWindow),
                ),
                (
                    Control::SubScreen(Layers::NONE),
                    Mirror::After(Field::SubLayers),
                ),
                (
                    Control::SubWindowMask(Layers::NONE),
                    Mirror::After(Field::SubWindow),
                ),
                (
                    Control::ColorSelection(0),
                    Mirror::After(Field::NextColorSelection),
                ),
                (Control::ColorMath(0), Mirror::After(Field::NextColorMath)),
            ],
        );
        game.apply(LOGO, Command::FixedColor(FixedColor::all(0)));
        set_fixed_color([32, 64, 0x80], game);
        apply_mirrored(
            game,
            LOGO,
            Control::ScreenSettings(0),
            Field::Geometry,
            false,
        );
        GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        crate::frame_display_byte!(game, Field::Brightness => 0x80);
        crate::cinematic::transfer_cinematic_words(
            256,
            2,
            &[(
                CinematicDestination::Palette {
                    offset: 0,
                    native: false,
                },
                crate::cinematic::cinematic_rom_words(TITLE_PALETTES, 0, game),
            )],
            game,
        );
        crate::display_refresh::enable(true, game);
        crate::screen_fade::store(game, 1, 1);
    }
    // Stages 1..=5 resume a pending NMI before drawing the next frame.
    while stage <= LOGO_END {
        if stage == LOGO_END {
            return end_nintendo_logo(game);
        }
        if stage != 0 {
            let yielded: u8 = crate::display_refresh::wait(true, game);
            if yielded != 0 {
                game.screen_set_primary(stage);
                return yielded;
            }
            match stage {
                2 => crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
                    options.screen_index = LOGO_HOLD
                }),
                4 => {
                    crate::screen_fade::store(game, 1, 1);
                }
                _ => {}
            }
        }
        crate::sprite_drawing::clear_high_table(game);
        crate::sprite_drawing::reset_cursor(game);
        draw_nintendo_logo(game);
        stage = match stage {
            0 | 1 => {
                crate::probe::hit("coroutine.logo.in");
                if advance_cinematic_fade(CinematicFade::FastIn, game) {
                    2
                } else {
                    1
                }
            }
            2 | 3 => {
                crate::probe::hit("coroutine.logo.hold");
                let held = crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
                    options.screen_index = options.screen_index.wrapping_sub(1);
                    options.screen_index
                });
                if held == 0 { 4 } else { 3 }
            }
            _ => {
                crate::probe::hit("coroutine.logo.out");
                if advance_cinematic_fade(CinematicFade::FastOut, game) {
                    LOGO_END
                } else {
                    5
                }
            }
        };
        crate::sprite_drawing::finish_frame(game);
    }
    game.screen_set_primary(0);
    0
}

/// Game state 1 - title sequence ($8B:9A22).
pub fn title_sequence<H: Host>(game: &mut H) -> u8 {
    Cinematic::lend(game, |game, cine| {
        run_function(cine, game);
        CinematicObjects::Sprites.advance(cine, game);
        CinematicObjects::Mode7.advance(cine, game);
        crate::palette_effects::execution::leased::advance(game);
        crate::cinematic_objects::draw_cinematic_sprites(false, cine, game);
        crate::title_screen::request_title_skip(cine, game);
        crate::title_sequence::handle_transitions(cine, game);
        crate::intro_objects::update_cinematic_transform(false, cine, game);
        0
    })
}

/// Runs Samus during the intro once she is shown (`intro_samus_display` is
/// `display`), with her timers.
pub fn handle_samus_during_intro<H: Host>(display: u16, game: &mut H) {
    if display != 0 {
        game.advance_intro_player();
        if game.invincibility_timer() != 0 {
            game.set_invincibility_timer(game.invincibility_timer() - 1);
        }
        if game.knockback_timer() != 0 {
            game.set_knockback_timer(game.knockback_timer() - 1);
        }
    }
}

/// Game state 1Eh/22h/25h (intro / Ceres goes boom, Samus goes to Zebes /
/// Ceres goes boom with Samus) ($8B:A35B).
pub fn intro<H: Host>(game: &mut H) -> u8 {
    let display = Cinematic::lend(game, |game, cine| {
        run_function(cine, game);
        if resuming(game) {
            return None;
        }
        count_frames(cine);
        Some(*cine.intro_samus_display())
    });
    let Some(display) = display else {
        return 1;
    };
    // Samus's frame reads the table as enemy projectiles, so it runs between
    // two loans, on the table as the cinematic function left it.
    handle_samus_during_intro(display, game);
    Cinematic::lend(game, |game, cine| {
        CinematicObjects::Sprites.advance(cine, game);
        CinematicObjects::Mode7.advance(cine, game);
        CinematicObjects::Background.advance(cine, game);
        crate::intro_pages::advance_text_glow(cine, game);
        crate::palette_effects::execution::leased::advance(game);
        crate::cinematic_display::draw_intro_sprites(cine, game);
        crate::intro_objects::update_cinematic_transform(true, cine, game);
        0
    })
}

/// Game state 27h (ending and credits) ($8B:D443).
pub fn ending_and_credits<H: Host>(game: &mut H) -> u8 {
    Cinematic::lend(game, |game, cine| {
        run_function(cine, game);
        if resuming(game) {
            return 1;
        }
        count_frames(cine);
        CinematicObjects::Sprites.advance(cine, game);
        advance_final_screen_background(cine, game);
        crate::credits::advance_credits(cine, game);
        crate::palette_effects::execution::leased::advance(game);
        crate::cinematic_objects::draw_cinematic_sprites(true, cine, game);
        crate::shooting_star_motion::advance_shooting_stars(*cine.shooting_stars_enabled(), game);
        crate::intro_objects::update_cinematic_transform(true, cine, game);
        0
    })
}

/// Handle final screen cinematic BG objects ($8B:D474): from the fade from
/// white on.
pub fn advance_final_screen_background<H: Host>(cine: &mut Cinematic, game: &mut H) {
    let function = crate::sequence_state::Access::sequence_flow(&*game).cinematic_function;
    if function.wrapping_sub(FINAL_SCREEN) < 0x8000 {
        CinematicObjects::Background.advance(cine, game);
    }
}

/// Runs cinematic sprite instruction `ea` for object `k` at cursor `j`;
/// returns the next cursor.
pub fn run_sprite_instruction<H: Host>(
    ea: u32,
    k: u16,
    j: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    if let Some(cursor) =
        crate::cinematic_objects::execute_cinematic_sprite_instruction(ea, k, j, cine, game)
    {
        cursor
    } else if ea == START_SCENE_3 {
        crate::title_sequence::start_scene_3(j, cine, game)
    } else {
        game.rtl_unreachable() as u16
    }
}

/// The original's assertion on a velocity index: its expression, line and
/// function.
struct NativeCheck {
    expression: &'static [u8],
    line: u32,
    function: &'static [u8],
}

/// Advances egg particle `k` on one axis by the velocity its index selects
/// from `table`, a table of 16.16 pairs; the original asserted the index.
fn advance_particle_axis<H: Host, W: Copy + Into<Word>>(
    cine: &mut Cinematic,
    game: &mut H,
    k: u16,
    vertical: bool,
    table: &[W],
    check: NativeCheck,
) {
    let slot = (k as i32 >> 1) as usize;
    // The velocity indices are the goto timers' low and high bytes.
    let index = 2 * usize::from(cine.spr_goto_timer(slot).to_le_bytes()[usize::from(vertical)]);
    if !(index + 1 < table.len()) {
        game.native_assertion(check.expression, check.line, check.function);
    }
    let (high, low) = (table[index].into().0, table[index + 1].into().0);
    let velocity = (u32::from(high) << 16) | u32::from(low);
    advance_cinematic_coordinate(k, vertical, velocity, cine);
}

/// A velocity table entry as the original read it, a 16-bit word.
struct Word(u16);

impl From<i16> for Word {
    fn from(value: i16) -> Self {
        Word(value as u16)
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value)
    }
}

const PARTICLE_FUNCTION: &[u8] = b"void SetSomeStuffForSpriteObject_7(uint16)\0";
const SLIME_FUNCTION: &[u8] = b"void SetSomeStuffForSpriteObject_9(uint16)\0";

/// Pre-instruction - cinematic sprite object $CECD-$CEEB (metroid egg
/// particle) ($8B:A994): fly until below the floor, then land.
pub fn egg_particle<H: Host>(k: u16, cine: &mut Cinematic, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    advance_particle_axis(
        cine,
        game,
        k,
        false,
        &PARTICLE_X,
        NativeCheck {
            expression: b"v3 + 1 < 12\0",
            line: 2389,
            function: PARTICLE_FUNCTION,
        },
    );
    advance_particle_axis(
        cine,
        game,
        k,
        true,
        &PARTICLE_Y,
        NativeCheck {
            expression: b"v6 + 1 < 82\0",
            line: 2392,
            function: PARTICLE_FUNCTION,
        },
    );
    if ((*cine.bg_arr8(slot)).wrapping_sub(PARTICLE_FLOOR) as i16) < 0 {
        *cine.spr_goto_timer(slot) = cine.spr_goto_timer(slot).wrapping_add(256);
    } else {
        *cine.spr_instruction_timer(slot) = 1;
        *cine.spr_instruction_pointer(slot) = PARTICLE_LANDED;
    }
}

/// Pre-instruction - cinematic sprite object $CEF1 (metroid egg slime drops)
/// ($8B:AAB3): odd drops fall slower; below the floor they splash.
pub fn slime_drop<H: Host>(k: u16, cine: &mut Cinematic, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    advance_particle_axis(
        cine,
        game,
        k,
        false,
        &SLIME_X,
        NativeCheck {
            expression: b"v3 + 1 < 10\0",
            line: 2442,
            function: SLIME_FUNCTION,
        },
    );
    if (*cine.spr_goto_timer(slot) & 1) != 0 {
        advance_particle_axis(
            cine,
            game,
            k,
            true,
            &SLIME_Y_SLOW,
            NativeCheck {
                expression: b"v12 + 1 < 124\0",
                line: 2446,
                function: SLIME_FUNCTION,
            },
        );
    } else {
        advance_particle_axis(
            cine,
            game,
            k,
            true,
            &SLIME_Y_FAST,
            NativeCheck {
                expression: b"v6 + 1 < 138\0",
                line: 2450,
                function: SLIME_FUNCTION,
            },
        );
    }
    if !(((*cine.bg_arr8(slot)).wrapping_sub(PARTICLE_FLOOR) as i16) < 0) {
        *cine.spr_instruction_timer(slot) = 1;
        *cine.spr_instruction_pointer(slot) = SLIME_LANDED;
        *cine.spr_pre_instruction(slot) = NO_PRE_INSTRUCTION;
    } else {
        *cine.spr_goto_timer(slot) = cine.spr_goto_timer(slot).wrapping_add(256);
    }
}
