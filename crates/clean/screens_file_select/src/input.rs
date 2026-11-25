//! What file select reads: how each frame ended, the player's presses, and save memory.

use enum_map::Enum;
use types::Set;
use types::file_select::Slot;

/// What the player asks of the menu, from the buttons just pressed as a finished frame began.
/// The binding is the caller's: `Choose` is A or Start, `Back` is B; other buttons ask nothing.
///
/// Listed in the order the menu looks at them (`ItemPressed`, `CursorMoves`): the first pressed
/// acts alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum MenuAction {
    /// A or Start: act on the item under the cursor.
    Choose,
    /// Up: the cursor to the item above.
    Up,
    /// Down: the cursor to the item below.
    Down,
    /// B: back to the title.
    Back,
}

/// How a frame ended, as the platform tells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEnd {
    /// The game was still busy: no frame finished, and nothing changes.
    Busy,
    /// A frame finished, with what the player asked as it began.
    Finished(Set<MenuAction>),
}

/// Save memory's owner, which file select asks, on the load's fourth finished frame end, about
/// the slots.
pub trait SaveMemory {
    /// The slots that hold a file.
    fn files(&self) -> Set<Slot>;
    /// The slot chosen last, if save memory holds a valid one.
    fn last_chosen(&self) -> Option<Slot>;
}
