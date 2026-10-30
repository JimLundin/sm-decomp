#![forbid(unsafe_code)]
use super::access::enemy;
use super::coordinates::{block_index, calculate_coordinates};
use super::host::*;
use super::shared::*;
use sm_game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};

/// Keep live enemy reads interleaved with writes, including broad slot aliases.
pub(super) fn copy_enemy_position(slot: isize, enemy: u16, y_offset: i16, ctx: &mut impl Host) {
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy), Le).x.whole => slot, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy), Le).x.fraction => slot, x_fraction);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy), Le).y.whole as i32 + y_offset as i32) as u16 => slot, y);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy), Le).y.fraction => slot, y_fraction);
}

pub(super) fn init_robot_laser(j: u16, y_velocity: u16, clear_graphics: bool, ctx: &mut impl Host) {
    let slot = (j >> 1) as isize;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(ctx.inputs().current_enemy), Le).variables[0] => slot, velocity_x);
    enemy!(ctx, y_velocity => slot, velocity_y);
    if clear_graphics {
        enemy!(ctx, 0 => slot, graphics);
    }
    EprojInit_WreckedShipRobotLaserCommon(ctx.inputs().current_enemy, j, ctx);
}

pub(super) fn init_landing_dust(j: u16, x_offset: i16, ctx: &mut impl Host) {
    let enemy = ctx.inputs().current_enemy;
    let slot = (j >> 1) as isize;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy), Le).y.whole.wrapping_add(48) => slot, y);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy), Le).x.whole.wrapping_add_signed(x_offset) => slot, x);
}

pub(super) fn init_statue_piece(j: u16, x: u16, y: u16, ctx: &mut impl Host) {
    let slot = (j >> 1) as isize;
    enemy!(ctx, x => slot, behavior_a);
    enemy!(ctx, x => slot, x);
    enemy!(ctx, y => slot, behavior_b);
    enemy!(ctx, y => slot, y);
}

static kEprojInit_MiniKraidSpikes_Tab0: [i16; 3] = [-2, 12, 24];
pub(super) fn init_skree_particle(
    j: u16,
    x_offset: i16,
    x_velocity: i16,
    y_velocity: i16,
    ctx: &mut impl Host,
) {
    let enemy_index = ctx.inputs().current_enemy;
    let word_index = usize::from(j >> 1);
    let y = ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole;
    enemy!(ctx, word_index as isize, y = y);
    enemy!(ctx, word_index as isize, velocity_y = y_velocity as u16);
    let x = ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .x
        .whole
        .wrapping_add_signed(x_offset);
    enemy!(ctx, word_index as isize, x = x);
    enemy!(ctx, word_index as isize, velocity_x = x_velocity as u16);
}

pub(super) fn EprojInit_FireballChild(j: u16, x: u16, y: u16, handler: u16, ctx: &mut impl Host) {
    {
        let parent = ctx.session().enemy_projectiles.spawn.primary;
        let slot = usize::from(j >> 1);
        enemy!(ctx, slot as isize, velocity_x = x);
        enemy!(ctx, slot as isize, velocity_y = y);
        EprojInit_Common(j, parent, handler, ctx);
    }
}

pub(super) fn EprojInit_MiniKraidSpikes(j: u16, a: u16, ctx: &mut impl Host) {
    {
        let v2: i32 = j as i32 >> 1;
        enemy!(ctx, a => (v2) as isize, velocity_x);
        let r18: u16 = kEprojInit_MiniKraidSpikes_Tab0
            [(ctx.mini_kraid_spike_variant(ctx.inputs().current_enemy) >> 1) as usize]
            as u16;
        let enemy_index = ctx.inputs().current_enemy;
        enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
        enemy!(ctx, (r18).wrapping_add(ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole) => (v2) as isize, y);

        enemy!(ctx, 0 => (v2) as isize, y_fraction);
        enemy!(ctx, 0 => (v2) as isize, x_fraction);
        enemy!(ctx, 0 => (v2) as isize, velocity_y);
    }
}

pub(super) fn EprojInit_KiHunterAcidSpitCommon(k: u16, j: u16, ctx: &mut impl Host) {
    {
        let v2: i32 = j as i32 >> 1;
        enemy!(ctx, 0 => (v2) as isize, velocity_y);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(k), Le).y.whole as i32 - 16i32) as u16 => (v2) as isize, y);

        enemy!(ctx, 0 => (v2) as isize, y_fraction);
        enemy!(ctx, 0 => (v2) as isize, x_fraction);
    }
}

pub(super) fn EprojInit_WreckedShipRobotLaserCommon(k: u16, j: u16, ctx: &mut impl Host) {
    {
        let v2 = k;
        let v3: i32 = j as i32 >> 1;
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32 - 16i32) as u16 => (v3) as isize, y);
        {
            let value = (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32
                + (if enemy!(ctx, (v3) as isize, velocity_x) & 0x8000 != 0 {
                    -4i32
                } else {
                    4
                })) as u16;
            enemy!(ctx, value => (v3) as isize, x);
        };

        enemy!(ctx, 0 => (v3) as isize, y_fraction);
        enemy!(ctx, 0 => (v3) as isize, x_fraction);

        if ((ctx.resolve_actor(Key::from_legacy(v2), Le).radius.x as i32
            + ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32
            - ctx.inputs().camera_x as i32) as i16)
            >= 0
            && ((ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32
                - ctx.resolve_actor(Key::from_legacy(v2), Le).radius.x as i32
                - 257i32
                - ctx.inputs().camera_x as i32) as i16)
                < 0
            && ((ctx.resolve_actor(Key::from_legacy(v2), Le).radius.y as i32
                + ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32
                - ctx.inputs().camera_y as i32) as i16)
                >= 0
            && ((ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32
                - ctx.resolve_actor(Key::from_legacy(v2), Le).radius.y as i32
                - 224i32) as i16)
                < 0
        {
            ctx.queue_sound(2, 6, 0x67);
        }
    }
}

pub(super) fn EprojInit_SpawnedGate_Common(j: u16, v1: u16, ctx: &mut impl Host) {
    {
        let v2: u16 = ctx.session().mechanisms.selected;
        calculate_coordinates(ctx.session().mechanisms.selected, ctx);
        let v3: i32 = j as i32 >> 1;
        enemy!(ctx, block_index(v2, ctx) => (v3) as isize, behavior_a);
        enemy!(ctx, (16u16).wrapping_mul(ctx.session().mechanisms.block_x) => (v3) as isize, x);
        enemy!(ctx, (v1 as i32 + 16 * ctx.session().mechanisms.block_y as i32) as u16 => (v3) as isize, y);
    }
}

fn EprojInit_Common(j: u16, k: u16, handler: u16, ctx: &mut impl Host) {
    let slot = (j >> 1) as isize;
    enemy!(ctx, slot, behavior_b = handler);
    let x = enemy!(ctx, (k >> 1) as isize, x);
    enemy!(ctx, slot, x = x);
    let y = enemy!(ctx, (k >> 1) as isize, y);
    enemy!(ctx, slot, y = y);
    let e = enemy!(ctx, (k >> 1) as isize, behavior_a);
    enemy!(ctx, slot, behavior_a = e);
    enemy!(ctx, slot, graphics = 2560);
    SetAreaDependentEprojPropertiesEx(
        crate::game::constants::bank_86::addr_kRidleysFireball_Tab0 as u16,
        j,
        ctx,
    );
}
