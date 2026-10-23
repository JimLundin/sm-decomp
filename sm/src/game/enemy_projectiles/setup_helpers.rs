#![forbid(unsafe_code)]
use super::SlotField as F;
use super::pre_helpers::store;
use super::shared::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use super::*;

/// Keep live enemy reads interleaved with writes, including broad slot aliases.
pub(super) fn copy_enemy_position(slot: isize, enemy: u16, y_offset: i16, ctx: &mut Execution<'_>) {
    store(ctx.enemy_x(enemy), F::X, slot, ctx);
    store(ctx.enemy_x_fraction(enemy), F::XFraction, slot, ctx);
    store(
        (ctx.enemy_y(enemy) as i32 + y_offset as i32) as u16,
        F::Y,
        slot,
        ctx,
    );
    store(ctx.enemy_y_fraction(enemy), F::YFraction, slot, ctx);
}

pub(super) fn init_robot_laser(
    j: u16,
    y_velocity: u16,
    clear_graphics: bool,
    ctx: &mut Execution<'_>,
) {
    let slot = (j >> 1) as isize;
    store(ctx.enemy_ai_a(ctx.current_enemy()), F::VelocityX, slot, ctx);
    store(y_velocity, F::VelocityY, slot, ctx);
    if clear_graphics {
        store(0, F::Graphics, slot, ctx);
    }
    EprojInit_WreckedShipRobotLaserCommon(ctx.current_enemy(), j, ctx);
}

pub(super) fn init_landing_dust(j: u16, x_offset: i16, ctx: &mut Execution<'_>) {
    let enemy = ctx.current_enemy();
    let slot = (j >> 1) as isize;
    store(ctx.enemy_y(enemy).wrapping_add(48), F::Y, slot, ctx);
    store(
        ctx.enemy_x(enemy).wrapping_add_signed(x_offset),
        F::X,
        slot,
        ctx,
    );
}

pub(super) fn init_statue_piece(j: u16, x: u16, y: u16, ctx: &mut Execution<'_>) {
    let slot = (j >> 1) as isize;
    store(x, F::BehaviorA, slot, ctx);
    store(x, F::X, slot, ctx);
    store(y, F::BehaviorB, slot, ctx);
    store(y, F::Y, slot, ctx);
}

static kEprojInit_MiniKraidSpikes_Tab0: [i16; 3] = [-2, 12, 24];
pub(super) fn init_skree_particle(
    j: u16,
    x_offset: i16,
    x_velocity: i16,
    y_velocity: i16,
    ctx: &mut Execution<'_>,
) {
    let enemy_index = ctx.current_enemy();
    let word_index = usize::from(j >> 1);
    let y = ctx.enemy_y(enemy_index);
    ctx.set_slot(F::Y, word_index as isize, y);
    ctx.set_slot(F::VelocityY, word_index as isize, y_velocity as u16);
    let x = ctx.enemy_x(enemy_index).wrapping_add_signed(x_offset);
    ctx.set_slot(F::X, word_index as isize, x);
    ctx.set_slot(F::VelocityX, word_index as isize, x_velocity as u16);
}

pub(super) fn EprojInit_FireballChild(
    j: u16,
    x: u16,
    y: u16,
    handler: u16,
    ctx: &mut Execution<'_>,
) {
    {
        let parent = ctx.state().spawn.primary;
        let slot = usize::from(j >> 1);
        ctx.set_slot(F::VelocityX, slot as isize, x);
        ctx.set_slot(F::VelocityY, slot as isize, y);
        EprojInit_Common(j, parent, handler, ctx);
    }
}

pub(super) fn EprojInit_MiniKraidSpikes(j: u16, a: u16, ctx: &mut Execution<'_>) {
    {
        let v2: i32 = j as i32 >> 1;
        store(a, F::VelocityX, (v2) as isize, ctx);
        let r18: u16 = kEprojInit_MiniKraidSpikes_Tab0
            [(ctx.mini_kraid_spike_variant(ctx.current_enemy()) >> 1) as usize]
            as u16;
        let enemy_index = ctx.current_enemy();
        store(ctx.enemy_x(enemy_index), F::X, (v2) as isize, ctx);
        store(
            (r18).wrapping_add(ctx.enemy_y(enemy_index)),
            F::Y,
            (v2) as isize,
            ctx,
        );

        store(0, F::YFraction, (v2) as isize, ctx);
        store(0, F::XFraction, (v2) as isize, ctx);
        store(0, F::VelocityY, (v2) as isize, ctx);
    }
}

pub(super) fn EprojInit_KiHunterAcidSpitCommon(k: u16, j: u16, ctx: &mut Execution<'_>) {
    {
        let v2: i32 = j as i32 >> 1;
        store(0, F::VelocityY, (v2) as isize, ctx);
        store(
            (ctx.enemy_y(k) as i32 - 16i32) as u16,
            F::Y,
            (v2) as isize,
            ctx,
        );

        store(0, F::YFraction, (v2) as isize, ctx);
        store(0, F::XFraction, (v2) as isize, ctx);
    }
}

pub(super) fn EprojInit_WreckedShipRobotLaserCommon(k: u16, j: u16, ctx: &mut Execution<'_>) {
    {
        let v2 = k;
        let v3: i32 = j as i32 >> 1;
        store(
            (ctx.enemy_y(v2) as i32 - 16i32) as u16,
            F::Y,
            (v3) as isize,
            ctx,
        );
        store(
            (ctx.enemy_x(v2) as i32
                + (if ctx.slot(F::VelocityX, (v3) as isize) & 0x8000 != 0 {
                    -4i32
                } else {
                    4
                })) as u16,
            F::X,
            (v3) as isize,
            ctx,
        );

        store(0, F::YFraction, (v3) as isize, ctx);
        store(0, F::XFraction, (v3) as isize, ctx);

        if ((ctx.enemy_width(v2) as i32 + ctx.enemy_x(v2) as i32 - ctx.layer1_x_pos() as i32)
            as i16)
            >= 0
            && ((ctx.enemy_x(v2) as i32
                - ctx.enemy_width(v2) as i32
                - 257i32
                - ctx.layer1_x_pos() as i32) as i16)
                < 0
            && ((ctx.enemy_height(v2) as i32 + ctx.enemy_y(v2) as i32 - ctx.layer1_y_pos() as i32)
                as i16)
                >= 0
            && ((ctx.enemy_y(v2) as i32 - ctx.enemy_height(v2) as i32 - 224i32) as i16) < 0
        {
            ctx.queue_sound(2, 6, 0x67);
        }
    }
}

pub(super) fn EprojInit_SpawnedGate_Common(j: u16, v1: u16, ctx: &mut Execution<'_>) {
    {
        let v2: u16 = room_mechanisms::selected(ctx);
        calculate_coordinates(room_mechanisms::selected(ctx), ctx);
        let v3: i32 = j as i32 >> 1;
        store(
            room_mechanisms::block_index(v2, ctx),
            F::BehaviorA,
            (v3) as isize,
            ctx,
        );
        store(
            (16u16).wrapping_mul(room_mechanisms::block_x(ctx)),
            F::X,
            (v3) as isize,
            ctx,
        );
        store(
            (v1 as i32 + 16 * room_mechanisms::block_y(ctx) as i32) as u16,
            F::Y,
            (v3) as isize,
            ctx,
        );
    }
}

fn EprojInit_Common(j: u16, k: u16, handler: u16, ctx: &mut Execution<'_>) {
    let slot = (j >> 1) as isize;
    ctx.set_slot(F::BehaviorB, slot, handler);
    let x = ctx.slot(F::X, (k >> 1) as isize);
    ctx.set_slot(F::X, slot, x);
    let y = ctx.slot(F::Y, (k >> 1) as isize);
    ctx.set_slot(F::Y, slot, y);
    let e = ctx.slot(F::BehaviorA, (k >> 1) as isize);
    ctx.set_slot(F::BehaviorA, slot, e);
    ctx.set_slot(F::Graphics, slot, 2560);
    SetAreaDependentEprojPropertiesEx(
        crate::game::constants::bank_86::addr_kRidleysFireball_Tab0 as u16,
        j,
        ctx,
    );
}
