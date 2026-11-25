//! The options menu's State: its stage, the screen, the cursor and the chosen file's settings.
//!
//! The game carries it from one frame end to the next, from file select's hand-over until the
//! menu hands on, and may save and restore it.

use serde::{Deserialize, Serialize};

use crate::power_on::{Level, ScreenLight};

/// Spec: `Dissolve`. The largest block size, 15.
const LARGEST_BLOCK_SIZE: u8 = 15;

/// Spec: `Dissolve`. The dissolve's block size, 0 to 15: blocks of `n + 1` pixels each side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct BlockSize(u8);

impl BlockSize {
    /// Blocks of one pixel: where every dissolve starts.
    pub const SMALLEST: Self = Self(0);
    /// Blocks of sixteen pixels: the size 15.
    pub const LARGEST: Self = Self(LARGEST_BLOCK_SIZE);

    /// The block size of this number, if it is one (0 to 15).
    #[must_use]
    pub const fn new(size: u8) -> Option<Self> {
        if size <= LARGEST_BLOCK_SIZE {
            Some(Self(size))
        } else {
            None
        }
    }

    /// Its number, 0 to 15.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Spec: `Dissolve`. One size larger as the screen darkens, `min(15, n + 1)`.
    #[must_use]
    pub fn grown(self) -> Self {
        Self::new(self.0.saturating_add(1)).unwrap_or(Self::LARGEST)
    }

    /// Spec: `Dissolve`. One size smaller as the screen brightens, `max(0, n - 1)`.
    #[must_use]
    pub const fn shrunk(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for BlockSize {
    type Error = &'static str;

    fn try_from(size: u8) -> Result<Self, Self::Error> {
        Self::new(size).ok_or("a block size must be 0 to 15")
    }
}

impl From<BlockSize> for u8 {
    fn from(size: BlockSize) -> Self {
        size.get()
    }
}

/// The language of the game's text. A new file's is English.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// English text.
    #[default]
    English,
    /// Japanese text.
    Japanese,
}

/// The chosen file's settings, which the menu changes and the game reads in play. A new file has
/// all three off: English, no weapon-reset, no moonwalk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The language of the text (spec: `japanese` on or off).
    pub language: Language,
    /// Weapon-reset, on or off.
    pub weapon_reset: bool,
    /// Moonwalk, on or off.
    pub moonwalk: bool,
}

/// An item of the options menu, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsItem {
    /// Start game.
    StartGame,
    /// English text.
    EnglishText,
    /// Japanese text.
    JapaneseText,
    /// The controller settings.
    ControllerSettings,
    /// The special settings.
    SpecialSettings,
}

/// An item of the special settings, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialItem {
    /// Weapon-reset.
    WeaponReset,
    /// Moonwalk.
    Moonwalk,
    /// End: back to the options.
    End,
}

/// Spec: `Dissolve`. The cursor's item while the screen dissolves, which is where the dissolve
/// goes: start game for the options, or the special or controller settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DissolveCursor {
    /// Start game: back to the options.
    StartGame,
    /// On to the controller settings.
    ControllerSettings,
    /// On to the special settings.
    SpecialSettings,
}

/// Spec: `Dissolve`. A dissolve under way: the cursor's item and the block size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dissolving {
    /// The cursor's item, the dissolve's destination.
    pub cursor: DissolveCursor,
    /// The size of the blocks the screen shows.
    pub blocks: BlockSize,
}

/// Spec: `OptionsLoad`. The menu loads over two finished frame ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadStep {
    /// Step 0: none of the load's frames has finished.
    First,
    /// Step 1: the first has.
    Second,
}

/// Spec: `OptionsLeave`. How a chosen file that leaves through the dark begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileBegins {
    /// With the intro: a new file.
    Intro,
    /// With the escape from the space station: a file saved after it.
    Escape,
}

/// The ways the menu hands on to the piece that follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WayOut {
    /// The intro begins, for a new file.
    IntroBegins,
    /// The escape from the space station begins.
    EscapeBegins,
    /// The chosen file goes straight on in play.
    PlayContinues,
    /// Back to file select.
    FileSelectAgain,
    /// The controller settings screen begins.
    ControlsBegin,
}

/// Where the options menu stands, with what each stage holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsStage {
    /// Black while the menu loads.
    Loading(LoadStep),
    /// The menu brightening, the cursor on its item.
    Brightening(OptionsItem),
    /// The menu lit, the cursor on its item.
    Choosing(OptionsItem),
    /// The special settings lit, the cursor on its item.
    Special(SpecialItem),
    /// The screen darkening as its blocks grow.
    DissolvingOut(Dissolving),
    /// The screen brightening as its blocks shrink.
    DissolvingIn(Dissolving),
    /// Darkening on the way into the game.
    Leaving {
        /// The cursor's item.
        cursor: OptionsItem,
        /// How the chosen file begins.
        begins: FileBegins,
    },
    /// Darkening on the way back to file select, the cursor on its item.
    Returning(OptionsItem),
    /// Handed on: every later frame end belongs to the piece that follows.
    HandedOn(WayOut),
}

/// The options menu's complete State. A caller may set any stage, screen and settings, read them
/// back, carry them forward and restore them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsState {
    /// Where the menu stands.
    pub stage: OptionsStage,
    /// The screen at the last frame end.
    pub screen: ScreenLight,
    /// The chosen file's settings.
    pub settings: Settings,
}

impl OptionsState {
    /// The State as the menu begins: loading at step 0, the screen blank at dark (file select
    /// always hands over dark), and the chosen file's settings.
    #[must_use]
    pub const fn begin(settings: Settings) -> Self {
        Self {
            stage: OptionsStage::Loading(LoadStep::First),
            screen: ScreenLight::Blank(Level::DARK),
            settings,
        }
    }
}
