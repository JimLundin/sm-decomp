//! Room PLM populations, decoded into typed entries.
//!
//! A population entry is six bytes, the PLM ID, the block's
//! X and Y and the room argument; a zero ID ends the list. A header is the setup routine's
//! pointer and the instruction list's.
//!
//! The scroll PLM's room argument points at its scroll changes: byte pairs,
//! a screen and its new scroll area, ended by a byte with bit 7 set.

use canonical::Canonical;

use crate::level::ScrollArea;
use crate::rom::{AddressError, Lorom, RomAddress};

/// Populations and scroll changes.
const ROOM_BANK: u8 = 0x8f;
/// A scroll change list ends at a byte with this bit set (`$84:8B6B`).
const END_OF_CHANGES: u8 = 0x80;

/// `$84:B703`, the scroll PLM: setup `$84:B371`, instruction list `$84:AF86`.
const SCROLL: u16 = 0xb703;
/// `$84:B63B`, the rightwards extension: setup `$84:B33A`.
const RIGHT_EXTENSION: u16 = 0xb63b;
/// `$84:B63F`, the leftwards extension: setup `$84:B345`.
const LEFT_EXTENSION: u16 = 0xb63f;
/// `$84:B643`, the downwards extension: setup `$84:B350`.
const DOWN_EXTENSION: u16 = 0xb643;
/// `$84:B647`, the upwards extension: setup `$84:B35B`.
const UP_EXTENSION: u16 = 0xb647;
/// `$84:EF23`, the Morph Ball: setup `$84:EE64`, instruction list `$84:E3EF`.
const MORPH_BALL: u16 = 0xef23;

/// Which way an extension block points to the block it extends, and the BTS its setup
/// writes: a horizontal extension (`$5`) or a vertical one (`$D`), offset 1 or -1.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Extension {
    /// Horizontal, BTS 0xFF, the block to the left.
    #[default]
    Right,
    /// Horizontal, BTS 1, the block to the right.
    Left,
    /// Vertical, BTS 0xFF, the block above.
    Down,
    /// Vertical, BTS 1, the block below.
    Up,
}

/// An item PLM.
// open: the other items join with the rooms that place them; the Morph Ball's pickup is C12.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemPlm {
    /// The Morph Ball's pickup.
    #[default]
    MorphBall,
}

/// A scroll change list in a room state's [`PlmPopulation`]: its place in `scroll_changes`.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScrollChangesId(pub u16);

/// One scroll change: screen `screen`'s scroll area becomes `area`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollChange {
    /// The screen, row-major.
    pub screen: u8,
    /// Its new scroll area.
    pub area: ScrollArea,
}

/// What a population entry spawns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlmKind {
    /// The scroll PLM, with the changes Samus touching it makes.
    Scroll(ScrollChangesId),
    /// An extension block.
    Extension(Extension),
    /// An item, with its collected bit's index (the room argument).
    Item(ItemPlm, u16),
    /// A PLM slice 0 doesn't run, by its ID: the Morph Ball Room's Zebes-awake state places
    /// one (`$84:C848`), which slice 0 never loads.
    // open: each joins as a kind with the room that runs it.
    Unmodelled(u16),
}

impl PlmKind {
    /// The PLM ID the original keeps in the slot.
    #[must_use]
    pub const fn id(self) -> u16 {
        match self {
            Self::Scroll(_) => SCROLL,
            Self::Extension(Extension::Right) => RIGHT_EXTENSION,
            Self::Extension(Extension::Left) => LEFT_EXTENSION,
            Self::Extension(Extension::Down) => DOWN_EXTENSION,
            Self::Extension(Extension::Up) => UP_EXTENSION,
            Self::Item(ItemPlm::MorphBall, _) => MORPH_BALL,
            Self::Unmodelled(id) => id,
        }
    }
}

/// A population entry: a PLM and its block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlmEntry {
    /// What it spawns.
    pub kind: PlmKind,
    /// The block's column.
    pub x: u8,
    /// The block's row.
    pub y: u8,
}

/// A room state's PLM population, in spawn order, and the scroll changes its scroll PLMs
/// make.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlmPopulation {
    /// The entries, in the original's order.
    pub entries: Vec<PlmEntry>,
    /// Each distinct scroll change list, first named first, with where it is.
    pub scroll_changes: Vec<(RomAddress, Vec<ScrollChange>)>,
}

impl PlmPopulation {
    /// The changes `id` names.
    #[must_use]
    pub fn changes(&self, id: ScrollChangesId) -> Option<&[ScrollChange]> {
        self.scroll_changes
            .get(usize::from(id.0))
            .map(|(_, changes)| changes.as_slice())
    }

    /// The list at `at`, a scroll PLM's room argument, if the population names it.
    #[must_use]
    pub fn changes_at(&self, at: RomAddress) -> Option<ScrollChangesId> {
        self.scroll_changes
            .iter()
            .position(|(known, _)| *known == at)
            .and_then(|index| u16::try_from(index).ok())
            .map(ScrollChangesId)
    }
}

/// Why a population doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlmError {
    /// A read outside the image.
    Address(AddressError),
    /// The list runs past its bank's end.
    BankEnd(RomAddress),
    /// A scroll change names an area that doesn't exist.
    ScrollArea {
        /// Where the change is.
        at: RomAddress,
        /// Its area byte.
        value: u8,
    },
    /// More scroll change lists than a [`ScrollChangesId`] counts.
    TooManyLists,
}

impl From<AddressError> for PlmError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// `N` bytes at `at`, and the address after them.
fn take<const N: usize>(rom: Lorom<'_>, at: RomAddress) -> Result<([u8; N], RomAddress), PlmError> {
    let bytes = rom.read(at, N)?;
    let array = <[u8; N]>::try_from(bytes).map_err(|_| PlmError::BankEnd(at))?;
    let next = at.advance(N).ok_or(PlmError::BankEnd(at))?;
    Ok((array, next))
}

/// The scroll change list at `at` (`$84:8B68`).
fn scroll_changes(rom: Lorom<'_>, at: RomAddress) -> Result<Vec<ScrollChange>, PlmError> {
    let mut changes = Vec::new();
    let mut cursor = at;
    loop {
        let ([screen], after_screen) = take::<1>(rom, cursor)?;
        if screen & END_OF_CHANGES != 0 {
            return Ok(changes);
        }
        let ([value], next) = take::<1>(rom, after_screen)?;
        let area = ScrollArea::try_from(value).map_err(|value| PlmError::ScrollArea {
            at: after_screen,
            value,
        })?;
        changes.push(ScrollChange { screen, area });
        cursor = next;
    }
}

/// Decode the population at `at`.
///
/// # Errors
///
/// [`PlmError`] if the list is outside the image or a scroll change names no area.
pub fn population(rom: Lorom<'_>, at: RomAddress) -> Result<PlmPopulation, PlmError> {
    let mut decoded = PlmPopulation::default();
    let mut cursor = at;
    loop {
        let ([id_low, id_high], after_id) = take::<2>(rom, cursor)?;
        let id = u16::from_le_bytes([id_low, id_high]);
        if id == 0 {
            return Ok(decoded);
        }
        let ([x, y, argument_low, argument_high], next) = take::<4>(rom, after_id)?;
        let argument = u16::from_le_bytes([argument_low, argument_high]);
        let kind = match id {
            SCROLL => {
                let list = RomAddress::new(ROOM_BANK, argument);
                if let Some(known) = decoded.changes_at(list) {
                    PlmKind::Scroll(known)
                } else {
                    let index = u16::try_from(decoded.scroll_changes.len())
                        .map_err(|_| PlmError::TooManyLists)?;
                    decoded
                        .scroll_changes
                        .push((list, scroll_changes(rom, list)?));
                    PlmKind::Scroll(ScrollChangesId(index))
                }
            }
            RIGHT_EXTENSION => PlmKind::Extension(Extension::Right),
            LEFT_EXTENSION => PlmKind::Extension(Extension::Left),
            DOWN_EXTENSION => PlmKind::Extension(Extension::Down),
            UP_EXTENSION => PlmKind::Extension(Extension::Up),
            MORPH_BALL => PlmKind::Item(ItemPlm::MorphBall, argument),
            id => PlmKind::Unmodelled(id),
        };
        decoded.entries.push(PlmEntry { kind, x, y });
        cursor = next;
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Room PLM populations, decoded from bank `$8F` into typed entries.
// Bank $84 log (<https://patrickjohnston.org/bank/84>), spawn room PLM (`$84:846A`): a
// population entry is six bytes, the PLM ID (its header's offset in bank `$84`), the block's
// The scroll PLM's room argument points into bank `$8F` at its scroll changes: byte pairs,
// a screen and its new scroll area, ended by a byte with bit 7 set (`$84:8B55`).
// `$B63B`: horizontal, BTS `$FF`, the block to the left.
// `$B63F`: horizontal, BTS 1.
// `$B643`: vertical, BTS `$FF`, the block above.
// `$B647`: vertical, BTS 1, the block below.
// `$EF23`.
// One scroll change: screen `screen`'s scroll area becomes `area` (`$84:8B71`).
// The PLM ID the original keeps in the slot (`$84:84AD`).
