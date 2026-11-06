//! PLMs: spawning a room's population, the scroll PLMs' trigger and the PLM pass for the
//! PLMs slice 0's rooms run (bank $84 log, <https://patrickjohnston.org/bank/84>).

use std::marker::PhantomData;

use data::plm::{Extension, PlmEntry, PlmKind, PlmPopulation};

use crate::state::room::{
    Block, BlockIndex, BlockKind, Bts, Offset, Plm, PlmBody, PlmCommon, RoomScoped, ScrollPlm,
    ScrollStep,
};
use crate::state::slots::{Frames, Occupancy, Slot, SlotIndex};

/// The scroll PLM's trigger block's BTS (`$84:B374`): special air whose inside reaction
/// spawns the scroll block touch PLM.
const SCROLL_TRIGGER: Bts = Bts(0x46);
/// The item PLMs' block's BTS (`$84:EE67`): air the item collision detection looks for.
const ITEM_BLOCK: Bts = Bts(0x45);

/// Why touching a scroll block can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollTouchFault {
    /// No PLM slot has the touched block: the original hangs (`$84:B3A6`, `BRA $FE`).
    NoPlmOnBlock(BlockIndex),
    /// The slot with the touched block holds no scroll PLM: the original would write its
    /// scroll-triggered word and instruction pointer anyway; no slice 0 room reaches it.
    NotAScrollPlm(SlotIndex<Plm>),
}

/// The free slot a spawn takes: the highest (`$84:8470`, `$84:84ED`, from `4Eh` down).
fn free_slot(room: &RoomScoped) -> Option<usize> {
    room.plms
        .slots
        .iter()
        .rposition(|slot| matches!(slot.occupancy, Occupancy::Free))
}

/// Fill `slot` with a PLM on `block` whose instruction timer is 1 (`$84:84C5`), its setup
/// leaving it `occupancy`. A field the spawn doesn't write keeps the last occupant's value.
const fn fill(
    slot: &mut Slot<PlmCommon, PlmBody>,
    block: BlockIndex,
    occupancy: Occupancy<PlmBody>,
) {
    slot.generation.0 = slot.generation.0.wrapping_add(1);
    slot.common.block = block;
    slot.common.cursor.delay = Frames(1);
    slot.occupancy = occupancy;
}

/// Replace block `block`'s type and BTS, keeping its tile (`$84:82B4`).
fn write_block(room: &mut RoomScoped, block: BlockIndex, kind: BlockKind) {
    if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
        cell.kind = kind;
    }
}

/// The block an extension's setup writes: horizontal or vertical, offset 1 or -1
/// (`$84:B33A`, `$84:B345`, `$84:B350`, `$84:B35B`).
const fn extension_block(extension: Extension) -> BlockKind {
    match extension {
        Extension::Right => BlockKind::HorizontalExtension(Offset(-1)),
        Extension::Left => BlockKind::HorizontalExtension(Offset(1)),
        Extension::Down => BlockKind::VerticalExtension(Offset(-1)),
        Extension::Up => BlockKind::VerticalExtension(Offset(1)),
    }
}

/// Spawn one room PLM and run its setup (`$84:846A`). With no slot free nothing spawns, as
/// the original returns with carry set.
fn spawn(room: &mut RoomScoped, entry: PlmEntry) {
    let Some(index) = free_slot(room) else {
        return;
    };
    // `$84:8495`: the row times the room's width in blocks, plus the column.
    let block = BlockIndex(
        u16::from(entry.y)
            .wrapping_mul(room.width.0)
            .wrapping_add(u16::from(entry.x)),
    );
    let body = match entry.kind {
        PlmKind::Scroll(changes) => {
            // `$84:B371`: the trigger block, not triggered, past the debug draw.
            write_block(room, block, BlockKind::SpecialAir(SCROLL_TRIGGER));
            PlmBody::Scroll(ScrollPlm {
                changes,
                triggered: false,
                step: ScrollStep::Sleep,
            })
        }
        PlmKind::Extension(extension) => {
            // The extension's setup writes its block and deletes it (`$84:B333`).
            write_block(room, block, extension_block(extension));
            if let Some(slot) = room.plms.slots.get_mut(index) {
                fill(slot, block, Occupancy::Free);
            }
            return;
        }
        // open: no slice 0 room state that loads places one; each joins as a kind.
        PlmKind::Unmodelled(_) => return,
        PlmKind::Item(item, collected_bit) => {
            // `$84:EE64`: air with the item's BTS.
            // open: the global items-loaded counter it increments (`$7E:D91A`).
            write_block(room, block, BlockKind::Air(ITEM_BLOCK));
            PlmBody::Item {
                item,
                collected_bit,
            }
        }
    };
    if let Some(slot) = room.plms.slots.get_mut(index) {
        fill(slot, block, Occupancy::Live(body));
    }
}

/// Spawn the room's PLMs, in population order (bank $82 log, room loading).
pub fn spawn_population(room: &mut RoomScoped, population: &PlmPopulation) {
    for &entry in &population.entries {
        spawn(room, entry);
    }
}

/// Samus touches scroll trigger block `block`.
///
/// The special air inside reaction spawns the scroll block touch PLM there (`$84:B6FF`,
/// through `$84:84E7`), whose setup (`$84:B393`) moves its own block to 0 and triggers the
/// PLM on `block` unless it already is: the scroll PLM steps to its update and its timer
/// is 1.
///
/// # Errors
///
/// [`ScrollTouchFault`] where the original hangs or would write into another kind of PLM.
pub fn touch_scroll_block(
    room: &mut RoomScoped,
    block: BlockIndex,
) -> Result<(), ScrollTouchFault> {
    if let Some(index) = free_slot(room)
        && let Some(slot) = room.plms.slots.get_mut(index)
    {
        fill(slot, BlockIndex(0), Occupancy::Live(PlmBody::ScrollTouch));
    }
    // `$84:B39D`: every slot's block, free or not, from the highest down.
    let found = room
        .plms
        .slots
        .iter()
        .rposition(|slot| slot.common.block == block)
        .ok_or(ScrollTouchFault::NoPlmOnBlock(block))?;
    let index = SlotIndex(u8::try_from(found).unwrap_or(u8::MAX), PhantomData);
    let Some(slot) = room.plms.slots.get_mut(found) else {
        return Err(ScrollTouchFault::NotAScrollPlm(index));
    };
    let Occupancy::Live(PlmBody::Scroll(scroll)) = &mut slot.occupancy else {
        return Err(ScrollTouchFault::NotAScrollPlm(index));
    };
    if !scroll.triggered {
        scroll.triggered = true;
        scroll.step = ScrollStep::Update;
        slot.common.cursor.delay = Frames(1);
    }
    Ok(())
}

/// The air scroll update (`$84:8B55`): not triggered, each change written to its screen,
/// and the block made special air again with its BTS kept.
fn update_scroll(
    room: &mut RoomScoped,
    block: BlockIndex,
    plm: &mut ScrollPlm,
    population: &PlmPopulation,
) {
    plm.triggered = false;
    for change in population.changes(plm.changes).unwrap_or_default() {
        // open: a screen past the room's would write past the scroll table; slice 0's lists
        // name the room's own screens.
        if let Some(area) = room.scroll_areas.get_mut(usize::from(change.screen)) {
            *area = change.area;
        }
    }
    if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
        let bts = Bts(cell.kind.bts());
        *cell = Block {
            kind: BlockKind::SpecialAir(bts),
            tile: cell.tile,
        };
    }
}

/// The PLM pass (`$84:85B4`): each live PLM from the highest slot down. Its pre-instruction
/// (an `RTS` for these) runs, its timer counts down, and when it runs out its list runs.
// open: the PLMs-enabled flag (`$1C23`) and the item's list (C12); an item's timer waits.
pub fn process(room: &mut RoomScoped, population: &PlmPopulation) {
    for index in (0..room.plms.slots.len()).rev() {
        let Some(slot) = room.plms.slots.get_mut(index) else {
            continue;
        };
        let Occupancy::Live(body) = slot.occupancy else {
            continue;
        };
        if matches!(body, PlmBody::Item { .. }) {
            continue;
        }
        // `$84:85E0`: the timer counts down, wrapping from 0 after a sleep.
        slot.common.cursor.delay = Frames(slot.common.cursor.delay.0.wrapping_sub(1));
        if slot.common.cursor.delay != Frames(0) {
            continue;
        }
        let block = slot.common.block;
        match body {
            PlmBody::Scroll(mut scroll) => {
                if scroll.step == ScrollStep::Update {
                    update_scroll(room, block, &mut scroll, population);
                    // `$84:AF8E`: back to the sleep.
                    scroll.step = ScrollStep::Sleep;
                }
                // `$84:86B4`: asleep; the timer stays at 0.
                if let Some(slot) = room.plms.slots.get_mut(index) {
                    slot.occupancy = Occupancy::Live(PlmBody::Scroll(scroll));
                }
            }
            // `$84:AAE3`: delete.
            PlmBody::ScrollTouch => slot.occupancy = Occupancy::Free,
            PlmBody::Item { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests;
