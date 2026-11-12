//! PLMs: spawning a room's population, the items' triggers and the PLM pass.
//!
//! Bank $84 log, <https://patrickjohnston.org/bank/84>. The items are slice 0's; the blocks'
//! places, the scroll cells' and the reactions', are the blocks model's (`world_blocks_bridge`), and
//! the pass runs them between the items'.

use std::marker::PhantomData;

use data::id::RoomId;
use data::level::{Tile, TileNumber};
use data::plm::{ItemPlm, PlmKind, PlmPopulation};
use data::tileset::TileTables;

use crate::state::room::{
    Block, BlockIndex, BlockKind, Bts, ItemFrame, ItemGfxSlot, ItemStep, Plm, PlmBody, PlmChoice,
    PlmCommon, RoomScoped,
};
use crate::state::samus::Item;
use crate::state::slots::{Frames, Occupancy, Slot, SlotIndex};
use crate::state::strand::Message;
use crate::state::{CollectedItems, State};
use crate::tick::Next;
use crate::world_blocks_bridge::{self, BlockContent, BlocksFault, Reads};
use crate::{message_box, run_progress_bridge};

use self::draw::Drawing;

pub mod draw;
pub mod item_gfx;

/// The item PLMs' block's BTS (`$84:EE67`): air the item collision detection looks for.
const ITEM_BLOCK: Bts = Bts(0x45);
/// How long an item shows each of its two frames: each draw instruction draws the PLM and
/// waits 4 frames (`$84:E04F`, `$84:E067`, through `$84:E07F`).
const ITEM_FRAME: Frames = Frames(4);

/// The tile an item's frame draws (`$84:A30F` on): its GFX slot's block for the frame, through
/// the frame tables `$84:E05F` and `$84:E077`.
fn item_frame_tile(slot: ItemGfxSlot, frame: ItemFrame) -> Tile {
    Tile {
        number: TileNumber(item_gfx::frame_block(slot, frame)),
        x_flip: false,
        y_flip: false,
    }
}

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

/// Why touching an item can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchFault {
    /// No PLM slot has the touched block: the original crashes (`$84:EEC3`, `BRK`).
    NoPlmOnBlock(BlockIndex),
    /// The slot with the touched item block holds no item: the original would trigger it
    /// anyway; no slice 0 room reaches it.
    NotAnItem(SlotIndex<Plm>),
}

/// The slot index of slot `index`.
fn slot_index(index: usize) -> SlotIndex<Plm> {
    SlotIndex(u8::try_from(index).unwrap_or(u8::MAX), PhantomData)
}

/// The highest slot, free or not, whose block is `block` (`$84:EEBA`).
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

/// Spawn one item of the room's population (`$84:846A`), into the place the blocks' start of
/// the room left for it. With no slot free nothing spawns, as the original returns with carry
/// set.
fn spawn_item(room: &mut RoomScoped, index: usize, block: BlockIndex, item: ItemPlm, bit: u16) {
    // `$84:EE64`: air with the item's BTS.
    // open: the global items-loaded counter it increments (`$7E:D91A`).
    write_block(room, block, BlockKind::Air(ITEM_BLOCK));
    let body = PlmBody::Item {
        item,
        collected_bit: bit,
        step: ItemStep::Spawned,
        // `$84:84B5`: a spawn zeroes it; the list's GFX load sets it.
        slot: ItemGfxSlot::default(),
    };
    if let Some(slot) = room.plms.slots.get_mut(index) {
        fill(slot, block, Occupancy::Live(body));
    }
}

/// Spawn the room's PLMs, in population order (bank $82 log, room loading): the scroll cells and
/// stand-ins through the blocks model's start of the room, each item into the place it leaves.
///
/// # Errors
///
/// A [`BlocksFault`] where a placement's cell is one the room's loading would refuse.
pub fn spawn_population(
    room: &mut RoomScoped,
    content: &BlockContent,
    population: &PlmPopulation,
) -> Result<(), BlocksFault> {
    let taken = world_blocks_bridge::start_room(
        room,
        Reads {
            content,
            population,
        },
    )?;
    for (entry, taken) in population.entries.iter().zip(taken) {
        if let (Some(index), PlmKind::Item(item, bit)) = (taken, entry.kind) {
            // `$84:8495`: the row times the room's width in blocks, plus the column.
            let block = BlockIndex(
                u16::from(entry.y)
                    .wrapping_mul(room.width.0)
                    .wrapping_add(u16::from(entry.x)),
            );
            spawn_item(room, index, block, item, bit);
        }
    }
    Ok(())
}

/// Draw item frame `frame` (`$84:E04F`, `$84:E067`) and wait: the block becomes the special
/// block Samus collides with, with the frame's tile (`$84:A30F`, `$84:A315`: block type
/// `Bh`), its BTS kept, and the draw queues it into the tilemap if it is on screen. The step
/// goes on to the other frame.
fn draw_frame(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    block: BlockIndex,
    slot: ItemGfxSlot,
    frame: ItemFrame,
) -> ItemStep {
    let tile = item_frame_tile(slot, frame);
    if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
        *cell = Block {
            kind: BlockKind::Special(PlmChoice::Common(ITEM_BLOCK.0)),
            tile,
        };
    }
    drawing.block(room, block, tile);
    ItemStep::Shown {
        next: match frame {
            ItemFrame::First => ItemFrame::Second,
            ItemFrame::Second => ItemFrame::First,
        },
    }
}

/// Draw the empty item (`$84:DFA9`): air with tile `FFh`, its BTS kept, for a frame before its
/// delete.
fn draw_empty(drawing: &mut Drawing, room: &mut RoomScoped, block: BlockIndex) -> ItemStep {
    if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
        *cell = Block {
            kind: BlockKind::Air(Bts(cell.kind.bts())),
            tile: EMPTY_ITEM_TILE,
        };
    }
    drawing.block(room, block, EMPTY_ITEM_TILE);
    ItemStep::Emptied
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
    // open: before its first run its pre-instruction isn't set, and its first run's would see
    // the trigger; its block is air until that run draws it, which no slice 0 touch reaches.
    if let ItemStep::Shown { .. } = *step {
        *step = ItemStep::Triggered;
    }
    Ok(())
}

/// The PLM pass (`$84:85B4`): each live PLM from the highest slot down.
///
/// An item's pre-instruction runs, its timer counts down, and when it runs out its list runs;
/// the blocks' places between the items' run as the blocks model's run.
///
/// An item's pickup calls out of the pass into the message box: the slot whose instruction
/// does is returned, the rest of the pass waiting for [`resume`].
///
/// # Errors
///
/// A [`BlocksFault`] where a place or a cell of the blocks is one the model isn't defined on.
// open: the PLMs-enabled flag (`$1C23`).
pub fn process(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    collected: &CollectedItems,
    blocks: &BlockContent,
    population: &PlmPopulation,
) -> Result<Option<SlotIndex<Plm>>, BlocksFault> {
    let end = room.plms.slots.len();
    run_below(drawing, room, collected, blocks, population, end)
}

/// The PLM pass with no video to draw into and no tile tables: its draws queue nothing, for
/// tests of the PLMs' own state.
#[cfg(test)]
pub(crate) fn process_unseen(
    room: &mut RoomScoped,
    collected: &CollectedItems,
    blocks: &BlockContent,
    population: &PlmPopulation,
) -> Result<Option<SlotIndex<Plm>>, BlocksFault> {
    let mut video = crate::state::video::Video::default();
    let tables = TileTables::default();
    process(
        &mut Drawing::new(&mut video, &tables),
        room,
        collected,
        blocks,
        population,
    )
}

/// The rest of the PLM pass from an item's pickup.
///
/// Returned from its message box (`$84:8916`), the item's list goes on to the empty item, whose
/// draw waits a frame (`$84:DFA9`), and the pass goes on below its slot. The slot an item called
/// out of that runs its pickup instruction first: set its place emptied and take its equipment
/// (`$84:8899`, `$84:88F3`, the progress model's `Taken`), then display its message box, the
/// call this returns. Another item's pickup further down moves the pass to its slot.
///
/// # Errors
///
/// A [`BlocksFault`] where a place or a cell of the blocks below is one the model isn't defined
/// on.
// open: the draw tilemap buffer's index (`$1C25`) restarts at 0 here instead of where the
// pass left it; it matters only when another PLM drew earlier in the same pass.
pub fn resume(
    state: &mut State,
    tables: &TileTables,
    blocks: &BlockContent,
    population: &PlmPopulation,
    at: SlotIndex<Plm>,
) -> Result<(SlotIndex<Plm>, Next), BlocksFault> {
    let index = usize::from(at.0);
    let mut drawing = Drawing::new(&mut state.video, tables);
    let room = &mut state.room;
    if let Some(slot) = room.plms.slots.get_mut(index)
        && let Occupancy::Live(PlmBody::Item {
            item,
            collected_bit,
            step,
            ..
        }) = &mut slot.occupancy
    {
        match *step {
            ItemStep::PickingUp => {
                let Pickup { item, message } = pickup(*item);
                let place = *collected_bit;
                *step = ItemStep::InBox;
                run_progress_bridge::take(state, item, place);
                // open: the room's music after 6 seconds (`$82:E118`) is C11's.
                return Ok((at, message_box::display(state, message)));
            }
            ItemStep::InBox => {
                slot.common.cursor.delay = Frames(1);
                let block = slot.common.block;
                *step = ItemStep::Emptied;
                draw_empty(&mut drawing, room, block);
            }
            ItemStep::Spawned
            | ItemStep::Shown { .. }
            | ItemStep::Triggered
            | ItemStep::Emptied => {}
        }
    }
    let next = run_below(
        &mut drawing,
        &mut state.room,
        &state.collected_items,
        blocks,
        population,
        index,
    )?;
    Ok(next.map_or((at, Next::Return), |next| (next, Next::Continue)))
}

/// Slot `index`'s item at `step`, its timer at `delay`.
fn set_item(room: &mut RoomScoped, index: usize, body: PlmBody, delay: Frames) {
    if let Some(slot) = room.plms.slots.get_mut(index) {
        slot.common.cursor.delay = delay;
        slot.occupancy = Occupancy::Live(body);
    }
}

/// Whether slot `index` holds an item's PLM, which the items' code runs.
fn holds_item(room: &RoomScoped, index: usize) -> bool {
    room.plms.slots.get(index).is_some_and(|slot| {
        matches!(
            slot.occupancy,
            Occupancy::Live(PlmBody::Item { .. } | PlmBody::ItemCollision)
        )
    })
}

/// The PLM pass over the slots below `end`, from the highest down: each run of the blocks'
/// places through the model, each item's slot by itself.
fn run_below(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    collected: &CollectedItems,
    blocks: &BlockContent,
    population: &PlmPopulation,
    end: usize,
) -> Result<Option<SlotIndex<Plm>>, BlocksFault> {
    let mut top = end;
    loop {
        let item = (0..top).rev().find(|&index| holds_item(room, index));
        let bottom = item.map_or(0, |index| index.wrapping_add(1));
        world_blocks_bridge::run(
            drawing,
            room,
            Reads {
                content: blocks,
                population,
            },
            bottom..top,
        )?;
        let Some(index) = item else {
            return Ok(None);
        };
        if let Some(callout) = run_item(drawing, room, collected, index) {
            return Ok(Some(callout));
        }
        top = index;
    }
}

/// The pass's turn of the item's PLM in slot `index`: the slot it calls out of the pass from,
/// if its pickup does.
fn run_item(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    collected: &CollectedItems,
    index: usize,
) -> Option<SlotIndex<Plm>> {
    let slot = room.plms.slots.get_mut(index)?;
    let Occupancy::Live(body) = slot.occupancy else {
        return None;
    };
    // `$84:DF89`: triggered, the pre-instruction goes to the link with a timer of 1.
    if let PlmBody::Item {
        step: ItemStep::Triggered,
        ..
    } = body
    {
        slot.common.cursor.delay = Frames(1);
    }
    // `$84:85E0`: the timer counts down.
    slot.common.cursor.delay = Frames(slot.common.cursor.delay.0.wrapping_sub(1));
    if slot.common.cursor.delay != Frames(0) {
        return None;
    }
    let block = slot.common.block;
    match body {
        // `$84:AAE3`: delete.
        PlmBody::ItemCollision => slot.occupancy = Occupancy::Free,
        PlmBody::Item {
            item,
            collected_bit,
            step,
            slot: gfx_slot,
        } => match step {
            // The list's start (`$84:E3EF`): load its GFX, then the empty item if it is
            // collected, else its link and pre-instruction and frame 0's draw.
            ItemStep::Spawned => {
                let gfx_slot = item_gfx::load(&mut room.item_gfx, item);
                let (step, delay) = if run_progress_bridge::is_emptied(collected, collected_bit) {
                    (draw_empty(drawing, room, block), Frames(1))
                } else {
                    (
                        draw_frame(drawing, room, block, gfx_slot, ItemFrame::First),
                        ITEM_FRAME,
                    )
                };
                let body = PlmBody::Item {
                    item,
                    collected_bit,
                    step,
                    slot: gfx_slot,
                };
                set_item(room, index, body, delay);
            }
            // The next frame's draw, then its wait (`$84:E407`, `$84:E40B`'s go-to back).
            ItemStep::Shown { next } => {
                let step = draw_frame(drawing, room, block, gfx_slot, next);
                let body = PlmBody::Item {
                    item,
                    collected_bit,
                    step,
                    slot: gfx_slot,
                };
                set_item(room, index, body, ITEM_FRAME);
            }
            // `$84:E40F`: the pickup calls out, and [`resume`] sets the room argument's
            // place emptied (`$84:8899`) with the pickup instruction: nothing reads it
            // between them.
            // open: clearing the music queue and queueing the fanfare (`$84:8BDD`) is C11's.
            ItemStep::Triggered => {
                slot.occupancy = Occupancy::Live(PlmBody::Item {
                    item,
                    collected_bit,
                    step: ItemStep::PickingUp,
                    slot: gfx_slot,
                });
                return Some(slot_index(index));
            }
            // `$84:DFAD`: delete.
            ItemStep::Emptied => slot.occupancy = Occupancy::Free,
            // Only [`resume`] runs these, the pass waiting on their message box.
            ItemStep::PickingUp | ItemStep::InBox => {}
        },
        PlmBody::Scroll(_) | PlmBody::ScrollTouch | PlmBody::Held | PlmBody::Reaction(_) => {}
    }
    None
}

#[cfg(test)]
mod tests;
