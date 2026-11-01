//! Title screen: Metroid palettes and cry, camera pans, skip and wait, sprite placement.
use crate::cinematic::{place_cinematic_sprite, set_cinematic_backdrop};
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{Cinematic, Word};
use crate::constants::buttons::{K_BUTTON_A, K_BUTTON_B, K_BUTTON_START};
use crate::menus::prelude::words;
use crate::ram::RawRam as _;
use crate::ram::word;

/// Bank $8B cinematic functions of the title sequence.
mod function {
    /// $8B:9A47: the return that leaves the cinematic idle.
    pub const IDLE: u16 = 0x9a47;
    /// $8B:9D17: scene 0, panning left over the lower half.
    pub const PAN_LEFT_LOWER: u16 = 0x9d17;
    /// $8B:9D90: scene 1, panning left over the upper half.
    pub const PAN_LEFT_UPPER: u16 = 0x9d90;
    /// $8B:9E12: scene 2, panning down.
    pub const PAN_DOWN: u16 = 0x9e12;
    /// $8B:9E8B: scene 3, zooming out.
    pub const ZOOM_OUT: u16 = 0x9e8b;
    /// $8B:9F29: the title screen itself; earlier functions can be skipped.
    pub const TITLE_SCREEN: u16 = 0x9f29;
    /// $8B:9F52: transition to the file select menu.
    pub const TO_FILE_SELECT: u16 = 0x9f52;
    /// $8B:9FAE: transition to the demos.
    pub const TO_DEMOS: u16 = 0x9fae;
}

/// The scrolling text cinematic objects, one per panning scene.
mod scrolling_text {
    pub const NINTENDO: u16 = 0xa0f5;
    pub const PRESENTS: u16 = 0xa0fb;
    pub const METROID_3: u16 = 0xa101;
}

/// Buttons that skip ahead or start the game from the title.
const CONFIRM_BUTTONS: u16 = (K_BUTTON_B | K_BUTTON_START | K_BUTTON_A) as u16;

const _: () = assert!(CONFIRM_BUTTONS == 0x9080);
const _: () = assert!(function::TITLE_SCREEN.wrapping_neg() == 0x60d7);

pub const METROID_PAL_1: [u16; 16] = [
    0x1000, 0x6bf5, 0x2e41, 0x2da1, 0x2d01, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
pub const METROID_PAL_2: [u16; 16] = [
    0x3800, 0x6bf5, 0x6e1, 0x641, 0x5a1, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
pub const METROID_PAL_3: [u16; 16] = [
    0x3800, 0x77f8, 0x1344, 0x12a4, 0x1204, 0x6abf, 0x249f, 0x1c77, 0x146d, 0x1067, 0x5bff, 0x4b38,
    0x3a72, 0x7bd3, 0x7fff, 0x6b43,
];
pub const METROID_PAL_4: [u16; 16] = [
    0x3800, 0x7ffb, 0x1fa7, 0x1f07, 0x1e67, 0x771f, 0x30ff, 0x28da, 0x20d0, 0x1cca, 0x67ff, 0x579b,
    0x46d5, 0x7bd6, 0x7fff, 0x77a6,
];

pub fn title_metroid_cry<H: crate::menus::host::Host + ?Sized>(
    sound: u16,
    cursor: u16,
    game: &mut H,
) -> u16 {
    game.queue_sound(2, (sound) as u8, 6);
    cursor
}

pub fn title_metroid_palette<H: crate::menus::host::Host + ?Sized>(
    palette: [u16; 16],
    cursor: u16,
    game: &mut H,
) -> u16 {
    crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 63) <- table(palette, 15));
    cursor
}

pub fn start_title_pan<H: crate::menus::host::Host + ?Sized>(
    stage: usize,
    cursor: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    let (next, zoom, x, y) = [
        (function::PAN_LEFT_LOWER, 72, 315, 225),
        (function::PAN_LEFT_UPPER, 96, 44, -155i16 as u16),
        (function::PAN_DOWN, 96, -177i16 as u16, -160i16 as u16),
        (function::ZOOM_OUT, 67, 0, 0),
    ][stage];
    game.ram().write_byte(crate::ram::mirror::TM, 17);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = next
    });
    *cine.mode7_zoom() = zoom;
    *cine.x() = x;
    *cine.x_sub() = 0;
    *cine.y() = y;
    *cine.y_sub() = 0;
    *cine.x_speed() = if stage < 2 { -2i16 as u16 } else { 0 };
    *cine.x_subspeed() = if stage < 2 { 0x8000 } else { 0 };
    if stage >= 2 {
        *cine.y_speed() = u16::from(stage == 2);
        *cine.y_subspeed() = if stage == 2 { 0x8000 } else { 0 };
    }
    if matches!(stage, 0 | 2) {
        set_cinematic_backdrop(true, game);
    }
    cursor
}

pub fn advance_title_pan<H: crate::menus::host::Host + ?Sized>(
    stage: usize,
    cine: &mut Cinematic,
    game: &mut H,
) {
    // The last stage pans vertically.
    let (whole, fraction, speed, subspeed) = if stage == 2 {
        (Word::Y, Word::YSub, Word::YSpeed, Word::YSubspeed)
    } else {
        (Word::X, Word::XSub, Word::XSpeed, Word::XSubspeed)
    };
    let cutoff = [7, 176, -163][stage];
    let velocity = words!((*cine.word(speed), *cine.word(subspeed),));
    cine.add_pair(whole, fraction, velocity);
    let negative = (*cine.word(whole) as i32 + cutoff) & 0x8000 != 0;
    // The vertical pan ends on the opposite side of its cutoff.
    if negative != (stage == 2) {
        game.ram().write_byte(crate::ram::mirror::TM, 16);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::IDLE
        });
        let definition = [
            scrolling_text::NINTENDO,
            scrolling_text::PRESENTS,
            scrolling_text::METROID_3,
        ][stage];
        CinematicObjects::Sprites.spawn(definition, function::IDLE, cine, game);
        set_cinematic_backdrop(false, game);
    }
    request_title_skip(cine, game);
}

pub fn request_title_skip<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    // Before the title screen: the original adds the function's negation plus one.
    if (crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function as i32
        + (function::TITLE_SCREEN.wrapping_neg() as i32 + 1))
        & 0x8000
        != 0
        && word!(game.JOYPAD1_NEWKEYS) & CONFIRM_BUTTONS != 0
        && *cine.skip_to_title_index() == 0
    {
        *cine.skip_to_title_index() = 1;
        crate::screen_fade::store(game, 0, 2);
    }
}

pub fn advance_title_wait<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let expired = {
        let flow_word = crate::session_cohort::load::<crate::flow::Flow, _>(game)
            .demo_timer
            .wrapping_sub(1);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.demo_timer = flow_word
        });
        flow_word
    } & 0x8000
        != 0;
    if crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_timer == 0 || expired {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::TO_DEMOS
        });
    } else {
        if word!(game.JOYPAD1_NEWKEYS) & CONFIRM_BUTTONS == 0 {
            return;
        }
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = function::TO_FILE_SELECT
        });
    }
    crate::screen_fade::store(game, 2, 2);
}

pub fn initialize_title_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    variant: usize,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let (y, palette, effect) =
        [(48, 1024, 0xe194), (81, 512, 0xe198), (196, 2048, 0xe19c)][variant];
    place_cinematic_sprite(object, (128, y), palette, cine);
    crate::palette_effects::execution::leased::spawn(effect, game);
}
