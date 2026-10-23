//! Enemy-projectile terrain movement and slope response.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::state::SlotField as F;
use crate::game::sm_94::{square_slope_occupied, square_slope_single_block};

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

// Every production caller creates this context locally; it cannot alias the
// projectile owner or external collision scratch.
struct Collision {
    direction: u16,
    position: u16,
    remaining: u16,
    x_radius: u16,
    y_radius: u16,
    span: u16,
    edge: u16,
}
impl Axis {
    fn position_field(self) -> F {
        match self {
            Self::Horizontal => F::X,
            Self::Vertical => F::Y,
        }
    }
    fn fraction_field(self) -> F {
        match self {
            Self::Horizontal => F::XFraction,
            Self::Vertical => F::YFraction,
        }
    }
    fn velocity_field(self) -> F {
        match self {
            Self::Horizontal => F::VelocityX,
            Self::Vertical => F::VelocityY,
        }
    }
    fn radius(self, c: &Collision) -> u16 {
        match self {
            Self::Horizontal => c.x_radius,
            Self::Vertical => c.y_radius,
        }
    }
    fn perpendicular_radius(self, c: &Collision) -> u16 {
        match self {
            Self::Horizontal => c.y_radius,
            Self::Vertical => c.x_radius,
        }
    }
    fn position(self, handle: u16, ctx: &Execution<'_>) -> u16 {
        ctx.slot(self.position_field(), (handle >> 1) as isize)
    }
    fn perpendicular_position(self, handle: u16, ctx: &Execution<'_>) -> u16 {
        ctx.slot(
            match self {
                Self::Horizontal => F::Y,
                Self::Vertical => F::X,
            },
            (handle >> 1) as isize,
        )
    }
}

fn extend(axis: Axis, ctx: &mut Execution<'_>) -> u8 {
    if ctx.block_bts(ctx.current_block()) != 0 {
        let previous = ctx.current_block();
        let delta = i32::from(ctx.block_bts(ctx.current_block()) as i8);
        let delta = match axis {
            Axis::Horizontal => delta,
            Axis::Vertical => delta * i32::from(ctx.room_width_in_blocks()),
        };
        let value = (i32::from(previous) + delta) as u16;
        ctx.set_current_block(value);
        return 0xff;
    }
    0
}

fn square_slope(c: &Collision, slope: u16, block: u16, axis: Axis, ctx: &mut Execution<'_>) -> u8 {
    crate::verify_probe::hit(match axis {
        Axis::Horizontal => "square-slope.eproj.horizontal",
        Axis::Vertical => "square-slope.eproj.vertical",
    });
    let captured = match axis {
        Axis::Horizontal => None,
        Axis::Vertical => Some(ctx.state().selected),
    };
    let orientation = u16::from(ctx.block_bts(block) >> 6);
    let shift = match axis {
        Axis::Horizontal => 3,
        Axis::Vertical => 2,
    };
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ ((c.edge & 8) >> shift));
    let other = match axis {
        Axis::Horizontal => 2,
        Axis::Vertical => 1,
    };
    let handle = captured.unwrap_or_else(|| ctx.state().selected);
    let position = axis.perpendicular_position(handle, ctx);
    let radius = axis.perpendicular_radius(c);
    let occupied = if c.span == 0 {
        square_slope_single_block(index, position, radius, other)
    } else {
        square_slope_occupied(index, c.remaining, c.span, position, radius, other)
    };
    if !occupied {
        return 0;
    }
    let slot = (ctx.state().selected >> 1) as isize;
    ctx.set_slot(axis.fraction_field(), slot, 0);
    let position = if c.direction & 0x8000 != 0 {
        axis.radius(c).wrapping_add(c.edge | 7).wrapping_add(1)
    } else {
        (c.edge & 0xfff8).wrapping_sub(axis.radius(c))
    };
    ctx.set_slot(axis.position_field(), slot, position);
    1
}

fn vertical_slope(c: &Collision, ctx: &mut Execution<'_>) -> u8 {
    let handle = ctx.state().selected;
    let negative = c.direction & 0x8000 != 0;
    let block = ctx.current_block();
    let dividend = ctx.current_block();
    let divisor = ctx.room_width_in_blocks() as u8;
    let column = if divisor == 0 {
        dividend
    } else {
        dividend % u16::from(divisor)
    };
    if ctx.slot(F::X, (handle >> 1) as isize) >> 4 != column {
        return 0;
    }
    let edge_offset = if negative {
        (c.position.wrapping_sub(c.y_radius) & 15) ^ 15
    } else {
        c.y_radius.wrapping_add(c.position).wrapping_sub(1) & 15
    };
    let row = u16::from(ctx.block_bts(block) & 0x1f) * 16;
    let bts = ctx.block_bts(block);
    if (bts & 0x80 != 0) != negative {
        return 0;
    }
    let x = if bts & 0x40 != 0 {
        ctx.slot(F::X, (handle >> 1) as isize) ^ 15
    } else {
        ctx.slot(F::X, (handle >> 1) as isize)
    };
    let index = row.wrapping_add(x & 15);
    // Keep both reads against the original raw-ROM source before any landing store.
    let height = ctx.raw_slope_height(index) & 0x1f;
    let adjustment = i16::from(height)
        .wrapping_sub(edge_offset as i16)
        .wrapping_sub(1);
    let height = ctx.raw_slope_height(index) & 0x1f;
    if i32::from(height) - i32::from(edge_offset) != 1 && adjustment >= 0 {
        return 0;
    }
    let position = if negative {
        c.position.wrapping_sub(adjustment as u16)
    } else {
        c.position.wrapping_add(adjustment as u16)
    };
    ctx.set_slot(F::Y, (handle >> 1) as isize, position);
    ctx.set_slot(
        F::YFraction,
        (handle >> 1) as isize,
        if negative { 0 } else { 0xffff },
    );
    1
}

fn slope(c: &Collision, axis: Axis, ctx: &mut Execution<'_>) -> u8 {
    let kind = u16::from(ctx.block_bts(ctx.current_block())) & 0x1f;
    if kind < 5 {
        square_slope(c, kind, ctx.current_block(), axis, ctx)
    } else {
        match axis {
            Axis::Horizontal => {
                let bts = u16::from(ctx.block_bts(ctx.current_block()));
                ctx.set_current_slope_bts(bts);
                0
            }
            Axis::Vertical => vertical_slope(c, ctx),
        }
    }
}

fn react(c: &Collision, cursor: u16, axis: Axis, ctx: &mut Execution<'_>) -> u8 {
    ctx.set_current_block(cursor >> 1);
    loop {
        let result = match ctx.current_level_block() >> 12 {
            0 | 2 | 3 | 4 | 6 | 7 | 14 => 0,
            1 => slope(c, axis, ctx),
            5 => extend(Axis::Horizontal, ctx),
            8 | 9 | 10 | 11 | 12 | 15 => 1,
            13 => extend(Axis::Vertical, ctx),
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            return result;
        }
    }
}

fn move_against_blocks(handle: u16, axis: Axis, ctx: &mut Execution<'_>) -> u8 {
    let slot = (handle >> 1) as isize;
    let amount = i32::from(ctx.slot(axis.velocity_field(), slot) as i16) << 8;
    let y_radius = u16::from(ctx.slot_byte(F::Radius, slot * 2 + 1));
    let x_radius = u16::from(ctx.slot_byte(F::Radius, slot * 2));
    let (radius, perpendicular_radius) = match axis {
        Axis::Horizontal => (x_radius, y_radius),
        Axis::Vertical => (y_radius, x_radius),
    };
    let first_edge = axis
        .perpendicular_position(handle, ctx)
        .wrapping_sub(perpendicular_radius)
        & 0xfff0;
    let span = perpendicular_radius
        .wrapping_add(axis.perpendicular_position(handle, ctx))
        .wrapping_sub(1)
        .wrapping_sub(first_edge)
        >> 4;
    let horizontal_product = match axis {
        Axis::Horizontal => {
            u16::from((ctx.slot(F::Y, slot).wrapping_sub(y_radius) >> 4) as u8)
                * u16::from(ctx.room_width_in_blocks() as u8)
        }
        Axis::Vertical => 0,
    };
    let position = ((amount as u32).wrapping_add(
        (u32::from(axis.position(handle, ctx)) << 16)
            | u32::from(ctx.slot(axis.fraction_field(), slot)),
    ) >> 16) as u16;
    let fraction = (amount + i32::from(ctx.slot(axis.fraction_field(), slot))) as u16;
    let edge = if amount < 0 {
        position.wrapping_sub(radius)
    } else {
        radius.wrapping_add(position).wrapping_sub(1)
    };
    let mut c = Collision {
        direction: (amount >> 16) as u16,
        position,
        remaining: span,
        x_radius,
        y_radius,
        span,
        edge,
    };
    let mut cursor = match axis {
        Axis::Horizontal => {
            i32::from((2 * (i32::from(horizontal_product) + i32::from(edge >> 4))) as u16)
        }
        Axis::Vertical => {
            let product =
                u16::from((edge >> 4) as u8) * u16::from(ctx.room_width_in_blocks() as u8);
            let column = (ctx.slot(F::X, slot).wrapping_sub(x_radius) >> 4) as i16;
            2 * (i32::from(product) + i32::from(column))
        }
    };
    loop {
        if react(&c, cursor as u16, axis, ctx) & 1 != 0 {
            break;
        }
        if matches!(axis, Axis::Horizontal) {
            cursor = i32::from(
                (cursor as u16).wrapping_add((i32::from(ctx.room_width_in_blocks()) * 2) as u16),
            );
        }
        c.remaining = c.remaining.wrapping_sub(1);
        if c.remaining & 0x8000 != 0 {
            ctx.set_slot(axis.fraction_field(), slot, fraction);
            ctx.set_slot(axis.position_field(), slot, position);
            if amount != 0 {
                crate::verify_probe::hit("eproj.motion.applied");
            }
            return 0;
        }
        if matches!(axis, Axis::Vertical) {
            cursor += 2;
        }
    }
    ctx.set_slot(axis.fraction_field(), slot, 0);
    if amount < 0 {
        let corrected = axis.radius(&c).wrapping_add(c.edge | 15).wrapping_add(1);
        if corrected <= axis.position(handle, ctx) {
            ctx.set_slot(axis.position_field(), slot, corrected);
        }
    } else {
        let corrected = (c.edge & 0xfff0).wrapping_sub(axis.radius(&c));
        if corrected >= axis.position(handle, ctx) {
            ctx.set_slot(axis.position_field(), slot, corrected);
        }
    }
    1
}

pub(super) fn EprojBlockCollisition_Horiz(handle: u16, ctx: &mut Execution<'_>) -> u8 {
    move_against_blocks(handle, Axis::Horizontal, ctx)
}
pub(super) fn EprojBlockCollisition_Vertical(handle: u16, ctx: &mut Execution<'_>) -> u8 {
    move_against_blocks(handle, Axis::Vertical, ctx)
}

pub(super) fn power_bomb(handle: u16, ctx: &mut Execution<'_>) {
    let radius = u16::from(ctx.power_bomb_radius_high());
    if ctx.power_bomb_radius_high() != 0 {
        let vertical_radius = ((i32::from(radius)
            + (i32::from(ctx.power_bomb_radius_high()) & 1)
            + (i32::from(ctx.power_bomb_radius()) >> 9)) as u16)
            >> 1;
        let slot = (handle >> 1) as isize;
        if ctx
            .power_bomb_x()
            .wrapping_sub(ctx.slot(F::X, slot))
            .cast_signed()
            .unsigned_abs()
            < radius
            && ctx
                .power_bomb_y()
                .wrapping_sub(ctx.slot(F::Y, slot))
                .cast_signed()
                .unsigned_abs()
                < vertical_radius
        {
            ctx.set_slot(F::Definition, slot, 0);
            ctx.set_player_speed_divisor(0);
        }
    }
}
pub(super) use power_bomb as EprojPowerBombCollision;
