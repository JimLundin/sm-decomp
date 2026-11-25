//! What the intro hands the others each frame: the colours and pictures to the renderer, the
//! clicks and the music's stop to audio, and, as it ends, the equipment returned.

use types::intro::{ColourSet, TextPlace};
use types::{Pair, Palette, Px};

use crate::script::{Letter, PictureName};

/// Something that happened this frame, for another part of the game to act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntroEvent {
    /// The text layer is cleared (`DissolveEnds`, `PressTurnsThePage`).
    TextCleared,
    /// A letter clicks: the sound is asked for (`LetterIsTyped`).
    Click,
    /// The music is asked to stop, as page 6 begins (`PressTurnsThePage`).
    MusicStops,
    /// The intro is over and the equipment lent for its played scenes is returned: her missiles
    /// and their capacity 0, and the player's own buttons back (`IntroEnds`).
    EquipmentReturned,
}

/// A picture shown at a place on the screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShownPicture {
    /// The picture.
    pub picture: PictureName,
    /// Its place, px.
    pub place: Pair<Px>,
}

/// A typed letter shown in another colour set (`LetterGlows`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recoloured {
    /// The letter's place.
    pub at: TextPlace,
    /// The set it is shown in now.
    pub set: ColourSet,
}

/// What one frame of the intro reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The 256 colours the screen shows, when the frame takes them (`PicturesDissolve`,
    /// `DissolveEnds`, `PageFiveBegins`); none when it leaves them.
    pub colours: Option<Palette>,
    /// The letter typed this frame, in colour set 0 (`LetterIsTyped`).
    pub typed: Option<Letter>,
    /// The letters shown in another colour set this frame (`LetterGlows`).
    pub recoloured: Vec<Recoloured>,
    /// The picture Samus's eyes show; their place is the picture's own.
    pub eyes: Option<PictureName>,
    /// The baby's picture and place.
    pub baby: Option<ShownPicture>,
    /// The caret's picture and place, when it shows (`CaretBlinks`).
    pub caret: Option<ShownPicture>,
    /// What happened, in order.
    pub events: Vec<IntroEvent>,
}

impl Report {
    /// A frame that has reported nothing yet.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            colours: None,
            typed: None,
            recoloured: Vec::new(),
            eyes: None,
            baby: None,
            caret: None,
            events: Vec::new(),
        }
    }
}
