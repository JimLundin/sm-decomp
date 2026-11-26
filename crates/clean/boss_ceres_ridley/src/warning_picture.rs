//! Checked warning content in, ordered picture updates out. Drawing retains the updated room.

use types::{Colour, Look, Pair};

use types::ceres_warning::WARNING_GLOW_LOOKS;

/// Spec: `WarningArt`. Which sheet an art batch modifies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningSheet {
    /// Sprite sheet (countdown and returning door art).
    Sprites,
    /// Foreground sheet (warning text art).
    Foreground,
}

/// Spec: `WarningArt`. Replace consecutive whole tiles starting at `first_tile`.
/// The importer checks that the batch fits. `T` is a whole tile in the caller's art format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningArtUpdate<'a, T> {
    /// Destination sheet.
    pub sheet: WarningSheet,
    /// Index of the first tile replaced.
    pub first_tile: usize,
    /// Replacement tiles in sheet order.
    pub tiles: &'a [T],
}

/// Spec: `WarningHeading`, `WarningTyping`. A glyph and its foreground column and row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionedGlyph {
    /// Foreground column and row.
    pub cell: Pair<u16>,
    /// Tile look in the prepared art.
    pub glyph: Look,
}

/// Spec: `WarningTyping`. A positioned English cell; a blank consumes time but writes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningEnglishCell {
    /// Foreground column and row.
    pub cell: Pair<u16>,
    /// Tile look, or none for a blank.
    pub glyph: Option<Look>,
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningGlow`.
///
/// Content admitted by the caller: the schedule and English text are nonempty, each art run
/// has at least one update, and every tile, glyph and position fits its destination.
pub struct WarningContent<'a, T> {
    /// Each ordinary art run's ordered updates.
    pub schedule: &'a [&'a [WarningArtUpdate<'a, T>]],
    /// Heading cells, placed together after the final batch.
    pub heading: &'a [PositionedGlyph],
    /// Japanese subtitle cells, supplied when admitting that language.
    pub subtitle: &'a [PositionedGlyph],
    /// English cells in typing order, including blanks.
    pub english: &'a [WarningEnglishCell],
    /// The three colours for heading slots 1 through 3, for each look in order.
    pub glow_palettes: &'a [[Colour; 3]; WARNING_GLOW_LOOKS],
}

/// Spec: `WarningArt`. The incoming room colours from slots 1 and 3 of one foreground palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningColourPair {
    /// Current colour in slot one.
    pub slot_one: Colour,
    /// Current colour in slot three.
    pub slot_three: Colour,
}

/// Spec: `WarningArt`. The two foreground palettes the warning takes colours from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningForeground {
    /// Foreground palette zero, the source for the heading colours.
    pub palette_zero: WarningColourPair,
    /// Foreground palette one, the source for the English text colours.
    pub palette_one: WarningColourPair,
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningGlow`.
/// A foreground palette used by the warning's high-priority text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningPalette {
    /// English text palette.
    English,
    /// Glowing heading palette.
    HeadingText,
    /// Japanese subtitle palette.
    Japanese,
}

impl WarningPalette {
    /// The palette's number in the foreground plane.
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::English => 5,
            Self::HeadingText => 6,
            Self::Japanese => 7,
        }
    }
}

/// Spec: `WarningArt`, `WarningGlow`. Replace only the named colour slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningColourWrite {
    /// Destination foreground palette.
    pub palette: WarningPalette,
    /// Colour slot within that palette.
    pub slot: u8,
    /// Replacement colour.
    pub colour: Colour,
}

const GLOW_SLOTS: [u8; 3] = [1, 2, 3];
const COPIED_SLOTS: [u8; 2] = [1, 3];

/// Spec: `WarningHeading`, `WarningTyping`. Write these tiles at high foreground priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningTextWrite<'a> {
    /// Foreground palette assigned to all these cells.
    pub palette: WarningPalette,
    /// Cells to replace, in drawing order.
    pub cells: &'a [PositionedGlyph],
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningGlow`.
///
/// Apply the art batch in order before text. Text writes are ordered as returned by `text()`.
/// Leave all other art, cells and colours intact. Save the resulting picture in drawing's state
/// alongside the warning state; the model never reconstructs or clears it on a restore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningPictureUpdates<'a, T> {
    /// Ordered replacement batches for this run.
    pub art: &'a [WarningArtUpdate<'a, T>],
    /// Colour slots to change on this run.
    pub colours: Vec<WarningColourWrite>,
    pub(crate) words: Option<WarningTextWrite<'a>>,
    pub(crate) letter: Option<PositionedGlyph>,
}

impl<T> WarningPictureUpdates<'_, T> {
    pub(crate) const fn unchanged() -> Self {
        Self {
            art: &[],
            colours: Vec::new(),
            words: None,
            letter: None,
        }
    }

    /// High-priority foreground writes, with the subtitle before any letter on the same run.
    pub fn text(&self) -> impl Iterator<Item = WarningTextWrite<'_>> {
        self.words
            .into_iter()
            .chain(self.letter.iter().map(|letter| WarningTextWrite {
                palette: WarningPalette::English,
                cells: core::slice::from_ref(letter),
            }))
    }

    /// Spec: `WarningArt`. Copy two slots from each of the incoming palettes.
    pub(crate) fn copy_colours(&mut self, foreground: WarningForeground) {
        let pairs = [
            (WarningPalette::HeadingText, foreground.palette_zero),
            (WarningPalette::English, foreground.palette_one),
        ];
        for (palette, pair) in pairs {
            self.write_colours(palette, COPIED_SLOTS, [pair.slot_one, pair.slot_three]);
        }
    }

    /// Spec: `WarningGlow`. Replace the three consecutive heading colour slots.
    pub(crate) fn glow_colours(&mut self, colours: [Colour; 3]) {
        self.write_colours(WarningPalette::HeadingText, GLOW_SLOTS, colours);
    }

    fn write_colours<const N: usize>(
        &mut self,
        palette: WarningPalette,
        slots: [u8; N],
        colours: [Colour; N],
    ) {
        self.colours
            .extend(
                slots
                    .into_iter()
                    .zip(colours)
                    .map(|(slot, colour)| WarningColourWrite {
                        palette,
                        slot,
                        colour,
                    }),
            );
    }
}
