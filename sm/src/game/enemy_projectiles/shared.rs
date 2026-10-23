//! Shared enemy-projectile rules, with ordered writes and explicit effects.
#![forbid(unsafe_code)]
use super::access::{enemy, read_member, read_member_byte, write_member, write_member_byte};
use super::host::{EnemyMember as F, Host, NativeRecord};
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session::{Point as Point16U, Vector32 as Point32};

fn move_axis(handle: u16, fraction: F, whole: F, velocity: F, ctx: &mut impl Host) -> u16 {
    let index = (handle >> 1) as isize;
    let speed = read_member(ctx, velocity, index);
    let byte_index = isize::try_from(handle).unwrap() + 1;
    let (fraction_value, carry) =
        read_member_byte(ctx, fraction, byte_index).overflowing_add(speed as u8);
    write_member_byte(ctx, fraction, byte_index, fraction_value);
    // Reread whole after the fraction write: broad/odd handles may overlap.
    let result = read_member(ctx, whole, index)
        .wrapping_add_signed(((speed as i16) >> 8) + i16::from(carry));
    write_member(ctx, whole, index, result);
    if speed != 0 {
        ctx.probe("eproj.motion.applied");
    }
    result
}
pub(super) fn MoveEprojWithVelocityX(k: u16, ctx: &mut impl Host) -> u16 {
    move_axis(k, F::XFraction, F::X, F::VelocityX, ctx)
}
pub(super) fn MoveEprojWithVelocityY(k: u16, ctx: &mut impl Host) -> u16 {
    move_axis(k, F::YFraction, F::Y, F::VelocityY, ctx)
}
pub(super) fn MoveEprojWithVelocity(k: u16, ctx: &mut impl Host) -> u16 {
    MoveEprojWithVelocityX(k, ctx);
    MoveEprojWithVelocityY(k, ctx)
}
pub(super) fn SetAreaDependentEprojPropertiesEx(table: u16, j: u16, ctx: &mut impl Host) {
    let source = ctx.native_record(0x86, table);
    let index = if ctx.session().room.area == 2 {
        1
    } else if ctx.session().room.area == 5 {
        2
    } else {
        0
    };
    let value = source.word_le(index);
    enemy!(ctx, (j >> 1) as isize, properties = value);
}
pub(super) fn SetAreaDependentEprojProperties(j: u16, ctx: &mut impl Host) {
    SetAreaDependentEprojPropertiesEx(
        crate::game::constants::bank_86::addr_kRidleysFireball_Tab0 as u16,
        j,
        ctx,
    );
}
pub(super) fn SetEprojSpawnPoint(k: u16, ctx: &mut impl Host) {
    let x = enemy!(ctx, (k >> 1) as isize, x);
    let y = enemy!(ctx, (k >> 1) as isize, y);
    ctx.session_mut().enemy_projectiles.spawn.point = crate::game::session::Point { x, y };
}
pub(super) fn sub_86A301(j: u16, ctx: &mut impl Host) {
    let index = (j >> 1) as isize;
    for field in [
        F::XFraction,
        F::YFraction,
        F::VelocityX,
        F::VelocityY,
        F::Graphics,
    ] {
        write_member(ctx, field, index, 0);
    }
    let x = ctx.session().player.x;
    enemy!(ctx, index, x = x);
}
pub(super) fn sub_86B279(k: u16, angle: u16, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    enemy!(
        ctx,
        index,
        velocity_x = (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle) + 64])) as u16
    );
    enemy!(
        ctx,
        index,
        velocity_y = (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle)])) as u16
    );
}
pub(super) fn Eproj_InitXYVelRandom(j: u16, table: u16, point: Point16U, ctx: &mut impl Host) {
    let source = ctx.native_record(0x86, table);
    let index = (j >> 1) as isize;
    enemy!(ctx, index, program = source.word_le(0));
    enemy!(ctx, index, x = source.word_le(1).wrapping_add(point.x));
    let speed = source.word_le(2);
    let random = ctx.next_random() as u8;
    enemy!(
        ctx,
        index,
        velocity_x = speed.wrapping_add(u16::from(random)).wrapping_sub(128)
    );
    enemy!(ctx, index, y = source.word_le(3).wrapping_add(point.y));
    let speed = source.word_le(4);
    let random = ctx.next_random() as u8;
    enemy!(
        ctx,
        index,
        velocity_y = speed.wrapping_add(u16::from(random)).wrapping_sub(128)
    );
}
fn set_random_velocity(k: u16, field: F, minimum: u8, ctx: &mut impl Host) {
    let random = ctx.next_random() as u8;
    write_member(
        ctx,
        field,
        (k >> 1) as isize,
        u16::from(random.max(minimum)),
    );
}
pub(super) fn Eproj_SetXvelRandom(k: u16, ctx: &mut impl Host) {
    set_random_velocity(k, F::VelocityX, 32, ctx);
}
pub(super) fn Eproj_SetYvelRandom(k: u16, ctx: &mut impl Host) {
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
pub(super) fn ConvertAngleToXy(angle: u16, amplitude: u16, ctx: &mut impl Host) -> Point32 {
    ctx.convert_angle_to_xy(angle, amplitude)
}
pub(super) fn Eproj_AngleToSamus(j: u16, x: u16, y: u16, ctx: &mut impl Host) {
    let index = (j >> 1) as isize;
    enemy!(ctx, index, x = x);
    enemy!(ctx, index, y = y);
    let x = ctx.session().player.x.wrapping_sub(enemy!(ctx, index, x));
    let y = ctx.session().player.y.wrapping_sub(enemy!(ctx, index, y));
    let angle = 64u16.wrapping_sub(ctx.calculate_angle_from_xy(x, y)) as u8 as u16;
    enemy!(ctx, index, angle = angle);
    let point = ctx.convert_angle_to_xy(angle, ctx.session().enemy_projectiles.spawn.primary);
    enemy!(ctx, index, velocity_x = (point.x >> 16) as u16);
    enemy!(ctx, index, behavior_a = point.x as u16);
    enemy!(ctx, index, velocity_y = (point.y >> 16) as u16);
    enemy!(ctx, index, behavior_b = point.y as u16);
}

pub(super) fn SpawnEprojWithRoomGfx(definition: u16, parameter: u16, ctx: &mut impl Host) -> u16 {
    super::lifecycle::spawn_room(ctx, definition, parameter)
}
pub(super) fn SpawnEprojWithGfx(
    parameter: u16,
    enemy: u16,
    definition: u16,
    ctx: &mut impl Host,
) -> u16 {
    super::lifecycle::spawn_from_enemy(ctx, parameter, enemy, definition)
}
