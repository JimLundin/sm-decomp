//! Complete attached stem/segment interactions over owned projectile state.
#![forbid(unsafe_code)]
use super::access::enemy;
use super::host::*;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
#[derive(Clone, Copy)]
enum StemAxis {
    X,
    Y,
}

impl StemAxis {
    fn read(self, slot: isize, ctx: &impl Host) -> u16 {
        match self {
            Self::X => enemy!(ctx, slot, x),
            Self::Y => enemy!(ctx, slot, y),
        }
    }
    fn write(self, slot: isize, value: u16, ctx: &mut impl Host) {
        match self {
            Self::X => enemy!(ctx, slot, x = value),
            Self::Y => enemy!(ctx, slot, y = value),
        }
    }
    fn fixed_anchor(self) -> u16 {
        match self {
            Self::X => 128,
            Self::Y => 560,
        }
    }
    fn head(self, ctx: &impl Host) -> u16 {
        match self {
            Self::X => ctx.resolve_actor(Key::from_legacy(0), Le).x.whole,
            Self::Y => ctx
                .resolve_actor(Key::from_legacy(0), Le)
                .y
                .whole
                .wrapping_sub(40),
        }
    }
    fn origin(self, ctx: &impl Host) -> u16 {
        match self {
            Self::X => ctx.spore_stem_anchor_x(),
            Self::Y => ctx.spore_stem_anchor_y(),
        }
    }
    fn write_control(self, part: u8, value: u16, ctx: &mut impl Host) {
        match self {
            Self::X => ctx.set_spore_stem_control_x(part, value),
            Self::Y => ctx.set_spore_stem_control_y(part, value),
        }
    }
}

pub fn update_spore_stem(ctx: &mut impl Host) {
    update_stem_axis(StemAxis::X, ctx);
    update_stem_axis(StemAxis::Y, ctx);
}

fn update_stem_axis(axis: StemAxis, ctx: &mut impl Host) {
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
    axis.write(14, axis.fixed_anchor(), ctx);
    axis.write(15, offset(axis.origin(ctx), quarter), ctx);
    axis.write_control(0, axis.read(15, ctx), ctx);
    axis.write(16, offset(axis.origin(ctx), half), ctx);
    axis.write_control(1, axis.read(16, ctx), ctx);
    let tip = offset(axis.origin(ctx), three_quarters);
    axis.write(17, tip, ctx);
    axis.write_control(2, tip, ctx);
}

pub fn update_maw_segment(part: u8, ctx: &mut impl Host) {
    let enemy = ctx.inputs().current_enemy;
    let segment = (ctx.maw_segment_handle(enemy, part) >> 1) as isize;
    if part == 0 {
        enemy!(ctx, ctx.maw_anchor_x(enemy) => segment, x);
        enemy!(ctx, ctx.maw_anchor_y(enemy) => segment, y);
    } else {
        {
            let value = ctx
                .maw_segment_offset_x(enemy, part)
                .wrapping_add(ctx.maw_anchor_x(enemy));
            enemy!(ctx, value => segment, x);
        };
        {
            let value = ctx
                .maw_segment_offset_y(enemy, part)
                .wrapping_add(ctx.maw_anchor_y(enemy));
            enemy!(ctx, value => segment, y);
        };
    }
}
pub fn update_maw_palette(handle: u16, ctx: &mut impl Host) {
    let segment = (handle >> 1) as isize;
    let enemy = ctx.inputs().current_enemy;
    enemy!(ctx, ctx.maw_palette(enemy) | enemy!(ctx, segment, graphics) & 0xf1ff => segment, graphics);
    if ctx.resolve_actor(Key::from_legacy(enemy), Le).frozen != 0 {
        enemy!(ctx, enemy!(ctx, segment, graphics) & 0xf1ff | 0xc00 => segment, graphics);
        let frozen_timer = ctx.resolve_actor(Key::from_legacy(enemy), Le).frozen;
        if (frozen_timer as i32 - 90i32) & 0x8000 != 0 && (frozen_timer & 2) == 0 {
            enemy!(ctx, ctx.maw_palette(enemy) | enemy!(ctx, segment, graphics) & 0xf1ff => segment, graphics);
        }
    }
}
