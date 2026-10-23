//! Pure pre-action operations. All indexes retain the caller's byte/word units.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::state::SlotField;
use super::*;

/// Value precedes index at the call site, matching assignment evaluation order.
pub(super) fn store(value: u16, field: SlotField, index: isize, ctx: &mut Execution<'_>) {
    ctx.set_slot(field, index, value);
}

pub(super) fn add_word(
    field: SlotField,
    index: isize,
    amount: u16,
    ctx: &mut Execution<'_>,
) -> u16 {
    let value = ctx.slot(field, index).wrapping_add(amount);
    ctx.set_slot(field, index, value);
    value
}

pub(super) fn start_instruction(index: isize, program: u16, ctx: &mut Execution<'_>) {
    ctx.set_slot(SlotField::Program, index, program);
    ctx.set_slot(SlotField::Delay, index, 1);
}

pub(super) fn set_property_bits(index: isize, bits: u16, ctx: &mut Execution<'_>) {
    let value = ctx.slot(SlotField::Properties, index) | bits;
    ctx.set_slot(SlotField::Properties, index, value);
}

/// The old turret write uses unrounded k, unlike ordinary slot accesses.
pub(super) fn clear_unrounded_definition(handle: u16, ctx: &mut Execution<'_>) {
    ctx.set_slot_byte(SlotField::Definition, handle as isize, 0);
    ctx.set_slot_byte(SlotField::Definition, handle as isize + 1, 0);
}

pub(super) fn integrate_shard(
    index: isize,
    x_high: SlotField,
    x_low: SlotField,
    ctx: &mut Execution<'_>,
) {
    let amount = ((ctx.slot(SlotField::VelocityX, index) as i16 as i32) << 8) as u32;
    ctx.add_split(x_high, x_low, index, amount);
    let amount = ((ctx.slot(SlotField::VelocityY, index) as i16 as i32) << 8) as u32;
    ctx.add_split(SlotField::Y, SlotField::YFraction, index, amount);
}

/// Preserve the original short circuit: an X rejection never writes Y.
pub(super) fn move_phantoon_orbit(
    index: isize,
    angle: u16,
    y_position: &mut u16,
    ctx: &mut Execution<'_>,
) {
    ctx.set_slot(SlotField::BehaviorA, index, angle);
    let point = Eproj_PhantomFireballs_Func1(angle, ctx.slot(SlotField::VelocityY, index));
    let negative_x = ((point.x as i32 + ctx.enemy_x(0) as i32) as i16) < 0;
    let x = (point.x as i32 + ctx.enemy_x(0) as i32) as i16;
    let value = (point.x as i32 + ctx.enemy_x(0) as i32) as u16;
    ctx.set_slot(SlotField::X, index, value);
    if negative_x
        || x as i32 - 256i32 & 0x8000 == 0
        || {
            let negative_y = ((point.y as i32 + ctx.enemy_y(0) as i32 + 16) as i16) < 0;
            *y_position = (point.y as i32 + ctx.enemy_y(0) as i32 + 16) as u16;
            ctx.set_slot(SlotField::Y, index, *y_position);
            negative_y
        }
        || *y_position as i32 - 256i32 & 0x8000 == 0
    {
        ctx.set_slot(SlotField::Program, index, 0x97f8);
        ctx.set_slot(SlotField::Delay, index, 1);
    }
}
