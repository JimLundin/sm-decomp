//! Samus's block inside handling: the cells her bottom, centre and top are inside.
//!
//! Bank $94 log (<https://patrickjohnston.org/bank/94>), "Samus block inside handling"
//! (`$94:9B60`), which the normal current state handler calls after the Y acceleration
//! (bank $90 log, `$90:E6BC`). It looks up the block at Samus's X and her bottom boundary,
//! then her centre if that is in another row, then her top if the original's chained XOR test
//! lets it (`$94:9BEB`), and runs each block's inside reaction (`$94:9B40`). The extensions
//! stand for the block they point at (`$94:9411`, `$94:9447`). What each cell then does with
//! the point inside it is the blocks model's (`blocks_world::inside`).

use clean_types::Point;

use super::Room;
use crate::blocks_world::{self, BlocksFault, Reads};
use crate::state::room::{BlockIndex, BlockKind, Offset, RoomScoped};
use crate::state::samus::Samus;

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

/// The block an inside reaction ends at from `index` and its kind: an extension with a
/// nonzero offset moves to the block it points at and dispatches again.
fn resolve(room: Room<'_>, index: u16) -> (u16, BlockKind) {
    let mut current = index;
    loop {
        let kind = room.kind(current);
        match kind {
            // An offset of 0 acts like air.
            BlockKind::HorizontalExtension(Offset(0))
            | BlockKind::VerticalExtension(Offset(0))
            | BlockKind::Air(_)
            | BlockKind::Slope(_)
            | BlockKind::SpikeAir(_)
            | BlockKind::SpecialAir(_)
            | BlockKind::ShootableAir(_)
            | BlockKind::UnusedAir(_)
            | BlockKind::BombableAir(_)
            | BlockKind::Solid(_)
            | BlockKind::Door(_)
            | BlockKind::Spike(_)
            | BlockKind::Special(_)
            | BlockKind::Shootable(_)
            | BlockKind::Grapple(_)
            | BlockKind::Bombable(_) => return (current, kind),
            BlockKind::HorizontalExtension(Offset(offset)) => {
                current = current.wrapping_add_signed(i16::from(offset));
            }
            BlockKind::VerticalExtension(Offset(offset)) => {
                let rows = i16::from(offset).cast_unsigned();
                current = current.wrapping_add(rows.wrapping_mul(room.width.0));
            }
        }
    }
}

/// The cell at `x`, `y` for `point`, its extensions followed (`$94:9B8E` and its repeats).
// open: past the level the original reads the level's length word and whatever follows as the
// block; slice 0 keeps Samus inside her room. Special air with bit 7 spawns the area's PLM
// (`$94:9B29`), which the blocks model holds a place for where it holds only that, and BTS 8 to
// `$B` are the treadmills (`$94:98EA` to `$94:9946`); the rest, like air, set the normal X speed
// table (`$94:98E3`), and spike air its own (`$94:98CC`). The State has no X speed table,
// deceleration multiplier or extra displacement yet, which the handling zeroes first
// (`$94:9B63`).
fn point(room: &RoomScoped, at: Point, x: u16, y: u16) -> Option<(BlockIndex, Point)> {
    let view = Room {
        blocks: &room.blocks,
        width: room.width,
        slopes: None,
    };
    let index = block_at(view, x, y)?;
    let (block, _) = resolve(view, index);
    Some((BlockIndex(block), at))
}

/// Samus block inside handling (`$94:9B60`): her bottom's cell, her centre's if it is in
/// another row than her bottom, and her top's if the chained XOR test passes, each to the
/// blocks model in that order.
///
/// # Errors
///
/// A [`BlocksFault`] where a cell is one the room's loading would refuse.
pub fn handle(room: &mut RoomScoped, samus: &Samus, reads: Reads<'_>) -> Result<(), BlocksFault> {
    let x = samus.motion.position.x.pixel();
    let y = samus.motion.position.y.pixel();
    let bottom = y.wrapping_add(samus.y_radius).wrapping_sub(1);
    let mut points = Vec::new();
    points.extend(point(room, Point::Bottom, x, bottom));
    if (y ^ bottom) & 0xfff0 != 0 {
        points.extend(point(room, Point::Centre, x, y));
    }
    let top = y.wrapping_sub(samus.y_radius);
    // `$94:9BE3`: the second test XORs her centre into the first's result, not her top.
    let rows = (top ^ bottom) & 0xfff0;
    if rows != 0 && (rows ^ y) & 0xfff0 != 0 {
        points.extend(point(room, Point::Top, x, top));
    }
    blocks_world::inside(room, reads, &points)
}

#[cfg(test)]
mod tests;
