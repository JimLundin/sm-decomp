//! The scene's content, in our format: its scripts, the colours its dissolves head for and move,
//! the caret's colours and its home. The importer checks its shape; the model reads it as given.

use enum_map::{Enum, EnumMap};
use types::intro::{BabyScript, CaretScript, EyesScript, PageScript};
use types::{Colour, Pair, Palette, Px};

use crate::script::Script;

/// One of the intro's sets of dissolve colours. The discovery shares the battle's sets
/// (`DiscoveryBegins`), the delivery the examination's (`DeliveryBegins`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Dissolve {
    /// The text dissolving into the battle on Zebes (`BattleDissolves`) or the discovery room.
    IntoFight,
    /// The battle or the discovery dissolving back to the text, for page 2 or 3
    /// (`BattleDissolves`, `DiscoveryReturns`).
    OutOfFight,
    /// The text dissolving into the delivery or the examination (`PicturesDissolve`).
    ToScene,
    /// The delivery or the examination dissolving back to the text, for page 4 or 5
    /// (`PicturesDissolve`).
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
    /// The baby's scripts, in the delivery and in the examination.
    pub baby: EnumMap<BabyScript, Script>,
    /// The caret's scripts.
    pub caret: EnumMap<CaretScript, Script>,
    /// The scene's 256 colours at their full, as the dissolves head for them (`PageFiveBegins`,
    /// `BattleBegins`, `PageTwoBegins`, `DiscoveryBegins`, `DiscoveryReturns`, `DeliveryBegins`,
    /// `PageFourBegins`, `ExaminationBegins`).
    pub full: Palette,
    /// Each dissolve's colours (`PicturesDissolve`, `BattleDissolves`); a dissolve's brightening
    /// set also starts black as it begins, but for page 5's (`PageFiveBegins`).
    pub sets: EnumMap<Dissolve, DissolveSets>,
    /// The text's colours, which start black as page 5 begins (`PageFiveBegins`).
    pub text: Vec<u8>,
    /// The colours the caret takes as the dissolve back to the text from the delivery or the
    /// examination ends, shown over the last step's colours (`DissolveEnds`). The return from
    /// the discovery, like the battle's, leaves them out and keeps its faded colours.
    pub caret_colours: Vec<IndexedColour>,
    /// The caret's place as a page begins, px (`PageOneIsMadeReady`, `PageTwoBegins`,
    /// `DiscoveryReturns`, `PageFourBegins`, `PageFiveBegins`, `PressTurnsThePage`).
    pub caret_home: Pair<Px>,
}
