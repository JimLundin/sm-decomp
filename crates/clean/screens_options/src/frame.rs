//! What a frame end brings the menu, and what it reports beyond the State.

use enum_map::Enum;
use types::Set;
use types::options::{FileBegins, WayOut};

/// What the player asks of the menu with a button just pressed. The binding to the pad's
/// buttons is the caller's: A and Start choose, B goes back, Left and Right go sideways.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum MenuAction {
    /// Up: the item above.
    Up,
    /// Down: the item below.
    Down,
    /// B: back.
    Back,
    /// A or Start: act on the item under the cursor.
    Choose,
    /// Left or Right: switch the special setting under the cursor.
    Sideways,
}

/// The actions just pressed as a finished frame began.
pub type Pressed = Set<MenuAction>;

/// How a frame ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEnd {
    /// The game had its work done: a finished frame, with the actions just pressed as it began.
    Finished(Pressed),
    /// The game was still at its work: nothing changes.
    Busy,
}

/// How the chosen file begins, as save memory's owner tells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChosenFile {
    /// Through the dark, with the intro or the escape.
    Begins(FileBegins),
    /// Straight on in play.
    InPlay,
}

/// A sound the menu asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSound {
    /// The cursor moved.
    CursorMoved,
    /// An item was chosen or a setting switched.
    Selected,
}

/// A frame end after the menu handed on: it belongs to the piece that follows, by this way out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NextPiece(pub WayOut);
