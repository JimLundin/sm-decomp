//! The scene's content, in our format: its scripts, the colours its dissolves head for and move,
//! the caret's colours and its home. The importer checks its shape; the model reads it as given.

use enum_map::{Enum, EnumMap};
use types::intro::{CaretScript, EyesScript, PageScript};
use types::{Colour, Pair, Palette, Px};

use crate::script::Script;

/// One of the intro's dissolves, each with its own colour sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Dissolve {
    /// Page 1's text dissolving into the battle on Zebes (`BattleDissolves`).
    IntoFight,
    /// The battle dissolving back to the text for page 2 (`BattleDissolves`).
    OutOfFight,
    /// Page 4's text dissolving into the picture of the baby (`PicturesDissolve`).
    ToScene,
    /// The picture dissolving back to the text for page 5 (`PicturesDissolve`).
    ToText,
}

/// A dissolve's colours, by index: those that dim to black and those that brighten from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DissolveSets {
    /// The colours that dim.
    pub dims: Vec<u8>,
    /// The colours that brighten.
    pub brightens: Vec<u8>,
}

/// A colour the screen shows at an index, whatever the dissolve left there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexedColour {
    /// The colour's index.
    pub index: u8,
    /// The colour.
    pub colour: Colour,
}

/// The intro's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntroContent {
    /// The pages' scripts: their letters, places and holds, and their signals.
    pub pages: EnumMap<PageScript, Script>,
    /// Samus's eyes' scripts.
    pub eyes: EnumMap<EyesScript, Script>,
    /// The baby's script.
    pub baby: Script,
    /// The caret's scripts.
    pub caret: EnumMap<CaretScript, Script>,
    /// The scene's 256 colours at their full, as the dissolves head for them (`PageFiveBegins`,
    /// `BattleBegins`, `PageTwoBegins`).
    pub full: Palette,
    /// Each dissolve's colours (`PicturesDissolve`, `BattleDissolves`); the battle's
    /// brightening sets also start black (`BattleBegins`, `PageTwoBegins`).
    pub sets: EnumMap<Dissolve, DissolveSets>,
    /// The text's colours, which start black as page 5 begins (`PageFiveBegins`).
    pub text: Vec<u8>,
    /// The colours the caret takes once the text is back (`DissolveEnds`).
    pub caret_colours: Vec<IndexedColour>,
    /// The caret's place as a page begins, px (`PageOneIsMadeReady`, `PageTwoBegins`,
    /// `PageFiveBegins`, `PressTurnsThePage`).
    pub caret_home: Pair<Px>,
}
