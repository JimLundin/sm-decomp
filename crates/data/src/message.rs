//! The message boxes' tables (bank $85 log, <https://patrickjohnston.org/bank/85>): the box
//! borders, each message's definition and tilemap, the special button letters and where they
//! are drawn.

use crate::rom::{AddressError, Lorom, RomAddress};

/// The bank all the tables are in.
const BANK: u8 = 0x85;
/// The large box's top and bottom border tilemap (`$85:8000`), then the small box's
/// (`$85:8040`).
const BORDERS: RomAddress = RomAddress::new(BANK, 0x8000);
/// Words in a border tilemap: one row of the box.
pub const BORDER_WORDS: usize = 0x20;
/// The message definitions (`$85:869B`): three words each, the set-up routine, the routine
/// that draws the box's tilemap and the message's tilemap.
const DEFINITIONS: RomAddress = RomAddress::new(BANK, 0x869b);
/// Words in a definition.
const DEFINITION_WORDS: usize = 3;
/// The definitions, the last of which is the terminator that ends the previous message's
/// tilemap.
const DEFINITION_COUNT: usize = 0x1d;
/// The special buttons' tilemap offsets (`$85:8749`), a word per message up to the last real
/// one; the messages after it have no special button.
const BUTTON_OFFSETS: RomAddress = RomAddress::new(BANK, 0x8749);
/// Messages with a button offset.
const BUTTON_OFFSET_COUNT: usize = 0x1a;
/// The tile numbers for the button letters (`$85:8426`).
const LETTERS: RomAddress = RomAddress::new(BANK, 0x8426);
/// Button letters, the blank included.
const LETTER_COUNT: usize = 8;

/// The routines a definition names for setting the box up (`$85:8436`, `$85:8441`,
/// `$85:83C5`, `$85:83CC`).
const SMALL_SETUP: u16 = 0x8436;
const LARGE_SETUP: u16 = 0x8441;
const SHOOT_SETUP: u16 = 0x83c5;
const RUN_SETUP: u16 = 0x83cc;
/// The routines a definition names for drawing the box's tilemap (`$85:825A`, `$85:8289`).
const LARGE_DRAW: u16 = 0x825a;
const SMALL_DRAW: u16 = 0x8289;

/// Where the box's rows start in BG3's tilemap, in words past its base (`$85:8420` and
/// `$85:8438`).
const LARGE_ROWS: u16 = 0x01a0;
const SMALL_ROWS: u16 = 0x01c0;

/// How big a box's borders are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderSize {
    /// The large box's borders.
    Large,
    /// The small box's borders.
    Small,
}

/// How a message sets its box up beyond the tilemap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    /// A small box.
    Small,
    /// A large box.
    Large,
    /// A large box that shows the shoot button.
    ShootButton,
    /// A large box that shows the run button.
    RunButton,
}

impl Setup {
    /// Where the box's rows start in BG3's tilemap, in words.
    #[must_use]
    pub const fn rows(self) -> u16 {
        match self {
            Self::Small => SMALL_ROWS,
            Self::Large | Self::ShootButton | Self::RunButton => LARGE_ROWS,
        }
    }
}

/// A letter the special button's tile can show: the button the control is bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Letter {
    /// A.
    A,
    /// B.
    B,
    /// X.
    X,
    /// Y.
    Y,
    /// Select.
    Select,
    /// L.
    L,
    /// R.
    R,
    /// A blank tile, for a control bound to no letter button.
    Blank,
}

impl Letter {
    /// The letter's place in the tile table.
    const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::X => 2,
            Self::Y => 3,
            Self::Select => 4,
            Self::L => 5,
            Self::R => 6,
            Self::Blank => 7,
        }
    }
}

/// A message: how its box is set up and drawn, and the words it writes between the borders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// How the box is set up.
    pub setup: Setup,
    /// Which borders the box is drawn with.
    pub borders: BorderSize,
    /// Where the special button is drawn, in words from the box's top border.
    pub button: u16,
    /// The words between the borders: the message's rows.
    pub tilemap: Vec<u16>,
}

/// Why the tables don't load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageError {
    /// A table is outside the image.
    Address(AddressError),
    /// A definition names a routine that none of the box kinds are.
    Routine(u16),
}

impl From<AddressError> for MessageError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Every message and the tables they share.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageBoxes {
    large_border: Vec<u16>,
    small_border: Vec<u16>,
    letters: Vec<u16>,
    messages: Vec<Message>,
}

impl MessageBoxes {
    /// The tables from their parts, for tests of what reads them.
    #[must_use]
    pub const fn new(
        large_border: Vec<u16>,
        small_border: Vec<u16>,
        letters: Vec<u16>,
        messages: Vec<Message>,
    ) -> Self {
        Self {
            large_border,
            small_border,
            letters,
            messages,
        }
    }

    /// The tables from the ROM.
    ///
    /// # Errors
    ///
    /// [`MessageError`] when a table is outside the image or names an unknown routine.
    pub fn load(rom: Lorom<'_>) -> Result<Self, MessageError> {
        let large_border = words(rom, BORDERS, BORDER_WORDS)?;
        let small_border = words(
            rom,
            offset(BORDERS, BORDER_WORDS.wrapping_mul(2))?,
            BORDER_WORDS,
        )?;
        let letters = words(rom, LETTERS, LETTER_COUNT)?;
        let button_offsets = words(rom, BUTTON_OFFSETS, BUTTON_OFFSET_COUNT)?;
        let definitions = words(
            rom,
            DEFINITIONS,
            DEFINITION_COUNT.wrapping_mul(DEFINITION_WORDS),
        )?;
        let records = definitions.as_chunks::<DEFINITION_WORDS>().0;
        let mut messages = Vec::with_capacity(records.len());
        for (index, &[setup, draw, tilemap]) in records.iter().enumerate() {
            // The message's size is the distance to the next definition's tilemap, which
            // is why the table ends in terminators.
            let end = records.get(index.wrapping_add(1)).map(|next| next[2]);
            let bytes = end.and_then(|end| end.checked_sub(tilemap));
            let tilemap = match bytes {
                Some(bytes) => words(rom, RomAddress::new(BANK, tilemap), usize::from(bytes) / 2)?,
                None => Vec::new(),
            };
            messages.push(Message {
                setup: match setup {
                    SMALL_SETUP => Setup::Small,
                    LARGE_SETUP => Setup::Large,
                    SHOOT_SETUP => Setup::ShootButton,
                    RUN_SETUP => Setup::RunButton,
                    other => return Err(MessageError::Routine(other)),
                },
                borders: match draw {
                    SMALL_DRAW => BorderSize::Small,
                    LARGE_DRAW => BorderSize::Large,
                    other => return Err(MessageError::Routine(other)),
                },
                button: button_offsets.get(index).copied().unwrap_or(0),
                tilemap,
            });
        }
        Ok(Self {
            large_border,
            small_border,
            letters,
            messages,
        })
    }

    /// The message with the one-based `index` the display routine takes.
    #[must_use]
    pub fn message(&self, index: u16) -> Option<&Message> {
        self.messages.get(usize::from(index.checked_sub(1)?))
    }

    /// The top and bottom border rows of boxes of `size`.
    #[must_use]
    pub fn border(&self, size: BorderSize) -> &[u16] {
        match size {
            BorderSize::Large => &self.large_border,
            BorderSize::Small => &self.small_border,
        }
    }

    /// The tile that shows `letter`.
    #[must_use]
    pub fn letter(&self, letter: Letter) -> Option<u16> {
        self.letters.get(letter.index()).copied()
    }
}

fn offset(at: RomAddress, bytes: usize) -> Result<RomAddress, AddressError> {
    at.advance(bytes).ok_or(AddressError::PastEnd(at))
}

/// `count` little-endian words from `at`.
fn words(rom: Lorom<'_>, at: RomAddress, count: usize) -> Result<Vec<u16>, AddressError> {
    Ok(rom
        .read(at, count.wrapping_mul(2))?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&word| u16::from_le_bytes(word))
        .collect())
}

#[cfg(test)]
mod tests;
