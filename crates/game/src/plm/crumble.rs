//! The 1x1 shot block (`$84:D074`): shooting a shootable block with BTS 4 spawns it, and its
//! instruction list (`$84:CBB7`) crumbles the block to air (bank $84 log).

use data::level::{Block, BlockKind, Bts, PlmChoice, Tile, TileNumber};

use crate::state::room::{BlockIndex, Crumble, PlmBody, RoomScoped};
use crate::state::slots::{Frames, Occupancy};

use super::draw::Drawing;
use super::{fill, free_slot};

/// One instruction of the shot block's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instruction {
    /// Queue the block crumbled sound, `Ah` in library 2 (`$84:8C7C`).
    // open: the sound queues join with the sound domain (C11).
    QueueCrumbledSound,
    /// Draw the block as air with `tile`, its BTS kept, then wait `frames` (the 1x1 air
    /// crumble draws, `$84:A345` on).
    DrawAir {
        /// How long the frame shows.
        frames: Frames,
        /// The tile it draws.
        tile: Tile,
    },
    /// Free the slot (`$84:86BC`).
    Delete,
}

/// Air crumble tile `number`, unflipped.
const fn crumble_tile(number: u16) -> Tile {
    Tile {
        number: TileNumber(number),
        x_flip: false,
        y_flip: false,
    }
}

/// The list `$84:CBB7`: the sound, three crumble frames for 4 frames each, the empty block
/// for 1, then the delete.
const LIST: [Instruction; 6] = [
    Instruction::QueueCrumbledSound,
    Instruction::DrawAir {
        frames: Frames(4),
        tile: crumble_tile(0x53),
    },
    Instruction::DrawAir {
        frames: Frames(4),
        tile: crumble_tile(0x54),
    },
    Instruction::DrawAir {
        frames: Frames(4),
        tile: crumble_tile(0x55),
    },
    Instruction::DrawAir {
        frames: Frames(1),
        tile: crumble_tile(0xff),
    },
    Instruction::Delete,
];

/// The instruction a place in the crumble runs next.
const fn next_instruction(crumble: Crumble) -> usize {
    match crumble {
        Crumble::Spawned => 0,
        Crumble::First => 2,
        Crumble::Second => 3,
        Crumble::Third => 4,
        Crumble::Cleared => 5,
    }
}

/// The place in the crumble whose next instruction is `index`: past a draw, the list waits
/// only there.
const fn place(index: usize) -> Option<Crumble> {
    match index {
        2 => Some(Crumble::First),
        3 => Some(Crumble::Second),
        4 => Some(Crumble::Third),
        5 => Some(Crumble::Cleared),
        _ => None,
    }
}

/// The shot block's BTS in the shootable PLM table (`$94:9EA6`): entry 4.
const SHOT_BLOCK_1X1: PlmChoice = PlmChoice::Common(4);

/// The block type bit the setup keeps (`AND #$8FFF`): the shootable type C becomes the solid
/// type 8, so shots and Samus stop at the block while it crumbles.
const TYPE_KEPT: u8 = 0x8;

/// Why a shot block reaction can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotFault {
    /// The shot block isn't a shootable block whose PLM is modelled: its reaction is another
    /// kind's.
    // open: the other shootable PLMs (`$94:9EA6`, `$94:9FC6`) join as the rooms that hold
    // them do.
    Unmodelled(BlockKind),
}

/// A projectile hits shootable block `block` (`$94:9E73`).
///
/// A block with BTS 4 spawns the 1x1 shot block there (`$84:84E7`), whose setup clears the
/// block's type's low three bits (`$84:B3C1`), so the shootable block turns solid while it
/// crumbles. Its timer is 1, so the PLM pass that frame runs its list's start. With no slot
/// free nothing spawns and no setup runs.
///
/// # Errors
///
/// [`ShotFault`] for a block whose reaction isn't the 1x1 shot block's.
// open: the projectile domain that calls this (bank $90 movement, bank $94 block collision)
// isn't built; until it is, the engine never shoots a block.
pub fn shoot_block(room: &mut RoomScoped, block: BlockIndex) -> Result<(), ShotFault> {
    let kind = room
        .blocks
        .get(usize::from(block.0))
        .map(|cell| cell.kind)
        .unwrap_or_default();
    if kind != BlockKind::Shootable(SHOT_BLOCK_1X1) {
        return Err(ShotFault::Unmodelled(kind));
    }
    let Some(slot) = free_slot(room).and_then(|index| room.plms.slots.get_mut(index)) else {
        return Ok(());
    };
    fill(
        slot,
        block,
        Occupancy::Live(PlmBody::ShotBlock {
            crumble: Crumble::Spawned,
        }),
    );
    if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
        cell.kind = BlockKind::new(cell.kind.nybble() & TYPE_KEPT, cell.kind.bts());
    }
    Ok(())
}

/// The shot block in slot `index`, on `block` at `crumble`, runs its list as its timer runs
/// out: instructions from its place until a draw waits or the delete frees the slot.
pub(super) fn run(
    drawing: &mut Drawing,
    room: &mut RoomScoped,
    index: usize,
    block: BlockIndex,
    crumble: Crumble,
) {
    let start = next_instruction(crumble);
    for (at, instruction) in LIST.iter().enumerate().skip(start) {
        match *instruction {
            Instruction::QueueCrumbledSound => {}
            Instruction::DrawAir { frames, tile } => {
                if let Some(cell) = room.blocks.get_mut(usize::from(block.0)) {
                    *cell = Block {
                        kind: BlockKind::Air(Bts(cell.kind.bts())),
                        tile,
                    };
                }
                drawing.block(room, block, tile);
                if let Some(slot) = room.plms.slots.get_mut(index) {
                    slot.common.cursor.delay = frames;
                    if let Some(crumble) = place(at.wrapping_add(1)) {
                        slot.occupancy = Occupancy::Live(PlmBody::ShotBlock { crumble });
                    }
                }
                return;
            }
            Instruction::Delete => {
                if let Some(slot) = room.plms.slots.get_mut(index) {
                    slot.occupancy = Occupancy::Free;
                }
                return;
            }
        }
    }
}
