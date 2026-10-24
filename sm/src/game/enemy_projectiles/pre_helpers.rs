//! Pure pre-action operations. All indexes retain the caller's byte/word units.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte, read_member, write_member};
use super::host::{EnemyMember as SlotField, Host, NativeRecord};
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};

use super::*;

pub(super) fn start_instruction(index: isize, program: u16, ctx: &mut impl Host) {
    enemy!(ctx, index, program = program);
    enemy!(ctx, index, delay = 1);
}

pub(super) fn set_property_bits(index: isize, bits: u16, ctx: &mut impl Host) {
    let value = enemy!(ctx, index, properties) | bits;
    enemy!(ctx, index, properties = value);
}

/// The old turret write uses unrounded k, unlike ordinary slot accesses.
pub(super) fn clear_unrounded_definition(handle: u16, ctx: &mut impl Host) {
    enemy_byte!(ctx, handle as isize, definition = 0);
    enemy_byte!(ctx, handle as isize + 1, definition = 0);
}

pub(super) fn integrate_shard(
    index: isize,
    x_high: SlotField,
    x_low: SlotField,
    ctx: &mut impl Host,
) {
    let amount = ((enemy!(ctx, index, velocity_x) as i16 as i32) << 8) as u32;
    add_split(x_high, x_low, index, amount, ctx);
    let amount = ((enemy!(ctx, index, velocity_y) as i16 as i32) << 8) as u32;
    add_split(SlotField::Y, SlotField::YFraction, index, amount, ctx);
}

/// Preserve the original short circuit: an X rejection never writes Y.
pub(super) fn move_phantoon_orbit(
    index: isize,
    angle: u16,
    y_position: &mut u16,
    ctx: &mut impl Host,
) {
    enemy!(ctx, index, behavior_a = angle);
    let point = Eproj_PhantomFireballs_Func1(angle, enemy!(ctx, index, velocity_y));
    let negative_x =
        ((point.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as i16) < 0;
    let x = (point.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as i16;
    let value = (point.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as u16;
    enemy!(ctx, index, x = value);
    if negative_x
        || x as i32 - 256i32 & 0x8000 == 0
        || {
            let negative_y = ((point.y as i32
                + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32
                + 16) as i16)
                < 0;
            *y_position = (point.y as i32
                + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32
                + 16) as u16;
            enemy!(ctx, index, y = *y_position);
            negative_y
        }
        || *y_position as i32 - 256i32 & 0x8000 == 0
    {
        enemy!(ctx, index, program = 0x97f8);
        enemy!(ctx, index, delay = 1);
    }
}

/// Both reads precede ordered low/high stores; aliases can overlap either lane.
pub(super) fn add_split(
    high: SlotField,
    low: SlotField,
    index: isize,
    amount: u32,
    ctx: &mut impl Host,
) {
    let low_word = read_member(ctx, low, index);
    let high_word = read_member(ctx, high, index);
    let sum = ((u32::from(high_word) << 16) | u32::from(low_word)).wrapping_add(amount);
    write_member(ctx, low, index, sum as u16);
    write_member(ctx, high, index, (sum >> 16) as u16);
    if amount != 0 && matches!(high, SlotField::X | SlotField::Y) {
        ctx.probe("eproj.motion.applied");
    }
}
pub(super) fn lavaquake_delta(index: isize, ctx: &mut impl Host) -> u32 {
    ctx.native_record(0xa0, 0xcbc7)
        .long_le(index.checked_mul(4).unwrap())
}
pub(super) fn spore_delta(index: isize, ctx: &mut impl Host) -> u8 {
    ctx.native_record(0x86, 0xdd6c).byte(index)
}
pub(super) fn turret_instruction(index: isize, ctx: &mut impl Host) -> u16 {
    ctx.native_record(0x86, 0xc040).word_le(index)
}
pub(super) fn turret_direction(cursor: u16, index: isize, ctx: &mut impl Host) -> u8 {
    ctx.native_record(0x86, cursor).byte(index)
}

pub(super) fn angle_to_player(handle: u16, ctx: &mut impl Host) -> u16 {
    let x = ctx
        .session()
        .player
        .x
        .wrapping_sub(enemy!(ctx, (handle >> 1) as isize, x));
    let y = ctx
        .session()
        .player
        .y
        .wrapping_sub(enemy!(ctx, (handle >> 1) as isize, y));
    ctx.calculate_angle_from_xy(x, y)
}
