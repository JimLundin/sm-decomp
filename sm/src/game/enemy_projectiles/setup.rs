//! Complete enemy-projectile setup dispatch, using owned fields and explicit effects.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte};
use super::coordinates::{calculate_coordinates, projectile_argument};
use super::host::*;
use super::pre_actions::{
    EprojPreInit_MotherBrainsDeathExplosion_0, EprojPreInstr_C84D,
    EprojPreInstr_MotherBrainRainbowBeam, EprojPreInstr_TimeBombSetJapaneseText, sub_86C320,
};
use super::setup_helpers::*;
use super::shared::*;
use super::shared_pre::*;
use super::tables::*;
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::game::constants::bank_86::*;
use crate::game::constants::bank_a0::*;
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session::{Point, Vector32};

fn native_setup_word(address: u16, index: isize, ctx: &mut impl Host) -> u16 {
    let record = ctx.loaded_record(0x86, address);
    record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
}

pub(super) fn dispatch(ea: u32, j: u16, ctx: &mut impl Host) {
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
pub(super) fn Eproj_Init_0x8aaf(j: u16, ctx: &mut impl Host) {
    ctx.next_random();
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 = (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole as i32 + 12) as u16;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, v2 => (v3) as isize, y);
    enemy!(ctx, (v2 as i32 + 72) as u16 => (v3) as isize, counter);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).variables[3] => (v3) as isize, velocity_y);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole as i32 + (ctx.inputs().random as i32 & 0x1f) - 16i32) as u16 => (v3) as isize, x);
    {
        let value = {
            let record = ctx.loaded_record(0x86, 0x8a75);
            let index: isize =
                ((enemy_byte!(ctx, v3 as isize * 2 + 1, velocity_y) as u16 & 6) >> 1) as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v3) as isize, program);
    };
}

pub(super) fn EprojInit_DraygonsWallTurretProjs(j: u16, ctx: &mut impl Host) {
    Eproj_AngleToSamus(
        j,
        ctx.session().enemy_projectiles.spawn.point.x,
        ctx.session().enemy_projectiles.spawn.point.y,
        ctx,
    );
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 2560 => (v1) as isize, graphics);
    enemy!(ctx, (0x868d54 & 0xffff) as u16 => (v1) as isize, pre_action);
}

pub(super) fn EprojInit_9642_RidleysFireball(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    let parameter = ctx.session().enemy_projectiles.spawn.secondary;
    enemy!(ctx, (slot) as isize, behavior_b = parameter);
    let v2: i16 = (if ctx.session().enemy_projectiles.spawn.primary != 0 {
        25
    } else {
        -25i32
    }) as i16;
    let x = (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole).wrapping_add((v2) as u16);
    enemy!(ctx, (slot) as isize, x = x);
    let y = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole).wrapping_sub(43);
    enemy!(ctx, (slot) as isize, y = y);
    enemy!(ctx, (slot) as isize, graphics = 2560);
    let x_velocity = ctx.ridley_fireball_x_velocity();
    enemy!(ctx, (slot) as isize, velocity_x = x_velocity);
    let y_velocity = ctx.ridley_fireball_y_velocity();
    enemy!(ctx, (slot) as isize, velocity_y = y_velocity);
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, ctx);
}

pub(super) fn EprojInit_PhantoonDestroyableFireballs(j: u16, ctx: &mut impl Host) {
    let mut v1: i8 = ((ctx.session().enemy_projectiles.spawn.primary & 0xff00) >> 8) as i8;
    match v1 as i32 {
        0 => {
            let v2: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v2) as isize, x_fraction);
            enemy!(ctx, 0 => (v2) as isize, y_fraction);
            enemy!(ctx, 0 => (v2) as isize, velocity_x);
            enemy!(ctx, 0 => (v2) as isize, velocity_y);

            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v2) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 32) as u16 => (v2) as isize, y);
            enemy!(ctx, addr_off_8697B4 as u16 => (v2) as isize, program);
            enemy!(ctx, (enemy!(ctx, (v2) as isize, properties) & 0xfff | 0x2000) => (v2) as isize, properties);
        }
        2 => {
            let v3: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v3) as isize, x_fraction);
            enemy!(ctx, 0 => (v3) as isize, y_fraction);
            enemy!(ctx, 0 => (v3) as isize, velocity_y);

            let v4: u16 = ctx.session().enemy_projectiles.spawn.primary as u8 as u16;
            if ((ctx.session().enemy_projectiles.spawn.primary as u8 as i32 - 8i32) as i16) >= 0 {
                enemy!(ctx, -2i32 as u16 => (v3) as isize, velocity_x);
            } else {
                enemy!(ctx, 2 => (v3) as isize, velocity_x);
            }
            {
                enemy!(ctx, byte_8698B4[v4 as usize] as u16 => (v3) as isize, behavior_a);
                enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v3) as isize, x);
                enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 32) as u16 => (v3) as isize, y);
                enemy!(ctx, (0x869a45 & 0xffff) as u16 => (v3) as isize, pre_action);
            }
        }
        4 => {
            let v5: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v5) as isize, x_fraction);
            enemy!(ctx, 0 => (v5) as isize, y_fraction);
            enemy!(ctx, 0 => (v5) as isize, velocity_y);

            enemy!(ctx, ((ctx.session().enemy_projectiles.spawn.primary as i32 & 0xf0) >> 1) as u16 => (v5) as isize, velocity_x);
            enemy!(ctx, byte_8698F7[(ctx.session().enemy_projectiles.spawn.primary as i32 & 0xf) as usize] as u16 => (v5) as isize, x);
            enemy!(ctx, 40 => (v5) as isize, y);
            enemy!(ctx, (0x869a94 & 0xffff) as u16 => (v5) as isize, pre_action);
        }
        6 => {
            let v7: i32 = j as i32 >> 1;

            enemy!(ctx, 0 => (v7) as isize, x_fraction);
            enemy!(ctx, 0 => (v7) as isize, y_fraction);
            enemy!(ctx, 0 => (v7) as isize, velocity_y);

            enemy!(ctx, 128 => (v7) as isize, velocity_x);
            enemy!(ctx, byte_869979[ctx.session().enemy_projectiles.spawn.primary as u8 as usize] as u16 => (v7) as isize, behavior_a);
            enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v7) as isize, x);
            enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v7) as isize, y);
            enemy!(ctx, (0x869ada & 0xffff) as u16 => (v7) as isize, pre_action);
        }
        _ => {
            ctx.unreachable();
        }
    };
}

pub(super) fn EprojInit_RocksKraidSpits(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole).wrapping_add(16) => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole).wrapping_sub(96) => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, velocity_x);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 1536 => (v2) as isize, graphics);
}

pub(super) fn EprojInit_RocksWhenKraidRises(j: u16, ctx: &mut impl Host) {
    let mut v1: i16 = ((ctx.inputs().random) as i16) & (0x3fi16);
    if (ctx.inputs().random & 1) == 0 {
        v1 = !(v1 as i32) as i16;
    }
    let v2: i32 = j as i32 >> 1;
    {
        let value = ({
            let enemy_index = ctx.inputs().current_enemy;
            ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole
        })
        .wrapping_add((v1) as u16);
        enemy!(ctx, value => (v2) as isize, x);
    };
    enemy!(ctx, 432 => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, velocity_x);
    enemy!(ctx, -1280i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 1536 => (v2) as isize, graphics);

    ctx.queue_sound(3, 6, 0x1e);
}

pub(super) fn EprojInit_WalkingLavaSeahorseFireball(j: u16, ctx: &mut impl Host) {
    static word_869EF9: [i16; 3] = [-0x100i16, 0, 0x100];
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole as i32 - 12i32) as u16 => (v2) as isize, y);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole as i32 - 16i32) as u16 => (v2) as isize, x);
    if (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).variables[3] & 0x8000) == 0 {
        enemy!(ctx, 1024 => (v2) as isize, velocity_x);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole as i32 + 16) as u16 => (v2) as isize, x);
    }
    enemy!(ctx, word_869EF9[(ctx.session().enemy_projectiles.spawn.primary >> 1) as usize] as u16 => (v2) as isize, velocity_y);

    enemy!(ctx, 0 => (v2) as isize, y_fraction);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);
}

pub(super) fn EprojInit_PirateClaw(j: u16, ctx: &mut impl Host) {
    let r = ctx.session().enemy_projectiles.spawn.rectangle;
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (r.origin.y as i32 + r.extent.y as i32) as u16 => (v1) as isize, y);
    enemy!(ctx, (r.origin.x as i32 + r.extent.x as i32) as u16 => (v1) as isize, x);
    {
        let value = (if ctx.session().enemy_projectiles.spawn.primary != 0 {
            addr_off_869FE1 as i32
        } else {
            addr_off_869FB9 as i32
        }) as u16;
        enemy!(ctx, value => (v1) as isize, program);
    };
    enemy!(ctx, (0x86a05b & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy!(ctx, 2048 => (v1) as isize, behavior_a);
    enemy!(ctx, 1 => (v1) as isize, behavior_b);
}

pub(super) fn EprojInit_BombTorizoLowHealthDrool(j: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    {
        let value = {
            let record = ctx.loaded_record(0x86, 0xa64d);
            let index: isize = (((ctx.next_random() >> 1) & (0xeu16)) >> 1) as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v1) as isize, program);
    };
    ctx.next_random();
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).y.whole).wrapping_sub(5) => (v1) as isize, y);
    let mut parameter_1: i16 = ctx
        .resolve_actor(Key::from_legacy(v2 as u16), Le)
        .parameters[0] as i16;
    let mut v4: u16 = if (parameter_1 as u16 & 0x4000) != 0 {
        ctx.inputs().random & 0x1fe
    } else {
        if parameter_1 < 0 {
            v5 = 32;
        } else {
            v5 = 224;
        }
        (2 * (v5 as i32 + (ctx.inputs().random as i32 & 0xf) - 8i32)) as u16
    };
    let v6: i32 = v4 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v6 + 64) as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kSinCosTable8bit_Sext[v6 as usize] as u16 => (v1) as isize, velocity_y);
    let v7 = ctx.inputs().current_enemy as i32 as u16;
    let mut v8: u16 = if (ctx
        .resolve_actor(Key::from_legacy(v7 as u16), Le)
        .parameters[0]
        & 0x8000)
        != 0
    {
        (ctx.resolve_actor(Key::from_legacy(v7 as u16), Le).x.whole).wrapping_add(8)
    } else {
        (ctx.resolve_actor(Key::from_legacy(v7 as u16), Le).x.whole as i32 - 8i32) as u16
    };
    enemy!(ctx, v8 => (v1) as isize, x);
}

pub(super) fn EprojInit_BombTorizoExplosiveSwipe(j: u16, ctx: &mut impl Host) {
    let enemy = ctx.inputs().current_enemy;
    let mirrored = ctx.resolve_actor(Key::from_legacy(enemy), Le).parameters[0] & 0x8000 != 0;
    let variant = (ctx.session().enemy_projectiles.spawn.primary >> 1) as usize;
    let slot = (j >> 1) as isize;
    let x = ctx.resolve_actor(Key::from_legacy(enemy), Le).x.whole;
    let x_offset = kEprojInit_BombTorizoExplosiveSwipe_Tab0[variant] as u16;
    let x = if mirrored {
        x.wrapping_sub(x_offset)
    } else {
        x.wrapping_add(x_offset)
    };
    enemy!(ctx, x => slot, x);
    let y = ctx.resolve_actor(Key::from_legacy(enemy), Le).y.whole;
    let y_offset = kEprojInit_BombTorizoExplosiveSwipe_Tab1[variant] as u16;
    enemy!(ctx, y.wrapping_add(y_offset) => slot, y);
}

pub(super) fn EprojInit_BombTorizoStatueBreaking(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let mut v1: i8 = ctx.session().enemy_projectiles.spawn.primary as i8;
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    {
        let value = native_setup_word(0xa7ab, v2 as isize, ctx);
        enemy!(ctx, value => (v3) as isize, program);
    };
    {
        let value = (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v3) as isize, x);
    };
    let v4: i32 = (v1 as i32 & 0xf) >> 1;
    {
        let value = (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v3) as isize, y);
    };
    enemy!(ctx, kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as u16 => (v3) as isize, velocity_y);
    enemy!(ctx, kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as u16 => (v3) as isize, behavior_b);
    {
        let index = (v3 as usize) as isize;
        let old = enemy!(ctx, index, properties);
        enemy!(ctx, index, properties = old | (0x1000));
    };
}

pub(super) fn EprojInit_AB07(j: u16, ctx: &mut impl Host) {
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
    enemy!(ctx, ctx.session().player.x => (v4) as isize, x);
    enemy!(ctx, (ctx.session().player.y as i32 - 36i32) as u16 => (v4) as isize, y);
}

pub(super) fn EprojInit_GoldenTorizosChozoOrbs(j: u16, ctx: &mut impl Host) {
    let E = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 =
        (if (ctx.resolve_actor(Key::from_legacy(E as u16), Le).parameters[0] & 0x8000) != 0 {
            addr_kEprojInit_GoldenTorizosChozoOrbs_init0 as i32
        } else {
            addr_kEprojInit_GoldenTorizosChozoOrbs_init1 as i32
        }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(E as u16), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(E as u16), Le).y.whole,
        },
        ctx,
    );
}

pub(super) fn EprojInit_TorizoSonicBoom(j: u16, ctx: &mut impl Host) {
    let v1: i32 = if (ctx.next_random() & 1) != 0 {
        -12i32
    } else {
        20
    };
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).y.whole as i32 + v1) as u16 => (v3) as isize, y);
    enemy!(ctx, 0 => (v3) as isize, velocity_y);

    if (ctx
        .resolve_actor(Key::from_legacy(v2 as u16), Le)
        .parameters[0]
        & 0x8000)
        != 0
    {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole as i32 + 32) as u16 => (v3) as isize, x);
        enemy!(ctx, 624 => (v3) as isize, velocity_x);
        enemy!(ctx, addr_off_86ADD2 as u16 => (v3) as isize, program);
    } else {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole as i32 - 32i32) as u16 => (v3) as isize, x);
        enemy!(ctx, -624i32 as u16 => (v3) as isize, velocity_x);
        enemy!(ctx, addr_off_86ADBF as u16 => (v3) as isize, program);
    };
}

pub(super) fn EprojInit_WreckedShipChozoSpikeFootsteps(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    {
        let value = (ctx.session().enemy_projectiles.spawn.primary as i32
            + ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole as i32)
            as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole as i32 + 28) as u16 => (v2) as isize, y);
}

pub(super) fn EprojInit_GoldenTorizoEgg(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ((0xe2 & 0x1f) + 64) as u16 => (v2) as isize, behavior_b);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).parameters[0] => (v2) as isize, behavior_a);

    let v4: u16 = (if ctx
        .resolve_actor(Key::from_legacy(v1 as u16), Le)
        .parameters[0]
        & 0x8000
        != 0
    {
        addr_kEprojInit_GoldenTorizoEgg0 as i32
    } else {
        addr_kEprojInit_GoldenTorizoEgg1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v4,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole,
        },
        ctx,
    );
}

pub(super) fn EprojInit_GoldenTorizoEyeBeam(j: u16, ctx: &mut impl Host) {
    let E = ctx.inputs().current_enemy as i32 as u16;
    let v2: u16 =
        (if (ctx.resolve_actor(Key::from_legacy(E as u16), Le).parameters[0] & 0x8000) != 0 {
            addr_stru_86B376 as i32
        } else {
            addr_stru_86B380 as i32
        }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(E as u16), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(E as u16), Le).y.whole,
        },
        ctx,
    );
    let mut v3: u16 = ((ctx.next_random() as i32 & 0x1e) - 16i32 + 192) as u16;
    if (ctx.resolve_actor(Key::from_legacy(E as u16), Le).parameters[0] & 0x8000) == 0 {
        v3 = (v3 as i32 + 128) as u16;
    }
    let v4: i32 = v3 as i32 >> 1;
    let v5: i32 = j as i32 >> 1;
    enemy!(ctx, (8 * kSinCosTable8bit_Sext[(v4 + 64) as usize] as i32) as u16 => (v5) as isize, velocity_x);
    enemy!(ctx, (8 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16 => (v5) as isize, velocity_y);
}

pub(super) fn EprojInit_LavaSeahorseFireball(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole).wrapping_sub(28) => (v2) as isize, y);
    enemy!(ctx, -961i32 as u16 => (v2) as isize, velocity_y);
    if (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).variables[0] & 0x8000) == 0 {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole as i32 + 12) as u16 => (v2) as isize, x);
        enemy!(ctx, 704 => (v2) as isize, velocity_x);
        enemy!(ctx, addr_word_86B4CB as u16 => (v2) as isize, program);
    } else {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole).wrapping_sub(12) => (v2) as isize, x);
        enemy!(ctx, -704i32 as u16 => (v2) as isize, velocity_x);
        enemy!(ctx, addr_word_86B4BF as u16 => (v2) as isize, program);
    };
}

pub(super) fn EprojInit_EyeDoorProjectile(j: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().mechanisms.selected;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, projectile_argument(ctx.session().mechanisms.selected, ctx) => (v2) as isize, behavior_b);
    calculate_coordinates(v1, ctx);
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    {
        let value = (word_86B65B[v3 as usize] as i32
            + 8 * (2 * ctx.session().mechanisms.block_x as i32 + 1)) as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    {
        let value = (word_86B65B[(v3 + 1) as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
}

pub(super) fn EprojInit_EyeDoorSweat(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (8 * (2 * (ctx.session().mechanisms.block_x as i32 - 1i32) + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_y as i32 + 1)) as u16 => (v1) as isize, y);
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    enemy!(ctx, word_86B6B1[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, word_86B6B1[(v2 + 1) as usize] as u16 => (v1) as isize, velocity_y);
}

pub(super) fn EprojInit_TourianStatueEyeGlow(j: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v2: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    enemy!(ctx, kEprojInit_TourianStatueEyeGlow_X[v2 as usize] => (v3) as isize, x);
    enemy!(ctx, kEprojInit_TourianStatueEyeGlow_Y[v2 as usize] => (v3) as isize, y);
    let mut v4: u16 = (4 * v1 as i32) as u16;
    for i in (498i32..506).step_by(2) {
        {
            let value = kEprojInit_TourianStatueEyeGlow_Colors[(v4 >> 1) as usize];
            ctx.set_palette_buffer((i >> 1) as isize, value);
        };
        v4 = (v4 as i32 + 2) as u16;
    }
}

pub(super) fn EprojInit_TourianStatueUnlockingParticle(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v2) as isize, x);
    enemy!(ctx, enemy!(ctx, (v1) as isize, y) => (v2) as isize, y);
    let v3: u16 = (2 * ((ctx.next_random() as i32 & 0x3f) - 32i32) as u8 as i32) as u16;
    enemy!(ctx, v3 => (v2) as isize, behavior_a);
    let v4: i32 = v3 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16 => (v2) as isize, velocity_y);
}

pub(super) fn EprojInit_TourianStatueSoul(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, kEprojInit_TourianStatueEyeGlow_X[v1 as usize] => (v2) as isize, x);
    enemy!(ctx, kEprojInit_TourianStatueEyeGlow_Y[v1 as usize] => (v2) as isize, y);
    enemy!(ctx, -1024i32 as u16 => (v2) as isize, velocity_y);
}

pub(super) fn EprojInit_NuclearWaffleBody(j: u16, ctx: &mut impl Host) {
    let v2 = ctx.inputs().current_enemy as i32 as u16;
    let v3: i32 = j as i32 >> 1;

    copy_enemy_position(v3 as isize, v2, 0, ctx);

    let v4 = ctx.inputs().current_enemy;
    let linked =
        (ctx.inputs().current_enemy as i32 + ctx.nuclear_waffle_part_offset(v4) as i32) as u16;
    ctx.set_enemy_linked_projectile(linked, j);
    enemy!(ctx, 1 => (v3) as isize, drop_flags);
}

pub(super) fn EprojInit_ShaktoolAttackMiddleBackCircle(j: u16, ctx: &mut impl Host) {
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (j >> 1) as isize, behavior_a);
    EprojInit_BDA2(j, ctx);
}

pub(super) fn EprojInit_MotherBrainBomb(j: u16, ctx: &mut impl Host) {
    enemy_byte!(
        ctx,
        j as isize,
        x_fraction = ctx.session().enemy_projectiles.spawn.primary as u8
    );
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 256 => (v1) as isize, velocity_y);
    enemy!(ctx, 224 => (v1) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 12) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
    enemy!(ctx, 112 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);

    let count = ctx.mother_brain_active_bombs();
    ctx.set_mother_brain_active_bombs(count.wrapping_add(1));
}

pub(super) fn EprojInit_MotherBrainEscapeDoorParticles(j: u16, ctx: &mut impl Host) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [i16; 16] = [
        0, -0x20i16, 0, -0x18i16, 0, -0x10i16, 0, -8i16, 0, 0, 0, 8, 0, 0x10, 0, 0x18,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [i16; 16] = [
        0x500, -0x200i16, 0x500, -0x100i16, 0x500, -0x100i16, 0x500, -0x80i16, 0x500, -0x80i16,
        0x500, 0x80, 0x500, -0x100i16, 0x500, 0x200,
    ];
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    let v2: i32 = 4 * ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    enemy!(ctx, (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize] as i32 + 16) as u16 => (v1) as isize, x);
    enemy!(ctx, (kEprojInit_MotherBrainEscapeDoorParticles_X[(v2 + 1) as usize] as i32 + 128) as u16 => (v1) as isize, y);
    enemy!(ctx, kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kEprojInit_MotherBrainEscapeDoorParticles_Xvel[(v2 + 1) as usize] as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, 32 => (v1) as isize, behavior_a);
}

pub(super) fn EprojInit_TimeBombSetJapaneseText(j: u16, ctx: &mut impl Host) {
    enemy!(ctx, 0 => (j >> 1) as isize, graphics);
    EprojPreInstr_TimeBombSetJapaneseText(j, ctx);
}

pub(super) fn EprojInit_MotherBrainGlassShatteringShard(j: u16, ctx: &mut impl Host) {
    let v1: u16 = (2 * ctx.next_random() as i32 & 0x1fe) as u16;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, v1 => (v2) as isize, behavior_a);
    let v3: i32 = v1 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v3 + 64) as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v3 as usize] as i32) as u16 => (v2) as isize, velocity_y);
    {
        let value = native_setup_word(0xce41, ((v1 as i32 >> 4 & 0x1e) as u16 >> 1) as isize, ctx);
        enemy!(ctx, value => (v2) as isize, program);
    };
    enemy!(ctx, 1600 => (v2) as isize, graphics);
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v4: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    {
        let value = (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v2) as isize, x);
    };
    {
        let value = (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };

    {
        let index = (v2) as isize;
        let old = enemy!(ctx, (index) as isize, x);
        let value = ((old as i32) + ((ctx.next_random() as i32 & 0xf) - 8i32)) as u16;
        enemy!(ctx, (index) as isize, x = value);
    };
    {
        let index = (v2) as isize;
        let old = enemy!(ctx, (index) as isize, y);
        let value = ((old as i32) + ((ctx.next_random() as i32 & 0xf) - 8i32)) as u16;
        enemy!(ctx, (index) as isize, y = value);
    };
}

pub(super) fn EprojInit_MotherBrainGlassShatteringSparkle(j: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, (enemy!(ctx, (v1) as isize, x) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (enemy!(ctx, (v1) as isize, y) as i32 + (ctx.next_random() as i32 & 0x1f) - 16i32) as u16 => (v2) as isize, y);
    enemy!(ctx, 1600 => (v2) as isize, graphics);
}

pub(super) fn EprojInit_N00bTubeCrack(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (16 * ctx.session().mechanisms.block_x as i32 + 96) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_y as i32 + 48) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_N00bTubeShards(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    {
        let value = (kEprojInit_N00bTubeShards_X[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32
            + 96) as u16;
        enemy!(ctx, value => (v2) as isize, behavior_b);
    };
    enemy!(ctx, 0 => (v2) as isize, behavior_a);
    {
        let value = (kEprojInit_N00bTubeShards_Y[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32
            + 48) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
    {
        let value = native_setup_word(0xd760, v1 as isize, ctx);
        enemy!(ctx, value => (v2) as isize, program);
    };
    enemy!(ctx, kEprojInit_N00bTubeShards_Xvel[v1 as usize] as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, kEprojInit_N00bTubeShards_Yvel[v1 as usize] as u16 => (v2) as isize, velocity_y);
}

pub(super) fn EprojInit_N00bTubeReleasedAirBubbles(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = j as i32 >> 1;

    {
        let value = (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_x as i32) as u16;
        enemy!(ctx, value => (v2) as isize, behavior_b);
    };
    enemy!(ctx, 0 => (v2) as isize, behavior_a);
    {
        let value = (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize] as i32
            + 16 * ctx.session().mechanisms.block_y as i32) as u16;
        enemy!(ctx, value => (v2) as isize, y);
    };
    enemy!(ctx, -1280i32 as u16 => (v2) as isize, velocity_y);
}

pub(super) fn EprojInit_SaveStationElectricity(j: u16, ctx: &mut impl Host) {
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    let v1: i32 = j as i32 >> 1;
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_x as i32 + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * (ctx.session().mechanisms.block_y as i32 - 2i32)) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_BotwoonsSpit(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy as i32 as u16;
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v1 as u16), Le).y.whole => (v2) as isize, y);
    enemy!(ctx, addr_word_86EBAE as u16 => (v2) as isize, program);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.angle as u16 => (v2) as isize, angle);

    let pt: Vector32 = ConvertAngleToXy(
        ctx.session().enemy_projectiles.spawn.angle as u16,
        ctx.session().enemy_projectiles.spawn.primary,
        ctx,
    );

    enemy!(ctx, (pt.x >> 16) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, pt.x as u16 => (v2) as isize, behavior_a);
    enemy!(ctx, (pt.y >> 16) as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, pt.y as u16 => (v2) as isize, behavior_b);
}

pub(super) fn EprojInit_Sparks(j: u16, ctx: &mut impl Host) {
    static word_86F3D4: [u16; 14] = [
        0xffff, 0xb800, 0xffff, 0xc000, 0xffff, 0xe000, 0xffff, 0xff00, 0, 0x100, 0, 0x2000, 0,
        0x4000,
    ];
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, addr_kEnemyDef_F353 as u16 => (v2) as isize, program);
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 8, ctx);

    enemy!(ctx, 0 => (v2) as isize, velocity_x);
    enemy!(ctx, 0 => (v2) as isize, velocity_y);

    let v4: u16 = ((ctx.next_random() as i32 & 0x1c) >> 1) as u16;
    {
        let value = native_setup_word(0xf3d4, (v4 as i32 + 0) as isize, ctx);
        enemy!(ctx, value => (v2) as isize, behavior_b);
    };
    {
        let value = native_setup_word(0xf3d4, (v4 as i32 + 1) as isize, ctx);
        enemy!(ctx, value => (v2) as isize, behavior_a);
    };
}

pub(super) fn EprojInit_9660_FireballExplosion(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    enemy!(ctx, (slot) as isize, graphics = 2560);
    let x = ctx.session().enemy_projectiles.spawn.point.x;
    enemy!(ctx, (slot) as isize, x = x);
    let y = ctx.session().enemy_projectiles.spawn.point.y;
    enemy!(ctx, (slot) as isize, y = y);
    let parameter = ctx.session().enemy_projectiles.spawn.primary;
    enemy!(ctx, (slot) as isize, behavior_a = parameter);
    enemy!(ctx, (slot) as isize, behavior_b = 0);
    enemy!(ctx, (slot) as isize, velocity_x = 0);
    enemy!(ctx, (slot) as isize, velocity_y = 0);
}

pub(super) fn EprojInit_9734_CeresFallingDebris(j: u16, ctx: &mut impl Host) {
    let slot = usize::from(j >> 1);
    enemy!(ctx, (slot) as isize, behavior_a = 0);
    enemy!(ctx, (slot) as isize, behavior_b = 0);
    enemy!(ctx, (slot) as isize, velocity_x = 0);
    enemy!(ctx, (slot) as isize, graphics = 3584);
    let x = ctx.session().enemy_projectiles.spawn.primary;
    enemy!(ctx, (slot) as isize, x = x);
    enemy!(ctx, (slot) as isize, y = 42);
    enemy!(ctx, (slot) as isize, velocity_y = 16);
}

pub(super) fn EprojInit_CeresElevatorPad(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;
    let eproj_y_pos: u16 = ctx.session().player.y.wrapping_add(28);
    enemy!(ctx, (v1 as usize) as isize, y = eproj_y_pos);
    enemy!(ctx, (v1 as usize) as isize, behavior_a = 60);
    sub_86A301(j, ctx);
}

pub(super) fn EprojInit_CeresElevatorPlatform(j: u16, ctx: &mut impl Host) {
    let eproj_word_index = (j >> 1) as usize;
    enemy!(ctx, (eproj_word_index) as isize, y = 97);
    sub_86A301(j, ctx);
}

pub(super) fn EprojPreInstr_PrePhantomRoom(j: u16, ctx: &mut impl Host) {
    {
        let value = 0;
        ctx.set_bg2_y_scroll(value);
    };
}

pub(super) fn sub_86BB30(j: u16, ctx: &mut impl Host) {
    let v1 = ctx.inputs().current_enemy;
    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.enemy_projectile_origin_x(v1) => (v2) as isize, x);
    enemy!(ctx, ctx.enemy_projectile_origin_y(v1) => (v2) as isize, y);
    {
        let value = {
            let record = ctx.loaded_record(0x86, 0xbb1e);
            let index: isize = ctx.session().enemy_projectiles.spawn.primary as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v2) as isize, program);
    };
}

pub(super) fn Eproj_MotherBrainsBlueRingLasers(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 8 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);

    let r18: u16 = ctx.session().enemy_projectiles.spawn.primary;

    enemy!(ctx, Math_MultBySin(0x450, r18) => (v1) as isize, velocity_x);
    enemy!(ctx, Math_MultByCos(0x450, r18) => (v1) as isize, velocity_y);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 10) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);

    sub_86C320(j, ctx);
}

pub(super) fn sub_86C605(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);

    ctx.set_mother_brain_beam_x_fraction(0);
    ctx.set_mother_brain_beam_y_fraction(0);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
    enemy!(
        ctx,
        v1 as isize,
        x = (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 + 64) as u16
    );
    ctx.set_mother_brain_beam_x(enemy!(ctx, v1 as isize, x));
    let x: u16 = (ctx.session().player.x as i32 - ctx.mother_brain_beam_x() as i32) as u16;
    enemy!(
        ctx,
        v1 as isize,
        y = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 - 48i32) as u16
    );
    ctx.set_mother_brain_beam_y(enemy!(ctx, v1 as isize, y));
    let y: u16 = (ctx.session().player.y as i32 - ctx.mother_brain_beam_y() as i32) as u16;
    let r18: i32 = -(ctx.calculate_angle_from_xy(x, y) as i32 + 0x80i32) as u8 as i32;
    ctx.set_mother_brain_beam_angle(r18 as u16);
    ctx.set_mother_brain_beam_velocity_x(Math_MultBySin(0xc00, r18 as u16));
    ctx.set_mother_brain_beam_velocity_y(Math_MultByCos(0xc00, r18 as u16));
}

pub(super) fn EprojIni_TourianStatueUnlockingParticleTail(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v2: i32 = v0 as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v2) as isize, x);
    enemy!(ctx, enemy!(ctx, (v1) as isize, y) => (v2) as isize, y);
}

pub(super) fn sub_86D992(v0: u16, ctx: &mut impl Host) {
    let v1: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v2: i32 = v0 as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v2) as isize, behavior_a);
    {
        let value = native_setup_word(0xd96a, (v1 >> 1) as isize, ctx);
        enemy!(ctx, value => (v2) as isize, program);
    };

    let enemy_index = ctx.inputs().current_enemy;

    copy_enemy_position(v2 as isize, enemy_index, 0, ctx);
    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 512 => (v2) as isize, velocity_x);

    if ctx.session().enemy_projectiles.spawn.primary as i32 - 12i32 & 0x8000 == 0 {
        enemy!(ctx, -384i32 as u16 => (v2) as isize, velocity_y);
        enemy!(ctx, 384 => (v2) as isize, velocity_x);
    }
}

pub(super) fn EprojInit_DustCloudOrExplosion(v0: u16, ctx: &mut impl Host) {
    let v1: i32 = v0 as i32 >> 1;
    {
        let value = {
            let record = ctx.loaded_record(0x86, 0xe42c);
            let index: isize = ctx.session().enemy_projectiles.spawn.primary as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v1) as isize, program);
    };
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
}

pub(super) fn EprojInit_DraygonsGunk(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.secondary => (v1) as isize, angle);
    let pt: Vector32 = ConvertAngleToXy(
        ctx.session().enemy_projectiles.spawn.secondary,
        ctx.session().enemy_projectiles.spawn.primary,
        ctx,
    );

    enemy!(ctx, (pt.x >> 16) as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, pt.x as u16 => (v1) as isize, behavior_a);
    enemy!(ctx, (pt.y >> 16) as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, pt.y as u16 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
}

pub(super) fn EprojInit_CrocomireSpikeWallPieces(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    {
        let value = native_setup_word(0x9105, ((j as i32 - 20i32) as u16 >> 1) as isize, ctx);
        enemy!(ctx, value => (v1) as isize, y);
    };
    enemy!(ctx, 528 => (v1) as isize, x);

    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);
    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);

    enemy!(ctx, -5i32 as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, -30720i32 as u16 => (v1) as isize, behavior_b);
}

pub(super) fn EprojInit_CrocomireBridgeCrumbling(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, x);
    enemy!(ctx, 187 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);

    enemy!(ctx, ((ctx.inputs().random as i32 & 0x3f) + 64) as u16 => (v1) as isize, velocity_y);
    enemy!(ctx, 1024 => (v1) as isize, graphics);
}

pub(super) fn EprojInit_9634(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_a);
    enemy!(ctx, 0 => (v1) as isize, behavior_b);

    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 - 29i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 - 35i32) as u16 => (v1) as isize, y);
    enemy!(ctx, 2560 => (v1) as isize, graphics);

    let v2: i32 = ctx.resolve_actor(Key::from_legacy(0), Le).parameters[0] as i32;
    enemy!(ctx, kEprojInit_9634_Xvel[v2 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kEprojInit_9634_Yvel[v2 as usize] as u16 => (v1) as isize, velocity_y);
}

pub(super) fn EprojInit_PhantoonStartingFireballs(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = byte_869979[ctx.session().enemy_projectiles.spawn.primary as usize] as u16;
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let pt: Point = sine_offset::<false>(v2, 0x30);

    enemy!(ctx, (pt.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (pt.y as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_RocksFallingKraidCeiling(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, x);
    enemy!(ctx, 312 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);

    enemy!(ctx, (ctx.inputs().random & 0x3f).wrapping_add(64) => (v1) as isize, velocity_y);
    enemy!(ctx, 1536 => (v1) as isize, graphics);
}

pub(super) fn EprojInit_A379(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = ctx.session().enemy_projectiles.spawn.primary;
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;

    enemy!(ctx, (word_86A2D6[v3 as usize] as i32 + ctx.session().player.x as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.session().player.y as i32 + 80) as u16 => (v1) as isize, y);
    {
        let value = native_setup_word(0xa2e2, v3 as isize, ctx);
        enemy!(ctx, value => (v1) as isize, program);
    };
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
}

pub(super) fn EprojInit_BombTorizoLowHealthInitialDrool(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    ctx.next_random();
    let v2 = ctx.inputs().current_enemy as i32 as u16;

    {
        let value = ((ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).y.whole as i32
            + (ctx.inputs().random as i32 & 3)) as u16)
            .wrapping_sub(5);
        enemy!(ctx, value => (v1) as isize, y);
    };
    enemy!(ctx, (ctx.inputs().random & 0x1f).wrapping_add(48) => (v1) as isize, velocity_y);

    ctx.next_random();
    let mut v3: i16 = ((ctx.inputs().random) as i16) & (3i16);
    let mut parameter_1: i16 = ctx
        .resolve_actor(Key::from_legacy(v2 as u16), Le)
        .parameters[0] as i16;
    if (parameter_1 as u16 & 0x4000) != 0 {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole as i32 + v3 as i32) as u16 => (v1) as isize, x);
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
    } else {
        if parameter_1 < 0 {
            enemy!(ctx, ((ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole as i32 + v3 as i32) as u16).wrapping_add(8) => (v1) as isize, x);
        } else {
            enemy!(ctx, ((ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole as i32 + v3 as i32) as u16).wrapping_sub(8) => (v1) as isize, x);
        }
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
    };
}

pub(super) fn EprojInit_A977(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_x as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (16 * ctx.session().mechanisms.block_y as i32 - 4i32) as u16 => (v1) as isize, y);

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
}

pub(super) fn EprojInit_TourianStatueDustClouds(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 128 => (v1) as isize, behavior_a);
    enemy!(ctx, 188 => (v1) as isize, behavior_b);
}

pub(super) fn EprojInit_TourianEscapeShaftFakeWallExplode(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 272 => (v1) as isize, x);
    enemy!(ctx, 272 => (v1) as isize, behavior_a);

    enemy!(ctx, 2184 => (v1) as isize, y);
    enemy!(ctx, 2184 => (v1) as isize, behavior_b);
}

pub(super) fn EprojInit_TourianStatueUnlockingParticleWaterSplash(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, enemy!(ctx, (ctx.session().enemy_projectiles.spawn.primary >> 1) as isize, x) => (v1) as isize, x);
    enemy!(ctx, (ctx.inputs().fx_y as i32 - 4i32) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_TourianStatueBaseDecoration(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 120, 184, ctx);
}

pub(super) fn EprojInit_TourianStatueRidley(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 142, 85, ctx);
}

pub(super) fn EprojInit_TourianStatuePhantoon(j: u16, ctx: &mut impl Host) {
    init_statue_piece(j, 132, 136, ctx);
}

pub(super) fn EprojInit_NorfairLavaquakeRocks(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, addr_word_86BBD5 as u16 => (v1) as isize, program);
    enemy!(ctx, (0x86bc16 & 0xffff) as u16 => (v1) as isize, behavior_a);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, velocity_y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.secondary => (v1) as isize, velocity_x);
    let v2 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v1 as isize, v2, 0, ctx);
}

pub(super) fn EprojInit_BDA2(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    let v2 = ctx.inputs().current_enemy as i32 as u16;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).y.whole => (v1) as isize, y);

    let v3: u16 =
        (2 * (ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).variables[3] as u8) as i32) as u16;
    let v4: i32 = v3 as i32 >> 1;
    enemy!(ctx, kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kSinCosTable8bit_Sext[v4 as usize] as u16 => (v1) as isize, velocity_y);
    let v5: i32 = (v3 >> 5) as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, x);
        let value = ((old as i32)
            + (kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as i32))
            as u16;
        enemy!(ctx, (index) as isize, x = value);
    };
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, y);
        let value = ((old as i32)
            + (kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as i32))
            as u16;
        enemy!(ctx, (index) as isize, y = value);
    };
}

pub(super) fn EprojInit_MotherBrainRoomTurrets(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 1024 => (v1) as isize, graphics);
    let v2: u16 = (2 * ctx.session().enemy_projectiles.spawn.primary as i32) as u16;
    let v3: u16 = ({
        let record = ctx.loaded_record(0x86, 0xbee1);
        let index: isize = ctx.session().enemy_projectiles.spawn.primary as isize;
        record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
    } | 0x100);
    enemy!(ctx, v3 => (v1) as isize, y_fraction);
    {
        let value = native_setup_word(0xbeb9, ((2 * v3 as u8 as i32) as u16 >> 1) as isize, ctx);
        enemy!(ctx, value => (v1) as isize, program);
    };
    let v4: i32 = v2 as i32 >> 1;
    enemy!(ctx, kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as u16 => (v1) as isize, x);
    enemy!(ctx, EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as u16 => (v1) as isize, y);
    {
        let value = native_setup_word(0xbec9, v4 as isize, ctx);
        enemy!(ctx, value => (v1) as isize, x_fraction);
    };
    Eproj_SetXvelRandom(j, ctx);
    Eproj_SetYvelRandom(j, ctx);
}

pub(super) fn EprojInit_MotherBrainRoomTurretBullets(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, behavior_b);
    enemy!(ctx, 1024 => (v1) as isize, graphics);

    let v2: u16 = (2 * enemy_byte!(
        ctx,
        (ctx.session().enemy_projectiles.spawn.primary >> 1) as isize * 2,
        y_fraction
    ) as i32) as u16;
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let v3: i32 = v2 as i32 >> 1;
    let pt: Point = Point {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as u16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as u16,
    };
    enemy!(ctx, kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as u16 => (v1) as isize, velocity_x);
    enemy!(ctx, kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as u16 => (v1) as isize, velocity_y);
    let v4: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    enemy!(ctx, (pt.x as i32 + enemy!(ctx, (v4) as isize, x) as i32) as u16 => (v1) as isize, x);
    enemy!(ctx, (pt.y as i32 + enemy!(ctx, (v4) as isize, y) as i32) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_MotherBrainDeathBeemFired(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.mother_brain_beam_x() => (v1) as isize, x);
    enemy!(ctx, ctx.mother_brain_beam_x_fraction() => (v1) as isize, x_fraction);
    enemy!(ctx, ctx.mother_brain_beam_y() => (v1) as isize, y);
    enemy!(ctx, ctx.mother_brain_beam_y_fraction() => (v1) as isize, y_fraction);
    enemy!(ctx, ctx.mother_brain_beam_velocity_x() => (v1) as isize, velocity_x);
    enemy!(ctx, ctx.mother_brain_beam_velocity_y() => (v1) as isize, velocity_y);

    MoveEprojWithVelocity(j, ctx);
    ctx.set_mother_brain_beam_x(enemy!(ctx, (v1) as isize, x));
    ctx.set_mother_brain_beam_x_fraction(enemy!(ctx, (v1) as isize, x_fraction));
    ctx.set_mother_brain_beam_y(enemy!(ctx, (v1) as isize, y));
    ctx.set_mother_brain_beam_y_fraction(enemy!(ctx, (v1) as isize, y_fraction));
    let r18: u16 =
        ((ctx.mother_brain_beam_angle() as u8) as i32 + ctx.next_random() as i32) as u8 as u16;
    let rv: u16 = ctx.next_random();
    enemy!(ctx, Math_MultBySin(rv & 0x700, r18) => (v1) as isize, velocity_x);
    enemy!(ctx, Math_MultByCos(ctx.inputs().random & 0x700, r18) => (v1) as isize, velocity_y);
    MoveEprojWithVelocity(j, ctx);
    if enemy!(ctx, (v1) as isize, y) as i32 - 34i32 & 0x8000 != 0
        || enemy!(ctx, (v1) as isize, y) as i32 - 206i32 & 0x8000 == 0
        || enemy!(ctx, (v1) as isize, x) as i32 - 2i32 & 0x8000 != 0
        || enemy!(ctx, (v1) as isize, x) as i32 - 238i32 & 0x8000 == 0
    {
        enemy!(ctx, 0 => (v1) as isize, definition);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1d, ctx);
        ctx.queue_sound(3, 6, 0x13);
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
    } else {
        enemy!(ctx, (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1 & 3) as u16 => (v1) as isize, behavior_a);

        enemy!(ctx, 0 => (v1) as isize, behavior_b);
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
    };
}

pub(super) fn EprojInit_MotherBrainRainbowBeam(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    EprojPreInstr_MotherBrainRainbowBeam(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsDrool(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, behavior_a);

    EprojPreInstr_C84D(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsDeathExplosion(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    {
        let value = {
            let record = ctx.loaded_record(0x86, 0xc929);
            let index: isize = ctx.session().enemy_projectiles.spawn.primary as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v1) as isize, program);
    };
    enemy!(ctx, 1 => (v1) as isize, delay);
    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, velocity_x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, velocity_y);
    EprojPreInit_MotherBrainsDeathExplosion_0(j, ctx);
}

pub(super) fn EprojInit_MotherBrainsRainbowBeamExplosion(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, velocity_x);
    {
        let value = (ctx.session().player.x as i32
            + ctx.session().enemy_projectiles.spawn.point.x as i32) as u16;
        enemy!(ctx, value => (v1) as isize, x);
    };
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, velocity_y);
    {
        let value = (ctx.session().player.y as i32
            + ctx.session().enemy_projectiles.spawn.point.y as i32) as u16;
        enemy!(ctx, value => (v1) as isize, y);
    };
}

pub(super) fn EprojInit_MotherBrainPurpleBreathBig(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, graphics);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 6) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_MotherBrainPurpleBreathSmall(j: u16, ctx: &mut impl Host) {
    EprojInit_MotherBrainPurpleBreathBig(j, ctx);
    ctx.set_mother_brain_hurt_frame(1);
}

pub(super) fn EprojInit_MotherBrainTubeFalling(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 3584 => (v1) as isize, graphics);

    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v1) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v1) as isize, y);
    enemy!(ctx, (0x86cbea & 0xffff) as u16 => (v1) as isize, behavior_a);
}

pub(super) fn EprojInit_MaridiaFloatersSpikes(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, 0 => (v1) as isize, x_fraction);
    enemy!(ctx, 0 => (v1) as isize, y_fraction);
    enemy!(ctx, 0 => (v1) as isize, velocity_x);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2 = ctx.inputs().current_enemy as i32 as u16;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(v2 as u16), Le).y.whole => (v1) as isize, y);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.primary => (v1) as isize, behavior_a);
}

pub(super) fn EprojInit_SporeSpawnStalk(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    {
        let value = (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32
            + word_86DCB9[ctx.session().enemy_projectiles.spawn.primary as usize] as i32)
            as u16;
        enemy!(ctx, value => (v1) as isize, y);
    };
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
}

pub(super) fn EprojInit_SporeSpawners(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, kEprojInit_SporeSpawners_X[ctx.session().enemy_projectiles.spawn.primary as usize] as u16 => (v1) as isize, x);
    enemy!(ctx, 520 => (v1) as isize, y);
}

pub(super) fn EprojInit_EyeDoorSmoke(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    {
        let value = {
            let record = ctx.loaded_record(0x86, 0xe42c);
            let index: isize = ctx.session().enemy_projectiles.spawn.primary as u8 as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        enemy!(ctx, value => (v1) as isize, program);
    };
    let v2: i32 = 8 * (ctx.session().enemy_projectiles.spawn.primary >> 8) as u8 as i32 >> 1;
    let x: u16 = ({
        let record = ctx.loaded_record(0x86, 0xe47e);
        let index: isize = (v2 + 2) as isize;
        record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
    } as i32
        + ({
            let record = ctx.loaded_record(0x86, 0xe47e);
            let index = v2 as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        } as i32
            & ctx.inputs().random as i32)) as u16;
    let y: u16 = ({
        let record = ctx.loaded_record(0x86, 0xe47e);
        let index: isize = (v2 + 3) as isize;
        record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
    } as i32
        + ({
            let record = ctx.loaded_record(0x86, 0xe47e);
            let index: isize = (v2 + 1) as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        } as i32
            & ctx.inputs().random as i32 >> 8)) as u16;
    calculate_coordinates(ctx.session().mechanisms.selected, ctx);
    enemy!(ctx, (x as i32 + 8 * (2 * ctx.session().mechanisms.block_x as i32 + 1)) as u16 => (v1) as isize, x);
    enemy!(ctx, (y as i32 + 8 * (2 * ctx.session().mechanisms.block_y as i32 + 1)) as u16 => (v1) as isize, y);
    ctx.next_random();
}

pub(super) fn EprojInit_BotwoonsBody(j: u16, ctx: &mut impl Host) {
    let v1: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).y.whole => (v1) as isize, y);
    enemy!(ctx, 0 => (v1) as isize, velocity_y);

    let v2: u16 = (if ctx.resolve_actor(Key::from_legacy(0), Le).variables[0] != 0 {
        16
    } else {
        48
    }) as u16;
    let v3: u16 = native_setup_word(0xe9f1, (v2 >> 1) as isize, ctx);

    enemy!(ctx, v3 => (v1) as isize, program);
    enemy!(ctx, v3 => (v1) as isize, behavior_b);
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    enemy!(ctx, (0x86ea98 & 0xffff) as u16 => (v1) as isize, velocity_x);

    let Ex = ctx.resolve_actor(Key::from_legacy(0), Le).variables[0];
    ctx.set_enemy_linked_projectile(Ex, j);
    enemy!(ctx, 2 => (v1) as isize, drop_flags);
    ctx.set_botwoon_body_active(Ex, 1);
}

pub(super) fn EprojInit_CrocomireProjectile(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, -512i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, 1 => (v2) as isize, velocity_y);

    let v3 = ctx.selected_actor();
    enemy!(ctx, (ctx.resolve_actor(v3, Le).x.whole as i32 - 32i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(v3, Le).y.whole as i32 - 16i32) as u16 => (v2) as isize, y);

    enemy!(ctx, 0 => (v2) as isize, counter);
    enemy!(ctx, 0 => (v2) as isize, x_fraction);
    enemy!(ctx, 0 => (v2) as isize, y_fraction);

    enemy!(ctx, 2560 => (v2) as isize, graphics);
}

pub(super) fn EprojInit_PirateMotherBrainLaser(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    {
        let value = (if ctx.session().enemy_projectiles.spawn.variant != 0 {
            addr_word_869F7D as i32
        } else {
            addr_word_869F41 as i32
        }) as u16;
        enemy!(ctx, value => (v2) as isize, program);
    };
    enemy!(ctx, (0x86a05b & 0xffff) as u16 => (v2) as isize, pre_action);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.x => (v2) as isize, x);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v2) as isize, y);
    let enemy_index = ctx.inputs().current_enemy;
    enemy!(ctx, ctx.native_record(0xa0, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).definition).word_le(3) | 0x1000 => (v2) as isize, properties);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).parameters[0] => (v2) as isize, behavior_a);
    ctx.queue_sound(2, 6, 0x67);
}

pub(super) fn EprojInit_GoldenTorizoSuperMissile(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, behavior_a);
    let v3 = ctx.inputs().current_enemy as i32 as u16;
    let v4: i32 = if (ctx
        .resolve_actor(Key::from_legacy(v3 as u16), Le)
        .parameters[0]
        & 0x8000)
        != 0
    {
        1
    } else {
        0
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v3 as u16), Le).x.whole as i32 + word_86B205[v4 as usize] as i32) as u16 => (v2) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v3 as u16), Le).y.whole as i32 - 52i32) as u16 => (v2) as isize, y);
    {
        let value = native_setup_word(0xb209, v4 as isize, ctx);
        enemy!(ctx, value => (v2) as isize, program);
    };
}

pub(super) fn EprojInit_KiHunterAcidSpitLeft(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, -768i32 as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(ctx.inputs().current_enemy), Le).x.whole as i32 - 22i32) as u16 => (v2) as isize, x);
    EprojInit_KiHunterAcidSpitCommon(ctx.inputs().current_enemy, j, ctx);
}

pub(super) fn EprojInit_KiHunterAcidSpitRight(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, 768 => (v2) as isize, velocity_x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(ctx.inputs().current_enemy), Le).x.whole as i32 + 22) as u16 => (v2) as isize, x);
    EprojInit_KiHunterAcidSpitCommon(ctx.inputs().current_enemy, j, ctx);
}

pub(super) fn EprojInit_KagosBugs(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, behavior_b);
    let enemy_index = ctx.inputs().current_enemy;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    let v4: u16 = ((ctx.inputs().random as i32 & 7) + 1) as u16;

    enemy!(ctx, v4 => (v2) as isize, behavior_c);
    enemy!(ctx, v4.wrapping_add(4) => (v2) as isize, behavior_a);
    enemy!(ctx, (0x86d0cai32) as u16 => (v2) as isize, pre_action);
}

pub(super) fn EprojInit_WreckedShipRobotLaserDown(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, 128, true, ctx);
}

pub(super) fn EprojInit_WreckedShipRobotLaserHorizontal(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, 0, false, ctx);
}

pub(super) fn EprojInit_WreckedShipRobotLaserUp(j: u16, ctx: &mut impl Host) {
    init_robot_laser(j, (-128i16) as u16, false, ctx);
}

pub(super) fn EprojInit_DBF2(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, addr_word_86DB0C as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86db62 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, (0x86db8c & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    enemy!(ctx, -256i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 256 => (v2) as isize, velocity_x);
}

pub(super) fn EprojInit_Spores(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(
        ctx,
        v2 as isize,
        x = ctx.session().enemy_projectiles.spawn.point.x
    );

    enemy!(ctx, enemy!(ctx, v2 as isize, x) => (v2) as isize, behavior_b);
    enemy!(ctx, ctx.session().enemy_projectiles.spawn.point.y => (v2) as isize, y);
    enemy!(ctx, 512 => (v2) as isize, graphics);
}

pub(super) fn EprojInit_NamiFuneFireball(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, addr_word_86DE96 as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86df40 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, addr_word_86DEA6 as u16 => (v2) as isize, program);
        enemy!(ctx, (0x86df6a & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 0, ctx);

    if (ctx
        .resolve_actor(Key::from_legacy(v3 as u16), Le)
        .parameters[0]
        & 0xf)
        != 0
    {
        {
            let index = (v2) as isize;
            let old = enemy!(ctx, (index) as isize, y);
            let value = ((old as i32) + (4)) as u16;
            enemy!(ctx, (index) as isize, y = value);
        };
    }
    let v4: i32 = (4
        * (ctx
            .resolve_actor(Key::from_legacy(v3 as u16), Le)
            .parameters[1] as u8) as i32) as u16 as i32
        >> 1;
    {
        let value = native_setup_word(0xdeb6, v4 as isize, ctx);
        enemy!(ctx, value => (v2) as isize, velocity_y);
    };
    {
        let value = native_setup_word(0xdeb6, (v4 + 1) as isize, ctx);
        enemy!(ctx, value => (v2) as isize, velocity_x);
    };
}

pub(super) fn EprojInit_LavaThrownByLavaman(j: u16, ctx: &mut impl Host) {
    let v2: i32 = j as i32 >> 1;

    enemy!(ctx, addr_word_86DFD8 as u16 => (v2) as isize, program);
    enemy!(ctx, (0x86e050 & 0xffff) as u16 => (v2) as isize, behavior_a);
    if ctx.session().enemy_projectiles.spawn.primary != 0 {
        enemy!(ctx, addr_word_86DFDE as u16 => (v2) as isize, program);
        enemy!(ctx, (0x86e07a & 0xffff) as u16 => (v2) as isize, behavior_a);
    }
    let v3 = ctx.inputs().current_enemy as i32 as u16;

    copy_enemy_position(v2 as isize, v3, 2, ctx);
    enemy!(ctx, -768i32 as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, 768 => (v2) as isize, velocity_x);
}

pub(super) fn EprojInit_MiniKraidSpit(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let extra_index = ctx.inputs().current_enemy;
    let v3: i32 = j as i32 >> 1;
    {
        let value = (ctx.mini_kraid_spit_x_offset(extra_index) as i32
            + ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole as i32)
            as u16;
        enemy!(ctx, value => (v3) as isize, x);
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole as i32 - 16i32) as u16 => (v3) as isize, y);

    enemy!(ctx, 0 => (v3) as isize, y_fraction);
    enemy!(ctx, 0 => (v3) as isize, x_fraction);

    enemy!(ctx, ctx.mini_kraid_spit_x_velocity(extra_index) => (v3) as isize, velocity_x);
    enemy!(ctx, ctx.mini_kraid_spit_y_velocity(extra_index) => (v3) as isize, velocity_y);
}

pub(super) fn EprojInit_BombTorizoLowHealthExplode(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[0]
        & 0x8000)
        == 0
    {
        {
            let old = ctx.session().enemy_projectiles.spawn.primary;
            let value = ((old as i32) + (2)) as u16;
            ctx.session_mut().enemy_projectiles.spawn.primary = value;
        };
    }
    {
        let old = ctx.session().enemy_projectiles.spawn.primary;
        ctx.session_mut().enemy_projectiles.spawn.primary = old.wrapping_add(2);
    };
    let v3: i32 = ctx.session().enemy_projectiles.spawn.primary as i32 >> 1;
    let v4: u16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize] as i32
        + enemy!(ctx, (v2) as isize, x) as i32) as u16;

    enemy!(ctx, v4 => (v2) as isize, x);
    enemy!(ctx, v4 => (v2) as isize, behavior_a);

    let v5: u16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize] as i32
        + enemy!(ctx, (v2) as isize, y) as i32) as u16;

    enemy!(ctx, v5 => (v2) as isize, y);
    enemy!(ctx, v5 => (v2) as isize, behavior_b);
}

pub(super) fn EprojInit_BombTorizoDeathExplosion(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let x_pos: u16 = ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole;
    let v3: i32 = j as i32 >> 1;

    enemy!(ctx, x_pos => (v3) as isize, x);
    enemy!(ctx, x_pos => (v3) as isize, behavior_a);

    let y_pos: u16 = ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole;

    enemy!(ctx, y_pos => (v3) as isize, y);
    enemy!(ctx, y_pos => (v3) as isize, behavior_b);
}

pub(super) fn EprojInit_BombTorizosChozoOrbs(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: u16 = (if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[0]
        & 0x8000)
        != 0
    {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point {
            x: ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole,
            y: ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole,
        },
        ctx,
    );
}

pub(super) fn EprojInit_TourianLandingDustCloudsRightFoot(j: u16, ctx: &mut impl Host) {
    init_landing_dust(j, 24, ctx);
}

pub(super) fn EprojInit_TorizoLandingDustCloudLeftFoot(j: u16, ctx: &mut impl Host) {
    init_landing_dust(j, -24, ctx);
}

pub(super) fn EprojInit_YappingMawsBody(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v3: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v3) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v3) as isize, y);
    enemy!(ctx, addr_word_86EC5C as u16 => (v3) as isize, program);
    if ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .parameters[1]
        == 0
    {
        enemy!(ctx, addr_word_86EC56 as u16 => (v3) as isize, program);
    }
    let base = ctx.inputs().current_enemy;
    let part = ctx.maw_body_part_index(ctx.inputs().current_enemy);
    ctx.set_maw_body_link((base as i32 + 2 * part as i32) as u16, j);
}

pub(super) fn EprojInit_EnemyDeathExplosion(j: u16, ctx: &mut impl Host) {
    let enemy_index = ctx.inputs().current_enemy;

    let v2: i32 = j as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).y.whole => (v2) as isize, y);
    enemy!(ctx, ctx.inputs().current_enemy => (v2) as isize, killed_enemy);
    if (ctx
        .resolve_actor(Key::from_legacy(enemy_index), Le)
        .properties
        & 0x4000)
        != 0
    {
        enemy!(ctx, (ctx.inputs().current_enemy) | (0x8000u16) => (v2) as isize, killed_enemy);
    }
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(enemy_index), Le).definition => (v2) as isize, drop_definition);
    enemy!(ctx, 0 => (v2) as isize, graphics);
    {
        let instruction: u16 = {
            let record = ctx.loaded_record(0x86, 0xefd5);
            let index: isize = ctx.session().enemy_projectiles.spawn.primary as isize;
            record.native_word(index.checked_mul(2).expect("ROM table offset overflow"))
        };
        {
            let index = (v2 as usize) as isize;
            enemy!(ctx, index, program = instruction);
            enemy!(ctx, index, delay = 1);
        };
    }
}

pub(super) fn EprojInit_9688(j: u16, ctx: &mut impl Host) {
    EprojInit_FireballChild(j, 0, (-3584i16) as u16, 0x9688, ctx)
}

pub(super) fn EprojInit_9696(j: u16, ctx: &mut impl Host) {
    EprojInit_FireballChild(j, 0, 3584, 0x9696, ctx)
}

pub(super) fn EprojInit_966C(j: u16, ctx: &mut impl Host) {
    EprojInit_FireballChild(j, 3584, 0, 0x966c, ctx)
}

pub(super) fn EprojInit_967A(j: u16, ctx: &mut impl Host) {
    EprojInit_FireballChild(j, (-3584i16) as u16, 0, 0x967a, ctx)
}

pub(super) fn EprojInit_MiniKraidSpikesLeft(j: u16, ctx: &mut impl Host) {
    EprojInit_MiniKraidSpikes(j, -0x200i32 as u16, ctx)
}

pub(super) fn EprojInit_MiniKraidSpikesRight(j: u16, ctx: &mut impl Host) {
    EprojInit_MiniKraidSpikes(j, 0x200, ctx)
}

pub(super) fn EprojInit_SpawnedShotGate(j: u16, ctx: &mut impl Host) {
    EprojInit_SpawnedGate_Common(j, 0, ctx)
}

pub(super) fn EprojInit_ClosedDownwardsShotGate(j: u16, ctx: &mut impl Host) {
    EprojInit_SpawnedGate_Common(j, 0x40, ctx)
}

pub(super) fn EprojInit_ClosedUpwardsShotGate(j: u16, ctx: &mut impl Host) {
    EprojInit_SpawnedGate_Common(j, -0x40i32 as u16, ctx)
}
