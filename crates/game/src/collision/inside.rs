//! Samus's block inside handling: the cells her bottom, centre and top are inside.
//!
//! Bank $94 log (<https://patrickjohnston.org/bank/94>), "Samus block inside handling"
//! (`$94:9B60`), which the normal current state handler calls after the Y acceleration
//! (bank $90 log, `$90:E6BC`). It looks up the block at Samus's X and her bottom boundary,
//! then her centre if that is in another row, then her top if the original's chained XOR test
//! lets it (`$94:9BEB`), and runs each block's inside reaction (`$94:9B40`). The extensions
//! stand for the block they point at (`$94:9411`, `$94:9447`). What each cell then does with
//! the point inside it is the blocks model's (`world_blocks_bridge::inside`).

use types::Point;

use super::Room;
use crate::state::room::{BlockIndex, BlockKind, RoomScoped};
use crate::state::samus::Samus;
use crate::world_blocks_bridge::{self, BlocksFault, Reads};
use data::level::{StandInLoop, stand_in_end};

/// The block lookup gives up from this position on, as it does for a negative one
/// (`$94:9C27`, `$94:9C3D`).
const POSITION_LIMIT: u16 = 0x1000;

/// The block at `x`, `y` (`$94:9C1D`): the row's start from the multiplier, which takes the
/// width's low byte, plus the column. `None` where the original gives up (block index
/// `FFFFh`).
fn block_at(room: Room<'_>, x: u16, y: u16) -> Option<u16> {
    if x >= POSITION_LIMIT || y >= POSITION_LIMIT {
        return None;
    }
    Some(super::row_start(y >> 4, room.width).wrapping_add(x >> 4))
}

/// The cell at `x`, `y` for `point`, its extensions followed (`$94:9B8E` and its repeats), if its
/// inside reaction is the blocks': of the inside reaction pointers (`$94:9B40`), special air's
/// alone spawns a PLM (`$94:9B16`).
// open: past the level the original reads the level's length word and whatever follows as the
// block; slice 0 keeps Samus inside her room. Special air with bit 7 spawns the area's PLM
// (`$94:9B29`), which the blocks model holds a place for where it holds only that, and BTS 8 to
// `$B` are the treadmills (`$94:98EA` to `$94:9946`); the rest, like air, set the normal X speed
// table (`$94:98E3`), and spike air its own (`$94:98CC`). The State has no X speed table,
// deceleration multiplier or extra displacement yet, which the handling zeroes first
// (`$94:9B63`).
//
// The room's loading refuses a chain that loops, but a reaction's show can close one as the room
// runs, so the follow is bounded and such a chain is a fault.
fn point(
    room: &RoomScoped,
    at: Point,
    x: u16,
    y: u16,
) -> Result<Option<(BlockIndex, Point)>, BlocksFault> {
    let view = Room {
        blocks: &room.blocks,
        width: room.width,
        slopes: None,
        doors: &[],
    };
    let Some(index) = block_at(view, x, y) else {
        return Ok(None);
    };
    let (block, kind) = stand_in_end(room.blocks.len(), room.width.0, |at| view.kind(at), index)
        .map_err(|StandInLoop(block)| BlocksFault::StandInLoop(BlockIndex(block)))?;
    // Past the level she meets air (above), which the blocks are never asked of.
    let special = matches!(kind, BlockKind::SpecialAir(_));
    Ok((special && usize::from(block) < room.blocks.len()).then_some((BlockIndex(block), at)))
}

/// Samus block inside handling (`$94:9B60`): her bottom's cell, her centre's if it is in
/// another row than her bottom, and her top's if the chained XOR test passes, each to the
/// blocks model in that order.
///
/// # Errors
///
/// A [`BlocksFault`] where a cell is one the room's loading would refuse, or a chain of stand-ins
/// loops.
pub fn handle(room: &mut RoomScoped, samus: &Samus, reads: Reads<'_>) -> Result<(), BlocksFault> {
    let x = samus.motion.position.x.pixel();
    let y = samus.motion.position.y.pixel();
    let bottom = y.wrapping_add(samus.y_radius).wrapping_sub(1);
    let mut points = Vec::new();
    points.extend(point(room, Point::Bottom, x, bottom)?);
    if (y ^ bottom) & 0xfff0 != 0 {
        points.extend(point(room, Point::Centre, x, y)?);
    }
    let top = y.wrapping_sub(samus.y_radius);
    // `$94:9BE3`: the second test XORs her centre into the first's result, not her top.
    let rows = (top ^ bottom) & 0xfff0;
    if rows != 0 && (rows ^ y) & 0xfff0 != 0 {
        points.extend(point(room, Point::Top, x, top)?);
    }
    world_blocks_bridge::inside(room, reads, &points)
}

#[cfg(test)]
mod tests;
