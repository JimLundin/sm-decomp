//! Samus's block inside handling: the reactions of the blocks her bottom, centre and top are
//! inside.
//!
//! Bank $94 log (<https://patrickjohnston.org/bank/94>), "Samus block inside handling"
//! (`$94:9B60`), which the normal current state handler calls after the Y acceleration
//! (bank $90 log, `$90:E6BC`). It looks up the block at Samus's X and her bottom boundary,
//! then her centre if that is in another row, then her top if the original's chained XOR test
//! lets it (`$94:9BEB`), and runs each block's inside reaction (`$94:9B40`). The extensions
//! stand for the block they point at (`$94:9411`, `$94:9447`); special air dispatches on its
//! BTS (`$94:9B16`), and BTS `$46`, the scroll PLM trigger, spawns the scroll block touch PLM
//! when the point is her centre (`$94:9956`).

use super::{Room, SCROLL_TRIGGER};
use crate::plm::{self, TouchFault};
use crate::state::room::{BlockIndex, BlockKind, Bts, Offset, RoomScoped};
use crate::state::samus::Samus;

/// The block lookup gives up from this position on, as it does for a negative one
/// (`$94:9C27`, `$94:9C3D`).
const POSITION_LIMIT: u16 = 0x1000;

/// The point of Samus a reaction is for (`$1E73`, set before each lookup).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReactionPoint {
    /// Her bottom boundary.
    Bottom,
    /// Her Y position.
    Centre,
    /// Her top boundary.
    Top,
}

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

/// The inside reaction of the block at `x`, `y` for `point` (`$94:9B8E` and its repeats).
fn react(room: &mut RoomScoped, point: ReactionPoint, x: u16, y: u16) -> Result<(), TouchFault> {
    let view = Room {
        blocks: &room.blocks,
        width: room.width,
        slopes: None,
    };
    // open: past the level the original reads the level's length word and whatever follows
    // as the block; slice 0 keeps Samus inside her room.
    let Some(index) = block_at(view, x, y) else {
        return Ok(());
    };
    let (block, kind) = resolve(view, index);
    match kind {
        BlockKind::SpecialAir(SCROLL_TRIGGER) => {
            if point == ReactionPoint::Centre {
                plm::touch_scroll_block(room, BlockIndex(block))?;
            }
        }
        // open: special air with bit 7 spawns the area's PLM (`$94:9B29`); BTS 8 to `$B` are
        // the treadmills (`$94:98EA` to `$94:9946`); the rest, like air, shootable, unused and
        // bombable air (`$94:97D0`), set the normal X speed table (`$94:98E3`), and spike air
        // its own (`$94:98CC`). The State has no X speed table, deceleration multiplier or
        // extra displacement yet, which the handling zeroes first (`$94:9B63`); slice 0's
        // rooms have no other special air.
        BlockKind::SpecialAir(Bts(_))
        | BlockKind::Air(_)
        | BlockKind::Slope(_)
        | BlockKind::SpikeAir(_)
        | BlockKind::ShootableAir(_)
        | BlockKind::HorizontalExtension(_)
        | BlockKind::UnusedAir(_)
        | BlockKind::BombableAir(_)
        | BlockKind::Solid(_)
        | BlockKind::Door(_)
        | BlockKind::Spike(_)
        | BlockKind::Special(_)
        | BlockKind::Shootable(_)
        | BlockKind::VerticalExtension(_)
        | BlockKind::Grapple(_)
        | BlockKind::Bombable(_) => {}
    }
    Ok(())
}

/// Samus block inside handling (`$94:9B60`): her bottom's block, her centre's if it is in
/// another row than her bottom, and her top's if the chained XOR test passes.
///
/// # Errors
///
/// [`TouchFault`] where a scroll trigger's touch PLM finds no scroll PLM on its block.
pub fn handle(room: &mut RoomScoped, samus: &Samus) -> Result<(), TouchFault> {
    let x = samus.motion.position.x.pixel();
    let y = samus.motion.position.y.pixel();
    let bottom = y.wrapping_add(samus.y_radius).wrapping_sub(1);
    react(room, ReactionPoint::Bottom, x, bottom)?;
    if (y ^ bottom) & 0xfff0 != 0 {
        react(room, ReactionPoint::Centre, x, y)?;
    }
    let top = y.wrapping_sub(samus.y_radius);
    // `$94:9BE3`: the second test XORs her centre into the first's result, not her top.
    let rows = (top ^ bottom) & 0xfff0;
    if rows != 0 && (rows ^ y) & 0xfff0 != 0 {
        react(room, ReactionPoint::Top, x, top)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
