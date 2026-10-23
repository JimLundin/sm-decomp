//! Shared pre-action behavior and numeric dispatch. No legacy projectile callbacks.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::pre_actions::*;
use super::pre_helpers::*;
use super::shared::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use super::state::{Point, SlotField};
use super::*;

#[derive(Clone, Copy)]
pub(super) struct CollisionRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

pub(super) use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as EprojPreInstrHelper_DBF2_Func2;
pub(super) use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as sub_86E0B0;
pub(super) use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as sub_86EC18;
pub(super) use super::pre_actions::Eproj_NorfairLavaquakeRocks_Func3 as EprojPreInstr_DBF2_MoveX2;
pub(super) use super::pre_actions::Eproj_NorfairLavaquakeRocks_Func3 as Eproj_LavaThrownByLavaman_MoveX2;
pub(super) use super::pre_actions::EprojPreInstr_DBF2_MoveX1 as Eproj_LavaThrownByLavaman_MoveX1;
pub(super) use super::pre_actions::sub_869DA5 as sub_86C42E;

pub(super) static kEprojPreInstr_SpikeShootingPlantSpikes: [fn(u16, &mut Execution<'_>); 10] = [
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

pub(super) fn Math_MultBySin(a: u16, angle: u16) -> u16 {
    Math_MultBySinCos(a, angle)
}
pub(super) fn Math_MultByCos(a: u16, angle: u16) -> u16 {
    Math_MultBySinCos(a, angle.wrapping_add(64))
}

pub(super) fn CallNorfairLavaquakeRocksFunc(ea: u32, k: u16, ctx: &mut Execution<'_>) {
    match ea {
        8829974 => Eproj_NorfairLavaquakeRocks_Func1(k, ctx),
        8830095 => Eproj_NorfairLavaquakeRocks_Func2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallMotherBrainTubeFallingFunc(ea: u32, k: u16, ctx: &mut Execution<'_>) {
    match ea {
        8834026 => MotherBrainTubeFallingFunc_GenerateExplosion(k, ctx),
        8834056 => MotherBrainTubeFallingFunc_Falling(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallNamiFuneFireballFunc(ea: u32, k: u16, ctx: &mut Execution<'_>) {
    match ea {
        8838976 => EprojInit_NamiFuneFireball_MoveX1(k, ctx),
        8839018 => EprojInit_NamiFuneFireball_MoveX2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallLavamanFunc(ea: u32, k: u16, ctx: &mut Execution<'_>) {
    match ea {
        8839248 => Eproj_LavaThrownByLavaman_MoveX1(k, ctx),
        8839290 => Eproj_LavaThrownByLavaman_MoveX2(k, ctx),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallBotwoonEprojFunc(ea: u32, k: u16, ctx: &mut Execution<'_>) {
    match ea {
        8841972 => Eproj_BotwonsBodyFunction_Dying(k, ctx),
        8841988 => Eproj_BotwonsBodyFunction_Dying2(k, ctx),
        8842015 => Eproj_BotwonsBodyFunction_DyingFalling(k, ctx),
        8841880 => Eproj_BotwoonsBody_Main(k, ctx),
        8842131 => (),
        _ => ctx.unreachable(),
    }
}

pub(super) fn CallEprojFunc(ea: u32, k: u32, ctx: &mut Execution<'_>) {
    match ea {
        8833019 => SpawnMotherBrainDeathBeam(k as u16, ctx),
        _ => ctx.unreachable(),
    }
}
pub(super) fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(k: u16, ctx: &mut Execution<'_>) -> u16 {
    let v1: i32 = k as i32 >> 1;
    (((ctx.slot(SlotField::X, (v1) as isize) as i32 - ctx.layer1_x_pos() as i32) as i16) < 0
        || ((ctx.layer1_x_pos() as i32 + 256 - ctx.slot(SlotField::X, (v1) as isize) as i32)
            as i16)
            < 0
        || ((ctx.slot(SlotField::Y, (v1) as isize) as i32 - ctx.layer1_y_pos() as i32) as i16) < 0
        || ((ctx.layer1_y_pos() as i32 + 256 - ctx.slot(SlotField::Y, (v1) as isize) as i32)
            as i16)
            < 0) as u16
}

pub(super) fn Eproj_MotherBrainRoomTurretBullets_Func2(v0: u16, ctx: &mut Execution<'_>) {
    let r20: u8 = (ctx.slot_byte(SlotField::YFraction, v0 as isize + 1) as i32
        + ctx.slot_byte(SlotField::YFraction, v0 as isize) as i32
        & 7) as u8;
    let r18: u16 = ctx.slot(SlotField::XFraction, (v0 >> 1) as isize);
    if ctx.raw_turret_direction(r18, (r20 as i32 & 7) as isize) != 0 {
        ctx.set_slot_byte(SlotField::YFraction, v0 as isize, r20);
    } else {
        let v1: i8 = -(ctx.slot_byte(SlotField::YFraction, v0 as isize + 1) as i32) as i8;
        ctx.set_slot_byte(SlotField::YFraction, v0 as isize + 1, v1 as u8);
        {
            let old = ctx.slot_byte(SlotField::YFraction, v0 as isize);
            ctx.set_slot_byte(
                SlotField::YFraction,
                v0 as isize,
                (old as i32 + v1 as i32) as u8,
            );
        }
    };
}

pub(super) fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    k: u16,
    ctx: &mut Execution<'_>,
) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = ctx.slot(SlotField::Y, (v1) as isize) as i16;
    let result: u8 = 1;
    if v2 >= 0 {
        let mut v3: i16 = (v2 as i32 + 16 - ctx.layer1_y_pos() as i32) as i16;
        if v3 >= 0 && v3 as i32 - 256i32 & 0x8000 != 0 {
            let mut v4: i16 = ctx.slot(SlotField::X, (v1) as isize) as i16;
            if v4 >= 0 {
                let mut v5: i16 = (v4 as i32 + 4 - ctx.layer1_x_pos() as i32) as i16;
                if v5 >= 0 && v5 as i32 - 264i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}

pub(super) fn Eproj_CheckForBombCollisionWithRect(
    rect: CollisionRect,
    ctx: &mut Execution<'_>,
) -> u8 {
    if ctx.bomb_counter() == 0 {
        return 0;
    }
    for v1 in (10i32..20).step_by(2) {
        let v2: i32 = v1 >> 1;
        if (ctx.player_projectile_type(v2 as u16) & 0xf00) == 1280
            && ctx.player_projectile_variables((v2) as isize) == 0
        {
            let x: u16 =
                abs16((ctx.player_projectile_x_pos((v2) as isize) as i32 - rect.x as i32) as u16);
            if x < ctx.player_projectile_x_radius((v2) as isize)
                || ((x as i32 - ctx.player_projectile_x_radius((v2) as isize) as i32) as u16)
                    < rect.w
            {
                let y: u16 = abs16(
                    (ctx.player_projectile_y_pos((v2) as isize) as i32 - rect.y as i32) as u16,
                );
                if y < ctx.player_projectile_y_radius((v2) as isize)
                    || ((y as i32 - ctx.player_projectile_y_radius((v2) as isize) as i32) as u16)
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
    ctx: &mut Execution<'_>,
) -> u8 {
    let v1 = k;
    ((abs16((ctx.enemy_x(v1) as i32 - rect.x as i32) as u16) as i32 - ctx.enemy_width(v1) as i32)
        < rect.w as i32
        && (abs16((ctx.enemy_y(v1) as i32 - rect.y as i32) as u16) as i32
            - ctx.enemy_height(v1) as i32)
            < rect.h as i32) as u8
}

pub(super) fn Eproj_CheckForCollisionWithSamus(v0: u16, ctx: &mut Execution<'_>) -> u8 {
    let rect: CollisionRect = Eproj_GetCollDetectRect(v0, ctx);
    let v1: u16 = abs16((ctx.player_x() as i32 - rect.x as i32) as u16);
    let mut v2: bool = v1 < ctx.player_x_radius();
    let v3: u16 = (v1 as i32 - ctx.player_x_radius() as i32) as u16;
    let result: u8 = 0;
    if v2 || v3 < rect.w {
        let v4: u16 = abs16((ctx.player_y() as i32 - rect.y as i32) as u16);
        v2 = v4 < ctx.player_y_radius();
        let v5: u16 = (v4 as i32 - ctx.player_y_radius() as i32) as u16;
        if v2 || v5 < rect.h {
            return 1;
        }
    }
    result
}

pub(super) fn Math_MultBySinCos(r38: u16, a: u16) -> u16 {
    let r46: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
    let prod: u16 = ((r38 as u32).wrapping_mul(abs16(r46) as u32) >> 8) as u16;
    ((if r46 & 0x8000 != 0 {
        -(prod as i32)
    } else {
        prod as i32
    }) as u16)
}

pub(super) fn CheckForCollisionWithShitroid_DoubleRet(v0: u16, ctx: &mut Execution<'_>) -> u8 {
    if ctx.mother_brain_target_enemy() == 0 {
        return 0;
    }
    if ctx.enemy_health(ctx.mother_brain_target_enemy()) != 0 {
        let rect: CollisionRect = Eproj_GetCollDetectRect(v0, ctx);
        return Eproj_CheckForEnemyCollisionWithRect(ctx.mother_brain_target_enemy(), rect, ctx);
    }
    store(0, SlotField::Definition, (v0 >> 1) as isize, ctx);
    0xff
}

pub(super) fn CheckForBlueRingCollisionWithRoom(k: u16, ctx: &mut Execution<'_>) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let result: u8 = 1;
    if ctx.slot(SlotField::Y, (v1) as isize) as i32 - 32i32 & 0x8000 == 0
        && ctx.slot(SlotField::Y, (v1) as isize) < 0xd8
    {
        let mut v2: i16 = ctx.slot(SlotField::X, (v1) as isize) as i16;
        if v2 >= 0 {
            let mut v3: i16 = (v2 as i32 - ctx.layer1_x_pos() as i32) as i16;
            if v3 >= 0 && v3 as i32 - 248i32 & 0x8000 != 0 {
                return 0;
            }
        }
    }
    result
}

pub(super) fn Eproj_GetCollDetectRect(k: u16, ctx: &mut Execution<'_>) -> CollisionRect {
    let v1: i32 = k as i32 >> 1;
    let rect: CollisionRect = CollisionRect {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: ctx.slot(SlotField::Y, (v1) as isize),
        w: ctx.slot_byte(SlotField::Radius, v1 as isize * 2) as u16,
        h: ctx.slot_byte(SlotField::Radius, v1 as isize * 2 + 1) as u16,
    };
    rect
}

pub(super) fn Eproj_Earthqhake5(k: u16, ctx: &mut Execution<'_>) {
    {
        {
            let value = 10;
            ctx.set_earthquake_timer(value);
        };
        {
            let value = 5;
            ctx.set_earthquake_type(value);
        };
    }
    BlueRingContactExplosion(k, ctx);
}

pub(super) fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(k: u16, ctx: &mut Execution<'_>) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let rect: CollisionRect = CollisionRect {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: ctx.slot(SlotField::Y, (v1) as isize),
        w: ctx.slot_byte(SlotField::Radius, v1 as isize * 2) as u16,
        h: ctx.slot_byte(SlotField::Radius, v1 as isize * 2 + 1) as u16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, ctx) == 0 {
        return 0;
    }
    {
        let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
        ctx.set_mother_brain_active_bombs(value);
    }
    {
        store(0, SlotField::VelocityX, (v1) as isize, ctx);
        store(0, SlotField::VelocityY, (v1) as isize, ctx);
        store(0, SlotField::Definition, (v1) as isize, ctx);
    }
    ctx.state_mut().spawn.point = Point {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: ctx.slot(SlotField::Y, (v1) as isize),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    ctx.state_mut().spawn.point = Point {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: ctx.slot(SlotField::Y, (v1) as isize),
    };
    super::drops::spawn(addr_kEnemyDef_EC3F as u16, k, 0, ctx);
    1
}

pub(super) fn MoveMotherBrainBomb(k: u16, a: u16, ctx: &mut Execution<'_>) -> u8 {
    let v2: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v2) as isize, a, ctx);
    MoveEprojWithVelocity(k, ctx);
    if ctx.slot(SlotField::X, (v2) as isize) as i32 - 240i32 & 0x8000 == 0 {
        store(
            -(ctx.slot(SlotField::VelocityX, (v2) as isize) as i32) as u16,
            SlotField::VelocityX,
            (v2) as isize,
            ctx,
        );
    }
    if ctx.slot(SlotField::Y, (v2) as isize) as i32 - 208i32 & 0x8000 != 0 {
        return 0;
    }
    store(208, SlotField::Y, (v2) as isize, ctx);
    store(
        (if ctx.slot(SlotField::VelocityX, (v2) as isize) & 0x8000 != 0 {
            -(ctx.slot(SlotField::BehaviorA, (v2) as isize) as i32)
        } else {
            ctx.slot(SlotField::BehaviorA, (v2) as isize) as i32
        }) as u16,
        SlotField::VelocityX,
        (v2) as isize,
        ctx,
    );
    store(-512i32 as u16, SlotField::VelocityY, (v2) as isize, ctx);
    1
}

pub(super) fn SpawnMotherBrainDeathBeam(x: u16, ctx: &mut Execution<'_>) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as u16,
        ctx.slot(SlotField::BehaviorA, (x >> 1) as isize),
        ctx,
    );
}

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1;

pub(super) use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2;

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as Eproj_NamiFuneFireball_After;

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as sub_86E0A4;

pub(super) fn CheckIfEprojIsOffScreen(k: u16, ctx: &mut Execution<'_>) -> u16 {
    if (ctx
        .slot(SlotField::X, (k >> 1) as isize)
        .wrapping_sub(ctx.layer1_x_pos()) as i16)
        >= 0
        && ((ctx.slot(SlotField::X, (k >> 1) as isize) as i32 - (ctx.layer1_x_pos() as i32 + 256))
            as i16)
            < 0
        && (ctx
            .slot(SlotField::Y, (k >> 1) as isize)
            .wrapping_sub(ctx.layer1_y_pos()) as i16)
            >= 0
        && ((ctx.slot(SlotField::Y, (k >> 1) as isize) as i32 - (ctx.layer1_y_pos() as i32 + 256))
            as i16)
            < 0
    {
        return 0;
    }
    1
}

pub(super) fn Eproj_FuncE722(k: u16, ctx: &mut Execution<'_>) -> u16 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = ctx.slot(SlotField::X, (v1) as isize) as i16;
    let result: u16 = 1;
    if v2 >= 0 && v2 as i32 - 512i32 & 0x8000 != 0 {
        let mut v3: i16 = ctx.slot(SlotField::Y, (v1) as isize) as i16;
        if v3 >= 0 && v3 as i32 - 512i32 & 0x8000 != 0 {
            return 0;
        }
    }
    result
}

pub(super) use super::pre_actions::Eproj_BotwoonsBodyHurtFlashHandling1 as Eproj_BotwoonsBodyHurtFlashHandling2;

pub(super) fn QueueSmallExplosionSfx(ctx: &mut Execution<'_>) {
    ctx.queue_sound(2, 6, 0x24);
}

pub(super) use super::pre_actions::EprojPreInstr_NorfairLavaquakeRocks_Inner as sub_86EC0C;

pub(super) use super::pre_actions::EprojPreInstr_SpikeShootingPlantSpikes_MoveX1 as EprojInit_NamiFuneFireball_MoveX1;

pub(super) use super::pre_actions::EprojPreInstr_SpikeShootingPlantSpikes_MoveX2 as EprojInit_NamiFuneFireball_MoveX2;

pub(super) fn Eproj_BotwonsBodyFunction_Dying(v0: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = v0 as i32 >> 1;
    {
        store(
            (4 * v0 as i32 + 96) as u16,
            SlotField::BehaviorA,
            (v1) as isize,
            ctx,
        );
        store(0xeb04, SlotField::VelocityX, (v1) as isize, ctx);
    }
    Eproj_BotwonsBodyFunction_Dying2(v0, ctx);
}

pub(super) fn Eproj_BotwonsBodyFunction_Dying2(v0: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = v0 as i32 >> 1;
    if add_word(SlotField::BehaviorA, (v1) as isize, 1, ctx) as i32 - 256i32 & 0x8000 == 0 {
        store(
            (0x86eb1f & 0xffff) as u16,
            SlotField::VelocityX,
            (v1) as isize,
            ctx,
        );
    }
    store(0, SlotField::Delay, (v1) as isize, ctx);
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, ctx);
}

pub(super) fn Eproj_BotwonsBodyFunction_DyingFalling(v0: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = v0 as i32 >> 1;
    let v3: i32 = 8 * ((ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 & 0xff00) >> 8) >> 1;
    {
        let index = v1 as isize;
        let amount = ctx.raw_lavaquake_delta((v3 >> 1) as isize);
        ctx.add_split(SlotField::Y, SlotField::YFraction, index, amount);
    };
    if ctx.slot(SlotField::Y, (v1) as isize) as i32 - 200i32 & 0x8000 != 0 {
        let v7: i32 = v0 as i32 >> 1;
        add_word(SlotField::VelocityY, (v7) as isize, 192, ctx);
        store(0, SlotField::Delay, (v7) as isize, ctx);
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, ctx);
    } else {
        {
            store(200, SlotField::Y, (v1) as isize, ctx);
            store(
                (0x86eb93 & 0xffff) as u16,
                SlotField::VelocityX,
                (v1) as isize,
                ctx,
            );
            store(
                addr_word_86E208 as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            store(2560, SlotField::Graphics, (v1) as isize, ctx);
            store(1, SlotField::Delay, (v1) as isize, ctx);
        }
        QueueSmallExplosionSfx(ctx);
        if v0 == 10 {
            ctx.set_botwoon_body_landed(1);
        }
    };
}
