//! Shared enemy-projectile rules, with ordered writes and explicit effects.
#![forbid(unsafe_code)]
use super::{Execution, SlotField as F};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::types::{Point16U, Point32};

fn move_axis(handle: u16, fraction: F, whole: F, velocity: F, ctx: &mut Execution<'_>) -> u16 {
    let index = (handle >> 1) as isize;
    let speed = ctx.slot(velocity, index);
    let byte_index = isize::try_from(handle).unwrap() + 1;
    let (fraction_value, carry) = ctx
        .slot_byte(fraction, byte_index)
        .overflowing_add(speed as u8);
    ctx.set_slot_byte(fraction, byte_index, fraction_value);
    // Reread whole after the fraction write: broad/odd handles may overlap.
    let result = ctx
        .slot(whole, index)
        .wrapping_add_signed(((speed as i16) >> 8) + i16::from(carry));
    ctx.set_slot(whole, index, result);
    if speed != 0 {
        crate::verify_probe::hit("eproj.motion.applied");
    }
    result
}
pub(super) fn MoveEprojWithVelocityX(k: u16, ctx: &mut Execution<'_>) -> u16 {
    move_axis(k, F::XFraction, F::X, F::VelocityX, ctx)
}
pub(super) fn MoveEprojWithVelocityY(k: u16, ctx: &mut Execution<'_>) -> u16 {
    move_axis(k, F::YFraction, F::Y, F::VelocityY, ctx)
}
pub(super) fn MoveEprojWithVelocity(k: u16, ctx: &mut Execution<'_>) -> u16 {
    MoveEprojWithVelocityX(k, ctx);
    MoveEprojWithVelocityY(k, ctx)
}
pub(super) fn SetAreaDependentEprojPropertiesEx(table: u16, j: u16, ctx: &mut Execution<'_>) {
    let source = ctx.raw_record(0x86, table);
    let index = if ctx.area() == 2 {
        1
    } else if ctx.area() == 5 {
        2
    } else {
        0
    };
    let value = source.word(index);
    ctx.set_slot(F::Properties, (j >> 1) as isize, value);
}
pub(super) fn SetAreaDependentEprojProperties(j: u16, ctx: &mut Execution<'_>) {
    SetAreaDependentEprojPropertiesEx(
        crate::game::constants::bank_86::addr_kRidleysFireball_Tab0 as u16,
        j,
        ctx,
    );
}
pub(super) fn SetEprojSpawnPoint(k: u16, ctx: &mut Execution<'_>) {
    let x = ctx.slot(F::X, (k >> 1) as isize);
    let y = ctx.slot(F::Y, (k >> 1) as isize);
    ctx.state_mut().spawn.point = super::state::Point { x, y };
}
pub(super) fn sub_86A301(j: u16, ctx: &mut Execution<'_>) {
    let index = (j >> 1) as isize;
    for field in [
        F::XFraction,
        F::YFraction,
        F::VelocityX,
        F::VelocityY,
        F::Graphics,
    ] {
        ctx.set_slot(field, index, 0);
    }
    let x = ctx.player_x();
    ctx.set_slot(F::X, index, x);
}
pub(super) fn sub_86B279(k: u16, angle: u16, ctx: &mut Execution<'_>) {
    let index = (k >> 1) as isize;
    ctx.set_slot(
        F::VelocityX,
        index,
        (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle) + 64])) as u16,
    );
    ctx.set_slot(
        F::VelocityY,
        index,
        (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle)])) as u16,
    );
}
pub(super) fn Eproj_InitXYVelRandom(j: u16, table: u16, point: Point16U, ctx: &mut Execution<'_>) {
    let source = ctx.raw_record(0x86, table);
    let index = (j >> 1) as isize;
    ctx.set_slot(F::Program, index, source.word(0));
    ctx.set_slot(F::X, index, source.word(1).wrapping_add(point.x));
    let speed = source.word(2);
    let random = ctx.next_random() as u8;
    ctx.set_slot(
        F::VelocityX,
        index,
        speed.wrapping_add(u16::from(random)).wrapping_sub(128),
    );
    ctx.set_slot(F::Y, index, source.word(3).wrapping_add(point.y));
    let speed = source.word(4);
    let random = ctx.next_random() as u8;
    ctx.set_slot(
        F::VelocityY,
        index,
        speed.wrapping_add(u16::from(random)).wrapping_sub(128),
    );
}
fn set_random_velocity(k: u16, field: F, minimum: u8, ctx: &mut Execution<'_>) {
    let random = ctx.next_random() as u8;
    ctx.set_slot(field, (k >> 1) as isize, u16::from(random.max(minimum)));
}
pub(super) fn Eproj_SetXvelRandom(k: u16, ctx: &mut Execution<'_>) {
    set_random_velocity(k, F::VelocityX, 32, ctx);
}
pub(super) fn Eproj_SetYvelRandom(k: u16, ctx: &mut Execution<'_>) {
    set_random_velocity(k, F::VelocityY, 128, ctx);
}
fn scale_sine_word(byte_offset: u16, amplitude: u16) -> u16 {
    let at = 128 + usize::from(byte_offset);
    let byte = |at: usize| kSinCosTable8bit_Sext[at / 2].to_le_bytes()[at % 2];
    let sample = u16::from_le_bytes([byte(at), byte(at + 1)]);
    ((u32::from(sample) * u32::from(amplitude as u8)) >> 8) as u16
}
pub(super) fn Eproj_PhantomFireballs_Func1(j: u16, amplitude: u16) -> Point16U {
    let x = if (i32::from(j) - 128) & 0x8000 != 0 {
        scale_sine_word((2 * i32::from(j)) as u16, amplitude)
    } else {
        scale_sine_word((2 * (i32::from(j) + 128) as u8 as i32) as u16, amplitude).wrapping_neg()
    };
    let angle = j.wrapping_sub(64) as u8;
    let y = if angle < 128 {
        scale_sine_word(2 * u16::from(angle), amplitude)
    } else {
        scale_sine_word(2 * u16::from(angle.wrapping_add(128)), amplitude).wrapping_neg()
    };
    Point16U { x, y }
}
pub(super) fn ConvertAngleToXy(angle: u16, amplitude: u16, ctx: &mut Execution<'_>) -> Point32 {
    ctx.convert_angle_to_xy(angle, amplitude)
}
pub(super) fn Eproj_AngleToSamus(j: u16, x: u16, y: u16, ctx: &mut Execution<'_>) {
    let index = (j >> 1) as isize;
    ctx.set_slot(F::X, index, x);
    ctx.set_slot(F::Y, index, y);
    let x = ctx.player_x().wrapping_sub(ctx.slot(F::X, index));
    let y = ctx.player_y().wrapping_sub(ctx.slot(F::Y, index));
    let angle = 64u16.wrapping_sub(ctx.calculate_angle_from_xy(x, y)) as u8 as u16;
    ctx.set_slot(F::Angle, index, angle);
    let point = ctx.convert_angle_to_xy(angle, ctx.state().spawn.primary);
    ctx.set_slot(F::VelocityX, index, (point.x >> 16) as u16);
    ctx.set_slot(F::BehaviorA, index, point.x as u16);
    ctx.set_slot(F::VelocityY, index, (point.y >> 16) as u16);
    ctx.set_slot(F::BehaviorB, index, point.y as u16);
}

pub(super) fn SpawnEprojWithRoomGfx(
    definition: u16,
    parameter: u16,
    ctx: &mut Execution<'_>,
) -> u16 {
    super::lifecycle::spawn_room(ctx, definition, parameter)
}
pub(super) fn SpawnEprojWithGfx(
    parameter: u16,
    enemy: u16,
    definition: u16,
    ctx: &mut Execution<'_>,
) -> u16 {
    super::lifecycle::spawn_from_enemy(ctx, parameter, enemy, definition)
}
