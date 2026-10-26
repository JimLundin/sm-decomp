//! Shared pre-action behavior and numeric dispatch. No legacy projectile callbacks.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_add, enemy_byte, shot_word};
use super::host::{EnemyMember as SlotField, Host};
use super::pre_actions::*;
use super::pre_helpers::*;
use super::shared::SpawnEprojWithRoomGfx;
use super::*;
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session::Point;

#[derive(Clone, Copy)]
pub(super) struct CollisionRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

pub(super) use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as EprojPreInstrHelper_DBF2_Func2;
pub(super) use super::pre_actions::Eproj_NorfairLavaquakeRocks_Func3 as EprojPreInstr_DBF2_MoveX2;
pub(super) use super::pre_actions::Eproj_NorfairLavaquakeRocks_Func3 as Eproj_LavaThrownByLavaman_MoveX2;
pub(super) use super::pre_actions::EprojPreInstr_DBF2_MoveX1 as Eproj_LavaThrownByLavaman_MoveX1;
pub(super) use super::pre_actions::sub_869DA5 as sub_86C42E;

pub(super) fn plant_pre<H: Host>(index: usize, k: u16, ctx: &mut H) {
    let actions: [fn(u16, &mut H); 10] = [
        EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1,
        EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1,
        EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2,
        EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1,
        EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2,
        EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2,
        EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1,
        EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1,
        EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2,
        EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2,
    ];
    actions[index](k, ctx);
}

pub(super) fn Math_MultBySin(a: u16, angle: u16) -> u16 {
    Math_MultBySinCos(a, angle)
}
pub(super) fn Math_MultByCos(a: u16, angle: u16) -> u16 {
    Math_MultBySinCos(a, angle.wrapping_add(64))
}

pub(super) fn CallNorfairLavaquakeRocksFunc(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8829974 => Eproj_NorfairLavaquakeRocks_Func1(k, ctx),
        8830095 => Eproj_NorfairLavaquakeRocks_Func2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallMotherBrainTubeFallingFunc(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8834026 => MotherBrainTubeFallingFunc_GenerateExplosion(k, ctx),
        8834056 => MotherBrainTubeFallingFunc_Falling(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallNamiFuneFireballFunc(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8838976 => EprojInit_NamiFuneFireball_MoveX1(k, ctx),
        8839018 => EprojInit_NamiFuneFireball_MoveX2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallLavamanFunc(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8839248 => Eproj_LavaThrownByLavaman_MoveX1(k, ctx),
        8839290 => Eproj_LavaThrownByLavaman_MoveX2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallBotwoonEprojFunc(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8841972 => Eproj_BotwonsBodyFunction_Dying(k, ctx),
        8841988 => Eproj_BotwonsBodyFunction_Dying2(k, ctx),
        8842015 => Eproj_BotwonsBodyFunction_DyingFalling(k, ctx),
        8841880 => Eproj_BotwoonsBody_Main(k, ctx),
        8842131 => (),
        _ => ctx.unreachable(),
    }
}

pub(super) fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(k: u16, ctx: &mut impl Host) -> u16 {
    let v1: i32 = k as i32 >> 1;
    (((enemy!(ctx, (v1) as isize, x) as i32 - ctx.inputs().camera_x as i32) as i16) < 0
        || ((ctx.inputs().camera_x as i32 + 256 - enemy!(ctx, (v1) as isize, x) as i32) as i16) < 0
        || ((enemy!(ctx, (v1) as isize, y) as i32 - ctx.inputs().camera_y as i32) as i16) < 0
        || ((ctx.inputs().camera_y as i32 + 256 - enemy!(ctx, (v1) as isize, y) as i32) as i16) < 0)
        as u16
}

pub(super) fn Eproj_MotherBrainRoomTurretBullets_Func2(v0: u16, ctx: &mut impl Host) {
    let r20: u8 = (enemy_byte!(ctx, v0 as isize + 1, y_fraction) as i32
        + enemy_byte!(ctx, v0 as isize, y_fraction) as i32
        & 7) as u8;
    let r18: u16 = enemy!(ctx, (v0 >> 1) as isize, x_fraction);
    if turret_direction(r18, (r20 as i32 & 7) as isize, ctx) != 0 {
        enemy_byte!(ctx, v0 as isize, y_fraction = r20);
    } else {
        let v1: i8 = -(enemy_byte!(ctx, v0 as isize + 1, y_fraction) as i32) as i8;
        enemy_byte!(ctx, v0 as isize + 1, y_fraction = v1 as u8);
        {
            let old = enemy_byte!(ctx, v0 as isize, y_fraction);
            enemy_byte!(
                ctx,
                v0 as isize,
                y_fraction = (old as i32 + v1 as i32) as u8
            );
        }
    };
}

pub(super) fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    k: u16,
    ctx: &mut impl Host,
) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = enemy!(ctx, (v1) as isize, y) as i16;
    let result: u8 = 1;
    if v2 >= 0 {
        let mut v3: i16 = (v2 as i32 + 16 - ctx.inputs().camera_y as i32) as i16;
        if v3 >= 0 && v3 as i32 - 256i32 & 0x8000 != 0 {
            let mut v4: i16 = enemy!(ctx, (v1) as isize, x) as i16;
            if v4 >= 0 {
                let mut v5: i16 = (v4 as i32 + 4 - ctx.inputs().camera_x as i32) as i16;
                if v5 >= 0 && v5 as i32 - 264i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}

pub(super) fn Eproj_CheckForBombCollisionWithRect(rect: CollisionRect, ctx: &mut impl Host) -> u8 {
    if ctx.session().projectiles.bomb_count == 0 {
        return 0;
    }
    for v1 in (10i32..20).step_by(2) {
        let v2: i32 = v1 >> 1;
        if (shot_word(ctx, super::host::ShotMember::Kind, (v2 as u16) as isize) & 0xf00) == 1280
            && shot_word(
                ctx,
                super::host::ShotMember::Variable,
                ((v2) as isize) as isize,
            ) == 0
        {
            let x: u16 = (((shot_word(ctx, super::host::ShotMember::X, ((v2) as isize) as isize)
                as i32
                - rect.x as i32) as u16) as i16)
                .unsigned_abs();
            if x < shot_word(
                ctx,
                super::host::ShotMember::XRadius,
                ((v2) as isize) as isize,
            ) || ((x as i32
                - shot_word(
                    ctx,
                    super::host::ShotMember::XRadius,
                    ((v2) as isize) as isize,
                ) as i32) as u16)
                < rect.w
            {
                let y: u16 =
                    (((shot_word(ctx, super::host::ShotMember::Y, ((v2) as isize) as isize) as i32
                        - rect.y as i32) as u16) as i16)
                        .unsigned_abs();
                if y < shot_word(
                    ctx,
                    super::host::ShotMember::YRadius,
                    ((v2) as isize) as isize,
                ) || ((y as i32
                    - shot_word(
                        ctx,
                        super::host::ShotMember::YRadius,
                        ((v2) as isize) as isize,
                    ) as i32) as u16)
                    < rect.h
                {
                    return 1;
                }
            }
        }
    }
    0
}

pub(super) fn Eproj_CheckForEnemyCollisionWithRect(
    k: u16,
    rect: CollisionRect,
    ctx: &mut impl Host,
) -> u8 {
    let v1 = k;
    (((((ctx.resolve_actor(Key::from_legacy(v1), Le).x.whole as i32 - rect.x as i32) as u16) as i16)
        .unsigned_abs() as i32
        - ctx.resolve_actor(Key::from_legacy(v1), Le).radius.x as i32)
        < rect.w as i32
        && ((((ctx.resolve_actor(Key::from_legacy(v1), Le).y.whole as i32 - rect.y as i32) as u16)
            as i16)
            .unsigned_abs() as i32
            - ctx.resolve_actor(Key::from_legacy(v1), Le).radius.y as i32)
            < rect.h as i32) as u8
}

pub(super) fn Eproj_CheckForCollisionWithSamus(v0: u16, ctx: &mut impl Host) -> u8 {
    let rect: CollisionRect = Eproj_GetCollDetectRect(v0, ctx);
    let v1: u16 = (((ctx.session().player.x as i32 - rect.x as i32) as u16) as i16).unsigned_abs();
    let mut v2: bool = v1 < ctx.session().player.contact.x_radius;
    let v3: u16 = (v1 as i32 - ctx.session().player.contact.x_radius as i32) as u16;
    let result: u8 = 0;
    if v2 || v3 < rect.w {
        let v4: u16 =
            (((ctx.session().player.y as i32 - rect.y as i32) as u16) as i16).unsigned_abs();
        v2 = v4 < ctx.session().player.contact.radius;
        let v5: u16 = (v4 as i32 - ctx.session().player.contact.radius as i32) as u16;
        if v2 || v5 < rect.h {
            return 1;
        }
    }
    result
}

pub(super) fn Math_MultBySinCos(r38: u16, a: u16) -> u16 {
    let r46: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
    let prod: u16 = ((r38 as u32).wrapping_mul(((r46) as i16).unsigned_abs() as u32) >> 8) as u16;
    (if r46 & 0x8000 != 0 {
        -(prod as i32)
    } else {
        prod as i32
    }) as u16
}

pub(super) fn CheckForCollisionWithShitroid_DoubleRet(v0: u16, ctx: &mut impl Host) -> u8 {
    if ctx.mother_brain_target_enemy() == 0 {
        return 0;
    }
    if ctx
        .resolve_actor(Key::from_legacy(ctx.mother_brain_target_enemy()), Le)
        .health
        != 0
    {
        let rect: CollisionRect = Eproj_GetCollDetectRect(v0, ctx);
        return Eproj_CheckForEnemyCollisionWithRect(ctx.mother_brain_target_enemy(), rect, ctx);
    }
    enemy!(ctx, 0 => (v0 >> 1) as isize, definition);
    0xff
}

pub(super) fn CheckForBlueRingCollisionWithRoom(k: u16, ctx: &mut impl Host) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let result: u8 = 1;
    if enemy!(ctx, (v1) as isize, y) as i32 - 32i32 & 0x8000 == 0
        && enemy!(ctx, (v1) as isize, y) < 0xd8
    {
        let mut v2: i16 = enemy!(ctx, (v1) as isize, x) as i16;
        if v2 >= 0 {
            let mut v3: i16 = (v2 as i32 - ctx.inputs().camera_x as i32) as i16;
            if v3 >= 0 && v3 as i32 - 248i32 & 0x8000 != 0 {
                return 0;
            }
        }
    }
    result
}

pub(super) fn Eproj_GetCollDetectRect(k: u16, ctx: &mut impl Host) -> CollisionRect {
    let v1: i32 = k as i32 >> 1;
    let rect: CollisionRect = CollisionRect {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
        w: enemy_byte!(ctx, v1 as isize * 2, radius) as u16,
        h: enemy_byte!(ctx, v1 as isize * 2 + 1, radius) as u16,
    };
    rect
}

pub(super) fn Eproj_Earthqhake5(k: u16, ctx: &mut impl Host) {
    {
        {
            let value = 10;
            ctx.session_mut().feedback.earthquake_timer = value;
        };
        {
            let value = 5;
            ctx.session_mut().feedback.earthquake_kind = value;
        };
    }
    BlueRingContactExplosion(k, ctx);
}

pub(super) fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(k: u16, ctx: &mut impl Host) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let rect: CollisionRect = CollisionRect {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
        w: enemy_byte!(ctx, v1 as isize * 2, radius) as u16,
        h: enemy_byte!(ctx, v1 as isize * 2 + 1, radius) as u16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, ctx) == 0 {
        return 0;
    }
    {
        let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
        ctx.set_mother_brain_active_bombs(value);
    }
    {
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    super::drops::spawn(addr_kEnemyDef_EC3F as u16, k, ctx);
    1
}

pub(super) fn MoveMotherBrainBomb(k: u16, a: u16, ctx: &mut impl Host) -> u8 {
    let v2: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v2) as isize, velocity_y, a);
    MoveEprojWithVelocity(k, ctx);
    if enemy!(ctx, (v2) as isize, x) as i32 - 240i32 & 0x8000 == 0 {
        enemy!(ctx, -(enemy!(ctx, (v2) as isize, velocity_x) as i32) as u16 => (v2) as isize, velocity_x);
    }
    if enemy!(ctx, (v2) as isize, y) as i32 - 208i32 & 0x8000 != 0 {
        return 0;
    }
    enemy!(ctx, 208 => (v2) as isize, y);
    enemy!(ctx, (if enemy!(ctx, (v2) as isize, velocity_x) & 0x8000 != 0 {
            -(enemy!(ctx, (v2) as isize, behavior_a) as i32)
        } else {
            enemy!(ctx, (v2) as isize, behavior_a) as i32
        }) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_y);
    1
}

pub(super) fn SpawnMotherBrainDeathBeam(x: u16, ctx: &mut impl Host) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as u16,
        enemy!(ctx, (x >> 1) as isize, behavior_a),
        ctx,
    );
}

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1;

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as Eproj_NamiFuneFireball_After;

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as sub_86E0A4;

pub(super) fn CheckIfEprojIsOffScreen(k: u16, ctx: &mut impl Host) -> u16 {
    if (enemy!(ctx, (k >> 1) as isize, x).wrapping_sub(ctx.inputs().camera_x) as i16) < 0 {
        return 1;
    }
    ctx.session_mut().compatibility_enemy_scratch = ctx.inputs().camera_x.wrapping_add(256);
    if (enemy!(ctx, (k >> 1) as isize, x).wrapping_sub(ctx.session().compatibility_enemy_scratch)
        as i16)
        >= 0
        || (enemy!(ctx, (k >> 1) as isize, y).wrapping_sub(ctx.inputs().camera_y) as i16) < 0
    {
        return 1;
    }
    ctx.session_mut().compatibility_enemy_scratch = ctx.inputs().camera_y.wrapping_add(256);
    u16::from(
        (enemy!(ctx, (k >> 1) as isize, y).wrapping_sub(ctx.session().compatibility_enemy_scratch)
            as i16)
            >= 0,
    )
}

pub(super) fn Eproj_FuncE722(k: u16, ctx: &mut impl Host) -> u16 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = enemy!(ctx, (v1) as isize, x) as i16;
    let result: u16 = 1;
    if v2 >= 0 && v2 as i32 - 512i32 & 0x8000 != 0 {
        let mut v3: i16 = enemy!(ctx, (v1) as isize, y) as i16;
        if v3 >= 0 && v3 as i32 - 512i32 & 0x8000 != 0 {
            return 0;
        }
    }
    result
}

pub(super) use super::pre_actions::Eproj_BotwoonsBodyHurtFlashHandling1 as Eproj_BotwoonsBodyHurtFlashHandling2;

pub(super) fn QueueSmallExplosionSfx(ctx: &mut impl Host) {
    ctx.queue_sound(2, 6, 0x24);
}

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as sub_86EC0C;

pub(super) use super::pre_actions::EprojPreInstr_SpikeShootingPlantSpikes_MoveX1 as EprojInit_NamiFuneFireball_MoveX1;

pub(super) use super::pre_actions::EprojPreInstr_SpikeShootingPlantSpikes_MoveX2 as EprojInit_NamiFuneFireball_MoveX2;

pub(super) fn Eproj_BotwonsBodyFunction_Dying(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    {
        enemy!(ctx, (4 * v0 as i32 + 96) as u16 => (v1) as isize, behavior_a);
        enemy!(ctx, 0xeb04 => (v1) as isize, velocity_x);
    }
    Eproj_BotwonsBodyFunction_Dying2(v0, ctx);
}

pub(super) fn Eproj_BotwonsBodyFunction_Dying2(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    if enemy_add!(ctx, (v1) as isize, behavior_a, 1) as i32 - 256i32 & 0x8000 == 0 {
        enemy!(ctx, (0x86eb1f & 0xffff) as u16 => (v1) as isize, velocity_x);
    }
    enemy!(ctx, 0 => (v1) as isize, delay);
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, ctx);
}

pub(super) fn Eproj_BotwonsBodyFunction_DyingFalling(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    let v3: i32 = 8 * ((enemy!(ctx, (v1) as isize, velocity_y) as i32 & 0xff00) >> 8) >> 1;
    {
        let index = v1 as isize;
        let amount = lavaquake_delta((v3 >> 1) as isize, ctx);
        add_split(SlotField::Y, SlotField::YFraction, index, amount, ctx);
    };
    if enemy!(ctx, (v1) as isize, y) as i32 - 200i32 & 0x8000 != 0 {
        let v7: i32 = v0 as i32 >> 1;
        enemy_add!(ctx, (v7) as isize, velocity_y, 192);
        enemy!(ctx, 0 => (v7) as isize, delay);
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, ctx);
    } else {
        {
            enemy!(ctx, 200 => (v1) as isize, y);
            enemy!(ctx, (0x86eb93 & 0xffff) as u16 => (v1) as isize, velocity_x);
            enemy!(ctx, addr_word_86E208 as u16 => (v1) as isize, program);
            enemy!(ctx, 2560 => (v1) as isize, graphics);
            enemy!(ctx, 1 => (v1) as isize, delay);
        }
        QueueSmallExplosionSfx(ctx);
        if v0 == 10 {
            ctx.set_botwoon_body_landed(1);
        }
    };
}
