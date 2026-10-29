//! Owned enemy-projectile pre-actions; compatibility and effects stay at the execution edge.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_add, enemy_byte, enemy_decrement, read_member};
use super::host::{EnemyMember as SlotField, Host, LoadedRecord};
use super::pre_helpers::*;
use super::shared::SpawnEprojWithRoomGfx;
use super::*;
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::{
    sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn},
    sprite_objects,
};
use sm_game::geometry::Point;

pub(super) fn dispatch(ea: u32, k: u16, ctx: &mut impl Host) {
    match ea {
        8817501 => Eproj_PreInstr_SkreeParticle(k, ctx),
        8837606 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx),
        8837690 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx),
        8837732 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx),
        8837794 => EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k, ctx),
        8837810 => EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k, ctx),
        8840446 => EprojPreInstr_DustCloudOrExplosion(k, ctx),
        8818122 => EprojPreInstr_8DCA(k, ctx),
        8818175 => EprojPreInstr_DraygonsTurret_8DFF(k, ctx),
        8818191 => EprojPreInstr_DraygonsGunk_8E0F(k, ctx),
        8818795 => EprojPreInstr_CrocomireProjectile(k, ctx),
        8818867 => sub_8690B3(k, ctx),
        8819386 => EprojPreInstr_CrocomireBridgeCrumbling(k, ctx),
        8819726 => EprojPreInstr_9642_RidleysFireball(k, ctx),
        8819981 => EprojPreInstr_966C(k, ctx),
        8820002 => EprojPreInstr_9688(k, ctx),
        8820023 => EprojPreInstr_96A4(k, ctx),
        8820032 => EprojPreInstr_96C0(k, ctx),
        8820481 => EprojPreInstr_9734_CeresFallingDebris(k, ctx),
        8821317 => EprojPreInstr_PhantoonDestroyableFireballs(k, ctx),
        8821466 => EprojPreInstr_PhantoonDestroyableFireballs_3(k, ctx),
        8821545 => EprojPreInstr_PhantoonStartingFireballsB(k, ctx),
        8822102 => EprojPreInstr_KraidRocks(k, ctx),
        8822153 => EprojPreInstr_RocksFallingKraidCeiling(k, ctx),
        8822181 => sub_869DA5(k, ctx),
        8822302 => EprojPreInit_MiniKraidSpit(k, ctx),
        8822403 => EprojPreInstr_MiniKraidSpikes(k, ctx),
        8822527 => EprojPreInstr_WalkingLavaSeahorseFireball(k, ctx),
        8823592 => EprojPreInstr_CeresElevatorPad(k, ctx),
        8823652 => EprojPreInstr_CeresElevatorPlatform(k, ctx),
        8825071 => EprojPreInstr_A977(k, ctx),
        8826029 => EprojPreInstr_BombTorizosChozoOrbs(k, ctx),
        8826106 => EprojPreInstr_GoldenTorizosChozoOrbs(k, ctx),
        8826476 => EprojPreInstr_TorizoSonicBoom(k, ctx),
        8827065 => sub_86B0B9(k, ctx),
        8827101 => sub_86B0DD(k, ctx),
        8827447 => EprojPreInstr_B237(k, ctx),
        8827786 => EprojPreInstr_GoldenTorizoEyeBeam(k, ctx),
        8828601 => EprojPreInstr_EyeDoorProjectile(k, ctx),
        8828692 => EprojPreInstr_EyeDoorSweat(k, ctx),
        8829303 => EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k, ctx),
        8829495 => EprojPreInstr_TourianStatueStuff(k, ctx),
        8830238 => EprojPreInstr_NorfairLavaquakeRocks_Inner(k, ctx),
        8829967 => EprojPreInstr_NorfairLavaquakeRocks(k, ctx),
        8830943 => EprojPreInstr_MotherBrainRoomTurrets(k, ctx),
        8832200 => EprojPreInstr_MotherBrainBomb(k, ctx),
        8833044 => EprojPreInstr_MotherBrainRainbowBeam(k, ctx),
        8833300 => EprojPreInit_MotherBrainsDeathExplosion_0(k, ctx),
        8834023 => EprojPreInstr_MotherBrainTubeFalling(k, ctx),
        8834715 => EprojPreInstr_MotherBrainGlassShatteringShard(k, ctx),
        8835274 => EprojPreInstr_KagosBugs(k, ctx),
        8835308 => EprojPreInstr_D0EC(k, ctx),
        8837117 => EprojPreInstr_N00bTubeShards(k, ctx),
        8837181 => sub_86D83D(k, ctx),
        8837279 => sub_86D89F(k, ctx),
        8837343 => sub_86D8DF(k, ctx),
        8837789 => EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k, ctx),
        8837802 => EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k, ctx),
        8837818 => EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k, ctx),
        8837595 => EprojPreInstr_SpikeShootingPlantSpikes(k, ctx),
        8838070 => EprojPreInstr_DBF2_Func1(k, ctx),
        8837979 => EprojPreInstr_DBF2(k, ctx),
        8838470 => EprojPreInstr_SporeSpawners(k, ctx),
        8841856 => EprojPreInstr_BotwoonsBody(k, ctx),
        8842245 => EprojPreInstr_BotwoonsSpit(k, ctx),
        8824967 => sub_86A887(k, ctx),
        8835368 => EprojPreInstr_D128(k, ctx),
        8838969 => EprojPreInstr_NamiFuneFireball(k, ctx),
        8839241 => sub_86E049(k, ctx),
        8829506 => EprojPreInstr_BA42(k, ctx),
        8833158 => EprojInit_MotherBrainsDrool_Falling(k, ctx),
        8830467 => EprojInit_ShaktoolAttackFrontCircle(k, ctx),
        8817277 => Eproj_PreInit_0x8aaf(k, ctx),
        8818965 => EprojPreInstr_CrocomireSpikeWallPieces(k, ctx),
        8819602 => EprojPreInstr_9634(k, ctx),
        8821121 => EprojPreInstr_PhantoonStartingFireballs(k, ctx),
        8821183 => EprojPreInstr_PhantoonStartingFireballs2(k, ctx),
        8821249 => EprojPreInstr_PhantoonStartingFireballs3(k, ctx),
        8821396 => EprojPreInstr_PhantoonDestroyableFireballs_2(k, ctx),
        8821569 => EprojPreInstr_PhantoonStartingFireballsB_2(k, ctx),
        8822876 => EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, ctx),
        8822906 => EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, ctx),
        8822993 => EprojPreInstr_PirateClawThrownLeft(k, ctx),
        8823076 => EprojPreInstr_PirateClawThrownRight(k, ctx),
        8825484 => EprojPreInstr_AB07(k, ctx),
        8826947 => EprojPreInstr_GoldenTorizoEgg(k, ctx),
        8827405 => EprojPreInstr_GoldenTorizoSuperMissile(k, ctx),
        8828213 => sub_86B535(k, ctx),
        8829314 => EprojPreInstr_TourianStatueUnlockingParticle(k, ctx),
        8829437 => EprojPreInstr_TourianStatueSoul(k, ctx),
        8830482 => EprojPreInstr_BE12(k, ctx),
        8831200 => EprojPreInstr_MotherBrainRoomTurretBullets(k, ctx),
        8831797 => Eproj_MoveToBlueRingSpawnPosition(k, ctx),
        8833101 => EprojPreInstr_C84D(k, ctx),
        8833356 => EprojPreInstr_MotherBrainsRainbowBeamExplosion(k, ctx),
        8833490 => EprojPreInstr_MotherBrainsExplodedDoorParticles(k, ctx),
        8833786 => EprojPreInstr_TimeBombSetJapaneseText(k, ctx),
        8835029 => sub_86CFD5(k, ctx),
        8835046 => sub_86CFE6(k, ctx),
        8835064 => sub_86CFF8(k, ctx),
        8835251 => EprojPreInstr_KagosBugs_Func1(k, ctx),
        8835556 => EprojPreInstr_KagosBugs_Func2(k, ctx),
        8835683 => EprojPreInstr_MaridiaFloatersSpikes(k, ctx),
        8836031 => EprojPreInstr_WreckedShipRobotLaser(k, ctx),
        8837055 => sub_86D7BF(k, ctx),
        8837086 => sub_86D7DE(k, ctx),
        8837648 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx),
        8837986 => EprojPreInstr_DBF2_MoveX1(k, ctx),
        8838382 => EprojPreInstr_Spores(k, ctx),
        8840709 => EprojPreInstr_E605(k, ctx),
        8844272 => EprojPreInstr_Sparks(k, ctx),
        8837774 => EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k, ctx),
        8837779 => EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k, ctx),
        8837784 => EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k, ctx),
        8843232 => collect_drop(k, ctx),
        8825112 => (),
        8814960 => (),
        8815867 => (),
        8818004 => (),
        8819980 => (),
        8820041 => EprojPreInstr_96CE(k, ctx),
        8821316 => (),
        8822875 => (),
        8823591 => (),
        8823715 => EprojPreInstr_PrePhantomRoom(k, ctx),
        8825113 => (),
        8828080 => (),
        8829775 => (),
        8829894 => (),
        8830250 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, ctx);
        }
        8833699 => (),
        8835063 => (),
        8835307 => (),
        8838028 => EprojPreInstr_DBF2_MoveX2(k, ctx),
        8838468 => (),
        8840456 => (),
        8840708 => (),
        8840913 => (),
        8842388 => (),
        8843231 => (),
        8832877 => (),
        _ => ctx.unreachable(),
    }
}

pub(super) fn Eproj_PreInstr_SkreeParticle(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    integrate_shard(v1 as isize, SlotField::X, SlotField::XFraction, ctx);
    enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 80);
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

fn move_spike_axis(k: u16, x: bool, velocity: SlotField, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    let amount = ((read_member(ctx, velocity, index) as i16 as i32) << 8) as u32;
    let (high, low) = if x {
        (SlotField::X, SlotField::XFraction)
    } else {
        (SlotField::Y, SlotField::YFraction)
    };
    add_split(high, low, index, amount, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, true, SlotField::VelocityY, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx);
}

pub(super) fn EprojPreInstr_DustCloudOrExplosion(k: u16, ctx: &mut impl Host) {
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    }
}

pub(super) fn EprojPreInstr_8DCA(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    if ctx.session().player.contact.contact_attack != 0
        || {
            let mut v1: i32 = k as i32 >> 1;
            {
                enemy!(ctx, ctx.session().player.x => (v1) as isize, x);
                enemy!(ctx, (ctx.session().player.y as i32 + 4 * enemy!(ctx, (v1) as isize, behavior_b) as i32
                    - 12i32) as u16 => (v1) as isize, y);
            }
            enemy_decrement!(ctx, (v1) as isize, behavior_a);
            enemy!(ctx, (v1) as isize, behavior_a) == 0
        }
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
        if ({
            let old = ctx.session().player.speed_divisor;
            let value = old.wrapping_sub(1);
            ctx.session_mut().player.speed_divisor = value;
            ctx.session().player.speed_divisor
        } & 0x8000)
            != 0
        {
            {
                let value = 0;
                ctx.session_mut().player.speed_divisor = value;
            };
        }
    }
}

pub(super) fn EprojPreInstr_DraygonsTurret_8DFF(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    Eproj_FuncE73E_MoveXY(k, ctx);
    let mut v1: i16 = Eproj_FuncE722(k, ctx) as i16;
    if v1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_DraygonsGunk_8E0F(k: u16, ctx: &mut impl Host) {
    EprojPowerBombCollision(k, ctx);
    Eproj_FuncE73E_MoveXY(k, ctx);
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (((ctx.session().player.x as i32 - enemy!(ctx, (v1) as isize, x) as i32) as u16)
        as i16)
        .unsigned_abs();
    if v2 as i32 - 16i32 & 0x8000 != 0 && {
        let mut v3: u16 = (((ctx.session().player.y as i32 - enemy!(ctx, (v1) as isize, y) as i32)
            as u16) as i16)
            .unsigned_abs();
        v3 as i32 - 20i32 & 0x8000 != 0
    } {
        {
            enemy!(ctx, 0x8c38 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
        }
    } else if Eproj_FuncE722(k, ctx) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_CrocomireProjectile(k: u16, ctx: &mut impl Host) {
    EprojBlockCollisition_Horiz(k, ctx);
    enemy!(ctx, 2560 => (0) as isize, graphics);
    {
        let index = (k >> 1) as isize;
        let old = enemy!(ctx, (index) as isize, counter);
        let value = ((old as i32) + (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32)) as u16;
        enemy!(ctx, (index) as isize, counter = value);
    };
    let x: u16 = -64i32 as u16;
    let y: u16 = {
        let record = ctx.loaded_record(0x86, 0x9059);
        record.native_word(
            ((ctx.resolve_actor(Key::from_legacy(0), Le).pre_instruction >> 1) as isize)
                .checked_mul(2)
                .unwrap(),
        )
    };
    let v1: i32 = ctx.calculate_angle_from_xy(x, y) as i32;
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[(v1 + 64) as usize] as i32) as u16 => (v2) as isize, velocity_x);
    enemy!(ctx, (4 * kSinCosTable8bit_Sext[v1 as usize] as i32) as u16 => (v2) as isize, velocity_y);
    enemy!(ctx, (0x8690b3 & 0xffff) as u16 => (v2) as isize, pre_action);
}

pub(super) fn sub_8690B3(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_CrocomireBridgeCrumbling(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    } else {
        enemy!(ctx, (enemy!(ctx, (k >> 1) as isize, velocity_y) as i32 + 24 & 0x3fff) as u16 => (k >> 1) as isize, velocity_y);
    };
}

pub(super) fn EprojPreInstr_9642_RidleysFireball(k: u16, ctx: &mut impl Host) {
    let mut v1: u16 = if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        -27042i32 as u16
    } else {
        if EprojBlockCollisition_Vertical(k, ctx) & 1 == 0 {
            return;
        }
        -27056i32 as u16
    };
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, (v2 as usize) as isize, definition = 0);
    if enemy!(ctx, (v2 as usize) as isize, behavior_b) == 0 {
        SetEprojSpawnPoint(k, ctx);
        SpawnEprojWithRoomGfx(v1, 3, ctx);
        ctx.queue_sound(2, 6, 0x2b);
    }
}

pub(super) fn EprojPreInstr_966C(k: u16, ctx: &mut impl Host) {
    MoveEprojWithVelocityX(k, ctx);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        enemy!(ctx, (slot) as isize, program = addr_off_869574 as u16);
        enemy!(ctx, (slot) as isize, delay = 1);
    }
}

pub(super) fn EprojPreInstr_9688(k: u16, ctx: &mut impl Host) {
    MoveEprojWithVelocityY(k, ctx);
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        enemy!(ctx, (slot) as isize, program = addr_off_869574 as u16);
        enemy!(ctx, (slot) as isize, delay = 1);
    }
}

pub(super) fn EprojPreInstr_96A4(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_96C0(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_9734_CeresFallingDebris(k: u16, ctx: &mut impl Host) {
    let slot = usize::from(k >> 1);
    enemy_add!(ctx, (slot) as isize, velocity_y, 16);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        enemy!(ctx, (slot) as isize, definition = 0);
        SetEprojSpawnPoint(k, ctx);
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
        ctx.queue_sound(2, 6, 0x6d);
    }
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs(k: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 4);
    let v2: u16 = (enemy_byte!(ctx, v1 as isize * 2, velocity_x) as i32
        + enemy!(ctx, (v1) as isize, behavior_a) as i32) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs_3(k: u16, ctx: &mut impl Host) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 2);
    let v2: u16 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 2) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballsB(k: u16, ctx: &mut impl Host) {
    if ctx.resolve_actor(Key::from_legacy(0), Le).variables[1] != 0 {
        let v1: i32 = k as i32 >> 1;
        {
            enemy!(ctx, (0x869b41 & 0xffff) as u16 => (v1) as isize, pre_action);
            enemy!(ctx, 180 => (v1) as isize, velocity_x);
            enemy!(ctx, 48 => (v1) as isize, velocity_y);
        }
    }
}

pub(super) fn EprojPreInstr_KraidRocks(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_x, 8);
        if v2 as i32 - 256i32 & 0x8000 == 0 {
            v2 = -256i32 as u16;
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_x);
        enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 64);
    };
}

pub(super) use self::EprojPreInstr_CrocomireBridgeCrumbling as EprojPreInstr_RocksFallingKraidCeiling;

pub(super) fn sub_869DA5(k: u16, ctx: &mut impl Host) {
    enemy!(ctx, 0 => (k >> 1) as isize, graphics);
}

pub(super) fn EprojPreInit_MiniKraidSpit(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: i16 = ((enemy!(ctx, (v1) as isize, velocity_y)) as i16).wrapping_add(64);
        if v2 >= 0 && enemy!(ctx, (v1) as isize, velocity_y) as i32 - 960i32 & 0x8000 == 0 {
            v2 = 1024;
        }
        enemy!(ctx, v2 as u16 => (v1) as isize, velocity_y);
    };
}

pub(super) fn EprojPreInstr_MiniKraidSpikes(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        enemy!(ctx, (usize::from(k >> 1)) as isize, definition = 0);
    }
}

pub(super) fn EprojPreInstr_WalkingLavaSeahorseFireball(k: u16, ctx: &mut impl Host) {
    let mut v2: u16 = 0;
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
        || EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    } else {
        let v1: i32 = k as i32 >> 1;
        if (enemy!(ctx, (v1) as isize, velocity_x) & 0x8000) == 0 {
            v2 = (enemy!(ctx, (v1) as isize, velocity_x) as i32 - 64i32) as u16;
            enemy!(ctx, v2 => (v1) as isize, velocity_x);
            if v2 as i32 - 512i32 & 0x8000 != 0 {
                v2 = 512;
            }
        } else {
            v2 = (enemy!(ctx, (v1) as isize, velocity_x) as i32 + 64) as u16;
            enemy!(ctx, v2 => (v1) as isize, velocity_x);
            if v2 as i32 + 512 & 0x8000 == 0 {
                v2 = -512i32 as u16;
            }
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_x);
    };
}

pub(super) fn EprojPreInstr_CeresElevatorPad(k: u16, ctx: &mut impl Host) {
    let mut v3: bool = false;
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) == 0
        || {
            let mut v2: bool = enemy!(ctx, (v1) as isize, behavior_a) == 1;
            v3 = ((enemy!(ctx, (v1) as isize, behavior_a)) as i16).wrapping_sub(1) < 0;
            enemy_decrement!(ctx, (v1) as isize, behavior_a);
            v2 as i32 != 0
        }
        || v3
    {
        enemy!(ctx, ctx.session().player.y.wrapping_add(28) => (v1) as isize, y);
        let fresh113 = {
            let value = ctx.session().player.y.wrapping_add(1);
            ctx.session_mut().player.y = value;
            value
        };
        if fresh113 as i32 - 73i32 & 0x8000 == 0 {
            {
                {
                    let value = 72;
                    ctx.session_mut().player.y = value;
                };
                enemy!(ctx, 1 => (v1) as isize, delay);
                enemy!(ctx, addr_off_86A28B as u16 => (v1) as isize, program);
            }
            ctx.resume_player_after_elevator();
        }
    }
}

pub(super) fn EprojPreInstr_CeresElevatorPlatform(k: u16, ctx: &mut impl Host) {
    if ctx.session().player.y == 72 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, addr_off_86A28B as u16 => (v1) as isize, program);
    }
}

pub(super) fn EprojPreInstr_A977(k: u16, ctx: &mut impl Host) {
    let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
        let v2: u16 = (enemy!(ctx, (0) as isize, behavior_b)).wrapping_add(enemy!(
            ctx,
            (v1) as isize,
            velocity_y
        ));
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if (v2 & 0xf000) == 4096 {
            enemy!(ctx, 4096 => (v1) as isize, velocity_y);
        }
    } else {
        enemy!(ctx, 0xa918 => (v1) as isize, pre_action);
    };
}

pub(super) fn EprojPreInstr_BombTorizosChozoOrbs(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) != 0 {
        let v3: i32 = k as i32 >> 1;
        start_instruction((v3 as usize) as isize, addr_off_86AB25 as u16, ctx);
    } else {
        let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
        let v1: i32 = k as i32 >> 1;
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
            let v2: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 18);
            if (v2 & 0xf000) == 4096 {
                enemy!(ctx, 0 => (v1) as isize, definition);
            }
        } else {
            let v4: i32 = k as i32 >> 1;
            enemy!(ctx, (enemy!(ctx, (k >> 1) as isize, y) & 0xfff0 | 8).wrapping_sub(2) => (v4) as isize, y);
            start_instruction((v4 as usize) as isize, addr_off_86AB41 as u16, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizosChozoOrbs(k: u16, ctx: &mut impl Host) {
    let mut v1: i32 = 0;
    let mut v3: u16 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        enemy!(ctx, -(enemy!(ctx, (k >> 1) as isize, velocity_x) as i32) as u16 => (k >> 1) as isize, velocity_x);
    }
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
        && {
            v1 = k as i32 >> 1;
            (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0
        }
        && {
            let mut v2: i16 = enemy!(ctx, (v1) as isize, velocity_x) as i16;
            if v2 >= 0 {
                v3 = (v2 as i32 - 64i32) as u16;
            } else {
                v3 = (v2 as i32 + 64) as u16;
            };
            enemy!(ctx, v3 => (v1) as isize, velocity_x);
            let mut v4: u16 = -(enemy!(ctx, (v1) as isize, velocity_y) as i32 >> 1i32) as u16;
            enemy!(ctx, v4 => (v1) as isize, velocity_y);
            (v4 & 0xff80) == 0xff80
        }
    {
        let v5: i32 = k as i32 >> 1;
        {
            enemy!(ctx, ((enemy!(ctx, (v5) as isize, y) as i32 & 0xfff0 | 8) - 2i32) as u16 => (v5) as isize, y);
            enemy!(ctx, addr_off_86AB41 as u16 => (v5) as isize, program);
            enemy!(ctx, 1 => (v5) as isize, delay);
        }
    } else {
        enemy_add!(ctx, (k >> 1) as isize, velocity_y, 24);
    };
}

pub(super) fn EprojPreInstr_TorizoSonicBoom(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let v4: i32 = k as i32 >> 1;
        {
            enemy!(ctx, addr_off_86ADE5 as u16 => (v4) as isize, program);
            enemy!(ctx, 1 => (v4) as isize, delay);
            enemy!(ctx, enemy!(ctx, (v4) as isize, x) => (v4) as isize, behavior_a);
            enemy!(ctx, enemy!(ctx, (v4) as isize, y) => (v4) as isize, behavior_b);
        }
    } else {
        let v1: i32 = k as i32 >> 1;
        {
            let index = (v1) as isize;
            let old = enemy!(ctx, (index) as isize, velocity_x);
            let value = ((old as i32)
                + (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                    -16i32
                } else {
                    16
                })) as u16;
            enemy!(ctx, (index) as isize, velocity_x = value);
        };
        if (enemy!(ctx, (v1) as isize, velocity_x) & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub(super) fn sub_86B0B9(k: u16, ctx: &mut impl Host) {
    let mut v2: i16 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let v3: i32 = k as i32 >> 1;
        {
            enemy!(ctx, 0xb0dd => (v3) as isize, pre_action);
            enemy!(ctx, 0 => (v3) as isize, velocity_y);
        }
    } else {
        let v1: i32 = k as i32 >> 1;
        if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
            v2 = 48;
        } else {
            v2 = -48i16;
        }
        enemy_add!(ctx, (v1) as isize, velocity_x, v2 as u16);
    };
}

pub(super) fn sub_86B0DD(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        enemy!(ctx, (if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
                addr_off_86B1A8 as i32
            } else {
                addr_off_86B190 as i32
            }) as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else {
        enemy_add!(ctx, (k >> 1) as isize, velocity_y, 48);
    };
}

pub(super) fn EprojPreInstr_B237(k: u16, ctx: &mut impl Host) {
    let mut v1: i32 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) != 0 || {
        let mut carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
        v1 = k as i32 >> 1;
        (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0 && carry != 0
    } {
        let v3: i32 = k as i32 >> 1;
        enemy!(ctx, addr_off_86B2EF as u16 => (v3) as isize, program);
        enemy!(ctx, 1 => (v3) as isize, delay);
    } else {
        let v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 16) as u16;
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if (v2 & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizoEyeBeam(k: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        enemy!(ctx, addr_off_86B3CD as u16 => (k >> 1) as isize, program);
        enemy!(ctx, 1 => (k >> 1) as isize, delay);
        return;
    }
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        {
            enemy!(ctx, ((enemy!(ctx, (v1) as isize, y) as i32 & 0xfff0 | 8) - 2i32) as u16 => (v1) as isize, y);
            enemy!(ctx, addr_off_86B3E5 as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
        }
    }
}

pub(super) fn EprojPreInstr_EyeDoorProjectile(k: u16, ctx: &mut impl Host) {
    let v3: i32 = k as i32 >> 1;
    if !(EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0)
    {
        let v1: i32 = enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 >> 1;
        {
            let index = (v3) as isize;
            let old = enemy!(ctx, (index) as isize, velocity_x);
            let value = old.wrapping_add((kSinCosTable8bit_Sext[(v1 + 64) as usize] >> 4) as u16);
            enemy!(ctx, (index) as isize, velocity_x = value);
        };
        {
            let index = (v3) as isize;
            let old = enemy!(ctx, (index) as isize, velocity_y);
            let value = ((old as i32) + (kSinCosTable8bit_Sext[v1 as usize] as i32 >> 4)) as u16;
            enemy!(ctx, (index) as isize, velocity_y = value);
        };
        let door = enemy!(ctx, (v3) as isize, behavior_b);
        if !crate::game::progress_flags::door_opened(ctx, door) {
            return;
        }
    }
    start_instruction((v3 as usize) as isize, addr_off_86B5F3 as u16, ctx);
}

pub(super) fn EprojPreInstr_EyeDoorSweat(k: u16, ctx: &mut impl Host) {
    EprojBlockCollisition_Horiz(k, ctx);
    let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
        enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 12);
    } else {
        enemy_add!(ctx, (v1 as usize) as isize, y, (-4i16) as u16);
        start_instruction((v1 as usize) as isize, addr_off_86B61D as u16, ctx);
    };
}

pub(super) fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k: u16, ctx: &mut impl Host) {
    enemy!(ctx, (ctx.inputs().fx_y as i32 - 4i32) as u16 => (k >> 1) as isize, y);
}

pub(super) fn EprojPreInstr_TourianStatueStuff(k: u16, ctx: &mut impl Host) {
    if ctx.session().mechanisms.statue_animation == 0 {
        {
            let old = ctx.session().mechanisms.statue_finished;
            let value = old | (0x8000);
            ctx.session_mut().mechanisms.statue_finished = value;
        };
    }
    EprojPreInstr_BA42(k, ctx);
}

pub(super) fn EprojPreInstr_NorfairLavaquakeRocks_Inner(k: u16, ctx: &mut impl Host) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, ctx) != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_NorfairLavaquakeRocks(k: u16, ctx: &mut impl Host) {
    CallNorfairLavaquakeRocksFunc(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, ctx);
}

pub(super) fn EprojPreInstr_MotherBrainRoomTurrets(k: u16, ctx: &mut impl Host) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, ctx) & 1 != 0 {
        if ctx.mother_brain_turret_destroyed() != 0 {
            clear_unrounded_definition(k, ctx);
        }
    } else if ctx.mother_brain_turret_destroyed() != 0 {
        clear_unrounded_definition(k, ctx);
        let v5: i32 = k as i32 >> 1;
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v5) as isize, x),
            y: enemy!(ctx, (v5) as isize, y),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, ctx);
    } else {
        let v1: i32 = k as i32 >> 1;
        let fresh89 = enemy_decrement!(ctx, v1 as isize, velocity_x);
        let mut v2: bool = fresh89 == 1;
        if v2 {
            Eproj_SetXvelRandom(k, ctx);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, ctx);
            let v3: i32 = k as i32 >> 1;
            enemy!(ctx, turret_instruction(enemy_byte!(ctx, v3 as isize * 2, y_fraction) as isize, ctx) => (v3) as isize, program);
            enemy!(ctx, 1 => (v3) as isize, delay);
        }
        let v4: i32 = k as i32 >> 1;
        let fresh93 = enemy_decrement!(ctx, v4 as isize, velocity_y);
        v2 = fresh93 == 1;
        if v2 {
            Eproj_SetYvelRandom(k, ctx);
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurretBullets as u16, k, ctx);
        }
    };
}

// Keep signed-word interpretation and narrowing before the zero clamp.
fn decelerated_magnitude(velocity: u16, decrement: u16) -> i16 {
    (((velocity as i16).unsigned_abs() as i32 - i32::from(decrement)) as i16).max(0)
}

pub(super) fn EprojPreInstr_MotherBrainBomb(k: u16, ctx: &mut impl Host) {
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, ctx) != 0 {
        return;
    }
    let v1: i32 = k as i32 >> 1;

    let advance_bomb_phase: bool = if enemy!(ctx, (v1) as isize, behavior_b) != 0 {
        let v4: i32 = k as i32 >> 1;
        let v5: u16 =
            kMotherBrainsBomb_Yaccel[(enemy!(ctx, (v4) as isize, behavior_b) >> 1) as usize] as u16;
        if v5 == 0 {
            {
                enemy!(ctx, 0 => (v4) as isize, velocity_x);
                enemy!(ctx, 0 => (v4) as isize, velocity_y);
            }
            {
                let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
                ctx.set_mother_brain_active_bombs(value);
            }
            enemy!(ctx, 0 => (v4) as isize, definition);
            ctx.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(ctx, (v4) as isize, x),
                y: enemy!(ctx, (v4) as isize, y),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as u16,
                enemy_byte!(ctx, v4 as isize * 2, x_fraction) as u16,
                ctx,
            );
            ctx.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(ctx, (v4) as isize, x),
                y: enemy!(ctx, (v4) as isize, y),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, ctx);
            ctx.queue_sound(3, 6, 0x13);
            return;
        }
        MoveMotherBrainBomb(k, v5, ctx) & 1 != 0
    } else {
        let v2 = decelerated_magnitude(enemy!(ctx, (v1) as isize, velocity_x), 2);
        enemy!(ctx, (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16 => (v1) as isize, velocity_x);
        MoveMotherBrainBomb(k, 7, ctx) & 1 != 0
    };
    if advance_bomb_phase {
        let v3: i32 = k as i32 >> 1;
        enemy_add!(ctx, (v3) as isize, behavior_b, 2);
    };
}

pub(super) fn EprojPreInstr_MotherBrainRainbowBeam(k: u16, ctx: &mut impl Host) {
    let v2: i32 = k as i32 >> 1;
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(64), Le).x.whole => (v2) as isize, x);
    enemy!(ctx, ctx.resolve_actor(Key::from_legacy(64), Le).y.whole => (v2) as isize, y);
}

pub(super) fn EprojPreInit_MotherBrainsDeathExplosion_0(k: u16, ctx: &mut impl Host) {
    let v2: i32 = k as i32 >> 1;
    {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32 + enemy!(ctx, (v2) as isize, velocity_x) as i32) as u16 => (v2) as isize, x);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + enemy!(ctx, (v2) as isize, velocity_y) as i32) as u16 => (v2) as isize, y);
    }
}

pub(super) fn EprojPreInstr_MotherBrainTubeFalling(k: u16, ctx: &mut impl Host) {
    CallMotherBrainTubeFallingFunc(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub(super) fn EprojPreInstr_MotherBrainGlassShatteringShard(k: u16, ctx: &mut impl Host) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::X, SlotField::XFraction, ctx);
    let _v6: u16 = enemy!(ctx, (v1) as isize, y_fraction);
    if (enemy!(ctx, (v1) as isize, y) & 0xff00) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    } else {
        enemy_add!(ctx, (v1) as isize, velocity_y, 32);
        if (ctx.next_random() & 0x420) == 0 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainGlassShatteringSparkle as u16, k, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_KagosBugs(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_KagosBugs_Func1(k, ctx);
    EprojPreInstr_KagosBugs_Func2(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_c) != 0 {
        let _fresh71 = enemy_add!(ctx, (v1 as usize) as isize, behavior_c, u16::MAX);
    } else {
        start_instruction((v1 as usize) as isize, addr_word_86D052 as u16, ctx);
        enemy!(ctx, (0x86d0ebi32) as u16 => (v1) as isize, pre_action);
    };
}

pub(super) fn EprojPreInstr_D0EC(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_KagosBugs_Func1(k, ctx);
    EprojPreInstr_KagosBugs_Func2(k, ctx);
    let reset_vertical_speed: bool = if EprojBlockCollisition_Horiz(k, ctx) != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, velocity_x);
        true
    } else if EprojBlockCollisition_Vertical(k, ctx) != 0 {
        true
    } else {
        let v2: u16 = enemy!(ctx, (k >> 1) as isize, velocity_y);
        let v3: bool = ((word_86D082) as i16).wrapping_add((v2) as i16) < 0;
        enemy!(ctx, (word_86D082).wrapping_add(v2) => (k >> 1) as isize, velocity_y);
        if !v3 {
            false
        } else {
            return;
        }
    };
    if reset_vertical_speed {
        enemy!(ctx, 256 => (k >> 1) as isize, velocity_y);
    }
    enemy!(ctx, (0x86d128i32) as u16 => (k >> 1) as isize, pre_action);
    start_instruction((usize::from(k >> 1)) as isize, addr_word_86D04A as u16, ctx);
}

pub(super) fn EprojPreInstr_N00bTubeShards(k: u16, ctx: &mut impl Host) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::BehaviorB, SlotField::BehaviorA, ctx);
    Eproj_DeleteIfYposOutside(k, ctx);
}

pub(super) fn sub_86D83D(k: u16, ctx: &mut impl Host) {
    let v2: i32 = (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32 & 0x17e | 0x80) >> 1;
    let v6: i32 = ctx.session().enemy_projectiles.selected as i32 >> 1;
    let mut amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v6 as isize,
        amt as u32,
        ctx,
    );
    enemy_add!(ctx, (v6) as isize, velocity_x, 2);
    amt = (enemy!(ctx, (v6) as isize, velocity_y) as i16 as i32) << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v6 as isize,
        amt as u32,
        ctx,
    );
    Eproj_DeleteIfYposOutside(ctx.session().enemy_projectiles.selected, ctx);
}

pub(super) fn sub_86D89F(k: u16, ctx: &mut impl Host) {
    let v1: u16 = (enemy!(ctx, (k >> 1) as isize, velocity_x) & 0x17e | 0x80);
    let v2: i32 = v1 as i32 >> 1;
    let amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    let v5: i32 = ctx.session().enemy_projectiles.selected as i32 >> 1;
    add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v5 as isize,
        amt as u32,
        ctx,
    );
    enemy_add!(ctx, (v5) as isize, velocity_x, 4);
    sub_86D8DF(ctx.session().enemy_projectiles.selected, ctx);
}

pub(super) fn sub_86D8DF(k: u16, ctx: &mut impl Host) {
    let v5: i32 = k as i32 >> 1;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    enemy!(ctx, enemy!(ctx, (v5) as isize, behavior_b) => (v5) as isize, x);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes(k: u16, ctx: &mut impl Host) {
    plant_pre(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) >> 1) as usize,
        k,
        ctx,
    );
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, ctx);
}

pub(super) fn EprojPreInstr_DBF2_Func1(k: u16, ctx: &mut impl Host) {
    if EprojPreInstrHelper_DBF2_Func2(k, ctx) != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_DBF2(k: u16, ctx: &mut impl Host) {
    dispatch(
        (enemy!(ctx, (k >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    EprojPreInstr_DBF2_Func1(k, ctx);
}

pub(super) fn EprojPreInstr_SporeSpawners(k: u16, ctx: &mut impl Host) {
    if ctx.kraid_unk9000() == 0 {
        let v1: i32 = k as i32 >> 1;
        if enemy!(ctx, (v1) as isize, behavior_b) == 0 {
            {
                enemy!(ctx, addr_word_86DC06 as u16 => (v1) as isize, program);
                enemy!(ctx, 1 => (v1) as isize, delay);
                enemy!(ctx, ctx.next_random() & 0x1ff => (v1) as isize, behavior_b);
            }
        }
        enemy_decrement!(ctx, (v1) as isize, behavior_b);
    }
}

pub(super) fn EprojPreInstr_BotwoonsBody(k: u16, ctx: &mut impl Host) {
    if ctx.botwoon_death_phase() != 0 {
        let v1: i32 = k as i32 >> 1;
        if enemy!(ctx, (v1) as isize, velocity_x) as i32 == 0x86ea98 & 0xffff {
            enemy!(ctx, (0x86eaf4 & 0xffff) as u16 => (v1) as isize, velocity_x);
        }
    }
    CallBotwoonEprojFunc(
        (enemy!(ctx, (k >> 1) as isize, velocity_x) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub(super) fn EprojPreInstr_BotwoonsSpit(k: u16, ctx: &mut impl Host) {
    Eproj_FuncE73E_MoveXY(k, ctx);
    sub_86EC0C(k, ctx);
}

pub(super) fn sub_86A887(v0: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(v0, ctx) & 1 != 0 {
        let v6: i32 = v0 as i32 >> 1;
        {
            enemy!(ctx, 0xa48a => (v6) as isize, program);
            enemy!(ctx, 1 => (v6) as isize, delay);
        }
    } else {
        let v1: i32 = v0 as i32 >> 1;
        let mut v2: i16 = enemy!(ctx, (v1) as isize, velocity_x) as i16;
        if v2 >= 0 {
            let mut v4: i16 = (v2).wrapping_sub(4);
            if v4 < 0 {
                v4 = 3;
            }
            enemy!(ctx, v4 as u16 => (v1) as isize, velocity_x);
        } else {
            let mut v3: i16 = (v2 as i32 + 4) as i16;
            if v3 >= 0 {
                v3 = 3;
            }
            enemy!(ctx, v3 as u16 => (v1) as isize, velocity_x);
        }
        let carry: u8 = EprojBlockCollisition_Vertical(v0, ctx);
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) != 0 || carry == 0 {
            let v5: u16 = enemy_add!(ctx, (v1 as usize) as isize, velocity_y, 16);
            if (v5 & 0xf000) == 4096 {
                enemy!(ctx, 0 => (v1) as isize, definition);
            }
        } else {
            let v7: i32 = v0 as i32 >> 1;
            enemy_add!(ctx, (v7 as usize) as isize, y, (-3i16) as u16);
            start_instruction((v7 as usize) as isize, 0xa48e, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_D128(v0: u16, ctx: &mut impl Host) {
    EprojPreInstr_KagosBugs_Func1(v0, ctx);
    EprojPreInstr_KagosBugs_Func2(v0, ctx);
    if EprojBlockCollisition_Horiz(v0, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (v0 >> 1) as isize, velocity_x);
    } else if EprojBlockCollisition_Vertical(v0, ctx) & 1 != 0 {
        let v1: i32 = v0 as i32 >> 1;
        enemy!(ctx, (0x86d0ebi32) as u16 => (v1) as isize, pre_action);
        start_instruction((v1 as usize) as isize, addr_word_86D03C as u16, ctx);
    } else {
        enemy_add!(ctx, (v0 >> 1) as isize, velocity_y, (word_86D082) as u16);
    };
}

pub(super) fn EprojPreInstr_NamiFuneFireball(v0: u16, ctx: &mut impl Host) {
    CallNamiFuneFireballFunc(
        (enemy!(ctx, (v0 >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    Eproj_NamiFuneFireball_After(v0, ctx);
}

pub(super) fn sub_86E049(v0: u16, ctx: &mut impl Host) {
    CallLavamanFunc(
        (enemy!(ctx, (v0 >> 1) as isize, behavior_a) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    sub_86E0A4(v0, ctx);
}

pub(super) fn EprojPreInstr_BA42(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, enemy!(ctx, (v1) as isize, behavior_a) => (v1) as isize, x);
    enemy!(ctx, (enemy!(ctx, (v1) as isize, behavior_b) as i32 + ctx.inputs().camera_y as i32
            - ctx.expanding_square_origin() as i32) as u16 => (v1) as isize, y);
}

pub(super) fn EprojInit_MotherBrainsDrool_Falling(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 12);
    if MoveEprojWithVelocityY(k, ctx) >= 0xd7 {
        enemy_add!(ctx, (v1) as isize, y, (-4i16) as u16);
        enemy!(ctx, addr_off_86C8E1 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
}

pub(super) fn EprojInit_ShaktoolAttackFrontCircle(v1: u16, ctx: &mut impl Host) {
    if EprojBlockCollisition_Horiz(v1, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(v1, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (v1 >> 1) as isize, definition);
    }
}

pub(super) fn Eproj_PreInit_0x8aaf(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    if enemy!(ctx, (v1) as isize, y) >= enemy!(ctx, (v1) as isize, counter) {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub(super) fn EprojPreInstr_CrocomireSpikeWallPieces(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = enemy!(ctx, (v1) as isize, behavior_a);
    if v2 != CrocomireSpikeWallPieces_Tab1[v1 as usize] {
        v2 = (v2 as i32 + CrocomireSpikeWallPieces_Tab2[v1 as usize] as i32) as u16;
        if v2 >= CrocomireSpikeWallPieces_Tab1[v1 as usize] {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    enemy!(ctx, v2 => (v1) as isize, behavior_a);
    let mut v3: i8 = enemy_byte!(ctx, k as isize + 1, behavior_a) as i8;
    let mut v4: bool = u8::overflowing_add(enemy_byte!(ctx, k as isize, velocity_x), v3 as u8).1;
    {
        let index = k as isize;
        let old = enemy_byte!(ctx, index, velocity_x);
        let value = (old as i32 + v3 as i32) as u8;
        enemy_byte!(ctx, index, velocity_x = value);
    }
    let mut v5: u8 = (v4 as i32 + enemy_byte!(ctx, k as isize + 1, velocity_x) as i32) as u8;
    if ((v5 as i32 - CrocomireSpikeWallPieces_Tab3[k as usize] as i32) as i8) >= 0 {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as u8;
    }
    enemy_byte!(ctx, k as isize + 1, velocity_x = v5);
    let mut v6: i8 = enemy_byte!(ctx, k as isize, velocity_x) as i8;
    v4 = u8::overflowing_add(enemy_byte!(ctx, k as isize + 1, x_fraction), v6 as u8).1;
    {
        let index = k as isize + 1;
        let old = enemy_byte!(ctx, index, x_fraction);
        let value = (old as i32 + v6 as i32) as u8;
        enemy_byte!(ctx, index, x_fraction = value);
    }
    let mut v7: i8 = enemy_byte!(ctx, k as isize + 1, velocity_x) as i8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as u8, v7 as u8).1;
    let mut v8: i8 = (v9 as i32 + v7 as i32) as i8;
    v4 = (v4 as i32 | u8::overflowing_add(enemy_byte!(ctx, k as isize, x), v8 as u8).1 as i32) != 0;
    {
        let index = k as isize;
        let old = enemy_byte!(ctx, index, x);
        let value = (old as i32 + v8 as i32) as u8;
        enemy_byte!(ctx, index, x = value);
    }
    {
        let index = k as isize + 1;
        let old = enemy_byte!(ctx, index, x);
        let value = (old as i32 + v4 as i32) as u8;
        enemy_byte!(ctx, index, x = value);
    }
    add_split(
        SlotField::VelocityY,
        SlotField::BehaviorB,
        v1 as isize,
        0x3000,
        ctx,
    );
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, velocity_y)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, behavior_b)) as u32),
        ctx,
    );
    if enemy!(ctx, (v1) as isize, y) >= 0xa8 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        if (k & 2) == 0 {
            ctx.queue_sound(2, 6, 0x29);
        }
        let v12: i32 = k as i32 >> 1;
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v12) as isize, x),
            y: enemy!(ctx, (v12) as isize, y),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, ctx);
        ctx.queue_sound(2, 6, 0x25);
    }
}

pub(super) fn EprojPreInstr_9634(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) >= 8 {
        MoveEprojWithVelocity(k, ctx);
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            enemy!(ctx, addr_off_869574 as u16 => (v1) as isize, program);
            enemy_add!(ctx, (v1) as isize, behavior_a, 1);
            enemy!(ctx, 1 => (v1) as isize, delay);
            {
                enemy!(ctx, 0 => (v1) as isize, velocity_x);
                enemy!(ctx, 0 => (v1) as isize, velocity_y);
            }
            enemy!(ctx, 2560 => (v1) as isize, graphics);
            ctx.queue_sound(2, 6, 0x2b);
        }
    } else {
        enemy_add!(ctx, (v1) as isize, behavior_a, 1);
    };
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        enemy!(ctx, (enemy!(ctx, (v1) as isize, properties) & 0xfff | 0x8000) => (v1) as isize, properties);
        enemy!(ctx, (0x8699bf & 0xffff) as u16 => (v1) as isize, pre_action);
        enemy!(ctx, addr_word_86976C as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, 8 => (v1) as isize, behavior_a);
        enemy_add!(ctx, (v1) as isize, y, 8);
    }
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs2(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: bool = enemy!(ctx, (v1) as isize, behavior_a) == 1;
    let v3: bool = (enemy_add!(ctx, (v1) as isize, behavior_a, (-1i16) as u16) & 0x8000) != 0;
    if v2 || v3 {
        {
            enemy!(ctx, (0x869a01 & 0xffff) as u16 => (v1) as isize, pre_action);
            enemy!(ctx, addr_word_869772 as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
        }
        enemy_add!(ctx, (v1) as isize, y, (-8i16) as u16);
        {
            enemy!(ctx, -768i32 as u16 => (v1) as isize, velocity_y);
            enemy!(ctx, 0 => (v1) as isize, behavior_a);
        }
        if (ctx.inputs().frame & 1) != 0 {
            enemy!(ctx, -128i32 as u16 => (v1) as isize, velocity_x);
        } else {
            enemy!(ctx, 128 => (v1) as isize, velocity_x);
        }
    }
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs3(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    let switch_fireball_phase: bool = if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v2: u16 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1) as u16;
        enemy!(ctx, v2 => (v1) as isize, behavior_a);
        if v2 as i32 - 3i32 & 0x8000 != 0 {
            enemy!(ctx, word_869A3E[v2 as usize] => (v1) as isize, velocity_y);
            return;
        }
        true
    } else {
        EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
    };
    if switch_fireball_phase {
        {
            enemy!(ctx, addr_word_869782 as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy!(ctx, (0x869a44 & 0xffff) as u16 => (v1) as isize, pre_action);
        }
    };
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs_2(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let apply_gravity: bool = (enemy!(ctx, (v1) as isize, velocity_x) == 0)
        || ({
            let mut v2: bool = enemy!(ctx, (v1) as isize, velocity_x) == 1;
            let mut v3: bool =
                (enemy_add!(ctx, (v1) as isize, velocity_x, (-1i16) as u16) & 0x8000) != 0;
            (v2 || v3)
                && ({
                    ctx.queue_sound(3, 6, 0x1d);
                    true
                })
        });
    if apply_gravity {
        enemy_add!(ctx, (v1) as isize, velocity_y, 16);
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            enemy!(ctx, addr_word_8697AC as u16 => (v1) as isize, program);
            enemy!(ctx, 1 => (v1) as isize, delay);
            enemy_add!(ctx, (v1) as isize, y, 8);
            enemy!(ctx, (0x869a44 & 0xffff) as u16 => (v1) as isize, pre_action);
            ctx.queue_sound(3, 6, 0x1d);
        }
    };
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballsB_2(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy!(ctx, (v1) as isize, velocity_x);
    if v2 != 0 {
        enemy!(ctx, (v2 as i32 - 1i32) as u16 => (v1) as isize, velocity_x);
    } else if (ctx.inputs().frame & 1) != 0 {
        let v3: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 - 1i32) as u16;
        enemy!(ctx, v3 => (v1) as isize, velocity_y);
        if v3 == 0 {
            {
                enemy!(ctx, ctx.resolve_actor(Key::from_legacy(0), Le).x.whole => (v1) as isize, x);
                enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
                enemy!(ctx, 1 => (v1) as isize, delay);
                enemy!(ctx, addr_off_8697F8 as u16 => (v1) as isize, program);
            }
            return;
        }
    }
    let v4: u8 = (enemy!(ctx, (v1) as isize, behavior_a) as i32 + 1) as u8;
    enemy!(ctx, v4 as u16 => (v1) as isize, behavior_a);
    let pt = sine_offset::<false>(v4 as u16, enemy!(ctx, (v1) as isize, velocity_y));
    {
        enemy!(ctx, (pt.x as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).x.whole as i32) as u16 => (v1) as isize, x);
        enemy!(ctx, (pt.y as i32 + ctx.resolve_actor(Key::from_legacy(0), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
    }
}

pub(super) fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1 as usize) as isize, x, (-2i16) as u16);
    if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) == 0 {
        enemy_add!(ctx, (v1) as isize, x, (-2i16) as u16);
    }
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub(super) fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1 as usize) as isize, x, 2);
    if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) == 0 {
        enemy_add!(ctx, (v1) as isize, x, 2);
    }
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub(super) fn EprojPreInstr_PirateClawThrownLeft(k: u16, ctx: &mut impl Host) {
    advance_thrown_claw(k, false, ctx);
}

pub(super) fn EprojPreInstr_PirateClawThrownRight(k: u16, ctx: &mut impl Host) {
    advance_thrown_claw(k, true, ctx);
}

pub(super) fn EprojPreInstr_AB07(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = ((8 * (ctx.session().player.x as i32 - enemy!(ctx, (v1) as isize, x) as i32))
        as i16 as i32)
        << 8;
    add_split(
        SlotField::X,
        SlotField::XFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
    amt = ((8 * (ctx.session().player.y as i32 - 36i32 - enemy!(ctx, (v1) as isize, y) as i32))
        as i16 as i32)
        << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
}

pub(super) fn EprojPreInstr_GoldenTorizoEgg(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if (enemy_add!(ctx, (v1) as isize, behavior_b, (-1i16) as u16) & 0x8000) != 0 {
        enemy_add!(ctx, (v1) as isize, program, 2);
        enemy!(ctx, 1 => (v1) as isize, delay);
        enemy!(ctx, (if (enemy!(ctx, (v1) as isize, behavior_a) & 0x8000) != 0 {
                256
            } else {
                -256i32
            }) as u16 => (v1) as isize, velocity_x);
    } else {
        if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
            enemy!(ctx, -(enemy!(ctx, (v1) as isize, velocity_x) as i32) as u16 => (v1) as isize, velocity_x);
            {
                let index = (v1) as isize;
                let old = enemy!(ctx, (index) as isize, behavior_a);
                let value = old ^ (0x8000);
                enemy!(ctx, (index) as isize, behavior_a = value);
            };
        }
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
            && (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0
        {
            {
                let index = (v1) as isize;
                let old = enemy!(ctx, (index) as isize, velocity_x);
                let value = ((old as i32)
                    + (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
                        32
                    } else {
                        -32i32
                    })) as u16;
                enemy!(ctx, (index) as isize, velocity_x = value);
            };
            enemy!(ctx, -(enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16 => (v1) as isize, velocity_y);
        }
        enemy_add!(ctx, (v1) as isize, velocity_y, 48);
        if (enemy!(ctx, (v1) as isize, velocity_y) & 0xf000) == 4096 {
            enemy!(ctx, 0 => (v1) as isize, definition);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizoSuperMissile(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2 = enemy!(ctx, (v1) as isize, behavior_a);
    let v3: i32 = if (ctx.resolve_actor(Key::from_legacy(v2), Le).parameters[0] & 0x8000) != 0 {
        32
    } else {
        -32i32
    };
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).x.whole as i32 + v3) as u16 => (v1) as isize, x);
    enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(v2), Le).y.whole as i32 - 52i32) as u16 => (v1) as isize, y);
}

pub(super) fn sub_86B535(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    let v6: i16 = enemy!(ctx, (v1 as usize) as isize, velocity_y) as i16;
    if v6 >= 0 {
        enemy!(
            ctx,
            (v1 as usize) as isize,
            velocity_y = (v6 as u16).wrapping_add(32)
        );
        Eproj_DeleteIfYposOutside(k, ctx);
    } else {
        enemy!(
            ctx,
            (v1 as usize) as isize,
            velocity_y = (v6 as u16).wrapping_add(32)
        );
        if (v6).wrapping_add(32) >= 0 {
            {
                let instruction: u16 = (if (enemy!(ctx, (v1) as isize, velocity_x) & 0x8000) == 0 {
                    addr_word_86B4E3 as i32
                } else {
                    addr_word_86B4D7 as i32
                }) as u16;
                start_instruction((v1 as usize) as isize, instruction, ctx);
            }
        }
    };
}

pub(super) fn Eproj_DeleteIfYposOutside(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if ((enemy!(ctx, (v1) as isize, y)) as i16).wrapping_sub((ctx.inputs().camera_y) as i16) >= 288
    {
        enemy!(ctx, (v1 as usize) as isize, definition = 0);
    }
}

pub(super) fn EprojPreInstr_TourianStatueUnlockingParticle(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    move_spike_axis(k, true, SlotField::VelocityX, ctx);
    let v7: u16 = (ctx.inputs().fx_y as i32 - enemy!(ctx, (v1) as isize, y) as i32) as u16;
    move_spike_axis(k, false, SlotField::VelocityY, ctx);
    if ((v7 as i32 ^ ctx.inputs().fx_y as i32 - enemy!(ctx, (v1) as isize, y) as i32) & 0x8000) != 0
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as u16, k, ctx);
    }
    if (enemy!(ctx, (v1) as isize, y) & 0xff00) == 256 {
        enemy!(ctx, addr_off_86B79F as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else {
        enemy_add!(ctx, (v1) as isize, velocity_y, 16);
    };
}

pub(super) fn EprojPreInstr_TourianStatueSoul(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx);
    if (enemy!(ctx, (v1) as isize, y) & 0x100) != 0 {
        enemy!(ctx, addr_off_86B79F as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
    enemy_add!(ctx, (v1) as isize, velocity_y, (-128i16) as u16);
}

pub(super) fn Eproj_NorfairLavaquakeRocks_Func1(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 - 2i32) as i16;
    enemy!(ctx, v2 as u16 => (v1) as isize, velocity_y);
    if v2 >= 0 {
        for n in (1i32..=2).rev() {
            let mut v3: i16 = (n + enemy!(ctx, (v1) as isize, velocity_y) as i32 - 1i32) as i16;
            if v3 < 0 {
                v3 = 0;
            }
            let t: i32 = 8 * v3 as i32 + 4 >> 1;
            apply_lavaquake_sample(v1 as isize, t, ctx);
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, ctx);
    } else {
        {
            enemy!(ctx, 0 => (v1) as isize, velocity_y);
            enemy!(ctx, (0x86bc8f & 0xffff) as u16 => (v1) as isize, behavior_a);
        }
    };
}

pub(super) fn Eproj_NorfairLavaquakeRocks_Func2(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 2) as u16;
    enemy!(ctx, v2 => (v1) as isize, velocity_y);
    if v2 as i32 - 64i32 & 0x8000 == 0 {
        enemy!(ctx, 64 => (v1) as isize, velocity_y);
    }
    for n in (1i32..=2).rev() {
        let t: i32 = 8 * (enemy!(ctx, (v1) as isize, velocity_y) as i32 - n + 1) >> 1;
        apply_lavaquake_sample(v1 as isize, t, ctx);
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, ctx);
}

pub(super) use self::EprojPreInstr_SpikeShootingPlantSpikes_MoveX2 as Eproj_NorfairLavaquakeRocks_Func3;

pub(super) fn EprojPreInstr_BE12(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(
        ctx,
        (enemy!(ctx, v1 as isize, behavior_a) >> 1) as isize,
        definition
    ) != 0
    {
        EprojBlockCollisition_Horiz(k, ctx);
        EprojBlockCollisition_Vertical(k, ctx);
    } else {
        enemy!(ctx, 0 => (v1) as isize, definition);
    };
}

pub(super) fn EprojPreInstr_MotherBrainRoomTurretBullets(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, properties);
        let value = old ^ (0x8000);
        enemy!(ctx, (index) as isize, properties = value);
    };
    MoveEprojWithVelocity(k, ctx);
    if ctx.ridley_offscreen(enemy!(ctx, (v1) as isize, x), enemy!(ctx, (v1) as isize, y)) & 1 != 0 {
        {
            let index = (ctx.session().enemy_projectiles.selected >> 1) as isize;
            enemy!(ctx, index, definition = 0);
        }
    }
}

pub(super) fn sub_86C320(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + 10) as u16 => (v1) as isize, x);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + 16) as u16 => (v1) as isize, y);
    }
}

pub(super) fn Eproj_MoveToBlueRingSpawnPosition(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, behavior_a) != 0 {
        enemy_decrement!(ctx, (v1) as isize, behavior_a);
        sub_86C320(k, ctx);
    } else {
        MoveEprojWithVelocity(k, ctx);
        let t: u8 = CheckForCollisionWithShitroid_DoubleRet(k, ctx);
        if t & 0x80 != 0 {
            return;
        }
        if t != 0 {
            {
                let value = ctx.mother_brain_blue_ring_contacts().wrapping_add(1);
                ctx.set_mother_brain_blue_ring_contacts(value);
            }
            BlueRingContactExplosion(k, ctx);
            let v3: u16 = ctx.mother_brain_target_enemy();
            {
                let target = ctx.mother_brain_target_enemy();
                ctx.set_enemy_extra_health_fraction(target, 0, 16);
            }
            let v4 = v3;
            ctx.edit_actor(Key::from_legacy(v4), Le, |actor| {
                let health = (i32::from(actor.health) - 80) as i16;
                actor.health = if health < 0 { 0 } else { health as u16 };
            });
        } else if Eproj_CheckForCollisionWithSamus(k, ctx) & 1 != 0 {
            BlueRingContactExplosion(k, ctx);
            {
                let damage = crate::game::combat_rules::suit_damage(
                    0x50,
                    ctx.session().inventory.equipment.active,
                );
                ctx.damage_player(damage);
            }
            {
                let value = 96;
                ctx.session_mut().player.invincibility = value;
            };
            {
                let value = 5;
                ctx.session_mut().player.knockback_timer = value;
            };
            {
                let value = (((ctx.session().player.x as i32
                    - enemy!(ctx, (k >> 1) as isize, x) as i32)
                    as i16)
                    >= 0) as u16;
                ctx.session_mut().player.knockback_x_direction = value;
            };
        } else if CheckForBlueRingCollisionWithRoom(k, ctx) & 1 != 0 {
            Eproj_Earthqhake5(k, ctx);
        }
    };
}

pub(super) fn BlueRingContactExplosion(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, definition);
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: enemy!(ctx, (v1) as isize, y),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, ctx);
    ctx.queue_sound(3, 6, 0x13);
    sub_86C42E(k, ctx);
}

pub(super) fn EprojPreInstr_C84D(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: i32 = (4 * enemy!(ctx, (v1) as isize, behavior_a) as i32) as u16 as i32 >> 1;
    {
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).x.whole as i32 + kEprojInit_MotherBrainsDrool[v2 as usize] as i32) as u16 => (v1) as isize, x);
        enemy!(ctx, (ctx.resolve_actor(Key::from_legacy(64), Le).y.whole as i32 + kEprojInit_MotherBrainsDrool[(v2 + 1) as usize] as i32)
                as u16 => (v1) as isize, y);
    }
    {
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
    }
}

pub(super) fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        enemy!(ctx, (ctx.session().player.x as i32 + enemy!(ctx, (v1) as isize, velocity_x) as i32) as u16 => (v1) as isize, x);
        enemy!(ctx, (ctx.session().player.y as i32 + enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16 => (v1) as isize, y);
    }
}

pub(super) fn EprojPreInstr_MotherBrainsExplodedDoorParticles(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2 = decelerated_magnitude(enemy!(ctx, (v1) as isize, velocity_x), 16);
    enemy!(ctx, (if enemy!(ctx, (v1) as isize, velocity_x) & 0x8000 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16 => (v1) as isize, velocity_x);
    enemy_add!(ctx, (v1) as isize, velocity_y, 32);
    MoveEprojWithVelocity(k, ctx);
    if (enemy_add!(ctx, (v1) as isize, behavior_a, (-1i16) as u16) & 0x8000) != 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        enemy_add!(ctx, (v1) as isize, y, (-4i16) as u16);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    }
}

pub(super) fn EprojPreInstr_TimeBombSetJapaneseText(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        enemy!(ctx, 0 => (v1) as isize, velocity_x);
        enemy!(ctx, 0 => (v1) as isize, velocity_y);
    }
    {
        enemy!(ctx, 128 => (v1) as isize, x);
        enemy!(ctx, 192 => (v1) as isize, y);
    }
}

pub(super) fn MotherBrainTubeFallingFunc_GenerateExplosion(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    ctx.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(ctx, (v1) as isize, x),
        y: (enemy!(ctx, (v1) as isize, y) as i32 + 8) as u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    enemy!(ctx, (0x86cc08 & 0xffff) as u16 => (v1) as isize, behavior_a);
    MotherBrainTubeFallingFunc_Falling(k, ctx);
}

pub(super) fn MotherBrainTubeFallingFunc_Falling(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy_add!(ctx, (v1) as isize, velocity_y, 6);
    let mut v2: i16 = MoveEprojWithVelocityY(k, ctx) as i16;
    if v2 as i32 - 208i32 & 0x8000 == 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
        ctx.session_mut().enemy_projectiles.spawn.point = Point {
            x: enemy!(ctx, (v1) as isize, x),
            y: enemy!(ctx, (v1) as isize, y),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, ctx);
    }
}

pub(super) fn sub_86CFD5(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (0x86cff8 & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy_add!(ctx, (v1) as isize, x, (-19i16) as u16);
}

pub(super) fn sub_86CFE6(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, (0x86cff8 & 0xffff) as u16 => (v1) as isize, pre_action);
    enemy_add!(ctx, (v1) as isize, x, 19);
}

pub(super) fn sub_86CFF8(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        enemy!(ctx, addr_off_86CF56 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    } else if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        enemy!(ctx, 0 => (k >> 1) as isize, velocity_x);
    } else {
        let mut v2: u16 = (enemy!(ctx, (v1) as isize, velocity_y) as i32 + 16) as u16;
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
        if v2 as i32 - 512i32 & 0x8000 == 0 {
            v2 = 512;
        }
        enemy!(ctx, v2 => (v1) as isize, velocity_y);
    };
}

pub(super) fn EprojPreInstr_KagosBugs_Func1(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy!(ctx, (v1 as usize) as isize, behavior_a);
    if v2 != 0 {
        let v3: u16 = v2.wrapping_sub(1);
        enemy!(ctx, (v1 as usize) as isize, behavior_a = v3);
        if v3 == 0 {
            ctx.queue_sound(2, 6, 0x6c);
        }
    }
}

pub(super) fn EprojPreInstr_KagosBugs_Func2(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let enemy_index = enemy!(ctx, (v1) as isize, behavior_b);
    let v3: u16 = (((ctx.resolve_actor(Key::from_legacy(enemy_index), Le).x.whole)
        .wrapping_sub(enemy!(ctx, (k >> 1) as isize, x))) as i16)
        .unsigned_abs();
    if v3 as i32 - 23i32 & 0x8000 == 0 {
        set_property_bits((v1 as usize) as isize, 0x8000, ctx);
    }
}

pub(super) fn EprojPreInstr_MaridiaFloatersSpikes(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, velocity_x);
        let value = ((old as i32)
            + (word_86D21A[enemy!(ctx, (v1) as isize, behavior_a) as usize] as i32))
            as u16;
        enemy!(ctx, (index) as isize, velocity_x = value);
    };
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 || {
        {
            let index = (v1) as isize;
            let old = enemy!(ctx, (index) as isize, velocity_y);
            let value = ((old as i32)
                + (word_86D22A[enemy!(ctx, (v1) as isize, behavior_a) as usize] as i32))
                as u16;
            enemy!(ctx, (index) as isize, velocity_y = value);
        };
        EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    } {
        enemy!(ctx, addr_off_86D218 as u16 => (v1) as isize, program);
        enemy!(ctx, 1 => (v1) as isize, delay);
    }
}

pub(super) fn EprojPreInstr_WreckedShipRobotLaser(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    enemy!(ctx, 0 => (v1) as isize, graphics);
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
}

pub(super) fn sub_86D7BF(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if enemy!(ctx, (v1) as isize, x) != 0xee00 {
        enemy!(ctx, enemy!(ctx, (v1) as isize, x) => (v1) as isize, behavior_a);
    }
    if ctx.inputs().frame & 1 != 0 {
        enemy!(ctx, -4608i32 as u16 => (v1) as isize, x);
    } else {
        enemy!(ctx, enemy!(ctx, (v1) as isize, behavior_a) => (v1) as isize, x);
    };
}

pub(super) fn sub_86D7DE(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (192i16 as i32) << 8;
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        amt as u32,
        ctx,
    );
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k: u16, ctx: &mut impl Host) {
    move_spike_axis(k, false, SlotField::VelocityX, ctx);
}

pub(super) use self::EprojPreInstr_SpikeShootingPlantSpikes_MoveX1 as EprojPreInstr_DBF2_MoveX1;

pub(super) fn EprojPreInstr_Spores(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = enemy_byte!(ctx, v1 as isize * 2, behavior_a) as u16;
    let mut r18: u16 = (spore_delta(enemy_byte!(ctx, v1 as isize * 2, behavior_a) as isize, ctx)
        as u16) as u8 as i8 as i16 as u16;
    if (enemy!(ctx, (v1) as isize, behavior_b) & 0x80) != 0 {
        r18 = r18.wrapping_neg();
    }
    enemy_add!(ctx, (v1) as isize, x, r18);
    r18 = (spore_delta((v2 as i32 + 1) as isize, ctx) as u16) as u8 as i8 as i16 as u16;
    let v3: u16 = (r18 as i32 + enemy!(ctx, (v1) as isize, y) as i32 + r18 as i32) as u16;
    enemy!(ctx, v3 => (v1) as isize, y);
    if v3 as i32 - 768i32 & 0x8000 == 0 {
        enemy!(ctx, 0 => (v1) as isize, definition);
    }
    enemy!(ctx, (enemy_byte!(ctx, v1 as isize * 2, behavior_a) as i32 + 2) as u8 as u16 => (v1) as isize, behavior_a);
}

pub(super) fn EprojPreInstr_E605(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = (enemy!(ctx, (v1) as isize, counter))
        .wrapping_add(((enemy!(ctx, (v1) as isize, velocity_y)) as i16).unsigned_abs());
    if v2 >= 0x1000 {
        enemy!(ctx, 1 => (v1) as isize, delay);
        let _fresh60 = enemy_add!(ctx, (v1 as usize) as isize, program, 1);
        let _fresh61 = enemy_add!(ctx, (v1 as usize) as isize, program, 1);
        v2 = 0;
    }
    enemy!(ctx, v2 => (v1) as isize, counter);
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, y_fraction);
        let value =
            ((old as i32) + ((enemy_byte!(ctx, v1 as isize * 2, velocity_y) as i32) << 8)) as u16;
        enemy!(ctx, (index) as isize, y_fraction = value);
    };
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, y);
        let value =
            old.wrapping_add((enemy_byte!(ctx, v1 as isize * 2 + 1, velocity_y) as i8) as u16);
        enemy!(ctx, (index) as isize, y = value);
    };
}

pub(super) fn Eproj_FuncE73E_MoveXY(k: u16, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    integrate_angle_axis(
        index,
        SlotField::X,
        SlotField::XFraction,
        SlotField::VelocityX,
        SlotField::BehaviorA,
        64,
        ctx,
    );
    integrate_angle_axis(
        index,
        SlotField::Y,
        SlotField::YFraction,
        SlotField::VelocityY,
        SlotField::BehaviorB,
        128,
        ctx,
    );
}

pub(super) fn Eproj_BotwoonsBody_Main(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = {
        let record = ctx.loaded_record(0x86, 0xe9f1);
        record.native_word(
            ((enemy!(ctx, (v1) as isize, behavior_a) >> 1) as isize)
                .checked_mul(2)
                .unwrap(),
        )
    };
    if v2 != enemy!(ctx, (v1) as isize, behavior_b) {
        {
            enemy!(ctx, v2 => (v1) as isize, program);
            enemy!(ctx, v2 => (v1) as isize, behavior_b);
            enemy!(ctx, 1 => (v1) as isize, delay);
        }
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, ctx);
}

pub(super) fn Eproj_BotwoonsBodyHurtFlashHandling1(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = enemy!(ctx, (index) as isize, graphics);
        let value = old | (0xe00);
        enemy!(ctx, (index) as isize, graphics = value);
    };
    if ctx.resolve_actor(Key::from_legacy(0), Le).flash != 0
        && (ctx.inputs().random_enemy_counter & 2) != 0
    {
        {
            let index = (v1) as isize;
            let old = enemy!(ctx, (index) as isize, graphics);
            let value = old & (0xf1ff);
            enemy!(ctx, (index) as isize, graphics = value);
        };
    }
}

pub(super) fn EprojPreInstr_Sparks(k: u16, ctx: &mut impl Host) {
    let v1: i32 = k as i32 >> 1;
    if (enemy!(ctx, (v1) as isize, velocity_y) & 0x8000) == 0 {
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            {
                enemy!(ctx, 0xf363 => (v1) as isize, program);
                enemy!(ctx, 1 => (v1) as isize, delay);
            }
            for _ in 0..2 {
                let high = (enemy!(ctx, v1 as isize, behavior_b) as i32 * 2
                    + (enemy!(ctx, v1 as isize, behavior_a) as i32 >> 15))
                    as u16;
                enemy!(ctx, v1 as isize, behavior_b = high);
                let low = enemy!(ctx, v1 as isize, behavior_a).wrapping_mul(2);
                enemy!(ctx, v1 as isize, behavior_a = low);
            }
            {
                enemy!(ctx, 0x8000 => (v1) as isize, velocity_x);
                enemy!(ctx, -1i32 as u16 => (v1) as isize, velocity_y);
            }
            enemy_add!(ctx, (v1) as isize, y, (-2i16) as u16);
            return;
        }
        let v2: u16 = enemy!(ctx, (v1) as isize, velocity_x);
        enemy!(ctx, (v2 as i32 + 0x4000) as u16 => (v1) as isize, velocity_x);
        let v3: u16 = (u16::overflowing_add(v2, 0x4000u16).1 as i32
            + enemy!(ctx, (v1) as isize, velocity_y) as i32) as u16;
        if v3 < 4 {
            enemy!(ctx, v3 => (v1) as isize, velocity_y);
        }
    }
    add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, velocity_y)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, velocity_x)) as u32),
        ctx,
    );
    add_split(
        SlotField::X,
        SlotField::XFraction,
        v1 as isize,
        (((enemy!(ctx, (v1) as isize, behavior_b)) as u32) << 16)
            | ((enemy!(ctx, (v1) as isize, behavior_a)) as u32),
        ctx,
    );
    if (ctx.inputs().nmi_counter_low & 3) == 0 {
        let request = SpriteSpawn {
            position: Point {
                x: enemy!(ctx, (v1) as isize, x),
                y: enemy!(ctx, (v1) as isize, y),
            },
            definition: LegacySpriteDefinition(48),
            appearance: enemy!(ctx, (v1) as isize, graphics),
        };
        sprite_objects::spawn(ctx, request);
    }
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx)
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx)
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k: u16, ctx: &mut impl Host) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx)
}

pub(super) use self::EprojPreInstr_96C0 as EprojPreInstr_96CE;

fn advance_thrown_claw(k: u16, initially_right: bool, ctx: &mut impl Host) {
    let index = (k >> 1) as isize;
    let outbound = enemy!(ctx, index, behavior_b) != 0;
    let x = enemy!(ctx, index, x);
    let speed = u16::from(enemy_byte!(ctx, index * 2 + 1, behavior_a));
    let right = if outbound {
        initially_right
    } else {
        !initially_right
    };
    enemy!(
        ctx,
        index,
        x = if right {
            x.wrapping_add(speed)
        } else {
            x.wrapping_sub(speed)
        }
    );
    if outbound {
        let speed = enemy!(ctx, index, behavior_a).wrapping_sub(32);
        enemy!(ctx, index, behavior_a = speed);
        if speed == 0 {
            enemy!(ctx, index, behavior_b = 0);
        }
    } else {
        enemy_add!(ctx, index, behavior_a, 32);
    }
    enemy_add!(ctx, index, y, 1);
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        enemy!(ctx, index, definition = 0);
    }
}

fn integrate_angle_axis(
    index: isize,
    high: SlotField,
    low: SlotField,
    velocity: SlotField,
    fraction: SlotField,
    phase: i32,
    ctx: &mut impl Host,
) {
    // Y must reread Angle after the complete X integration.
    let negative = (enemy!(ctx, index, angle) as i32 + phase & 0x80) != 0;
    let amount = (((read_member(ctx, velocity, index)) as u32) << 16)
        | ((read_member(ctx, fraction, index)) as u32);
    // Preserve the original signed-min negation panic in checked builds.
    let amount = if negative {
        -(amount as i32) as u32
    } else {
        amount
    };
    add_split(high, low, index, amount, ctx);
}

fn apply_lavaquake_sample(index: isize, table_index: i32, ctx: &mut impl Host) {
    let amount = lavaquake_delta((table_index >> 1) as isize, ctx);
    add_split(SlotField::Y, SlotField::YFraction, index, amount, ctx);
    let record = ctx.loaded_record(0xa0, 0xcbc7);
    let value = record.native_word(((table_index + 1) as isize).checked_mul(2).unwrap());
    enemy!(ctx, index, behavior_b = value);
}
