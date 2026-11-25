//! The intro's state (screens-intro), carried between frames, saved and restored.
//!
//! Where the scene has got to, its colours, the baby, the page, the caret, the glowing letters,
//! Samus's eyes and the screen's brightness.

use serde::{Deserialize, Serialize};

use crate::{Pair, Px};

/// The intro's stage: where the scene has got to. Every entry reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntroStage {
    /// Page 4's text dissolving into the picture of the baby.
    ToScene,
    /// The picture shown, the baby on its script.
    Scene,
    /// The baby has signalled page 5; the page begins on the next frame.
    ToPageFive,
    /// The picture dissolving back to the text.
    ToText,
    /// A page being typed, or holding.
    Text,
    /// Page 5 typed, the caret blinking, the game waiting for a button.
    WaitingForPress,
    /// The picture fading to black.
    FadingOut,
    /// The intro is over: the next frame is the flight's.
    Done,
}

/// One of the scene's 256 colours as the dissolves move it (`PicturesDissolve`).
///
/// Each channel, red, green and blue, has a level and the step it moves by, in 256ths of a
/// level. A level wraps at 16 bits as it dims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FadingColour {
    /// The channels' levels, red, green and blue.
    pub levels: [u16; 3],
    /// The channels' steps: a colour's full level × 8, so 32 steps make it full.
    pub steps: [u16; 3],
}

impl FadingColour {
    /// Black, and still: every level and every step 0.
    pub const STILL_BLACK: Self = Self {
        levels: [0; 3],
        steps: [0; 3],
    };
}

/// The number of the scene's colours.
pub const SCENE_COLOURS: usize = 256;

/// The scene's 256 colours, by index, as the dissolves move them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<FadingColour>", into = "Vec<FadingColour>")]
pub struct FadingColours(pub [FadingColour; SCENE_COLOURS]);

impl FadingColours {
    /// Every colour black and still: the colours before any dissolve has moved them.
    #[must_use]
    pub const fn still_black() -> Self {
        Self([FadingColour::STILL_BLACK; SCENE_COLOURS])
    }

    /// The colour of this index, to change.
    pub fn colour_mut(&mut self, index: u8) -> Option<&mut FadingColour> {
        self.0.get_mut(usize::from(index))
    }
}

impl TryFrom<Vec<FadingColour>> for FadingColours {
    type Error = &'static str;

    fn try_from(colours: Vec<FadingColour>) -> Result<Self, Self::Error> {
        colours
            .try_into()
            .map(Self)
            .map_err(|_| "the scene has 256 colours")
    }
}

impl From<FadingColours> for Vec<FadingColour> {
    fn from(colours: FadingColours) -> Self {
        Self::from(colours.0)
    }
}

/// The step a script is at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepAt {
    /// Placed at its start: its first step comes the next time it runs (`ScriptSteps`).
    Start,
    /// At this step, counted from 0.
    Step(usize),
}

/// Where a script has got to (`ScriptSteps`): its step, the frames its picture still holds and
/// the count left of a repeat it is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// The step it is at.
    pub step: StepAt,
    /// The frames its picture still holds, counting this one.
    pub hold: u16,
    /// The count left of a repeat it is in, or none.
    pub repeat_left: Option<u16>,
}

impl Progress {
    /// `ScriptSteps`: a script is placed at its start with its hold 1.
    pub const START: Self = Self {
        step: StepAt::Start,
        hold: 1,
        repeat_left: None,
    };
}

/// A script's place: which script runs, and where it has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPlace<S> {
    /// Which of the scripts.
    pub script: S,
    /// Where it has got to.
    pub progress: Progress,
}

impl<S> ScriptPlace<S> {
    /// The script placed at its start.
    pub const fn start(script: S) -> Self {
        Self {
            script,
            progress: Progress::START,
        }
    }
}

/// The page scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum, Serialize, Deserialize)]
pub enum PageScript {
    /// Page 5, placed as the picture starts back toward the text.
    Five,
    /// Page 6, placed by the press ("CERES STATION WAS UNDER ATTACK!!").
    Six,
}

/// Samus's eyes' scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum, Serialize, Deserialize)]
pub enum EyesScript {
    /// Her eyes blinking, through the scene and page 5.
    Blinking,
    /// Her eyes staring, from page 6 on.
    Staring,
}

/// The caret's scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum, Serialize, Deserialize)]
pub enum CaretScript {
    /// While a page types.
    Steady,
    /// Once the page waits.
    Blinking,
}

/// The baby, on the scene's picture: x 112, its y and its script.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baby {
    /// Its y on the screen, px.
    pub y: Px,
    /// Where its script has got to.
    pub progress: Progress,
}

/// The caret: where the next letter goes, and its script.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caret {
    /// Its place on the screen, px.
    pub place: Pair<Px>,
    /// Its script's place.
    pub script: ScriptPlace<CaretScript>,
}

/// A place on the text layer: a column and a row, each of 8 px.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPlace {
    /// The column, from the left.
    pub column: u16,
    /// The row, from the top.
    pub row: u16,
}

/// The last colour set a letter takes.
const LAST_SET: u8 = 3;

/// One of the four colour sets a letter is shown in as it glows (`LetterGlows`): 0 to 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct ColourSet(u8);

impl ColourSet {
    /// Set 0, the set a new letter takes.
    pub const FIRST: Self = Self(0);

    /// The set of this number, if it is 0 to 3.
    #[must_use]
    pub const fn new(set: u8) -> Option<Self> {
        if set <= LAST_SET {
            Some(Self(set))
        } else {
            None
        }
    }

    /// Its number.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0
    }

    /// The set after this one; none after set 3, the one a letter keeps.
    #[must_use]
    pub const fn after(self) -> Option<Self> {
        if self.0 < LAST_SET {
            Some(Self(self.0.wrapping_add(1)))
        } else {
            None
        }
    }
}

impl TryFrom<u8> for ColourSet {
    type Error = &'static str;

    fn try_from(set: u8) -> Result<Self, Self::Error> {
        Self::new(set).ok_or("a colour set is 0 to 3")
    }
}

impl From<ColourSet> for u8 {
    fn from(set: ColourSet) -> Self {
        set.number()
    }
}

/// A glowing letter: its place, the colour set it takes next and the frames left before it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glow {
    /// The letter's place.
    pub at: TextPlace,
    /// The set it takes next.
    pub next: ColourSet,
    /// The frames left before it takes it.
    pub left: u16,
}

/// `LetterIsTyped`: at most 8 letters glow at once.
pub const MOST_GLOWS: usize = 8;

/// The glowing letters, at most 8, in the order they were typed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Glow>", into = "Vec<Glow>")]
pub struct Glows(Vec<Glow>);

impl Glows {
    /// No letter glowing.
    #[must_use]
    pub const fn none() -> Self {
        Self(Vec::new())
    }

    /// These glows, if there are at most 8.
    #[must_use]
    pub fn new(glows: Vec<Glow>) -> Option<Self> {
        if glows.len() <= MOST_GLOWS {
            Some(Self(glows))
        } else {
            None
        }
    }

    /// The glows, oldest first.
    #[must_use]
    pub fn as_slice(&self) -> &[Glow] {
        &self.0
    }

    /// `LetterIsTyped`: the glow placed if fewer than 8 glow; otherwise none is.
    pub fn place(&mut self, glow: Glow) {
        if self.0.len() < MOST_GLOWS {
            self.0.push(glow);
        }
    }

    /// Each glow replaced by what `next` makes of it, or removed where it makes none.
    pub fn update(&mut self, next: impl FnMut(Glow) -> Option<Glow>) {
        self.0 = self.0.drain(..).filter_map(next).collect();
    }
}

impl TryFrom<Vec<Glow>> for Glows {
    type Error = &'static str;

    fn try_from(glows: Vec<Glow>) -> Result<Self, Self::Error> {
        Self::new(glows).ok_or("at most 8 letters glow")
    }
}

impl From<Glows> for Vec<Glow> {
    fn from(glows: Glows) -> Self {
        glows.0
    }
}

/// The click turn: whether the last letter's turn clicked, so the next one's doesn't
/// (`LetterIsTyped`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClickTurn {
    /// Off: the next letter's turn clicks.
    Off,
    /// On: the next letter's turn is silent.
    On,
}

impl ClickTurn {
    /// The turn the other way.
    #[must_use]
    pub const fn turned(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }
}

/// The highest brightness level, full.
const FULL_LEVEL: u8 = 15;

/// A level of the screen's brightness: 0 (black) to 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct BrightnessLevel(u8);

impl BrightnessLevel {
    /// Level 0: black.
    pub const BLACK: Self = Self(0);
    /// Level 15: full.
    pub const FULL: Self = Self(FULL_LEVEL);

    /// The level of this number, if it is 0 to 15.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= FULL_LEVEL {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its number.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0
    }

    /// One level darker, staying at 0.
    #[must_use]
    pub const fn darker(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for BrightnessLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("a brightness level is 0 to 15")
    }
}

impl From<BrightnessLevel> for u8 {
    fn from(level: BrightnessLevel) -> Self {
        level.number()
    }
}

/// Whether the screen shows anything at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blanking {
    /// Nothing shown, at any level.
    Blanked,
    /// Shown at its level.
    Shown,
}

/// The screen's brightness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brightness {
    /// Its level.
    pub level: BrightnessLevel,
    /// Whether anything shows.
    pub blanking: Blanking,
}

impl Brightness {
    /// Full and shown.
    pub const FULL: Self = Self {
        level: BrightnessLevel::FULL,
        blanking: Blanking::Shown,
    };
}

/// The most frames of the fade's count and its rhythm.
const MOST_FADE_FRAMES: u16 = 32_768;

/// Frames of the brightness fade, 0 to 32,768: its count, or its rhythm `d` (`PictureDarkens`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct FadeFrames(u16);

impl FadeFrames {
    /// No frames.
    pub const ZERO: Self = Self(0);
    /// `PictureDarkens`: the reference rhythm, 1 frame.
    pub const REFERENCE_RHYTHM: Self = Self(1);

    /// These frames, if they are 0 to 32,768.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= MOST_FADE_FRAMES {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its number of frames.
    #[must_use]
    pub const fn frames(self) -> u16 {
        self.0
    }

    /// One frame fewer; none at 0.
    #[must_use]
    pub const fn fewer(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(frames) => Some(Self(frames)),
            None => None,
        }
    }
}

impl TryFrom<u16> for FadeFrames {
    type Error = &'static str;

    fn try_from(frames: u16) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("the fade's frames are 0 to 32,768")
    }
}

impl From<FadeFrames> for u16 {
    fn from(frames: FadeFrames) -> Self {
        frames.frames()
    }
}

/// The intro's state: everything it remembers from one frame to the next (screens-intro).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntroState {
    /// Where the scene has got to.
    pub stage: IntroStage,
    /// The scene's colours as the dissolves move them.
    pub colours: FadingColours,
    /// The dissolve's count, frames: when the dissolve steps and ends. It wraps at 16 bits.
    pub dissolve: u16,
    /// The picture's height, px: where the scene's picture stands.
    pub height: i16,
    /// The baby, or none.
    pub baby: Option<Baby>,
    /// The page being typed, or none.
    pub page: Option<ScriptPlace<PageScript>>,
    /// Samus's eyes, or none.
    pub eyes: Option<ScriptPlace<EyesScript>>,
    /// The caret, or none.
    pub caret: Option<Caret>,
    /// The glowing letters.
    pub glows: Glows,
    /// The click turn.
    pub click: ClickTurn,
    /// The screen's brightness.
    pub brightness: Brightness,
    /// The fade's count, frames: when the picture next darkens.
    pub fade: FadeFrames,
}
