//! The PLM slots, decoded from the original's PLM tables (bank $84 log): one array of 40
//! words per field, from `$1C37` on in bank $7E and the instruction timers' at `$DE1C`.
//!
//! A slot's ID word names its PLM through the room's population: the PLMs it places, and
//! the scroll block touch PLM its scroll PLMs' trigger blocks spawn. The item collision
//! detection PLM and the 1x1 shot block, which Samus's touches and shots spawn wherever
//! the blocks for them are, the ID names directly.

use data::plm::{ItemPlm, PlmKind, PlmPopulation};
use data::rom::RomAddress;
use game::state::room::{
    BlockIndex, Crumble, ItemFrame, ItemStep, PLM_SLOTS, PlmBody, PlmCommon, ScrollPlm, ScrollStep,
};
use game::state::slots::{Cursor, Frames, Generation, Occupancy, Slot, SlotStore};

use crate::decode::DecodeError;
use crate::source::{self, WramArray};

/// `$84:B6FF`, the scroll block touch PLM: setup `$84:B393`.
const SCROLL_TOUCH: u16 = 0xb6ff;
/// `$84:EED3`, the item collision detection PLM: setup `$84:EEAB`.
const ITEM_COLLISION: u16 = 0xeed3;
/// `$84:D074`, the 1x1 shot block: setup `$84:B3C1`, list `$84:CBB7`.
const SHOT_BLOCK: u16 = 0xd074;
/// The delete a touch PLM's list is (`$84:AAE3`): its pointer until its first run deletes it.
const DELETE_LIST: u16 = 0xaae3;
/// The 1x1 shot block's instruction pointer at each place in its crumble (`$84:CBB7`): its
/// list's start, then past each draw.
const CRUMBLE: [(u16, Crumble); 5] = [
    (0xcbb7, Crumble::Spawned),
    (0xcbbe, Crumble::First),
    (0xcbc2, Crumble::Second),
    (0xcbc6, Crumble::Third),
    (0xcbca, Crumble::Cleared),
];
/// The bank a scroll PLM's room argument points into, at its scroll changes.
const ROOM_BANK: u8 = 0x8f;
/// A scroll PLM's instruction pointer asleep: its list's sleep (`$84:AF8A`), where its setup
/// leaves it past the debug draw and its update's go-to returns.
const SCROLL_SLEEP: u16 = 0xaf8a;
/// Touched: the air scroll update (`$84:AF8C`), where the touch PLM's setup steps it.
const SCROLL_UPDATE: u16 = 0xaf8c;
/// An item's instruction pointer from its spawn to its first run: its list's start, the GFX
/// load (`$84:8764`).
const fn item_list(item: ItemPlm) -> u16 {
    match item {
        ItemPlm::MorphBall => 0xe3ef,
    }
}
/// An item's instruction pointer inside its pickup's message box: its link (`$84:DF89`
/// leaves it there and the pickup's instructions run from `Y`).
const fn item_link(item: ItemPlm) -> u16 {
    match item {
        ItemPlm::MorphBall => 0xe40f,
    }
}
/// An item's instruction pointer past frame 0's draw, waiting to draw frame 1 (`$84:E067`).
/// Anywhere else before its link, its next draw is frame 0's.
const fn item_second_frame(item: ItemPlm) -> u16 {
    match item {
        ItemPlm::MorphBall => 0xe409,
    }
}
/// An item's instruction pointer once the empty item is drawn: its delete (`$84:DFAD`).
const EMPTY_ITEM_DELETE: u16 = 0xdfad;
/// An item's triggered byte once touched (`$84:EEC4`).
const ITEM_TRIGGERED: u16 = 0x00ff;

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
    /// The item collision detection PLM.
    ItemCollision,
    /// The 1x1 shot block.
    ShotBlock,
}

/// The PLM `id` names in a room with `population`, if the room runs one with it: an
/// extension deletes itself in its setup, so no slot keeps one. A touch PLM runs only in a
/// room whose population places the PLMs it touches.
fn known(population: &PlmPopulation, id: u16) -> Option<Known> {
    let mut placed = population.entries.iter().map(|entry| entry.kind);
    match id {
        SCROLL_TOUCH => placed
            .any(|kind| matches!(kind, PlmKind::Scroll(_)))
            .then_some(Known::ScrollTouch),
        ITEM_COLLISION => placed
            .any(|kind| matches!(kind, PlmKind::Item(..)))
            .then_some(Known::ItemCollision),
        SHOT_BLOCK => Some(Known::ShotBlock),
        _ => placed
            .find(|kind| kind.id() == id)
            .and_then(|kind| match kind {
                PlmKind::Scroll(_) => Some(Known::Scroll),
                PlmKind::Item(item, _) => Some(Known::Item(item)),
                PlmKind::Extension(_) | PlmKind::Unmodelled(_) => None,
            }),
    }
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
    /// A scroll PLM's step and the shot block's crumble: the instruction pointer.
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
    /// index, a scroll PLM whose argument names no list of the population's, whose
    /// triggered word is neither 0 nor `$8000`, or whose instruction pointer is neither its
    /// sleep nor its update, an item collision detection PLM past its delete, or a shot block
    /// whose instruction pointer is no place in its crumble; [`DecodeError::WramTooShort`] when a table lies past the
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
                        step: match word(self.instruction)? {
                            at if at == item_list(item) => ItemStep::Spawned,
                            at if at == item_link(item) => ItemStep::InBox,
                            EMPTY_ITEM_DELETE => ItemStep::Emptied,
                            _ if word(self.variable)? & 0x00ff == ITEM_TRIGGERED => {
                                ItemStep::Triggered
                            }
                            at if at == item_second_frame(item) => ItemStep::Shown {
                                next: ItemFrame::Second,
                            },
                            _ => ItemStep::Shown {
                                next: ItemFrame::First,
                            },
                        },
                    },
                    Known::ItemCollision => match word(self.instruction)? {
                        DELETE_LIST => PlmBody::ItemCollision,
                        value => return Err(unrepresentable(self.instruction, value)),
                    },
                    Known::ShotBlock => {
                        let at = word(self.instruction)?;
                        let crumble = CRUMBLE
                            .iter()
                            .find_map(|&(pointer, crumble)| (pointer == at).then_some(crumble))
                            .ok_or_else(|| unrepresentable(self.instruction, at))?;
                        PlmBody::ShotBlock { crumble }
                    }
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
