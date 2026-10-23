//! Complete enemy-projectile setup dispatch, using owned fields and explicit effects.
#![forbid(unsafe_code)]
use super::SlotField as F;
use super::pre_helpers::store;
use super::shared::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use super::*;
use crate::game::constants::bank_86::*;
use crate::game::constants::bank_a0::*;
use crate::types::{Point16U, Point32};

pub(super) fn dispatch(ea: u32, j: u16, ctx: &mut Execution<'_>) {
    match ea {
        8817209 => Eproj_Init_0x8aaf(j, ctx),
        8817984 => EprojInit_DraygonsWallTurretProjs(j, ctx),
        8819658 => EprojInit_9642_RidleysFireball(j, ctx),
        8820772 => EprojInit_PhantoonDestroyableFireballs(j, ctx),
        8821923 => EprojInit_RocksKraidSpits(j, ctx),
        8822028 => EprojInit_RocksWhenKraidRises(j, ctx),
        8822450 => EprojInit_WalkingLavaSeahorseFireball(j, ctx),
        8822936 => EprojInit_PirateClaw(j, ctx),
        8824275 => EprojInit_BombTorizoLowHealthDrool(j, ctx),
        8824566 => EprojInit_BombTorizoExplosiveSwipe(j, ctx),
        8824676 => EprojInit_BombTorizoStatueBreaking(j, ctx),
        8825405 => EprojInit_AB07(j, ctx),
        8825980 => EprojInit_GoldenTorizosChozoOrbs(j, ctx),
        8826389 => EprojInit_TorizoSonicBoom(j, ctx),
        8826620 => EprojInit_WreckedShipChozoSpikeFootsteps(j, ctx),
        8826881 => EprojInit_GoldenTorizoEgg(j, ctx),
        8827688 => EprojInit_GoldenTorizoEyeBeam(j, ctx),
        8828143 => EprojInit_LavaSeahorseFireball(j, ctx),
        8828461 => EprojInit_EyeDoorProjectile(j, ctx),
        8828547 => EprojInit_EyeDoorSweat(j, ctx),
        8829070 => EprojInit_TourianStatueEyeGlow(j, ctx),
        8829109 => EprojInit_TourianStatueUnlockingParticle(j, ctx),
        8829176 => EprojInit_TourianStatueSoul(j, ctx),
        8829842 => EprojInit_NuclearWaffleBody(j, ctx),
        8830364 => EprojInit_ShaktoolAttackMiddleBackCircle(j, ctx),
        8832130 => EprojInit_MotherBrainBomb(j, ctx),
        8833377 => EprojInit_MotherBrainEscapeDoorParticles(j, ctx),
        8833782 => EprojInit_TimeBombSetJapaneseText(j, ctx),
        8834501 => EprojInit_MotherBrainGlassShatteringShard(j, ctx),
        8834669 => EprojInit_MotherBrainGlassShatteringSparkle(j, ctx),
        8836773 => EprojInit_N00bTubeCrack(j, ctx),
        8836809 => EprojInit_N00bTubeShards(j, ctx),
        8836980 => EprojInit_N00bTubeReleasedAirBubbles(j, ctx),
        8840877 => EprojInit_SaveStationElectricity(j, ctx),
        8842182 => EprojInit_BotwoonsSpit(j, ctx),
        8844177 => EprojInit_Sparks(j, ctx),
        8819839 => EprojInit_9660_FireballExplosion(j, ctx),
        8820444 => EprojInit_9734_CeresFallingDebris(j, ctx),
        8823534 => EprojInit_CeresElevatorPad(j, ctx),
        8823579 => EprojInit_CeresElevatorPlatform(j, ctx),
        8823715 => EprojPreInstr_PrePhantomRoom(j, ctx),
        8829744 => sub_86BB30(j, ctx),
        8831731 => Eproj_MotherBrainsBlueRingLasers(j, ctx),
        8832517 => sub_86C605(j, ctx),
        8829160 => EprojIni_TourianStatueUnlockingParticleTail(j, ctx),
        8837522 => sub_86D992(j, ctx),
        8840296 => EprojInit_DustCloudOrExplosion(j, ctx),
        8817924 => EprojInit_DraygonsGunk(j, ctx),
        8818895 => EprojInit_CrocomireSpikeWallPieces(j, ctx),
        8819334 => EprojInit_CrocomireBridgeCrumbling(j, ctx),
        8819533 => EprojInit_9634(j, ctx),
        8821050 => EprojInit_PhantoonStartingFireballs(j, ctx),
        8821976 => EprojInit_RocksFallingKraidCeiling(j, ctx),
        8823457 => EprojInit_A379(j, ctx),
        8824413 => EprojInit_BombTorizoLowHealthInitialDrool(j, ctx),
        8824519 => EprojInit_A977(j, ctx),
        8826691 => EprojInit_TourianStatueDustClouds(j, ctx),
        8828061 => EprojInit_TourianEscapeShaftFakeWallExplode(j, ctx),
        8829050 => EprojInit_TourianStatueUnlockingParticleWaterSplash(j, ctx),
        8829246 => EprojInit_TourianStatueBaseDecoration(j, ctx),
        8829265 => EprojInit_TourianStatueRidley(j, ctx),
        8829284 => EprojInit_TourianStatuePhantoon(j, ctx),
        8829915 => EprojInit_NorfairLavaquakeRocks(j, ctx),
        8830370 => EprojInit_BDA2(j, ctx),
        8830543 => EprojInit_MotherBrainRoomTurrets(j, ctx),
        8830809 => EprojInit_MotherBrainRoomTurretBullets(j, ctx),
        8832644 => EprojInit_MotherBrainDeathBeemFired(j, ctx),
        8833034 => EprojInit_MotherBrainRainbowBeam(j, ctx),
        8833091 => EprojInit_MotherBrainsDrool(j, ctx),
        8833269 => EprojInit_MotherBrainsDeathExplosion(j, ctx),
        8833327 => EprojInit_MotherBrainsRainbowBeamExplosion(j, ctx),
        8833642 => EprojInit_MotherBrainPurpleBreathBig(j, ctx),
        8833667 => EprojInit_MotherBrainPurpleBreathSmall(j, ctx),
        8833993 => EprojInit_MotherBrainTubeFalling(j, ctx),
        8835642 => EprojInit_MaridiaFloatersSpikes(j, ctx),
        8838307 => EprojInit_SporeSpawnStalk(j, ctx),
        8838356 => EprojInit_SporeSpawners(j, ctx),
        8840358 => EprojInit_EyeDoorSmoke(j, ctx),
        8841777 => EprojInit_BotwoonsBody(j, ctx),
        8818723 => EprojInit_CrocomireProjectile(j, ctx),
        8822793 => EprojInit_PirateMotherBrainLaser(j, ctx),
        8827342 => EprojInit_GoldenTorizoSuperMissile(j, ctx),
        8834960 => EprojInit_KiHunterAcidSpitLeft(j, ctx),
        8834982 => EprojInit_KiHunterAcidSpitRight(j, ctx),
        8835208 => EprojInit_KagosBugs(j, ctx),
        8835852 => EprojInit_WreckedShipRobotLaserDown(j, ctx),
        8835886 => EprojInit_WreckedShipRobotLaserHorizontal(j, ctx),
        8835905 => EprojInit_WreckedShipRobotLaserUp(j, ctx),
        8837912 => EprojInit_DBF2(j, ctx),
        8838285 => EprojInit_Spores(j, ctx),
        8838870 => EprojInit_NamiFuneFireball(j, ctx),
        8839168 => EprojInit_LavaThrownByLavaman(j, ctx),
        8822252 => EprojInit_MiniKraidSpit(j, ctx),
        8824859 => EprojInit_BombTorizoLowHealthExplode(j, ctx),
        8824945 => EprojInit_BombTorizoDeathExplosion(j, ctx),
        8825835 => EprojInit_BombTorizosChozoOrbs(j, ctx),
        8826704 => EprojInit_TourianLandingDustCloudsRightFoot(j, ctx),
        8826829 => EprojInit_TorizoLandingDustCloudLeftFoot(j, ctx),
        8842338 => EprojInit_YappingMawsBody(j, ctx),
        8843145 => EprojInit_EnemyDeathExplosion(j, ctx),
        8819872 => EprojInit_9688(j, ctx),
        8819892 => EprojInit_9696(j, ctx),
        8819912 => EprojInit_966C(j, ctx),
        8819932 => EprojInit_967A(j, ctx),
        8822342 => EprojInit_MiniKraidSpikesLeft(j, ctx),
        8822347 => EprojInit_MiniKraidSpikesRight(j, ctx),
        8840656 => EprojInit_SpawnedShotGate(j, ctx),
        8840661 => EprojInit_ClosedDownwardsShotGate(j, ctx),
        8840666 => EprojInit_ClosedUpwardsShotGate(j, ctx),
        8843049 => super::drops::initialize(j, ctx),
        8817357 => {
            init_skree_particle(j, 6, 320, -769, ctx);
        }
        8817393 => {
            init_skree_particle(j, 6, 96, -1025, ctx);
        }
        8817429 => {
            init_skree_particle(j, -6, -320, -769, ctx);
        }
        8817465 => {
            init_skree_particle(j, -6, -96, -1025, ctx);
        }
        _ => {
            ctx.unreachable();
        }
    }
}
pub(super) fn Eproj_Init_0x8aaf(j: u16, ctx: &mut Execution<'_>) {
    ctx.next_random();
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: u16 = (ctx.enemy_y(v1 as u16) as i32 + 12) as u16;
    let v3: i32 = j as i32 >> 1;

    store(v2, F::Y, (v3) as isize, ctx);
    store((v2 as i32 + 72) as u16, F::Counter, (v3) as isize, ctx);
    store(ctx.enemy_ai_d(v1 as u16), F::VelocityY, (v3) as isize, ctx);
    store(
        (ctx.enemy_x(v1 as u16) as i32 + (ctx.random_number() as i32 & 0x1f) - 16i32) as u16,
        F::X,
        (v3) as isize,
        ctx,
    );
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x868a75) as usize,
            ((ctx.slot_byte(F::VelocityY, v3 as isize * 2 + 1) as u16 & 6) >> 1) as isize,
        ),
        F::Program,
        (v3) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_DraygonsWallTurretProjs(j: u16, ctx: &mut Execution<'_>) {
    Eproj_AngleToSamus(j, ctx.state().spawn.point.x, ctx.state().spawn.point.y, ctx);
    let v1: i32 = j as i32 >> 1;

    store(2560, F::Graphics, (v1) as isize, ctx);
    store((0x868d54 & 0xffff) as u16, F::PreAction, (v1) as isize, ctx);
}

pub(super) fn EprojInit_9642_RidleysFireball(j: u16, ctx: &mut Execution<'_>) {
    let slot = usize::from(j >> 1);
    let parameter = ctx.state().spawn.secondary;
    ctx.set_slot(F::BehaviorB, (slot) as isize, parameter);
    let v2: i16 = (if ctx.state().spawn.primary != 0 {
        25
    } else {
        -25i32
    }) as i16;
    let x = (ctx.enemy_x(0)).wrapping_add((v2) as u16);
    ctx.set_slot(F::X, (slot) as isize, x);
    let y = (ctx.enemy_y(0)).wrapping_sub(43);
    ctx.set_slot(F::Y, (slot) as isize, y);
    ctx.set_slot(F::Graphics, (slot) as isize, 2560);
    let x_velocity = ctx.ridley_fireball_x_velocity();
    ctx.set_slot(F::VelocityX, (slot) as isize, x_velocity);
    let y_velocity = ctx.ridley_fireball_y_velocity();
    ctx.set_slot(F::VelocityY, (slot) as isize, y_velocity);
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, ctx);
}

pub(super) fn EprojInit_PhantoonDestroyableFireballs(j: u16, ctx: &mut Execution<'_>) {
    let mut v1: i8 = ((ctx.state().spawn.primary & 0xff00) >> 8) as i8;
    match v1 as i32 {
        0 => {
            let v2: i32 = j as i32 >> 1;

            store(0, F::XFraction, (v2) as isize, ctx);
            store(0, F::YFraction, (v2) as isize, ctx);
            store(0, F::VelocityX, (v2) as isize, ctx);
            store(0, F::VelocityY, (v2) as isize, ctx);

            store(ctx.enemy_x(0), F::X, (v2) as isize, ctx);
            store(
                (ctx.enemy_y(0) as i32 + 32) as u16,
                F::Y,
                (v2) as isize,
                ctx,
            );
            store(addr_off_8697B4 as u16, F::Program, (v2) as isize, ctx);
            store(
                (ctx.slot(F::Properties, (v2) as isize) & 0xfff | 0x2000),
                F::Properties,
                (v2) as isize,
                ctx,
            );
        }
        2 => {
            let v3: i32 = j as i32 >> 1;

            store(0, F::XFraction, (v3) as isize, ctx);
            store(0, F::YFraction, (v3) as isize, ctx);
            store(0, F::VelocityY, (v3) as isize, ctx);

            let v4: u16 = ctx.state().spawn.primary as u8 as u16;
            if ((ctx.state().spawn.primary as u8 as i32 - 8i32) as i16) >= 0 {
                store(-2i32 as u16, F::VelocityX, (v3) as isize, ctx);
            } else {
                store(2, F::VelocityX, (v3) as isize, ctx);
            }
            {
                store(
                    byte_8698B4[v4 as usize] as u16,
                    F::BehaviorA,
                    (v3) as isize,
                    ctx,
                );
                store(ctx.enemy_x(0), F::X, (v3) as isize, ctx);
                store(
                    (ctx.enemy_y(0) as i32 + 32) as u16,
                    F::Y,
                    (v3) as isize,
                    ctx,
                );
                store((0x869a45 & 0xffff) as u16, F::PreAction, (v3) as isize, ctx);
            }
        }
        4 => {
            let v5: i32 = j as i32 >> 1;

            store(0, F::XFraction, (v5) as isize, ctx);
            store(0, F::YFraction, (v5) as isize, ctx);
            store(0, F::VelocityY, (v5) as isize, ctx);

            store(
                ((ctx.state().spawn.primary as i32 & 0xf0) >> 1) as u16,
                F::VelocityX,
                (v5) as isize,
                ctx,
            );
            store(
                byte_8698F7[(ctx.state().spawn.primary as i32 & 0xf) as usize] as u16,
                F::X,
                (v5) as isize,
                ctx,
            );
            store(40, F::Y, (v5) as isize, ctx);
            store((0x869a94 & 0xffff) as u16, F::PreAction, (v5) as isize, ctx);
        }
        6 => {
            let v7: i32 = j as i32 >> 1;

            store(0, F::XFraction, (v7) as isize, ctx);
            store(0, F::YFraction, (v7) as isize, ctx);
            store(0, F::VelocityY, (v7) as isize, ctx);

            store(128, F::VelocityX, (v7) as isize, ctx);
            store(
                byte_869979[ctx.state().spawn.primary as u8 as usize] as u16,
                F::BehaviorA,
                (v7) as isize,
                ctx,
            );
            store(ctx.enemy_x(0), F::X, (v7) as isize, ctx);
            store(
                (ctx.enemy_y(0) as i32 + 16) as u16,
                F::Y,
                (v7) as isize,
                ctx,
            );
            store((0x869ada & 0xffff) as u16, F::PreAction, (v7) as isize, ctx);
        }
        _ => {
            ctx.unreachable();
        }
    };
}

pub(super) fn EprojInit_RocksKraidSpits(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();
    let v2: i32 = j as i32 >> 1;
    store(
        (ctx.enemy_x(enemy_index)).wrapping_add(16),
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(enemy_index)).wrapping_sub(96),
        F::Y,
        (v2) as isize,
        ctx,
    );

    store(0, F::YFraction, (v2) as isize, ctx);
    store(0, F::XFraction, (v2) as isize, ctx);

    store(ctx.state().spawn.primary, F::VelocityX, (v2) as isize, ctx);
    store(-1024i32 as u16, F::VelocityY, (v2) as isize, ctx);
    store(1536, F::Graphics, (v2) as isize, ctx);
}

pub(super) fn EprojInit_RocksWhenKraidRises(j: u16, ctx: &mut Execution<'_>) {
    let mut v1: i16 = ((ctx.random_number()) as i16) & (0x3fi16);
    if (ctx.random_number() & 1) == 0 {
        v1 = !(v1 as i32) as i16;
    }
    let v2: i32 = j as i32 >> 1;
    store(
        ({
            let enemy_index = ctx.current_enemy();
            ctx.enemy_x(enemy_index)
        })
        .wrapping_add((v1) as u16),
        F::X,
        (v2) as isize,
        ctx,
    );
    store(432, F::Y, (v2) as isize, ctx);

    store(0, F::YFraction, (v2) as isize, ctx);
    store(0, F::XFraction, (v2) as isize, ctx);

    store(ctx.state().spawn.primary, F::VelocityX, (v2) as isize, ctx);
    store(-1280i32 as u16, F::VelocityY, (v2) as isize, ctx);
    store(1536, F::Graphics, (v2) as isize, ctx);

    ctx.queue_sound(3, 6, 0x1e);
}

pub(super) fn EprojInit_WalkingLavaSeahorseFireball(j: u16, ctx: &mut Execution<'_>) {
    static word_869EF9: [i16; 3] = [-0x100i16, 0, 0x100];
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: i32 = j as i32 >> 1;
    store(
        (ctx.enemy_y(v1 as u16) as i32 - 12i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(-1024i32 as u16, F::VelocityX, (v2) as isize, ctx);
    store(
        (ctx.enemy_x(v1 as u16) as i32 - 16i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    if (ctx.enemy_ai_d(v1 as u16) & 0x8000) == 0 {
        store(1024, F::VelocityX, (v2) as isize, ctx);
        store(
            (ctx.enemy_x(v1 as u16) as i32 + 16) as u16,
            F::X,
            (v2) as isize,
            ctx,
        );
    }
    store(
        word_869EF9[(ctx.state().spawn.primary >> 1) as usize] as u16,
        F::VelocityY,
        (v2) as isize,
        ctx,
    );

    store(0, F::YFraction, (v2) as isize, ctx);
    store(0, F::XFraction, (v2) as isize, ctx);
}

pub(super) fn EprojInit_PirateClaw(j: u16, ctx: &mut Execution<'_>) {
    let r = ctx.state().spawn.rectangle;
    let v1: i32 = j as i32 >> 1;
    store(
        (r.origin.y as i32 + r.extent.y as i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(
        (r.origin.x as i32 + r.extent.x as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (if ctx.state().spawn.primary != 0 {
            addr_off_869FE1 as i32
        } else {
            addr_off_869FB9 as i32
        }) as u16,
        F::Program,
        (v1) as isize,
        ctx,
    );
    store((0x86a05b & 0xffff) as u16, F::PreAction, (v1) as isize, ctx);
    store(2048, F::BehaviorA, (v1) as isize, ctx);
    store(1, F::BehaviorB, (v1) as isize, ctx);
}

pub(super) fn EprojInit_BombTorizoLowHealthDrool(j: u16, ctx: &mut Execution<'_>) {
    let mut v5: u16 = 0;
    let v1: i32 = j as i32 >> 1;
    store(0, F::Graphics, (v1) as isize, ctx);
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86a64d) as usize,
            (((ctx.next_random() >> 1) & (0xeu16)) >> 1) as isize,
        ),
        F::Program,
        (v1) as isize,
        ctx,
    );
    ctx.next_random();
    let v2 = (ctx.current_enemy() as i32 as u16);
    store(
        (ctx.enemy_y(v2 as u16)).wrapping_sub(5),
        F::Y,
        (v1) as isize,
        ctx,
    );
    let mut parameter_1: i16 = ctx.enemy_parameter_1(v2 as u16) as i16;
    let mut v4: u16 = if (parameter_1 as u16 & 0x4000) != 0 {
        (ctx.random_number() & 0x1fe)
    } else {
        if parameter_1 < 0 {
            v5 = 32;
        } else {
            v5 = 224;
        }
        (2 * (v5 as i32 + (ctx.random_number() as i32 & 0xf) - 8i32)) as u16
    };
    let v6: i32 = v4 as i32 >> 1;
    store(
        kSinCosTable8bit_Sext[(v6 + 64) as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        kSinCosTable8bit_Sext[v6 as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    let v7 = (ctx.current_enemy() as i32 as u16);
    let mut v8: u16 = if (ctx.enemy_parameter_1(v7 as u16) & 0x8000) != 0 {
        (ctx.enemy_x(v7 as u16)).wrapping_add(8)
    } else {
        (ctx.enemy_x(v7 as u16) as i32 - 8i32) as u16
    };
    store(v8, F::X, (v1) as isize, ctx);
}

pub(super) fn EprojInit_BombTorizoExplosiveSwipe(j: u16, ctx: &mut Execution<'_>) {
    let enemy = ctx.current_enemy();
    let mirrored = ctx.enemy_parameter_1(enemy) & 0x8000 != 0;
    let variant = (ctx.state().spawn.primary >> 1) as usize;
    let slot = (j >> 1) as isize;
    let x = ctx.enemy_x(enemy);
    let x_offset = kEprojInit_BombTorizoExplosiveSwipe_Tab0[variant] as u16;
    let x = if mirrored {
        x.wrapping_sub(x_offset)
    } else {
        x.wrapping_add(x_offset)
    };
    store(x, F::X, slot, ctx);
    let y = ctx.enemy_y(enemy);
    let y_offset = kEprojInit_BombTorizoExplosiveSwipe_Tab1[variant] as u16;
    store(y.wrapping_add(y_offset), F::Y, slot, ctx);
}

pub(super) fn EprojInit_BombTorizoStatueBreaking(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let mut v1: i8 = ctx.state().spawn.primary as i8;
    let v2: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86a7ab) as usize, v2 as isize),
        F::Program,
        (v3) as isize,
        ctx,
    );
    store(
        (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize] as i32
            + 16 * room_mechanisms::block_x(ctx) as i32) as u16,
        F::X,
        (v3) as isize,
        ctx,
    );
    let v4: i32 = (v1 as i32 & 0xf) >> 1;
    store(
        (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize] as i32
            + 16 * room_mechanisms::block_y(ctx) as i32) as u16,
        F::Y,
        (v3) as isize,
        ctx,
    );
    store(
        kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as u16,
        F::VelocityY,
        (v3) as isize,
        ctx,
    );
    store(
        kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as u16,
        F::BehaviorB,
        (v3) as isize,
        ctx,
    );
    {
        let index = (v3 as usize) as isize;
        let old = ctx.slot(F::Properties, index);
        ctx.set_slot(F::Properties, index, old | (0x1000));
    };
}

pub(super) fn EprojInit_AB07(j: u16, ctx: &mut Execution<'_>) {
    let mut v1: u16 = ctx.vram_write_queue_tail();
    let v2 = ctx.vram_write_queue_tail();
    ctx.set_vram_queue_size(v2, 64);
    ctx.set_vram_queue_source(v2, addr_kEprojInit_AB07_Tile0 as u16);
    ctx.set_vram_queue_bank_word(v2, 134);
    ctx.set_vram_queue_destination(v2, 28160);
    v1 = (v1 as i32 + 7) as u16;
    let v3 = v1;
    ctx.set_vram_queue_size(v3, 64);
    ctx.set_vram_queue_source(v3, addr_kEprojInit_AB07_Tile1 as u16);
    ctx.set_vram_queue_bank_word(v3, 134);
    ctx.set_vram_queue_destination(v3, 28416);
    {
        let value = (v1 as i32 + 7) as u16;
        ctx.set_vram_write_queue_tail(value);
    };
    let v4: i32 = j as i32 >> 1;
    store(ctx.player_x(), F::X, (v4) as isize, ctx);
    store(
        (ctx.player_y() as i32 - 36i32) as u16,
        F::Y,
        (v4) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_GoldenTorizosChozoOrbs(j: u16, ctx: &mut Execution<'_>) {
    let E = (ctx.current_enemy() as i32 as u16);
    let v2: u16 = (if (ctx.enemy_parameter_1(E as u16) & 0x8000) != 0 {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: ctx.enemy_x(E as u16),
            y: ctx.enemy_y(E as u16),
        },
        ctx,
    );
}

pub(super) fn EprojInit_TorizoSonicBoom(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = if (ctx.next_random() & 1) != 0 {
        -12i32
    } else {
        20
    };
    let v2 = (ctx.current_enemy() as i32 as u16);
    let v3: i32 = j as i32 >> 1;

    store(
        (ctx.enemy_y(v2 as u16) as i32 + v1) as u16,
        F::Y,
        (v3) as isize,
        ctx,
    );
    store(0, F::VelocityY, (v3) as isize, ctx);

    if (ctx.enemy_parameter_1(v2 as u16) & 0x8000) != 0 {
        store(
            (ctx.enemy_x(v2 as u16) as i32 + 32) as u16,
            F::X,
            (v3) as isize,
            ctx,
        );
        store(624, F::VelocityX, (v3) as isize, ctx);
        store(addr_off_86ADD2 as u16, F::Program, (v3) as isize, ctx);
    } else {
        store(
            (ctx.enemy_x(v2 as u16) as i32 - 32i32) as u16,
            F::X,
            (v3) as isize,
            ctx,
        );
        store(-624i32 as u16, F::VelocityX, (v3) as isize, ctx);
        store(addr_off_86ADBF as u16, F::Program, (v3) as isize, ctx);
    };
}

pub(super) fn EprojInit_WreckedShipChozoSpikeFootsteps(j: u16, ctx: &mut Execution<'_>) {
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: i32 = j as i32 >> 1;
    store(
        (ctx.state().spawn.primary as i32 + ctx.enemy_x(v1 as u16) as i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(v1 as u16) as i32 + 28) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_GoldenTorizoEgg(j: u16, ctx: &mut Execution<'_>) {
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: i32 = j as i32 >> 1;

    store(
        ((0xe2 & 0x1f) + 64) as u16,
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.enemy_parameter_1(v1 as u16),
        F::BehaviorA,
        (v2) as isize,
        ctx,
    );

    let v4: u16 = (if ctx.enemy_parameter_1(v1 as u16) & 0x8000 != 0 {
        addr_kEprojInit_GoldenTorizoEgg0 as i32
    } else {
        addr_kEprojInit_GoldenTorizoEgg1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v4,
        Point16U {
            x: ctx.enemy_x(v1 as u16),
            y: ctx.enemy_y(v1 as u16),
        },
        ctx,
    );
}

pub(super) fn EprojInit_GoldenTorizoEyeBeam(j: u16, ctx: &mut Execution<'_>) {
    let E = (ctx.current_enemy() as i32 as u16);
    let v2: u16 = (if (ctx.enemy_parameter_1(E as u16) & 0x8000) != 0 {
        addr_stru_86B376 as i32
    } else {
        addr_stru_86B380 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: ctx.enemy_x(E as u16),
            y: ctx.enemy_y(E as u16),
        },
        ctx,
    );
    let mut v3: u16 = ((ctx.next_random() as i32 & 0x1e) - 16i32 + 192) as u16;
    if (ctx.enemy_parameter_1(E as u16) & 0x8000) == 0 {
        v3 = (v3 as i32 + 128) as u16;
    }
    let v4: i32 = v3 as i32 >> 1;
    let v5: i32 = j as i32 >> 1;
    store(
        (8 * kSinCosTable8bit_Sext[(v4 + 64) as usize] as i32) as u16,
        F::VelocityX,
        (v5) as isize,
        ctx,
    );
    store(
        (8 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16,
        F::VelocityY,
        (v5) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_LavaSeahorseFireball(j: u16, ctx: &mut Execution<'_>) {
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: i32 = j as i32 >> 1;
    store(
        (ctx.enemy_y(v1 as u16)).wrapping_sub(28),
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(-961i32 as u16, F::VelocityY, (v2) as isize, ctx);
    if (ctx.enemy_ai_a(v1 as u16) & 0x8000) == 0 {
        store(
            (ctx.enemy_x(v1 as u16) as i32 + 12) as u16,
            F::X,
            (v2) as isize,
            ctx,
        );
        store(704, F::VelocityX, (v2) as isize, ctx);
        store(addr_word_86B4CB as u16, F::Program, (v2) as isize, ctx);
    } else {
        store(
            (ctx.enemy_x(v1 as u16)).wrapping_sub(12),
            F::X,
            (v2) as isize,
            ctx,
        );
        store(-704i32 as u16, F::VelocityX, (v2) as isize, ctx);
        store(addr_word_86B4BF as u16, F::Program, (v2) as isize, ctx);
    };
}

pub(super) fn EprojInit_EyeDoorProjectile(j: u16, ctx: &mut Execution<'_>) {
    let v1: u16 = room_mechanisms::selected(ctx);
    let v2: i32 = j as i32 >> 1;
    store(
        room_mechanisms::projectile_argument(room_mechanisms::selected(ctx), ctx),
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    calculate_coordinates(v1, ctx);
    let v3: i32 = ctx.state().spawn.primary as i32 >> 1;

    store(
        (word_86B65B[v3 as usize] as i32 + 8 * (2 * room_mechanisms::block_x(ctx) as i32 + 1))
            as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (word_86B65B[(v3 + 1) as usize] as i32 + 16 * room_mechanisms::block_y(ctx) as i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_EyeDoorSweat(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v1: i32 = j as i32 >> 1;
    store(
        (8 * (2 * (room_mechanisms::block_x(ctx) as i32 - 1i32) + 1)) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (16 * (room_mechanisms::block_y(ctx) as i32 + 1)) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    let v2: i32 = ctx.state().spawn.primary as i32 >> 1;
    store(
        word_86B6B1[v2 as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        word_86B6B1[(v2 + 1) as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_TourianStatueEyeGlow(j: u16, ctx: &mut Execution<'_>) {
    let v1: u16 = ctx.state().spawn.primary;
    let v2: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    store(
        kEprojInit_TourianStatueEyeGlow_X[v2 as usize],
        F::X,
        (v3) as isize,
        ctx,
    );
    store(
        kEprojInit_TourianStatueEyeGlow_Y[v2 as usize],
        F::Y,
        (v3) as isize,
        ctx,
    );
    let mut v4: u16 = (4 * v1 as i32) as u16;
    for i in (498i32..506).step_by(2) {
        {
            let value = kEprojInit_TourianStatueEyeGlow_Colors[(v4 >> 1) as usize];
            ctx.set_palette_buffer((i >> 1) as isize, value);
        };
        v4 = (v4 as i32 + 2) as u16;
    }
}

pub(super) fn EprojInit_TourianStatueUnlockingParticle(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    store(ctx.slot(F::X, (v1) as isize), F::X, (v2) as isize, ctx);
    store(ctx.slot(F::Y, (v1) as isize), F::Y, (v2) as isize, ctx);
    let v3: u16 = (2 * ((ctx.next_random() as i32 & 0x3f) - 32i32) as u8 as i32) as u16;
    store(v3, F::BehaviorA, (v2) as isize, ctx);
    let v4: i32 = v3 as i32 >> 1;
    store(
        kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16,
        F::VelocityX,
        (v2) as isize,
        ctx,
    );
    store(
        (4 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16,
        F::VelocityY,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_TourianStatueSoul(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    store(
        kEprojInit_TourianStatueEyeGlow_X[v1 as usize],
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        kEprojInit_TourianStatueEyeGlow_Y[v1 as usize],
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(-1024i32 as u16, F::VelocityY, (v2) as isize, ctx);
}

pub(super) fn EprojInit_NuclearWaffleBody(j: u16, ctx: &mut Execution<'_>) {
    let v2 = (ctx.current_enemy() as i32 as u16);
    let v3: i32 = j as i32 >> 1;

    copy_enemy_position(v3 as isize, v2, 0, ctx);

    let v4 = ctx.current_enemy();
    let linked = (ctx.current_enemy() as i32 + ctx.nuclear_waffle_part_offset(v4) as i32) as u16;
    ctx.set_enemy_linked_projectile(linked, j);
    store(1, F::DropFlags, (v3) as isize, ctx);
}

pub(super) fn EprojInit_ShaktoolAttackMiddleBackCircle(j: u16, ctx: &mut Execution<'_>) {
    store(
        ctx.state().spawn.primary,
        F::BehaviorA,
        (j >> 1) as isize,
        ctx,
    );
    EprojInit_BDA2(j, ctx);
}

pub(super) fn EprojInit_MotherBrainBomb(j: u16, ctx: &mut Execution<'_>) {
    ctx.set_slot_byte(F::XFraction, j as isize, ctx.state().spawn.primary as u8);
    let v1: i32 = j as i32 >> 1;

    store(256, F::VelocityY, (v1) as isize, ctx);
    store(224, F::VelocityX, (v1) as isize, ctx);
    store(
        (ctx.enemy_x(64) as i32 + 12) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(64) as i32 + 16) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(1024, F::Graphics, (v1) as isize, ctx);
    store(112, F::BehaviorA, (v1) as isize, ctx);
    store(0, F::BehaviorB, (v1) as isize, ctx);

    let count = ctx.mother_brain_active_bombs();
    ctx.set_mother_brain_active_bombs(count.wrapping_add(1));
}

pub(super) fn EprojInit_MotherBrainEscapeDoorParticles(j: u16, ctx: &mut Execution<'_>) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [i16; 16] = [
        0, -0x20i16, 0, -0x18i16, 0, -0x10i16, 0, -8i16, 0, 0, 0, 8, 0, 0x10, 0, 0x18,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [i16; 16] = [
        0x500, -0x200i16, 0x500, -0x100i16, 0x500, -0x100i16, 0x500, -0x80i16, 0x500, -0x80i16,
        0x500, 0x80, 0x500, -0x100i16, 0x500, 0x200,
    ];
    let v1: i32 = j as i32 >> 1;
    store(0, F::Graphics, (v1) as isize, ctx);
    let v2: i32 = 4 * ctx.state().spawn.primary as i32 >> 1;

    store(
        (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize] as i32 + 16) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (kEprojInit_MotherBrainEscapeDoorParticles_X[(v2 + 1) as usize] as i32 + 128) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(
        kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        kEprojInit_MotherBrainEscapeDoorParticles_Xvel[(v2 + 1) as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    store(32, F::BehaviorA, (v1) as isize, ctx);
}

pub(super) fn EprojInit_TimeBombSetJapaneseText(j: u16, ctx: &mut Execution<'_>) {
    store(0, F::Graphics, (j >> 1) as isize, ctx);
    EprojPreInstr_TimeBombSetJapaneseText(j, ctx);
}

pub(super) fn EprojInit_MotherBrainGlassShatteringShard(j: u16, ctx: &mut Execution<'_>) {
    let v1: u16 = (2 * ctx.next_random() as i32 & 0x1fe) as u16;
    let v2: i32 = j as i32 >> 1;
    store(v1, F::BehaviorA, (v2) as isize, ctx);
    let v3: i32 = v1 as i32 >> 1;
    store(
        kSinCosTable8bit_Sext[(v3 + 64) as usize] as u16,
        F::VelocityX,
        (v2) as isize,
        ctx,
    );
    store(
        (4 * kSinCosTable8bit_Sext[v3 as usize] as i32) as u16,
        F::VelocityY,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86ce41) as usize,
            ((v1 as i32 >> 4 & 0x1e) as u16 >> 1) as isize,
        ),
        F::Program,
        (v2) as isize,
        ctx,
    );
    store(1600, F::Graphics, (v2) as isize, ctx);
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v4: i32 = ctx.state().spawn.primary as i32 >> 1;

    store(
        (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize] as i32
            + 16 * room_mechanisms::block_x(ctx) as i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize] as i32
            + 16 * room_mechanisms::block_y(ctx) as i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );

    {
        let index = (v2) as isize;
        let old = ctx.slot(F::X, (index) as isize);
        let value = ((old as i32) + ((ctx.next_random() as i32 & 0xf) - 8i32)) as u16;
        ctx.set_slot(F::X, (index) as isize, value);
    };
    {
        let index = (v2) as isize;
        let old = ctx.slot(F::Y, (index) as isize);
        let value = ((old as i32) + ((ctx.next_random() as i32 & 0xf) - 8i32)) as u16;
        ctx.set_slot(F::Y, (index) as isize, value);
    };
}

pub(super) fn EprojInit_MotherBrainGlassShatteringSparkle(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    store(
        (ctx.slot(F::X, (v1) as isize) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (ctx.slot(F::Y, (v1) as isize) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(1600, F::Graphics, (v2) as isize, ctx);
}

pub(super) fn EprojInit_N00bTubeCrack(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v1: i32 = j as i32 >> 1;
    store(
        (16 * room_mechanisms::block_x(ctx) as i32 + 96) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (16 * room_mechanisms::block_y(ctx) as i32 + 48) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_N00bTubeShards(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    store(
        (kEprojInit_N00bTubeShards_X[v1 as usize] as i32
            + 16 * room_mechanisms::block_x(ctx) as i32
            + 96) as u16,
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    store(0, F::BehaviorA, (v2) as isize, ctx);
    store(
        (kEprojInit_N00bTubeShards_Y[v1 as usize] as i32
            + 16 * room_mechanisms::block_y(ctx) as i32
            + 48) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86d760) as usize, v1 as isize),
        F::Program,
        (v2) as isize,
        ctx,
    );
    store(
        kEprojInit_N00bTubeShards_Xvel[v1 as usize] as u16,
        F::VelocityX,
        (v2) as isize,
        ctx,
    );
    store(
        kEprojInit_N00bTubeShards_Yvel[v1 as usize] as u16,
        F::VelocityY,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_N00bTubeReleasedAirBubbles(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    store(
        (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize] as i32
            + 16 * room_mechanisms::block_x(ctx) as i32) as u16,
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    store(0, F::BehaviorA, (v2) as isize, ctx);
    store(
        (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize] as i32
            + 16 * room_mechanisms::block_y(ctx) as i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(-1280i32 as u16, F::VelocityY, (v2) as isize, ctx);
}

pub(super) fn EprojInit_SaveStationElectricity(j: u16, ctx: &mut Execution<'_>) {
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    let v1: i32 = j as i32 >> 1;
    store(
        (16 * (room_mechanisms::block_x(ctx) as i32 + 1)) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (16 * (room_mechanisms::block_y(ctx) as i32 - 2i32)) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_BotwoonsSpit(j: u16, ctx: &mut Execution<'_>) {
    let v1 = (ctx.current_enemy() as i32 as u16);
    let v2: i32 = j as i32 >> 1;

    store(ctx.enemy_x(v1 as u16), F::X, (v2) as isize, ctx);
    store(ctx.enemy_y(v1 as u16), F::Y, (v2) as isize, ctx);
    store(addr_word_86EBAE as u16, F::Program, (v2) as isize, ctx);
    store(ctx.state().spawn.angle as u16, F::Angle, (v2) as isize, ctx);

    let pt: Point32 = ConvertAngleToXy(
        ctx.state().spawn.angle as u16,
        ctx.state().spawn.primary,
        ctx,
    );

    store((pt.x >> 16) as u16, F::VelocityX, (v2) as isize, ctx);
    store(pt.x as u16, F::BehaviorA, (v2) as isize, ctx);
    store((pt.y >> 16) as u16, F::VelocityY, (v2) as isize, ctx);
    store(pt.y as u16, F::BehaviorB, (v2) as isize, ctx);
}

pub(super) fn EprojInit_Sparks(j: u16, ctx: &mut Execution<'_>) {
    static word_86F3D4: [u16; 14] = [
        0xffff, 0xb800, 0xffff, 0xc000, 0xffff, 0xe000, 0xffff, 0xff00, 0, 0x100, 0, 0x2000, 0,
        0x4000,
    ];
    let v2: i32 = j as i32 >> 1;
    store(addr_kEnemyDef_F353 as u16, F::Program, (v2) as isize, ctx);
    let v3 = (ctx.current_enemy() as i32 as u16);

    copy_enemy_position(v2 as isize, v3, 8, ctx);

    store(0, F::VelocityX, (v2) as isize, ctx);
    store(0, F::VelocityY, (v2) as isize, ctx);

    let v4: u16 = ((ctx.next_random() as i32 & 0x1c) >> 1) as u16;
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86f3d4) as usize,
            (v4 as i32 + 0) as isize,
        ),
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86f3d4) as usize,
            (v4 as i32 + 1) as isize,
        ),
        F::BehaviorA,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_9660_FireballExplosion(j: u16, ctx: &mut Execution<'_>) {
    let slot = usize::from(j >> 1);
    ctx.set_slot(F::Graphics, (slot) as isize, 2560);
    let x = ctx.state().spawn.point.x;
    ctx.set_slot(F::X, (slot) as isize, x);
    let y = ctx.state().spawn.point.y;
    ctx.set_slot(F::Y, (slot) as isize, y);
    let parameter = ctx.state().spawn.primary;
    ctx.set_slot(F::BehaviorA, (slot) as isize, parameter);
    ctx.set_slot(F::BehaviorB, (slot) as isize, 0);
    ctx.set_slot(F::VelocityX, (slot) as isize, 0);
    ctx.set_slot(F::VelocityY, (slot) as isize, 0);
}

pub(super) fn EprojInit_9734_CeresFallingDebris(j: u16, ctx: &mut Execution<'_>) {
    let slot = usize::from(j >> 1);
    ctx.set_slot(F::BehaviorA, (slot) as isize, 0);
    ctx.set_slot(F::BehaviorB, (slot) as isize, 0);
    ctx.set_slot(F::VelocityX, (slot) as isize, 0);
    ctx.set_slot(F::Graphics, (slot) as isize, 3584);
    let x = ctx.state().spawn.primary;
    ctx.set_slot(F::X, (slot) as isize, x);
    ctx.set_slot(F::Y, (slot) as isize, 42);
    ctx.set_slot(F::VelocityY, (slot) as isize, 16);
}

pub(super) fn EprojInit_CeresElevatorPad(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;
    let eproj_y_pos: u16 = ctx.player_y().wrapping_add(28);
    ctx.set_slot(F::Y, (v1 as usize) as isize, eproj_y_pos);
    ctx.set_slot(F::BehaviorA, (v1 as usize) as isize, 60);
    sub_86A301(j, ctx);
}

pub(super) fn EprojInit_CeresElevatorPlatform(j: u16, ctx: &mut Execution<'_>) {
    let eproj_word_index = (j >> 1) as usize;
    ctx.set_slot(F::Y, (eproj_word_index) as isize, 97);
    sub_86A301(j, ctx);
}

pub(super) fn EprojPreInstr_PrePhantomRoom(j: u16, ctx: &mut Execution<'_>) {
    {
        let value = 0;
        ctx.set_bg2_y_scroll(value);
    };
}

pub(super) fn sub_86BB30(j: u16, ctx: &mut Execution<'_>) {
    let v1 = ctx.current_enemy();
    let v2: i32 = j as i32 >> 1;
    store(ctx.enemy_projectile_origin_x(v1), F::X, (v2) as isize, ctx);
    store(ctx.enemy_projectile_origin_y(v1), F::Y, (v2) as isize, ctx);
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86bb1e) as usize,
            ctx.state().spawn.primary as isize,
        ),
        F::Program,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn Eproj_MotherBrainsBlueRingLasers(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(8, F::BehaviorA, (v1) as isize, ctx);
    store(0, F::BehaviorB, (v1) as isize, ctx);
    store(1024, F::Graphics, (v1) as isize, ctx);

    let r18: u16 = ctx.state().spawn.primary;

    store(Math_MultBySin(0x450, r18), F::VelocityX, (v1) as isize, ctx);
    store(Math_MultByCos(0x450, r18), F::VelocityY, (v1) as isize, ctx);
    store(
        (ctx.enemy_x(64) as i32 + 10) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(64) as i32 + 16) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );

    sub_86C320(j, ctx);
}

pub(super) fn sub_86C605(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::BehaviorA, (v1) as isize, ctx);
    store(0, F::BehaviorB, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);
    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);

    ctx.set_mother_brain_beam_x_fraction(0);
    ctx.set_mother_brain_beam_y_fraction(0);
    store(1024, F::Graphics, (v1) as isize, ctx);
    ctx.set_slot(F::X, v1 as isize, (ctx.enemy_x(0) as i32 + 64) as u16);
    ctx.set_mother_brain_beam_x(ctx.slot(F::X, v1 as isize));
    let x: u16 = (ctx.player_x() as i32 - ctx.mother_brain_beam_x() as i32) as u16;
    ctx.set_slot(F::Y, v1 as isize, (ctx.enemy_y(0) as i32 - 48i32) as u16);
    ctx.set_mother_brain_beam_y(ctx.slot(F::Y, v1 as isize));
    let y: u16 = (ctx.player_y() as i32 - ctx.mother_brain_beam_y() as i32) as u16;
    let r18: i32 = -(ctx.calculate_angle_from_xy(x, y) as i32 + 0x80i32) as u8 as i32;
    ctx.set_mother_brain_beam_angle(r18 as u16);
    ctx.set_mother_brain_beam_velocity_x(Math_MultBySin(0xc00, r18 as u16));
    ctx.set_mother_brain_beam_velocity_y(Math_MultByCos(0xc00, r18 as u16));
}

pub(super) fn EprojIni_TourianStatueUnlockingParticleTail(v0: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v2: i32 = v0 as i32 >> 1;
    store(ctx.slot(F::X, (v1) as isize), F::X, (v2) as isize, ctx);
    store(ctx.slot(F::Y, (v1) as isize), F::Y, (v2) as isize, ctx);
}

pub(super) fn sub_86D992(v0: u16, ctx: &mut Execution<'_>) {
    let v1: u16 = ctx.state().spawn.primary;
    let v2: i32 = v0 as i32 >> 1;

    store(ctx.state().spawn.primary, F::BehaviorA, (v2) as isize, ctx);
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86d96a) as usize,
            (v1 >> 1) as isize,
        ),
        F::Program,
        (v2) as isize,
        ctx,
    );

    let enemy_index = ctx.current_enemy();

    copy_enemy_position(v2 as isize, enemy_index, 0, ctx);
    store(-512i32 as u16, F::VelocityY, (v2) as isize, ctx);
    store(512, F::VelocityX, (v2) as isize, ctx);

    if ctx.state().spawn.primary as i32 - 12i32 & 0x8000 == 0 {
        store(-384i32 as u16, F::VelocityY, (v2) as isize, ctx);
        store(384, F::VelocityX, (v2) as isize, ctx);
    }
}

pub(super) fn EprojInit_DustCloudOrExplosion(v0: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = v0 as i32 >> 1;
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86e42c) as usize,
            ctx.state().spawn.primary as isize,
        ),
        F::Program,
        (v1) as isize,
        ctx,
    );
    store(ctx.state().spawn.point.x, F::X, (v1) as isize, ctx);
    store(ctx.state().spawn.point.y, F::Y, (v1) as isize, ctx);
}

pub(super) fn EprojInit_DraygonsGunk(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(ctx.state().spawn.point.x, F::X, (v1) as isize, ctx);
    store(ctx.state().spawn.point.y, F::Y, (v1) as isize, ctx);
    store(ctx.state().spawn.secondary, F::Angle, (v1) as isize, ctx);
    let pt: Point32 = ConvertAngleToXy(ctx.state().spawn.secondary, ctx.state().spawn.primary, ctx);

    store((pt.x >> 16) as u16, F::VelocityX, (v1) as isize, ctx);
    store(pt.x as u16, F::BehaviorA, (v1) as isize, ctx);
    store((pt.y >> 16) as u16, F::VelocityY, (v1) as isize, ctx);
    store(pt.y as u16, F::BehaviorB, (v1) as isize, ctx);
    store(1024, F::Graphics, (v1) as isize, ctx);
}

pub(super) fn EprojInit_CrocomireSpikeWallPieces(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x869105) as usize,
            ((j as i32 - 20i32) as u16 >> 1) as isize,
        ),
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(528, F::X, (v1) as isize, ctx);

    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);
    store(0, F::BehaviorA, (v1) as isize, ctx);
    store(0, F::BehaviorB, (v1) as isize, ctx);
    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);

    store(-5i32 as u16, F::VelocityY, (v1) as isize, ctx);
    store(-30720i32 as u16, F::BehaviorB, (v1) as isize, ctx);
}

pub(super) fn EprojInit_CrocomireBridgeCrumbling(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(ctx.state().spawn.primary, F::X, (v1) as isize, ctx);
    store(187, F::Y, (v1) as isize, ctx);

    store(0, F::YFraction, (v1) as isize, ctx);
    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);

    store(
        ((ctx.random_number() as i32 & 0x3f) + 64) as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    store(1024, F::Graphics, (v1) as isize, ctx);
}

pub(super) fn EprojInit_9634(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::BehaviorA, (v1) as isize, ctx);
    store(0, F::BehaviorB, (v1) as isize, ctx);

    store(
        (ctx.enemy_x(0) as i32 - 29i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(0) as i32 - 35i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(2560, F::Graphics, (v1) as isize, ctx);

    let v2: i32 = ctx.enemy_parameter_1(0) as i32;
    store(
        kEprojInit_9634_Xvel[v2 as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        kEprojInit_9634_Yvel[v2 as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_PhantoonStartingFireballs(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    let v2: u16 = byte_869979[ctx.state().spawn.primary as usize] as u16;
    store(v2, F::BehaviorA, (v1) as isize, ctx);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, 0x30);

    store(
        (pt.x as i32 + ctx.enemy_x(0) as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (pt.y as i32 + ctx.enemy_y(0) as i32 + 16) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_RocksFallingKraidCeiling(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(ctx.state().spawn.primary, F::X, (v1) as isize, ctx);
    store(312, F::Y, (v1) as isize, ctx);

    store(0, F::YFraction, (v1) as isize, ctx);
    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);

    store(
        (ctx.random_number() & 0x3f).wrapping_add(64),
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    store(1536, F::Graphics, (v1) as isize, ctx);
}

pub(super) fn EprojInit_A379(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    let v2: u16 = ctx.state().spawn.primary;
    let v3: i32 = ctx.state().spawn.primary as i32 >> 1;

    store(
        (word_86A2D6[v3 as usize] as i32 + ctx.player_x() as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.player_y() as i32 + 80) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86a2e2) as usize, v3 as isize),
        F::Program,
        (v1) as isize,
        ctx,
    );
    store(v2, F::BehaviorA, (v1) as isize, ctx);
}

pub(super) fn EprojInit_BombTorizoLowHealthInitialDrool(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    ctx.next_random();
    let v2 = (ctx.current_enemy() as i32 as u16);

    store(
        ((ctx.enemy_y(v2 as u16) as i32 + (ctx.random_number() as i32 & 3)) as u16).wrapping_sub(5),
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.random_number() & 0x1f).wrapping_add(48),
        F::VelocityY,
        (v1) as isize,
        ctx,
    );

    ctx.next_random();
    let mut v3: i16 = ((ctx.random_number()) as i16) & (3i16);
    let mut parameter_1: i16 = ctx.enemy_parameter_1(v2 as u16) as i16;
    if (parameter_1 as u16 & 0x4000) != 0 {
        store(
            (ctx.enemy_x(v2 as u16) as i32 + v3 as i32) as u16,
            F::X,
            (v1) as isize,
            ctx,
        );
        store(0, F::VelocityX, (v1) as isize, ctx);
    } else {
        if parameter_1 < 0 {
            store(
                ((ctx.enemy_x(v2 as u16) as i32 + v3 as i32) as u16).wrapping_add(8),
                F::X,
                (v1) as isize,
                ctx,
            );
        } else {
            store(
                ((ctx.enemy_x(v2 as u16) as i32 + v3 as i32) as u16).wrapping_sub(8),
                F::X,
                (v1) as isize,
                ctx,
            );
        }
        store(0, F::VelocityX, (v1) as isize, ctx);
    };
}

pub(super) fn EprojInit_A977(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    store(
        (16 * room_mechanisms::block_x(ctx) as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (16 * room_mechanisms::block_y(ctx) as i32 - 4i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );

    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);
}

pub(super) fn EprojInit_TourianStatueDustClouds(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(128, F::BehaviorA, (v1) as isize, ctx);
    store(188, F::BehaviorB, (v1) as isize, ctx);
}

pub(super) fn EprojInit_TourianEscapeShaftFakeWallExplode(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(272, F::X, (v1) as isize, ctx);
    store(272, F::BehaviorA, (v1) as isize, ctx);

    store(2184, F::Y, (v1) as isize, ctx);
    store(2184, F::BehaviorB, (v1) as isize, ctx);
}

pub(super) fn EprojInit_TourianStatueUnlockingParticleWaterSplash(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        ctx.slot(F::X, (ctx.state().spawn.primary >> 1) as isize),
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.fx_y_pos() as i32 - 4i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_TourianStatueBaseDecoration(j: u16, ctx: &mut Execution<'_>) {
    init_statue_piece(j, 120, 184, ctx);
}

pub(super) fn EprojInit_TourianStatueRidley(j: u16, ctx: &mut Execution<'_>) {
    init_statue_piece(j, 142, 85, ctx);
}

pub(super) fn EprojInit_TourianStatuePhantoon(j: u16, ctx: &mut Execution<'_>) {
    init_statue_piece(j, 132, 136, ctx);
}

pub(super) fn EprojInit_NorfairLavaquakeRocks(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(addr_word_86BBD5 as u16, F::Program, (v1) as isize, ctx);
    store((0x86bc16 & 0xffff) as u16, F::BehaviorA, (v1) as isize, ctx);
    store(ctx.state().spawn.primary, F::VelocityY, (v1) as isize, ctx);
    store(
        ctx.state().spawn.secondary,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    let v2 = (ctx.current_enemy() as i32 as u16);

    copy_enemy_position(v1 as isize, v2, 0, ctx);
}

pub(super) fn EprojInit_BDA2(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    let v2 = (ctx.current_enemy() as i32 as u16);

    store(ctx.enemy_x(v2 as u16), F::X, (v1) as isize, ctx);
    store(ctx.enemy_y(v2 as u16), F::Y, (v1) as isize, ctx);

    let v3: u16 = (2 * (ctx.enemy_ai_d(v2 as u16) as u8) as i32) as u16;
    let v4: i32 = v3 as i32 >> 1;
    store(
        kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        kSinCosTable8bit_Sext[v4 as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    let v5: i32 = (v3 >> 5) as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = ctx.slot(F::X, (index) as isize);
        let value = ((old as i32)
            + (kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as i32))
            as u16;
        ctx.set_slot(F::X, (index) as isize, value);
    };
    {
        let index = (v1) as isize;
        let old = ctx.slot(F::Y, (index) as isize);
        let value = ((old as i32)
            + (kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as i32))
            as u16;
        ctx.set_slot(F::Y, (index) as isize, value);
    };
}

pub(super) fn EprojInit_MotherBrainRoomTurrets(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(1024, F::Graphics, (v1) as isize, ctx);
    let v2: u16 = (2 * ctx.state().spawn.primary as i32) as u16;
    let v3: u16 = (ctx.image().table_word(
        crate::rtl::lorom_offset(0x86bee1) as usize,
        ctx.state().spawn.primary as isize,
    ) | 0x100);
    store(v3, F::YFraction, (v1) as isize, ctx);
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86beb9) as usize,
            ((2 * v3 as u8 as i32) as u16 >> 1) as isize,
        ),
        F::Program,
        (v1) as isize,
        ctx,
    );
    let v4: i32 = v2 as i32 >> 1;
    store(
        kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86bec9) as usize, v4 as isize),
        F::XFraction,
        (v1) as isize,
        ctx,
    );
    Eproj_SetXvelRandom(j, ctx);
    Eproj_SetYvelRandom(j, ctx);
}

pub(super) fn EprojInit_MotherBrainRoomTurretBullets(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::BehaviorB, (v1) as isize, ctx);
    store(1024, F::Graphics, (v1) as isize, ctx);

    let v2: u16 = (2 * ctx.slot_byte(F::YFraction, (ctx.state().spawn.primary >> 1) as isize * 2)
        as i32) as u16;
    store(v2, F::BehaviorA, (v1) as isize, ctx);
    let v3: i32 = v2 as i32 >> 1;
    let pt: Point16U = Point16U {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as u16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as u16,
    };
    store(
        kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as u16,
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as u16,
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    let v4: i32 = ctx.state().spawn.primary as i32 >> 1;
    store(
        (pt.x as i32 + ctx.slot(F::X, (v4) as isize) as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (pt.y as i32 + ctx.slot(F::Y, (v4) as isize) as i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_MotherBrainDeathBeemFired(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(ctx.mother_brain_beam_x(), F::X, (v1) as isize, ctx);
    store(
        ctx.mother_brain_beam_x_fraction(),
        F::XFraction,
        (v1) as isize,
        ctx,
    );
    store(ctx.mother_brain_beam_y(), F::Y, (v1) as isize, ctx);
    store(
        ctx.mother_brain_beam_y_fraction(),
        F::YFraction,
        (v1) as isize,
        ctx,
    );
    store(
        ctx.mother_brain_beam_velocity_x(),
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        ctx.mother_brain_beam_velocity_y(),
        F::VelocityY,
        (v1) as isize,
        ctx,
    );

    MoveEprojWithVelocity(j, ctx);
    ctx.set_mother_brain_beam_x(ctx.slot(F::X, (v1) as isize));
    ctx.set_mother_brain_beam_x_fraction(ctx.slot(F::XFraction, (v1) as isize));
    ctx.set_mother_brain_beam_y(ctx.slot(F::Y, (v1) as isize));
    ctx.set_mother_brain_beam_y_fraction(ctx.slot(F::YFraction, (v1) as isize));
    let r18: u16 =
        ((ctx.mother_brain_beam_angle() as u8) as i32 + ctx.next_random() as i32) as u8 as u16;
    let rv: u16 = ctx.next_random();
    store(
        Math_MultBySin((rv & 0x700), r18),
        F::VelocityX,
        (v1) as isize,
        ctx,
    );
    store(
        Math_MultByCos((ctx.random_number() & 0x700), r18),
        F::VelocityY,
        (v1) as isize,
        ctx,
    );
    MoveEprojWithVelocity(j, ctx);
    if ctx.slot(F::Y, (v1) as isize) as i32 - 34i32 & 0x8000 != 0
        || ctx.slot(F::Y, (v1) as isize) as i32 - 206i32 & 0x8000 == 0
        || ctx.slot(F::X, (v1) as isize) as i32 - 2i32 & 0x8000 != 0
        || ctx.slot(F::X, (v1) as isize) as i32 - 238i32 & 0x8000 == 0
    {
        store(0, F::Definition, (v1) as isize, ctx);
        ctx.state_mut().spawn.point = super::state::Point {
            x: ctx.slot(F::X, (v1) as isize),
            y: ctx.slot(F::Y, (v1) as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1d, ctx);
        ctx.queue_sound(3, 6, 0x13);
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
    } else {
        store(
            (ctx.slot(F::BehaviorA, (v1) as isize) as i32 + 1 & 3) as u16,
            F::BehaviorA,
            (v1) as isize,
            ctx,
        );

        store(0, F::BehaviorB, (v1) as isize, ctx);
        store(0, F::VelocityX, (v1) as isize, ctx);
        store(0, F::VelocityY, (v1) as isize, ctx);
    };
}

pub(super) fn EprojInit_MotherBrainRainbowBeam(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    EprojPreInstr_MotherBrainRainbowBeam(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsDrool(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    store(ctx.state().spawn.primary, F::BehaviorA, (v1) as isize, ctx);

    EprojPreInstr_C84D(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsDeathExplosion(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86c929) as usize,
            ctx.state().spawn.primary as isize,
        ),
        F::Program,
        (v1) as isize,
        ctx,
    );
    store(1, F::Delay, (v1) as isize, ctx);
    store(0, F::Graphics, (v1) as isize, ctx);
    store(ctx.state().spawn.point.x, F::VelocityX, (v1) as isize, ctx);
    store(ctx.state().spawn.point.y, F::VelocityY, (v1) as isize, ctx);
    EprojPreInit_MotherBrainsDeathExplosion_0(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsRainbowBeamExplosion(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    store(ctx.state().spawn.point.x, F::VelocityX, (v1) as isize, ctx);
    store(
        (ctx.player_x() as i32 + ctx.state().spawn.point.x as i32) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(ctx.state().spawn.point.y, F::VelocityY, (v1) as isize, ctx);
    store(
        (ctx.player_y() as i32 + ctx.state().spawn.point.y as i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_MotherBrainPurpleBreathBig(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::Graphics, (v1) as isize, ctx);
    store(
        (ctx.enemy_x(64) as i32 + 6) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(64) as i32 + 16) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_MotherBrainPurpleBreathSmall(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_MotherBrainPurpleBreathBig(j, ctx);
    ctx.set_mother_brain_hurt_frame(1);
}

pub(super) fn EprojInit_MotherBrainTubeFalling(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(3584, F::Graphics, (v1) as isize, ctx);

    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    store(ctx.state().spawn.point.x, F::X, (v1) as isize, ctx);
    store(ctx.state().spawn.point.y, F::Y, (v1) as isize, ctx);
    store((0x86cbea & 0xffff) as u16, F::BehaviorA, (v1) as isize, ctx);
}

pub(super) fn EprojInit_MaridiaFloatersSpikes(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(0, F::XFraction, (v1) as isize, ctx);
    store(0, F::YFraction, (v1) as isize, ctx);
    store(0, F::VelocityX, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    let v2 = (ctx.current_enemy() as i32 as u16);

    store(ctx.enemy_x(v2 as u16), F::X, (v1) as isize, ctx);
    store(ctx.enemy_y(v2 as u16), F::Y, (v1) as isize, ctx);
    store(ctx.state().spawn.primary, F::BehaviorA, (v1) as isize, ctx);
}

pub(super) fn EprojInit_SporeSpawnStalk(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        (ctx.enemy_y(0) as i32 + word_86DCB9[ctx.state().spawn.primary as usize] as i32) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    store(ctx.enemy_x(0), F::X, (v1) as isize, ctx);
}

pub(super) fn EprojInit_SporeSpawners(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        kEprojInit_SporeSpawners_X[ctx.state().spawn.primary as usize] as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(520, F::Y, (v1) as isize, ctx);
}

pub(super) fn EprojInit_EyeDoorSmoke(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86e42c) as usize,
            ctx.state().spawn.primary as u8 as isize,
        ),
        F::Program,
        (v1) as isize,
        ctx,
    );
    let v2: i32 = 8 * (ctx.state().spawn.primary >> 8) as u8 as i32 >> 1;
    let x: u16 = (ctx.image().table_word(
        crate::rtl::lorom_offset(0x86e47e) as usize,
        (v2 + 2) as isize,
    ) as i32
        + (ctx
            .image()
            .table_word(crate::rtl::lorom_offset(0x86e47e) as usize, v2 as isize)
            as i32
            & ctx.random_number() as i32)) as u16;
    let y: u16 = (ctx.image().table_word(
        crate::rtl::lorom_offset(0x86e47e) as usize,
        (v2 + 3) as isize,
    ) as i32
        + (ctx.image().table_word(
            crate::rtl::lorom_offset(0x86e47e) as usize,
            (v2 + 1) as isize,
        ) as i32
            & ctx.random_number() as i32 >> 8)) as u16;
    calculate_coordinates(room_mechanisms::selected(ctx), ctx);
    store(
        (x as i32 + 8 * (2 * room_mechanisms::block_x(ctx) as i32 + 1)) as u16,
        F::X,
        (v1) as isize,
        ctx,
    );
    store(
        (y as i32 + 8 * (2 * room_mechanisms::block_y(ctx) as i32 + 1)) as u16,
        F::Y,
        (v1) as isize,
        ctx,
    );
    ctx.next_random();
}

pub(super) fn EprojInit_BotwoonsBody(j: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = j as i32 >> 1;

    store(ctx.enemy_x(0), F::X, (v1) as isize, ctx);
    store(ctx.enemy_y(0), F::Y, (v1) as isize, ctx);
    store(0, F::VelocityY, (v1) as isize, ctx);

    let v2: u16 = (if ctx.enemy_ai_a(0) != 0 { 16 } else { 48 }) as u16;
    let v3: u16 = ctx.image().table_word(
        crate::rtl::lorom_offset(0x86e9f1) as usize,
        (v2 >> 1) as isize,
    );

    store(v3, F::Program, (v1) as isize, ctx);
    store(v3, F::BehaviorB, (v1) as isize, ctx);
    store(v2, F::BehaviorA, (v1) as isize, ctx);
    store((0x86ea98 & 0xffff) as u16, F::VelocityX, (v1) as isize, ctx);

    let Ex = ctx.enemy_ai_a(0);
    ctx.set_enemy_linked_projectile(Ex, j);
    store(2, F::DropFlags, (v1) as isize, ctx);
    ctx.set_botwoon_body_active(Ex, 1);
}

pub(super) fn EprojInit_CrocomireProjectile(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(-512i32 as u16, F::VelocityX, (v2) as isize, ctx);
    store(1, F::VelocityY, (v2) as isize, ctx);

    let v3 = (ctx.current_enemy() as i32 as u16);
    store(
        (ctx.enemy_x(v3 as u16) as i32 - 32i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(v3 as u16) as i32 - 16i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );

    store(0, F::Counter, (v2) as isize, ctx);
    store(0, F::XFraction, (v2) as isize, ctx);
    store(0, F::YFraction, (v2) as isize, ctx);

    store(2560, F::Graphics, (v2) as isize, ctx);
}

pub(super) fn EprojInit_PirateMotherBrainLaser(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(
        (if ctx.state().spawn.variant != 0 {
            addr_word_869F7D as i32
        } else {
            addr_word_869F41 as i32
        }) as u16,
        F::Program,
        (v2) as isize,
        ctx,
    );
    store((0x86a05b & 0xffff) as u16, F::PreAction, (v2) as isize, ctx);
    store(ctx.state().spawn.point.x, F::X, (v2) as isize, ctx);
    store(ctx.state().spawn.point.y, F::Y, (v2) as isize, ctx);
    let enemy_index = ctx.current_enemy();
    store(
        (ctx.raw_enemy_definition_properties(ctx.enemy_definition(enemy_index)) | 0x1000),
        F::Properties,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.enemy_parameter_1(enemy_index),
        F::BehaviorA,
        (v2) as isize,
        ctx,
    );
    ctx.queue_sound(2, 6, 0x67);
}

pub(super) fn EprojInit_GoldenTorizoSuperMissile(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(ctx.current_enemy(), F::BehaviorA, (v2) as isize, ctx);
    let v3 = (ctx.current_enemy() as i32 as u16);
    let v4: i32 = if (ctx.enemy_parameter_1(v3 as u16) & 0x8000) != 0 {
        1
    } else {
        0
    };
    store(
        (ctx.enemy_x(v3 as u16) as i32 + word_86B205[v4 as usize] as i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(v3 as u16) as i32 - 52i32) as u16,
        F::Y,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86b209) as usize, v4 as isize),
        F::Program,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_KiHunterAcidSpitLeft(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(-768i32 as u16, F::VelocityX, (v2) as isize, ctx);
    store(
        (ctx.enemy_x(ctx.current_enemy()) as i32 - 22i32) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    EprojInit_KiHunterAcidSpitCommon(ctx.current_enemy(), j, ctx);
}

pub(super) fn EprojInit_KiHunterAcidSpitRight(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(768, F::VelocityX, (v2) as isize, ctx);
    store(
        (ctx.enemy_x(ctx.current_enemy()) as i32 + 22) as u16,
        F::X,
        (v2) as isize,
        ctx,
    );
    EprojInit_KiHunterAcidSpitCommon(ctx.current_enemy(), j, ctx);
}

pub(super) fn EprojInit_KagosBugs(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(ctx.current_enemy(), F::BehaviorB, (v2) as isize, ctx);
    let enemy_index = ctx.current_enemy();
    store(ctx.enemy_x(enemy_index), F::X, (v2) as isize, ctx);
    store(ctx.enemy_y(enemy_index), F::Y, (v2) as isize, ctx);
    let v4: u16 = ((ctx.random_number() as i32 & 7) + 1) as u16;

    store(v4, F::BehaviorC, (v2) as isize, ctx);
    store(v4.wrapping_add(4), F::BehaviorA, (v2) as isize, ctx);
    store(((0x86d0cai32) as u16), F::PreAction, (v2) as isize, ctx);
}

pub(super) fn EprojInit_WreckedShipRobotLaserDown(j: u16, ctx: &mut Execution<'_>) {
    init_robot_laser(j, 128, true, ctx);
}

pub(super) fn EprojInit_WreckedShipRobotLaserHorizontal(j: u16, ctx: &mut Execution<'_>) {
    init_robot_laser(j, 0, false, ctx);
}

pub(super) fn EprojInit_WreckedShipRobotLaserUp(j: u16, ctx: &mut Execution<'_>) {
    init_robot_laser(j, (-128i16) as u16, false, ctx);
}

pub(super) fn EprojInit_DBF2(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(addr_word_86DB0C as u16, F::Program, (v2) as isize, ctx);
    store((0x86db62 & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    if ctx.state().spawn.primary != 0 {
        store((0x86db8c & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    }
    let v3 = (ctx.current_enemy() as i32 as u16);

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    store(-256i32 as u16, F::VelocityY, (v2) as isize, ctx);
    store(256, F::VelocityX, (v2) as isize, ctx);
}

pub(super) fn EprojInit_Spores(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    ctx.set_slot(F::X, v2 as isize, ctx.state().spawn.point.x);

    store(
        ctx.slot(F::X, v2 as isize),
        F::BehaviorB,
        (v2) as isize,
        ctx,
    );
    store(ctx.state().spawn.point.y, F::Y, (v2) as isize, ctx);
    store(512, F::Graphics, (v2) as isize, ctx);
}

pub(super) fn EprojInit_NamiFuneFireball(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(addr_word_86DE96 as u16, F::Program, (v2) as isize, ctx);
    store((0x86df40 & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    if ctx.state().spawn.primary != 0 {
        store(addr_word_86DEA6 as u16, F::Program, (v2) as isize, ctx);
        store((0x86df6a & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    }
    let v3 = (ctx.current_enemy() as i32 as u16);

    copy_enemy_position(v2 as isize, v3, 0, ctx);

    if (ctx.enemy_parameter_1(v3 as u16) & 0xf) != 0 {
        {
            let index = (v2) as isize;
            let old = ctx.slot(F::Y, (index) as isize);
            let value = ((old as i32) + (4)) as u16;
            ctx.set_slot(F::Y, (index) as isize, value);
        };
    }
    let v4: i32 = (4 * (ctx.enemy_parameter_2(v3 as u16) as u8) as i32) as u16 as i32 >> 1;
    store(
        ctx.image()
            .table_word(crate::rtl::lorom_offset(0x86deb6) as usize, v4 as isize),
        F::VelocityY,
        (v2) as isize,
        ctx,
    );
    store(
        ctx.image().table_word(
            crate::rtl::lorom_offset(0x86deb6) as usize,
            (v4 + 1) as isize,
        ),
        F::VelocityX,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_LavaThrownByLavaman(j: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = j as i32 >> 1;

    store(addr_word_86DFD8 as u16, F::Program, (v2) as isize, ctx);
    store((0x86e050 & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    if ctx.state().spawn.primary != 0 {
        store(addr_word_86DFDE as u16, F::Program, (v2) as isize, ctx);
        store((0x86e07a & 0xffff) as u16, F::BehaviorA, (v2) as isize, ctx);
    }
    let v3 = (ctx.current_enemy() as i32 as u16);

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    store(-768i32 as u16, F::VelocityY, (v2) as isize, ctx);
    store(768, F::VelocityX, (v2) as isize, ctx);
}

pub(super) fn EprojInit_MiniKraidSpit(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let extra_index = ctx.current_enemy();
    let v3: i32 = j as i32 >> 1;
    store(
        (ctx.mini_kraid_spit_x_offset(extra_index) as i32 + ctx.enemy_x(enemy_index) as i32) as u16,
        F::X,
        (v3) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(enemy_index) as i32 - 16i32) as u16,
        F::Y,
        (v3) as isize,
        ctx,
    );

    store(0, F::YFraction, (v3) as isize, ctx);
    store(0, F::XFraction, (v3) as isize, ctx);

    store(
        ctx.mini_kraid_spit_x_velocity(extra_index),
        F::VelocityX,
        (v3) as isize,
        ctx,
    );
    store(
        ctx.mini_kraid_spit_y_velocity(extra_index),
        F::VelocityY,
        (v3) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_BombTorizoLowHealthExplode(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let v2: i32 = j as i32 >> 1;
    store(ctx.enemy_x(enemy_index), F::X, (v2) as isize, ctx);
    store(ctx.enemy_y(enemy_index), F::Y, (v2) as isize, ctx);
    if (ctx.enemy_parameter_1(enemy_index) & 0x8000) == 0 {
        {
            let old = ctx.state().spawn.primary;
            let value = ((old as i32) + (2)) as u16;
            ctx.state_mut().spawn.primary = value;
        };
    }
    {
        let old = ctx.state().spawn.primary;
        ctx.state_mut().spawn.primary = old.wrapping_add(2);
    };
    let v3: i32 = ctx.state().spawn.primary as i32 >> 1;
    let v4: u16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize] as i32
        + ctx.slot(F::X, (v2) as isize) as i32) as u16;

    store(v4, F::X, (v2) as isize, ctx);
    store(v4, F::BehaviorA, (v2) as isize, ctx);

    let v5: u16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize] as i32
        + ctx.slot(F::Y, (v2) as isize) as i32) as u16;

    store(v5, F::Y, (v2) as isize, ctx);
    store(v5, F::BehaviorB, (v2) as isize, ctx);
}

pub(super) fn EprojInit_BombTorizoDeathExplosion(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let x_pos: u16 = ctx.enemy_x(enemy_index);
    let v3: i32 = j as i32 >> 1;

    store(x_pos, F::X, (v3) as isize, ctx);
    store(x_pos, F::BehaviorA, (v3) as isize, ctx);

    let y_pos: u16 = ctx.enemy_y(enemy_index);

    store(y_pos, F::Y, (v3) as isize, ctx);
    store(y_pos, F::BehaviorB, (v3) as isize, ctx);
}

pub(super) fn EprojInit_BombTorizosChozoOrbs(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let v2: u16 = (if (ctx.enemy_parameter_1(enemy_index) & 0x8000) != 0 {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: ctx.enemy_x(enemy_index),
            y: ctx.enemy_y(enemy_index),
        },
        ctx,
    );
}

pub(super) fn EprojInit_TourianLandingDustCloudsRightFoot(j: u16, ctx: &mut Execution<'_>) {
    init_landing_dust(j, 24, ctx);
}

pub(super) fn EprojInit_TorizoLandingDustCloudLeftFoot(j: u16, ctx: &mut Execution<'_>) {
    init_landing_dust(j, -24, ctx);
}

pub(super) fn EprojInit_YappingMawsBody(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let v3: i32 = j as i32 >> 1;
    store(ctx.enemy_x(enemy_index), F::X, (v3) as isize, ctx);
    store(ctx.enemy_y(enemy_index), F::Y, (v3) as isize, ctx);
    store(addr_word_86EC5C as u16, F::Program, (v3) as isize, ctx);
    if ctx.enemy_parameter_2(enemy_index) == 0 {
        store(addr_word_86EC56 as u16, F::Program, (v3) as isize, ctx);
    }
    let base = ctx.current_enemy();
    let part = ctx.maw_body_part_index(ctx.current_enemy());
    ctx.set_maw_body_link((base as i32 + 2 * part as i32) as u16, j);
}

pub(super) fn EprojInit_EnemyDeathExplosion(j: u16, ctx: &mut Execution<'_>) {
    let enemy_index = ctx.current_enemy();

    let v2: i32 = j as i32 >> 1;
    store(ctx.enemy_x(enemy_index), F::X, (v2) as isize, ctx);
    store(ctx.enemy_y(enemy_index), F::Y, (v2) as isize, ctx);
    store(ctx.current_enemy(), F::KilledEnemy, (v2) as isize, ctx);
    if (ctx.enemy_properties(enemy_index) & 0x4000) != 0 {
        store(
            (ctx.current_enemy()) | (0x8000u16),
            F::KilledEnemy,
            (v2) as isize,
            ctx,
        );
    }
    store(
        ctx.enemy_definition(enemy_index),
        F::DropDefinition,
        (v2) as isize,
        ctx,
    );
    store(0, F::Graphics, (v2) as isize, ctx);
    {
        let instruction: u16 = ctx.image().table_word(
            crate::rtl::lorom_offset(0x86efd5) as usize,
            ctx.state().spawn.primary as isize,
        );
        {
            let index = (v2 as usize) as isize;
            ctx.set_slot(F::Program, index, instruction);
            ctx.set_slot(F::Delay, index, 1);
        };
    }
}

pub(super) fn EprojInit_9688(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_FireballChild(j, 0, (-3584i16) as u16, 0x9688, ctx)
}

pub(super) fn EprojInit_9696(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_FireballChild(j, 0, 3584, 0x9696, ctx)
}

pub(super) fn EprojInit_966C(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_FireballChild(j, 3584, 0, 0x966c, ctx)
}

pub(super) fn EprojInit_967A(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_FireballChild(j, (-3584i16) as u16, 0, 0x967a, ctx)
}

pub(super) fn EprojInit_MiniKraidSpikesLeft(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_MiniKraidSpikes(j, -0x200i32 as u16, ctx)
}

pub(super) fn EprojInit_MiniKraidSpikesRight(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_MiniKraidSpikes(j, 0x200, ctx)
}

pub(super) fn EprojInit_SpawnedShotGate(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_SpawnedGate_Common(j, 0, ctx)
}

pub(super) fn EprojInit_ClosedDownwardsShotGate(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_SpawnedGate_Common(j, 0x40, ctx)
}

pub(super) fn EprojInit_ClosedUpwardsShotGate(j: u16, ctx: &mut Execution<'_>) {
    EprojInit_SpawnedGate_Common(j, -0x40i32 as u16, ctx)
}
