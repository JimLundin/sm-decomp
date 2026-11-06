//! The PLM slots, decoded from the original's PLM tables (bank $84 log): one array of 40
//! words per field, from `$1C37` on in bank $7E and the instruction timers' at `$DE1C`.
//!
//! A slot's ID word names its PLM through the room's population: the PLMs it places, and
//! the scroll block touch PLM its scroll PLMs' trigger blocks spawn.

use data::plm::{ItemPlm, PlmKind, PlmPopulation};
use data::rom::RomAddress;
use game::state::room::{BlockIndex, PLM_SLOTS, PlmBody, PlmCommon, ScrollPlm, ScrollStep};
use game::state::slots::{Cursor, Frames, Generation, Occupancy, Slot, SlotStore};

use crate::decode::DecodeError;
use crate::source::{self, WramArray};

/// `$84:B6FF`, the scroll block touch PLM: setup `$84:B393`.
const SCROLL_TOUCH: u16 = 0xb6ff;
/// The bank a scroll PLM's room argument points into, at its scroll changes.
const ROOM_BANK: u8 = 0x8f;
/// A scroll PLM's instruction pointer asleep: its list's sleep (`$84:AF8A`), where its setup
/// leaves it past the debug draw and its update's go-to returns.
const SCROLL_SLEEP: u16 = 0xaf8a;
/// Touched: the air scroll update (`$84:AF8C`), where the touch PLM's setup steps it.
const SCROLL_UPDATE: u16 = 0xaf8c;
/// A scroll PLM's triggered word once touched (`$84:B393`); its update clears it.
const TRIGGERED: u16 = 0x8000;

/// What a slot's ID word names in its room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Known {
    /// The scroll PLM, `$B703`.
    Scroll,
    /// The scroll block touch PLM.
    ScrollTouch,
    /// An item PLM.
    Item(ItemPlm),
}

/// The PLM `id` names in a room with `population`, if the room runs one with it: an
/// extension deletes itself in its setup, so no slot keeps one.
fn known(population: &PlmPopulation, id: u16) -> Option<Known> {
    let mut placed = population.entries.iter().map(|entry| entry.kind);
    if id == SCROLL_TOUCH {
        return placed
            .any(|kind| matches!(kind, PlmKind::Scroll(_)))
            .then_some(Known::ScrollTouch);
    }
    placed
        .find(|kind| kind.id() == id)
        .and_then(|kind| match kind {
            PlmKind::Scroll(_) => Some(Known::Scroll),
            PlmKind::Item(item, _) => Some(Known::Item(item)),
            PlmKind::Extension(_) | PlmKind::Unmodelled(_) => None,
        })
}

/// Where the original keeps each PLM slot's fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlmSources {
    /// The occupancy and its body's kind: the PLM ID.
    pub id: WramArray,
    /// `.common.block`, times two.
    pub block: WramArray,
    /// `.common.cursor.delay`.
    pub timer: WramArray,
    /// A scroll PLM's step: its instruction pointer.
    pub instruction: WramArray,
    /// A scroll PLM's changes and an item's collected bit: the room argument.
    pub argument: WramArray,
    /// A scroll PLM's triggered flag.
    pub variable: WramArray,
}

impl PlmSources {
    /// The original's layout.
    pub const ORIGINAL: Self = Self {
        id: source::PLM_HEADER_PTR,
        block: source::PLM_BLOCK_INDICES,
        timer: source::PLM_INSTRUCTION_TIMER,
        instruction: source::PLM_INSTR_LIST_PTRS,
        argument: source::PLM_ROOM_ARGUMENTS,
        variable: source::PLM_VARIABLE,
    };

    /// Every slot in `wram`, its ID resolved through the room's `population`.
    ///
    /// The generation stays at its default: the original keeps no fill count. A free slot
    /// keeps its last occupant's block and timer, which the original leaves.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for an ID the room runs no PLM with, an odd block
    /// index, or a scroll PLM whose argument names no list of the population's, whose
    /// triggered word is neither 0 nor `$8000`, or whose instruction pointer is neither its
    /// sleep nor its update; [`DecodeError::WramTooShort`] when a table lies past the
    /// record's work RAM.
    pub fn read(
        &self,
        wram: &[u8],
        population: &PlmPopulation,
    ) -> Result<SlotStore<PlmCommon, PlmBody, PLM_SLOTS>, DecodeError> {
        let mut store = SlotStore::default();
        for (slot, index) in store.slots.iter_mut().zip(0_u16..) {
            *slot = self.slot(wram, index, population)?;
        }
        Ok(store)
    }

    /// Slot `index`.
    fn slot(
        &self,
        wram: &[u8],
        index: u16,
        population: &PlmPopulation,
    ) -> Result<Slot<PlmCommon, PlmBody>, DecodeError> {
        let word = |array: WramArray| array.word(index)?.read(wram);
        let unrepresentable = |array: WramArray, value| DecodeError::Unrepresentable {
            source: array.name,
            value,
        };
        let doubled = word(self.block)?;
        if doubled % 2 != 0 {
            return Err(unrepresentable(self.block, doubled));
        }
        let occupancy = match word(self.id)? {
            0 => Occupancy::Free,
            id => Occupancy::Live(
                match known(population, id).ok_or_else(|| unrepresentable(self.id, id))? {
                    Known::Scroll => PlmBody::Scroll(self.scroll(wram, index, population)?),
                    Known::ScrollTouch => PlmBody::ScrollTouch,
                    Known::Item(item) => PlmBody::Item {
                        item,
                        collected_bit: word(self.argument)?,
                    },
                },
            ),
        };
        Ok(Slot {
            generation: Generation::default(),
            common: PlmCommon {
                block: BlockIndex(doubled / 2),
                cursor: Cursor {
                    delay: Frames(word(self.timer)?),
                    ..Cursor::default()
                },
            },
            occupancy,
        })
    }

    /// The scroll PLM in slot `index`.
    fn scroll(
        &self,
        wram: &[u8],
        index: u16,
        population: &PlmPopulation,
    ) -> Result<ScrollPlm, DecodeError> {
        let word = |array: WramArray| array.word(index)?.read(wram);
        let unrepresentable = |array: WramArray, value| DecodeError::Unrepresentable {
            source: array.name,
            value,
        };
        let argument = word(self.argument)?;
        let changes = population
            .changes_at(RomAddress::new(ROOM_BANK, argument))
            .ok_or_else(|| unrepresentable(self.argument, argument))?;
        let triggered = match word(self.variable)? {
            0 => false,
            TRIGGERED => true,
            value => return Err(unrepresentable(self.variable, value)),
        };
        let step = match word(self.instruction)? {
            SCROLL_SLEEP => ScrollStep::Sleep,
            SCROLL_UPDATE => ScrollStep::Update,
            value => return Err(unrepresentable(self.instruction, value)),
        };
        Ok(ScrollPlm {
            changes,
            triggered,
            step,
        })
    }
}
