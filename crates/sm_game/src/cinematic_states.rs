//! The cinematic game states and what they share: the Nintendo logo, the
//! title sequence, intro and ending frames, the final screen's background
//! objects, the cinematic sprite instructions and the metroid egg's particle
//! pre-instructions (bank $8B:8000-$8B:A39F and $8B:D443-$8B:D47F).
//!
//! Was `InitializeIoDisplayLogo_Async`, `GameState_1_OpeningCinematic_`,
//! `GameState_37_CeresGoesBoomWithSamus_`, `GameState_39_EndingAndCredits_`,
//! `MaybeHandleCinematicBgObjects`, `CallCinematicSprInstr` and
//! `SetSomeStuffForSpriteObject_7`/`9` in the original decompilation.
#![forbid(unsafe_code)]
use crate::cinematic::{advance_cinematic_coordinate, advance_cinematic_fade, set_fixed_color};
use crate::cinematic_host::Host;
use crate::cinematic_objects::CinematicObjects;
use crate::display::registers as reg;
use crate::display_refresh::GraphicsAsset;
use crate::ram::RawRam as _;
use crate::ram::mirror;
use crate::ram::word;
use crate::visual_state::CinematicFade;

/// The cinematic functions run in bank $8B.
const BANK: u32 = 0x8b;
/// $7E:0612, the logo coroutine's stage.
const LOGO_STAGE: usize = 0x612;
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
/// $7E:1B7D: the metroid egg particles' velocity indices, horizontal then
/// vertical per slot.
const PARTICLE_VELOCITY_INDEX: usize = 0x1b7d;
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

fn run_function<H: Host + ?Sized>(game: &mut H) {
    let address = (BANK << 16) | u32::from(word!(game.cinematic_function));
    game.run_cinematic_function(address);
}

/// The intro and ending's frame counters.
fn count_frames<H: Host + ?Sized>(game: &mut H) {
    if (word!(game.cinematic_var11) & 0x8000) == 0 {
        word!(game.cinematic_var11 = word!(game.cinematic_var11).wrapping_add(1));
    }
    word!(game.cinematic_var12 = word!(game.cinematic_var12).wrapping_add(1));
}

/// Draws the Nintendo logo. Was `AddNintentoLogoToOam`.
pub fn draw_nintendo_logo<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::paletted(
        0x8c,
        NINTENDO_LOGO,
        NINTENDO_LOGO_POSITION,
        0,
        crate::sprite_drawing::Clip::Bounded,
        game,
    );
}

/// The Nintendo logo coroutine: set up the display, then fade the logo in,
/// hold it and fade it out, yielding to the frame loop at each refresh.
/// Was `InitializeIoDisplayLogo_Async`.
pub fn nintendo_logo<H: Host + ?Sized>(game: &mut H) -> u8 {
    crate::probe::hit("coroutine.logo");
    let mut stage = game.compatibility_byte(LOGO_STAGE);
    if stage == 0 {
        crate::display::sequence::write_register_and_mirror(
            game,
            reg::INIDISP as u16,
            0x8f,
            mirror::INIDISP,
            false,
        );
        word!(game.oam_next_ptr = 0);
        crate::sprite_drawing::clear_high_table(game);
        crate::sprite_drawing::finish_frame(game);
        word!(game.nmi_copy_samus_halves = 0);
        word!(game.nmi_copy_samus_top_half_src = 0);
        word!(game.nmi_copy_samus_bottom_half_src = 0);
        crate::display_refresh::initialize_io_and_oam(None, game);
        crate::display::sequence::register_writes! { game;
            mirror(reg::BGMODE as u16, 1, 0x55, false); mirror(reg::MOSAIC as u16, 0, 0x57, false);
            mirror(reg::BG1SC as u16, 0, 0x58, false); mirror(reg::BG2SC as u16, 0, 0x59, false);
            mirror(reg::BG3SC as u16, 0, 0x5a, false); mirror(reg::BG4SC as u16, 0, 0x5c, false);
            mirror(reg::BG12NBA as u16, 0, 0x5d, false); mirror(reg::BG34NBA as u16, 0, 0x5e, false);
            byte(reg::BG1HOFS as u16, 0); byte(reg::BG1HOFS as u16, 0);
            byte(reg::BG1VOFS as u16, 0); byte(reg::BG1VOFS as u16, 0);
            byte(reg::BG2HOFS as u16, 0); byte(reg::BG2HOFS as u16, 0);
            byte(reg::BG2VOFS as u16, 0); byte(reg::BG2VOFS as u16, 0);
            byte(reg::BG3HOFS as u16, 0); byte(reg::BG3HOFS as u16, 0);
            byte(reg::BG3VOFS as u16, 0); byte(reg::BG3VOFS as u16, 0);
            byte(reg::BG4HOFS as u16, 0); byte(reg::BG4HOFS as u16, 0);
            byte(reg::BG4VOFS as u16, 0); byte(reg::BG4VOFS as u16, 0);
            byte(reg::VMAIN as u16, 0); mirror(reg::M7SEL as u16, 0, 0x5f, false);
            byte(reg::M7A as u16, 0); byte(reg::M7B as u16, 0);
            byte(reg::M7C as u16, 0); byte(reg::M7D as u16, 0);
            byte(reg::M7X as u16, 0); byte(reg::M7Y as u16, 0);
            mirror(reg::W12SEL as u16, 0, 0x60, false); mirror(reg::W34SEL as u16, 0, 0x61, false);
            mirror(reg::WOBJSEL as u16, 0, 0x62, false); mirror(reg::WH0 as u16, 0, 0x63, false);
            mirror(reg::WH1 as u16, 0, 0x64, false); mirror(reg::WH2 as u16, 0, 0x65, false);
            mirror(reg::WH3 as u16, 0, 0x66, false); mirror(reg::WBGLOG as u16, 0, 0x67, false);
            mirror(reg::WOBJLOG as u16, 0, 0x68, false); mirror(reg::TM as u16, 0x10, 0x69, false);
            mirror(reg::TMW as u16, 0, 0x6c, false); mirror(reg::TS as u16, 0, 0x6b, false);
            mirror(reg::TSW as u16, 0, 0x6d, false); mirror(reg::CGWSEL as u16, 0, 0x6e, false);
            mirror(reg::CGADSUB as u16, 0, 0x71, false); byte(reg::COLDATA as u16, 0xe0);
        }
        set_fixed_color([32, 64, 0x80], game);
        crate::display::sequence::write_register_and_mirror(
            game,
            reg::SETINI as u16,
            0,
            mirror::SETINI,
            false,
        );
        GraphicsAsset::Decompress(0x9580d8, 0x15000).load(game);
        GraphicsAsset::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        game.ram().write_byte(mirror::INIDISP, 0x80);
        crate::word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(TITLE_PALETTES, 0));
        game.enable_display_nmi(true);
        word!(game.screen_fade_delay = 1);
        word!(game.screen_fade_counter = 1);
    }
    // Stages 1..=6 resume a pending NMI before drawing the next frame.
    while stage <= 6 {
        if stage != 0 {
            let yielded: u8 = crate::display_refresh::wait(true, game);
            if yielded != 0 {
                game.write_owned_byte(LOGO_STAGE, stage);
                return yielded;
            }
            match stage {
                2 => word!(game.game_options_screen_index = LOGO_HOLD),
                4 => {
                    word!(game.screen_fade_delay = 1);
                    word!(game.screen_fade_counter = 1);
                }
                6 => {
                    crate::display::sequence::write_register_and_mirror(
                        game,
                        reg::INIDISP as u16,
                        0x8f,
                        mirror::INIDISP,
                        false,
                    );
                    break;
                }
                _ => {}
            }
        }
        crate::sprite_drawing::clear_high_table(game);
        word!(game.oam_next_ptr = 0);
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
                if word!(game.game_options_screen_index -= 1, reread) == 0 {
                    4
                } else {
                    3
                }
            }
            _ => {
                crate::probe::hit("coroutine.logo.out");
                if advance_cinematic_fade(CinematicFade::FastOut, game) {
                    6
                } else {
                    5
                }
            }
        };
        crate::sprite_drawing::finish_frame(game);
    }
    game.write_owned_byte(LOGO_STAGE, 0);
    0
}

/// Game state 1 - title sequence ($8B:9A22). Was `GameState_1_OpeningCinematic_`.
pub fn title_sequence<H: Host + ?Sized>(game: &mut H) -> u8 {
    run_function(game);
    CinematicObjects::Sprites.advance(game);
    CinematicObjects::Mode7.advance(game);
    game.advance_palette_effects();
    crate::cinematic_objects::draw_cinematic_sprites(false, game);
    crate::title_screen::request_title_skip(game);
    crate::title_sequence::handle_transitions(game);
    crate::intro_objects::update_cinematic_transform(false, game);
    0
}

/// Runs Samus during the intro once she is shown, with her timers. Was
/// `HandleSamusDuringIntro`.
pub fn handle_samus_during_intro<H: Host + ?Sized>(game: &mut H) {
    if word!(game.cinematic_var15) != 0 {
        game.advance_intro_player();
        if word!(game.samus_invincibility_timer) != 0 {
            word!(
                game.samus_invincibility_timer =
                    word!(game.samus_invincibility_timer).wrapping_sub(1)
            );
        }
        if word!(game.samus_knockback_timer) != 0 {
            word!(game.samus_knockback_timer = word!(game.samus_knockback_timer).wrapping_sub(1));
        }
    }
}

/// Game state 1Eh/22h/25h (intro / Ceres goes boom, Samus goes to Zebes /
/// Ceres goes boom with Samus) ($8B:A35B). Was
/// `GameState_37_CeresGoesBoomWithSamus_`.
pub fn intro<H: Host + ?Sized>(game: &mut H) -> u8 {
    run_function(game);
    count_frames(game);
    handle_samus_during_intro(game);
    CinematicObjects::Sprites.advance(game);
    CinematicObjects::Mode7.advance(game);
    CinematicObjects::Background.advance(game);
    crate::intro_pages::advance_text_glow(game);
    game.advance_palette_effects();
    crate::cinematic_display::draw_intro_sprites(game);
    crate::intro_objects::update_cinematic_transform(true, game);
    0
}

/// Game state 27h (ending and credits) ($8B:D443). Was
/// `GameState_39_EndingAndCredits_`.
pub fn ending_and_credits<H: Host + ?Sized>(game: &mut H) -> u8 {
    run_function(game);
    count_frames(game);
    CinematicObjects::Sprites.advance(game);
    advance_final_screen_background(game);
    crate::credits::advance_credits(game);
    game.advance_palette_effects();
    crate::cinematic_objects::draw_cinematic_sprites(true, game);
    crate::shooting_star_motion::advance_shooting_stars(game);
    crate::intro_objects::update_cinematic_transform(true, game);
    0
}

/// Handle final screen cinematic BG objects ($8B:D474): from the fade from
/// white on. Was `MaybeHandleCinematicBgObjects`.
pub fn advance_final_screen_background<H: Host + ?Sized>(game: &mut H) {
    if !(((word!(game.cinematic_function)).wrapping_sub(FINAL_SCREEN) as i16) < 0) {
        CinematicObjects::Background.advance(game);
    }
}

/// Runs cinematic sprite instruction `ea` for object `k` at cursor `j`;
/// returns the next cursor. Was `CallCinematicSprInstr`.
pub fn run_sprite_instruction<H: Host + ?Sized>(ea: u32, k: u16, j: u16, game: &mut H) -> u16 {
    if let Some(cursor) =
        crate::cinematic_objects::execute_cinematic_sprite_instruction(ea, k, j, game)
    {
        cursor
    } else if ea == START_SCENE_3 {
        crate::title_sequence::start_scene_3(j, game)
    } else {
        game.rtl_unreachable() as u16
    }
}

/// Advances egg particle `k` on one axis by the velocity its index selects
/// from `table`, a table of 16.16 pairs; the original asserted the index.
fn advance_particle_axis<H: Host + ?Sized, W: Copy + Into<Word>>(
    game: &mut H,
    k: u16,
    vertical: bool,
    table: &[W],
    expression: &'static [u8],
    line: u32,
    function: &'static [u8],
) {
    let slot = (k as i32 >> 1) as usize;
    let index = 2 * game
        .ram()
        .read_byte(PARTICLE_VELOCITY_INDEX + 2 * slot + usize::from(vertical))
        as usize;
    if !(index + 1 < table.len()) {
        game.native_assertion(expression, line, function);
    }
    let (high, low) = (table[index].into().0, table[index + 1].into().0);
    let velocity = (u32::from(high) << 16) | u32::from(low);
    advance_cinematic_coordinate(k, vertical, velocity, game);
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
/// particle) ($8B:A994): fly until below the floor, then land. Was
/// `SetSomeStuffForSpriteObject_7`.
pub fn egg_particle<H: Host + ?Sized>(k: u16, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    advance_particle_axis(
        game,
        k,
        false,
        &PARTICLE_X,
        b"v3 + 1 < 12\0",
        2389,
        PARTICLE_FUNCTION,
    );
    advance_particle_axis(
        game,
        k,
        true,
        &PARTICLE_Y,
        b"v6 + 1 < 82\0",
        2392,
        PARTICLE_FUNCTION,
    );
    if ((word!(game.cinematicbg_arr8[slot])).wrapping_sub(PARTICLE_FLOOR) as i16) < 0 {
        word!(
            game.cinematicspr_goto_timer[slot] =
                word!(game.cinematicspr_goto_timer[slot]).wrapping_add(256)
        );
    } else {
        word!(game.cinematicspr_instr_timer[slot] = 1);
        word!(game.cinematicspr_instr_ptr[slot] = PARTICLE_LANDED);
    }
}

/// Pre-instruction - cinematic sprite object $CEF1 (metroid egg slime drops)
/// ($8B:AAB3): odd drops fall slower; below the floor they splash. Was
/// `SetSomeStuffForSpriteObject_9`.
pub fn slime_drop<H: Host + ?Sized>(k: u16, game: &mut H) {
    let slot = (k as i32 >> 1) as usize;
    advance_particle_axis(
        game,
        k,
        false,
        &SLIME_X,
        b"v3 + 1 < 10\0",
        2442,
        SLIME_FUNCTION,
    );
    if (word!(game.cinematicspr_goto_timer[slot]) & 1) != 0 {
        advance_particle_axis(
            game,
            k,
            true,
            &SLIME_Y_SLOW,
            b"v12 + 1 < 124\0",
            2446,
            SLIME_FUNCTION,
        );
    } else {
        advance_particle_axis(
            game,
            k,
            true,
            &SLIME_Y_FAST,
            b"v6 + 1 < 138\0",
            2450,
            SLIME_FUNCTION,
        );
    }
    if !(((word!(game.cinematicbg_arr8[slot])).wrapping_sub(PARTICLE_FLOOR) as i16) < 0) {
        word!(game.cinematicspr_instr_timer[slot] = 1);
        word!(game.cinematicspr_instr_ptr[slot] = SLIME_LANDED);
        word!(game.cinematicspr_preinstr_func[slot] = NO_PRE_INSTRUCTION);
    } else {
        word!(
            game.cinematicspr_goto_timer[slot] =
                word!(game.cinematicspr_goto_timer[slot]).wrapping_add(256)
        );
    }
}
