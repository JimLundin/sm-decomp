//! Complete attached stem/segment interactions over owned projectile state.
#![forbid(unsafe_code)]
use super::pre_helpers::store;
use super::{Execution, SlotField};
#[derive(Clone, Copy)]
enum StemAxis {
    X,
    Y,
}

impl StemAxis {
    fn field(self) -> SlotField {
        match self {
            Self::X => SlotField::X,
            Self::Y => SlotField::Y,
        }
    }
    fn fixed_anchor(self) -> u16 {
        match self {
            Self::X => 128,
            Self::Y => 560,
        }
    }
    fn head(self, ctx: &Execution<'_>) -> u16 {
        match self {
            Self::X => ctx.enemy_x(0),
            Self::Y => ctx.enemy_y(0).wrapping_sub(40),
        }
    }
    fn origin(self, ctx: &Execution<'_>) -> u16 {
        match self {
            Self::X => ctx.spore_stem_anchor_x(),
            Self::Y => ctx.spore_stem_anchor_y(),
        }
    }
    fn write_control(self, part: u8, value: u16, ctx: &mut Execution<'_>) {
        match self {
            Self::X => ctx.set_spore_stem_control_x(part, value),
            Self::Y => ctx.set_spore_stem_control_y(part, value),
        }
    }
}

pub(super) fn update_spore_stem(ctx: &mut Execution<'_>) {
    update_stem_axis(StemAxis::X, ctx);
    update_stem_axis(StemAxis::Y, ctx);
}

fn update_stem_axis(axis: StemAxis, ctx: &mut Execution<'_>) {
    // Initial delta reads head before origin; the negative half-distance rereads
    // origin before head. Keep both observations even when values usually agree.
    let delta = axis.head(ctx).wrapping_sub(axis.origin(ctx));
    let negative = delta & 0x8000 != 0;
    let (half, quarter) = if negative {
        let half = axis.origin(ctx).wrapping_sub(axis.head(ctx)) >> 1;
        (half, delta.wrapping_neg() >> 2)
    } else {
        (delta >> 1, delta >> 2)
    };
    let three_quarters = half.wrapping_add(quarter);
    let offset = |origin: u16, distance: u16| {
        if negative {
            origin.wrapping_sub(distance)
        } else {
            origin.wrapping_add(distance)
        }
    };
    let field = axis.field();
    ctx.set_slot(field, 14, axis.fixed_anchor());
    store(offset(axis.origin(ctx), quarter), field, 15, ctx);
    axis.write_control(0, ctx.slot(field, 15), ctx);
    store(offset(axis.origin(ctx), half), field, 16, ctx);
    axis.write_control(1, ctx.slot(field, 16), ctx);
    let tip = offset(axis.origin(ctx), three_quarters);
    ctx.set_slot(field, 17, tip);
    axis.write_control(2, tip, ctx);
}

pub(super) fn update_maw_segment(part: u8, ctx: &mut Execution<'_>) {
    let enemy = ctx.current_enemy();
    let segment = (ctx.maw_segment_handle(enemy, part) >> 1) as isize;
    if part == 0 {
        store(ctx.maw_anchor_x(enemy), SlotField::X, segment, ctx);
        store(ctx.maw_anchor_y(enemy), SlotField::Y, segment, ctx);
    } else {
        store(
            ctx.maw_segment_offset_x(enemy, part)
                .wrapping_add(ctx.maw_anchor_x(enemy)),
            SlotField::X,
            segment,
            ctx,
        );
        store(
            ctx.maw_segment_offset_y(enemy, part)
                .wrapping_add(ctx.maw_anchor_y(enemy)),
            SlotField::Y,
            segment,
            ctx,
        );
    }
}
pub(super) fn update_maw_palette(handle: u16, ctx: &mut Execution<'_>) {
    let segment = (handle >> 1) as isize;
    let enemy = ctx.current_enemy();
    store(
        ctx.maw_palette(enemy) | ctx.slot(SlotField::Graphics, segment) & 0xf1ff,
        SlotField::Graphics,
        segment,
        ctx,
    );
    if ctx.maw_frozen_timer(enemy) != 0 {
        store(
            ctx.slot(SlotField::Graphics, segment) & 0xf1ff | 0xc00,
            SlotField::Graphics,
            segment,
            ctx,
        );
        let frozen_timer = ctx.maw_frozen_timer(enemy);
        if frozen_timer as i32 - 90i32 & 0x8000 != 0 && (frozen_timer & 2) == 0 {
            store(
                ctx.maw_palette(enemy) | ctx.slot(SlotField::Graphics, segment) & 0xf1ff,
                SlotField::Graphics,
                segment,
                ctx,
            );
        }
    }
}
