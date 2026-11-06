//! PLMs: spawning a room's population, the scroll PLMs' and items' triggers and the PLM pass
//! for the PLMs slice 0's rooms run (bank $84 log, <https://patrickjohnston.org/bank/84>).

use std::marker::PhantomData;

use data::id::RoomId;
use data::level::{Tile, TileNumber};
use data::plm::{Extension, ItemPlm, PlmEntry, PlmKind, PlmPopulation};

use crate::message_box;
use crate::state::room::{
    Block, BlockIndex, BlockKind, Bts, ItemStep, Offset, Plm, PlmBody, PlmChoice, PlmCommon,
    RoomScoped, ScrollPlm, ScrollStep,
};
use crate::state::samus::Item;
use crate::state::slots::{Frames, Occupancy, Slot, SlotIndex};
use crate::state::strand::Message;
use crate::state::{CollectedItems, State};
use crate::tick::Next;

/// The scroll PLM's trigger block's BTS (`$84:B374`): special air whose inside reaction
/// spawns the scroll block touch PLM.
const SCROLL_TRIGGER: Bts = Bts(0x46);
/// The item PLMs' block's BTS (`$84:EE67`): air the item collision detection looks for.
const ITEM_BLOCK: Bts = Bts(0x45);
/// How long an item shows each of its two frames: each draw instruction draws the PLM and
/// waits 4 frames (`$84:E04F`, `$84:E067`, through `$84:E07F`).
const ITEM_FRAME: Frames = Frames(4);

/// The empty item's draw (`$84:A2B5`): one block, air with tile `FFh`.
const EMPTY_ITEM_TILE: Tile = Tile {
    number: TileNumber(0xff),
    x_flip: false,
    y_flip: false,
};

/// What an item's pickup instruction gives and shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pickup {
    /// The equipment equipped and collected.
    item: Item,
    /// The message box it displays.
    message: Message,
}

/// An item PLM's pickup (`$84:88F3` with its list's arguments).
const fn pickup(item: ItemPlm) -> Pickup {
    match item {
        // `$84:E413`: equipment 4, message 9.
        ItemPlm::MorphBall => Pickup {
            item: Item::MorphBall,
            message: Message::MORPH_BALL,
        },
    }
}

/// A room state with no PLMs.
static NO_PLMS: PlmPopulation = PlmPopulation {
    entries: Vec::new(),
    scroll_changes: Vec::new(),
};

/// Each loaded room's PLM population: the one its state places, whose scroll change lists
/// the PLM pass reads.
// open: a room whose states place different populations needs the loaded state in
// `RoomScoped`; slice 0 loads only the Morph Ball Room's state for a new game's progress.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomPlms(pub Vec<(RoomId, PlmPopulation)>);

impl RoomPlms {
    /// Room `room`'s population; none for a room the content doesn't hold.
    #[must_use]
    pub fn of(&self, room: RoomId) -> &PlmPopulation {
        self.0
            .iter()
            .find_map(|(id, population)| (*id == room).then_some(population))
            .unwrap_or(&NO_PLMS)
    }
}

/// Why touching a scroll block or an item can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchFault {
    /// No PLM slot has the touched block: the original hangs (`$84:B3A6`, `BRA $FE`) or
    /// crashes (`$84:EEC3`, `BRK`).
    NoPlmOnBlock(BlockIndex),
    /// The slot with the touched block holds no scroll PLM: the original would write its
    /// scroll-triggered word and instruction pointer anyway; no slice 0 room reaches it.
    NotAScrollPlm(SlotIndex<Plm>),
    /// The slot with the touched item block holds no item: the original would trigger it
    /// anyway; no slice 0 room reaches it.
    NotAnItem(SlotIndex<Plm>),
}

/// The slot index of slot `index`.
fn slot_index(index: usize) -> SlotIndex<Plm> {
    SlotIndex(u8::try_from(index).unwrap_or(u8::MAX), PhantomData)
}

/// The highest slot, free or not, whose block is `block` (`$84:B39D`, `$84:EEBA`).
fn slot_on(room: &RoomScoped, block: BlockIndex) -> Result<usize, TouchFault> {
    room.plms
        .slots
        .iter()
        .rposition(|slot| slot.common.block == block)
        .ok_or(TouchFault::NoPlmOnBlock(block))
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
                step: ItemStep::Shown,
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
/// is 1. With no slot free nothing spawns and no setup runs (`$84:84F9`).
///
/// # Errors
///
/// [`TouchFault`] where the original hangs or would write into another kind of PLM.
pub fn touch_scroll_block(room: &mut RoomScoped, block: BlockIndex) -> Result<(), TouchFault> {
    let Some(slot) = free_slot(room).and_then(|index| room.plms.slots.get_mut(index)) else {
        return Ok(());
    };
    fill(slot, BlockIndex(0), Occupancy::Live(PlmBody::ScrollTouch));
    let found = slot_on(room, block)?;
    let index = slot_index(found);
    let Some(slot) = room.plms.slots.get_mut(found) else {
        return Err(TouchFault::NotAScrollPlm(index));
    };
    let Occupancy::Live(PlmBody::Scroll(scroll)) = &mut slot.occupancy else {
        return Err(TouchFault::NotAScrollPlm(index));
    };
    if !scroll.triggered {
        scroll.triggered = true;
        scroll.step = ScrollStep::Update;
        slot.common.cursor.delay = Frames(1);
    }
    Ok(())
}

/// Samus touches item block `block`.
///
/// The special block collision reaction for BTS `$45` spawns the item collision detection PLM
/// there (`$94:9139` + `$45` * 2 = `$84:EED3`, through `$84:84E7`), whose setup (`$84:EEAB`)
/// moves its own block to 0 and triggers the PLM on `block`: the item's pre-instruction picks
/// it up when the PLM pass next reaches it. With no slot free nothing spawns.
///
/// # Errors
///
/// [`TouchFault`] where the original crashes or would trigger another kind of PLM.
// open: with time frozen (`$0A78`) the setup deletes the PLM and triggers nothing; slice 0
// never freezes time.
pub fn touch_item_block(room: &mut RoomScoped, block: BlockIndex) -> Result<(), TouchFault> {
    let Some(slot) = free_slot(room).and_then(|index| room.plms.slots.get_mut(index)) else {
        return Ok(());
    };
    fill(slot, BlockIndex(0), Occupancy::Live(PlmBody::ItemCollision));
    let found = slot_on(room, block)?;
    let index = slot_index(found);
    let Some(slot) = room.plms.slots.get_mut(found) else {
        return Err(TouchFault::NotAnItem(index));
    };
    let Occupancy::Live(PlmBody::Item { step, .. }) = &mut slot.occupancy else {
        return Err(TouchFault::NotAnItem(index));
    };
    // Once its pre-instruction has gone to the pickup, the trigger no longer matters.
    if *step == ItemStep::Shown {
        *step = ItemStep::Triggered;
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
/// runs, its timer counts down, and when it runs out its list runs.
///
/// An item's pickup calls out of the pass into the message box: the slot whose instruction
/// does is returned, the rest of the pass waiting for [`resume`].
// open: the PLMs-enabled flag (`$1C23`); the item's GFX load and collected check at its
// list's start, and its draws' tiles and tilemap writes; the shot block's crumble, whose draws
// need the same writes, its timer waits. An untriggered item's list from its start or either
// frame comes to a draw, so its timer counts without the list's place.
pub fn process(
    room: &mut RoomScoped,
    collected: &mut CollectedItems,
    population: &PlmPopulation,
) -> Option<SlotIndex<Plm>> {
    let end = room.plms.slots.len();
    run_below(room, collected, population, end)
}

/// The rest of the PLM pass from an item's pickup.
///
/// Returned from its message box (`$84:8916`), the item's list goes on to the empty item, whose draw waits a frame
/// (`$84:DFA9`), and the pass goes on below its slot. The slot an item called out of that
/// runs its pickup instruction first (`$84:88F3`): equip and collect its equipment, then
/// display its message box, the call this returns. Another item's pickup further down
/// moves the pass to its slot.
pub fn resume(
    state: &mut State,
    population: &PlmPopulation,
    at: SlotIndex<Plm>,
) -> (SlotIndex<Plm>, Next) {
    let index = usize::from(at.0);
    let room = &mut state.room;
    if let Some(slot) = room.plms.slots.get_mut(index)
        && let Occupancy::Live(PlmBody::Item { item, step, .. }) = &mut slot.occupancy
    {
        match *step {
            ItemStep::PickingUp => {
                let Pickup { item, message } = pickup(*item);
                *step = ItemStep::InBox;
                let equipment = &mut state.samus.equipment;
                equipment.equipped.insert(item);
                equipment.collected.insert(item);
                // open: the room's music after 6 seconds (`$82:E118`) is C11's.
                return (at, message_box::display(state, message));
            }
            ItemStep::InBox => {
                *step = ItemStep::Emptied;
                slot.common.cursor.delay = Frames(1);
                let block = slot.common.block;
                if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
                    *cell = Block {
                        kind: BlockKind::Air(Bts(cell.kind.bts())),
                        tile: EMPTY_ITEM_TILE,
                    };
                }
            }
            ItemStep::Shown | ItemStep::Triggered | ItemStep::Emptied => {}
        }
    }
    run_below(
        &mut state.room,
        &mut state.collected_items,
        population,
        index,
    )
    .map_or((at, Next::Return), |next| (next, Next::Continue))
}

/// The PLM pass over the slots below `end`, from the highest down.
fn run_below(
    room: &mut RoomScoped,
    collected: &mut CollectedItems,
    population: &PlmPopulation,
    end: usize,
) -> Option<SlotIndex<Plm>> {
    for index in (0..end).rev() {
        let Some(slot) = room.plms.slots.get_mut(index) else {
            continue;
        };
        let Occupancy::Live(body) = slot.occupancy else {
            continue;
        };
        match body {
            PlmBody::ShotBlock { .. } => continue,
            // `$84:DF89`: triggered, the pre-instruction goes to the link with a timer of 1.
            PlmBody::Item {
                step: ItemStep::Triggered,
                ..
            } => slot.common.cursor.delay = Frames(1),
            PlmBody::Item { .. }
            | PlmBody::Scroll(_)
            | PlmBody::ScrollTouch
            | PlmBody::ItemCollision => {}
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
            PlmBody::ScrollTouch | PlmBody::ItemCollision => slot.occupancy = Occupancy::Free,
            PlmBody::Item {
                item,
                collected_bit,
                step,
            } => match step {
                // The next frame's draw, then its wait (`$84:E407`, `$84:E40B`'s go-to back):
                // the block becomes the special block Samus collides with (`$84:A30F`,
                // `$84:A315`: block type `Bh`), its BTS kept.
                // open: the frames' tiles (`$08E`, `$08F` for the GFX index) and tilemap.
                ItemStep::Shown => {
                    slot.common.cursor.delay = ITEM_FRAME;
                    write_block(
                        room,
                        block,
                        BlockKind::Special(PlmChoice::Common(ITEM_BLOCK.0)),
                    );
                }
                // `$84:E40F`: set the room argument item collected (`$84:8899`), then the
                // pickup instruction calls out.
                // open: clearing the music queue and queueing the fanfare (`$84:8BDD`) is C11's.
                ItemStep::Triggered => {
                    collected.insert(collected_bit);
                    slot.occupancy = Occupancy::Live(PlmBody::Item {
                        item,
                        collected_bit,
                        step: ItemStep::PickingUp,
                    });
                    return Some(slot_index(index));
                }
                // `$84:DFAD`: delete.
                ItemStep::Emptied => slot.occupancy = Occupancy::Free,
                // Only [`resume`] runs these, the pass waiting on their message box.
                ItemStep::PickingUp | ItemStep::InBox => {}
            },
            PlmBody::ShotBlock { .. } => {}
        }
    }
    None
}

#[cfg(test)]
mod tests;
