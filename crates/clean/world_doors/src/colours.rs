//! The screen's colours as the passage reads and sets them: the dark colours it heads for, the
//! fade's steps toward the colours headed for, and her visor's colour as she lands.

use core::num::{NonZeroU16, Wrapping};

use enum_map::Enum;
use types::{Colour, Set};

/// The screen's colours: 256 of them.
pub const COLOURS: usize = 256;

/// Which colours a colour of the screen is, as the HUD's content and hers name them
/// (`TheFades`). Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ColourGroup {
    /// The HUD's icons and empty tanks: kept lit unless either room keeps the items dark.
    Icon,
    /// The HUD's mini-map and text: always kept lit.
    Lit,
    /// Any other colour: it goes dark.
    Other,
    /// The escape timer's four: kept lit while it runs, unless either room keeps the items dark.
    Timer,
    /// Her visor's, which goes dark and is lit again as she lands (`Arriving`).
    Visor,
}

/// The screen's colours and the colours they head for, which belong to the palettes; the
/// passage reads and sets them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// The colours as they stand.
    pub screen: [Colour; COLOURS],
    /// The colours a fade heads for: the dark colours the passage sets as it leaves, then the new
    /// room's, which the room's loading sets.
    pub goal: [Colour; COLOURS],
}

/// `TheFades`: the dark colours. Each colour heads for black, but a colour of a group kept lit,
/// which heads for its own colour.
pub fn head_for_dark(
    palette: &mut Palette,
    groups: &[ColourGroup; COLOURS],
    kept: Set<ColourGroup>,
) {
    for ((goal, &now), &group) in palette.goal.iter_mut().zip(&palette.screen).zip(groups) {
        *goal = if kept[group] { now } else { Colour::BLACK };
    }
}

/// `Arriving`: her visor's colour as she lands, (0, 31, 14) in red, green and blue.
const VISOR_LIT: [u16; 3] = [0, 31, 14];

/// `Arriving`: her visor's colours become lit at once.
pub fn light_visor(palette: &mut Palette, groups: &[ColourGroup; COLOURS]) {
    for (now, &group) in palette.screen.iter_mut().zip(groups) {
        if group == ColourGroup::Visor {
            *now = Colour::from_channels(VISOR_LIT);
        }
    }
}

/// A fade's step, `t`: 0 to `d + 2` for a fade of `d` steps (14 for the game's 12). Its range
/// depends on the rules, so any step is admitted: one past `d + 2` acts as `d + 2`, the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeTime(pub u16);

/// What a fade's step does (`Fading`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FadeStage {
    /// `t` = 0: no colour changes.
    Start,
    /// `t` from 1 to `d`: each colour moves its share of the way left, `1 / (d + 1 − t)`.
    Toward(NonZeroU16),
    /// `t` = `d + 1`: each colour is its goal.
    Goal,
    /// `t` = `d + 2`: no colour changes, and the fade ends.
    End,
}

impl FadeTime {
    /// A fade's first step, 0.
    pub const START: Self = Self(0);

    /// `Fading`: what this step does in a fade of `steps` steps.
    fn stage(self, steps: u8) -> FadeStage {
        let Some(before) = self.0.checked_sub(1) else {
            return FadeStage::Start;
        };
        // d + 1 − t, the steps left counting this one: none left is the goal, fewer is the end.
        u16::from(steps)
            .checked_sub(before)
            .map_or(FadeStage::End, |left| {
                NonZeroU16::new(left).map_or(FadeStage::Goal, FadeStage::Toward)
            })
    }

    /// `TheFades`: whether this is the fade's last step, `d + 2`.
    #[must_use]
    pub fn is_last(self, steps: u8) -> bool {
        self.stage(steps) == FadeStage::End
    }

    /// `TheFades`: the fade's next step, or none after its last.
    #[must_use]
    pub fn next(self, steps: u8) -> Option<Self> {
        (!self.is_last(steps)).then(|| Self(self.0.saturating_add(1)))
    }
}

/// `Fading`: one step `t` of a fade of `steps` steps, every colour of the screen toward the colour
/// it heads for. Darkening and brightening are this one rule; only the colours headed for differ.
pub fn fade(palette: &mut Palette, t: FadeTime, steps: u8) {
    match t.stage(steps) {
        FadeStage::Start | FadeStage::End => {}
        FadeStage::Toward(left) => {
            for (now, &goal) in palette.screen.iter_mut().zip(&palette.goal) {
                *now = faded(*now, goal, left);
            }
        }
        FadeStage::Goal => palette.screen = palette.goal,
    }
}

/// `Fading`: a channel's sums are kept in 1/256ths, its value their high byte.
const CHANNEL_SCALE: Wrapping<u16> = Wrapping(256);

/// `Fading`: a colour stepped toward its goal with `left` steps left, each channel on its own.
///
/// A channel `c` heading for `g` moves by `q = ⌊|g − c| × 256 / left⌋`, toward `g`: the sum
/// `c × 256 ± q` is taken mod 2^16 and its high byte kept, which rounds down. A channel at its
/// goal moves by 0, so a colour at its goal stays.
fn faded(now: Colour, goal: Colour, left: NonZeroU16) -> Colour {
    let mut channels = now.channels();
    for (channel, goal) in channels.iter_mut().zip(goal.channels()) {
        let share = Wrapping((Wrapping(channel.abs_diff(goal)) * CHANNEL_SCALE).0 / left);
        let scaled = Wrapping(*channel) * CHANNEL_SCALE;
        let sum = if goal < *channel {
            scaled - share
        } else {
            scaled + share
        };
        let [high, _] = sum.0.to_be_bytes();
        *channel = u16::from(high);
    }
    Colour::from_channels(channels)
}
