//! What file select reports at each frame end, beside its State.

use types::file_select::WayOut;

/// A sound file select asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSound {
    /// The menu opens, as it loads.
    MenuOpens,
    /// The cursor moved, or the menu was left for copy, clear or the title.
    CursorMoved,
    /// A file was chosen.
    ChoiceMade,
}

/// Whether the frame of work goes on through the frame end.
///
/// The game's frame owner reads it: where it goes on, no new frame of work begins at the next
/// finished frame end, and so no random number is taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameOfWork {
    /// The frame of work ended with the frame end.
    Ends,
    /// The frame of work goes on through the frame end.
    GoesOn,
}

/// What one frame end reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameReport {
    /// The sounds asked for, oldest first.
    pub sounds: Vec<MenuSound>,
    /// Whether the frame of work goes on.
    pub frame: FrameOfWork,
}

impl FrameReport {
    /// A frame end that asks for nothing, its frame of work ended.
    pub const NOTHING: Self = Self {
        sounds: Vec::new(),
        frame: FrameOfWork::Ends,
    };

    /// A frame end that asks for these sounds, its frame of work ended.
    #[must_use]
    pub fn asking(sounds: &[MenuSound]) -> Self {
        Self {
            sounds: sounds.to_vec(),
            frame: FrameOfWork::Ends,
        }
    }
}

/// A frame end after file select went out, which belongs to the piece that follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unsupported(pub WayOut);
