//! What the intro hands the others each frame: the colours and pictures to the renderer, the
//! picture layout to the line split and the picture layers, the text tiles' publishing to the
//! tile writer, the clicks and the music's requests to audio, the flashback's requests to production
//! and the fight's owners, and, as it begins and ends, the equipment lent and returned.

use types::intro::{ColourSet, TextPlace};
use types::{Pair, Palette, Px};

use crate::music::MusicRequest;
use crate::script::{Letter, PictureName};

/// Which buttons the pad's presses mean.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSetting {
    /// The game's default buttons, which the played scenes use.
    Default,
    /// The player's own buttons.
    PlayersOwn,
}

/// Her equipment as the intro sets it for its played scenes, and undoes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equipment {
    /// Her missiles.
    pub missiles: u16,
    /// Their capacity.
    pub missile_capacity: u16,
    /// The buttons.
    pub buttons: ButtonSetting,
}

impl Equipment {
    /// `IntroBegins`: lent for the played scenes: missiles and their capacity 900, and the
    /// default buttons.
    pub const LENT: Self = Self {
        missiles: 900,
        missile_capacity: 900,
        buttons: ButtonSetting::Default,
    };
    /// `IntroEnds`: returned: missiles and their capacity 0, and the player's own buttons back.
    pub const RETURNED: Self = Self {
        missiles: 0,
        missile_capacity: 0,
        buttons: ButtonSetting::PlayersOwn,
    };
}

/// What the intro asks of production and the fight's owners during the flashback to Zebes.
/// Production applies the start before room play, and the hide after room play, before drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashbackRequest {
    /// Start the encounter: the Zebes flashback room and its demo (`BattleBegins`).
    StartZebesFight,
    /// Hide Samus, her shots and Mother Brain (`BattleDissolves`).
    HideFightSamusAndShots,
}

/// How the fight and the text are shown together, for the line split's colour blend (E3) and
/// the picture layers and their scroll mirrors (E4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureLayout {
    /// The two blended along the line split, while a battle dissolve runs.
    SplitBlend,
    /// The fight's picture alone.
    PlainFight,
    /// The text's picture alone.
    PlainText,
}

/// Whether the room and text tile writer (E2) publishes the text's tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTiles {
    /// Publishing paused.
    Paused,
    /// Publishing resumed.
    Resumed,
}

/// Something that happened this frame, for another part of the game to act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntroEvent {
    /// The text layer is cleared (`DissolveEnds`, `BattleDissolves`, `PressTurnsThePage`,
    /// `PageOneIsMadeReady`).
    TextCleared,
    /// A letter clicks: the sound is asked for (`LetterIsTyped`).
    Click,
    /// Her equipment set for the played scenes (`Equipment::LENT`, `IntroBegins`) or set back
    /// (`Equipment::RETURNED`, `IntroEnds`).
    EquipmentSet(Equipment),
    /// A request of the flashback's encounter (`BattleBegins`, `BattleDissolves`).
    Flashback(FlashbackRequest),
    /// The fight's flashing lights begin, as the dissolve into it finishes (`BattleDissolves`).
    FlashingLightsBegin,
    /// The way the fight and the text are shown changes, for the line split and the picture
    /// layers (`BattleBegins`, `BattleDissolves`, `PageTwoBegins`).
    Layout(PictureLayout),
    /// The tile writer's publishing of the text's tiles is paused or resumed (`PageTwoBegins`).
    TextUpdates(TextTiles),
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
    /// The music's requests, in order (`IntroBegins`, `OpeningUnfolds`, `PageOneIsMadeReady`,
    /// `PressTurnsThePage`).
    pub music: Vec<MusicRequest>,
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
            music: Vec::new(),
        }
    }
}
