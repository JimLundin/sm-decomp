//! Enemy projectile presentation selection; rendering stays an explicit edge.
#![forbid(unsafe_code)]
use super::{Execution, SlotField as F};
use crate::types::Point16U;

pub enum Priority {
    Low,
    High,
}
pub(super) fn draw(priority: Priority, ctx: &mut Execution<'_>) {
    let shake = screen_shake(ctx);
    let low = matches!(priority, Priority::Low);
    for slot in (0..18).rev() {
        if ctx.slot(F::Definition, slot) != 0
            && (ctx.slot(F::Properties, slot) & 0x1000 != 0) == low
        {
            draw_one((slot * 2) as u16, shake, ctx);
        }
    }
}
pub(super) fn draw_one(handle: u16, shake: Point16U, ctx: &mut Execution<'_>) {
    let slot = (handle >> 1) as isize;
    let map = ctx.slot(F::Spritemap, slot);
    let tile = u16::from(ctx.slot(F::Graphics, slot) as u8);
    let graphics = ctx.slot(F::Graphics, slot) & 0xff00;
    let x = shake
        .x
        .wrapping_add(ctx.slot(F::X, slot))
        .wrapping_sub(ctx.layer1_x_pos());
    if (i32::from(x) + 128) & 0xfe00 == 0 {
        let y = shake
            .y
            .wrapping_add(ctx.slot(F::Y, slot))
            .wrapping_sub(ctx.layer1_y_pos());
        if y & 0xff00 != 0 {
            if (i32::from(y) + 128) & 0xfe00 == 0 {
                ctx.draw_spritemap(true, 0x8d, map, x, y, tile, graphics);
            }
        } else {
            ctx.draw_spritemap(false, 0x8d, map, x, y, tile, graphics);
        }
    }
}
pub(super) fn screen_shake(ctx: &mut Execution<'_>) -> Point16U {
    if ctx.earthquake_timer() != 0
        && ctx.time_frozen() == 0
        && (i32::from(ctx.earthquake_type()) - 36) & 0x8000 != 0
    {
        let index = isize::try_from(4u16.wrapping_mul(ctx.earthquake_type()) >> 1).unwrap();
        let negative = ctx.earthquake_timer() & 2 != 0;
        let x = ctx
            .image()
            .table_word(crate::rtl::lorom_offset(0x86846b) as usize, index);
        let y = ctx
            .image()
            .table_word(crate::rtl::lorom_offset(0x86846b) as usize, index + 1);
        Point16U {
            x: if negative { x.wrapping_neg() } else { x },
            y: if negative { y.wrapping_neg() } else { y },
        }
    } else {
        Point16U { x: 0, y: 0 }
    }
}
