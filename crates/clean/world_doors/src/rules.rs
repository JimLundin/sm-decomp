//! The tuning values the spec lists as parameters, with the spec's values as their default.

use core::num::Wrapping;

use enum_map::{Enum, EnumMap, enum_map};
use types::{Fixed, Heading, Px};

/// The axis a slide runs along, which sets its length and her speed: across, for a way left or
/// right, or down and up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Axis {
    /// A way left or right.
    Across,
    /// A way down or up.
    DownUp,
}

impl Axis {
    /// The axis the way she leaves by runs along.
    #[must_use]
    pub const fn of(way: Heading) -> Self {
        if way.vertical() {
            Self::DownUp
        } else {
            Self::Across
        }
    }
}

/// The rules the passage reads its tuning from. `Rules::default()` holds the spec's values, the
/// game's; any other is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// `Fading`: the fade's steps, `d`. A fade takes `d + 3` frames of work, its step `t` running
    /// 0 to `d + 2`.
    pub fade_steps: u8,
    /// `Sliding`: how far the view moves on each slide frame that moves it, `s`, in px.
    pub slide_step: Px,
    /// `Sliding`: the slide's length in slide frames, `n`, on each axis.
    pub slide_length: EnumMap<Axis, u16>,
    /// `Carrying`: her speed in the slide, `P.S` a frame, on each axis, when the doorway gives
    /// none.
    pub carry_speed: EnumMap<Axis, Fixed>,
}

/// `Fading`: the game's steps, `d` = 12.
const FADE_STEPS: u8 = 12;
/// `Sliding`: the game's step, `s` = 4 px.
const SLIDE_STEP: Px = Wrapping(4);
/// `Sliding`: the game's slide left or right, 64 frames.
const SLIDE_ACROSS: u16 = 64;
/// `Sliding`: the game's slide down or up, 57 frames.
const SLIDE_DOWN_UP: u16 = 57;
/// `Carrying`: the game's speed left or right, `0.C800` px a frame.
const SPEED_ACROSS: Fixed = Wrapping(0x0000_C800);
/// `Carrying`: the game's speed down or up, `1.8000` px a frame.
const SPEED_DOWN_UP: Fixed = Wrapping(0x0001_8000);

impl Default for Rules {
    /// The spec's values: the game as it is.
    fn default() -> Self {
        Self {
            fade_steps: FADE_STEPS,
            slide_step: SLIDE_STEP,
            slide_length: enum_map! {
                Axis::Across => SLIDE_ACROSS,
                Axis::DownUp => SLIDE_DOWN_UP,
            },
            carry_speed: enum_map! {
                Axis::Across => SPEED_ACROSS,
                Axis::DownUp => SPEED_DOWN_UP,
            },
        }
    }
}
