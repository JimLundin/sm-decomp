//! Owned enemy-projectile pre-actions; compatibility and effects stay at the execution edge.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::pre_helpers::*;
use super::shared::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use super::state::{Point, SlotField};
use super::*;
use crate::rtl::words;

pub(super) fn dispatch(ea: u32, k: u16, ctx: &mut Execution<'_>) {
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

pub(super) fn Eproj_PreInstr_SkreeParticle(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((ctx.slot(SlotField::VelocityX, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::X,
        SlotField::XFraction,
        (v1 as usize) as isize,
        split_amount,
    );
    let split_amount =
        ((ctx.slot(SlotField::VelocityY, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        (v1 as usize) as isize,
        split_amount,
    );
    add_word(SlotField::VelocityY, (v1 as usize) as isize, 80, ctx);
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        ctx.set_slot(SlotField::Definition, (v1 as usize) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((ctx.slot(SlotField::VelocityY, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        (v1 as usize) as isize,
        split_amount,
    );
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((ctx.slot(SlotField::VelocityY, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::X,
        SlotField::XFraction,
        (v1 as usize) as isize,
        split_amount,
    );
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((ctx.slot(SlotField::VelocityX, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::X,
        SlotField::XFraction,
        (v1 as usize) as isize,
        split_amount,
    );
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx);
}

pub(super) fn EprojPreInstr_DustCloudOrExplosion(k: u16, ctx: &mut Execution<'_>) {
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        ctx.set_slot(SlotField::Definition, (usize::from(k >> 1)) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_8DCA(k: u16, ctx: &mut Execution<'_>) {
    EprojPowerBombCollision(k, ctx);
    if ctx.player_contact_damage_index() != 0 || {
        let mut v1: i32 = k as i32 >> 1;
        {
            store(ctx.player_x(), SlotField::X, (v1) as isize, ctx);
            store(
                (ctx.player_y() as i32 + 4 * ctx.slot(SlotField::BehaviorB, (v1) as isize) as i32
                    - 12i32) as u16,
                SlotField::Y,
                (v1) as isize,
                ctx,
            );
        }
        ctx.decrement(SlotField::BehaviorA, (v1) as isize);
        ctx.slot(SlotField::BehaviorA, (v1) as isize) == 0
    } {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
        if ({
            let old = ctx.player_speed_divisor();
            let value = old.wrapping_sub(1);
            ctx.set_player_speed_divisor(value);
            ctx.player_speed_divisor()
        } & 0x8000)
            != 0
        {
            {
                let value = 0;
                ctx.set_player_speed_divisor(value);
            };
        }
    }
}

pub(super) fn EprojPreInstr_DraygonsTurret_8DFF(k: u16, ctx: &mut Execution<'_>) {
    EprojPowerBombCollision(k, ctx);
    Eproj_FuncE73E_MoveXY(k, ctx);
    let mut v1: i16 = Eproj_FuncE722(k, ctx) as i16;
    if v1 != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_DraygonsGunk_8E0F(k: u16, ctx: &mut Execution<'_>) {
    EprojPowerBombCollision(k, ctx);
    Eproj_FuncE73E_MoveXY(k, ctx);
    let v1: i32 = k as i32 >> 1;
    let v2: u16 =
        abs16((ctx.player_x() as i32 - ctx.slot(SlotField::X, (v1) as isize) as i32) as u16);
    if v2 as i32 - 16i32 & 0x8000 != 0 && {
        let mut v3: u16 =
            abs16((ctx.player_y() as i32 - ctx.slot(SlotField::Y, (v1) as isize) as i32) as u16);
        v3 as i32 - 20i32 & 0x8000 != 0
    } {
        {
            store(0x8c38, SlotField::Program, (v1) as isize, ctx);
            store(1, SlotField::Delay, (v1) as isize, ctx);
        }
    } else if Eproj_FuncE722(k, ctx) != 0 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_CrocomireProjectile(k: u16, ctx: &mut Execution<'_>) {
    EprojBlockCollisition_Horiz(k, ctx);
    store(2560, SlotField::Graphics, (0) as isize, ctx);
    {
        let index = (k >> 1) as isize;
        let old = ctx.slot(SlotField::Counter, (index) as isize);
        let value =
            ((old as i32) + (ctx.slot(SlotField::VelocityX, (k >> 1) as isize) as i32)) as u16;
        ctx.set_slot(SlotField::Counter, (index) as isize, value);
    };
    let x: u16 = -64i32 as u16;
    let y: u16 = {
        let image = ctx.image();
        image.table_word(0x31059, (ctx.enemy_ai_preinstr(0) >> 1) as isize)
    };
    let v1: i32 = ctx.calculate_angle_from_xy(x, y) as i32;
    let v2: i32 = k as i32 >> 1;
    store(
        (4 * kSinCosTable8bit_Sext[(v1 + 64) as usize] as i32) as u16,
        SlotField::VelocityX,
        (v2) as isize,
        ctx,
    );
    store(
        (4 * kSinCosTable8bit_Sext[v1 as usize] as i32) as u16,
        SlotField::VelocityY,
        (v2) as isize,
        ctx,
    );
    store(
        (0x8690b3 & 0xffff) as u16,
        SlotField::PreAction,
        (v2) as isize,
        ctx,
    );
}

pub(super) fn sub_8690B3(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_CrocomireBridgeCrumbling(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    } else {
        store(
            (ctx.slot(SlotField::VelocityY, (k >> 1) as isize) as i32 + 24 & 0x3fff) as u16,
            SlotField::VelocityY,
            (k >> 1) as isize,
            ctx,
        );
    };
}

pub(super) fn EprojPreInstr_9642_RidleysFireball(k: u16, ctx: &mut Execution<'_>) {
    let mut v1: u16 = if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        -27042i32 as u16
    } else {
        if EprojBlockCollisition_Vertical(k, ctx) & 1 == 0 {
            return;
        }
        -27056i32 as u16
    };
    let v2: i32 = k as i32 >> 1;
    ctx.set_slot(SlotField::Definition, (v2 as usize) as isize, 0);
    if ctx.slot(SlotField::BehaviorB, (v2 as usize) as isize) == 0 {
        SetEprojSpawnPoint(k, ctx);
        SpawnEprojWithRoomGfx(v1, 3, ctx);
        ctx.queue_sound(2, 6, 0x2b);
    }
}

pub(super) fn EprojPreInstr_966C(k: u16, ctx: &mut Execution<'_>) {
    MoveEprojWithVelocityX(k, ctx);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        ctx.set_slot(SlotField::Program, (slot) as isize, addr_off_869574 as u16);
        ctx.set_slot(SlotField::Delay, (slot) as isize, 1);
    }
}

pub(super) fn EprojPreInstr_9688(k: u16, ctx: &mut Execution<'_>) {
    MoveEprojWithVelocityY(k, ctx);
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let slot = usize::from(k >> 1);
        ctx.set_slot(SlotField::Program, (slot) as isize, addr_off_869574 as u16);
        ctx.set_slot(SlotField::Delay, (slot) as isize, 1);
    }
}

pub(super) fn EprojPreInstr_96A4(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_96C0(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_9734_CeresFallingDebris(k: u16, ctx: &mut Execution<'_>) {
    let slot = usize::from(k >> 1);
    add_word(SlotField::VelocityY, (slot) as isize, 16, ctx);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        ctx.set_slot(SlotField::Definition, (slot) as isize, 0);
        SetEprojSpawnPoint(k, ctx);
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
        ctx.queue_sound(2, 6, 0x6d);
    }
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs(k: u16, ctx: &mut Execution<'_>) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 4, ctx);
    let v2: u16 = (ctx.slot_byte(SlotField::VelocityX, v1 as isize * 2) as i32
        + ctx.slot(SlotField::BehaviorA, (v1) as isize) as i32) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs_3(k: u16, ctx: &mut Execution<'_>) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 2, ctx);
    let v2: u16 = (ctx.slot(SlotField::BehaviorA, (v1) as isize) as i32 + 2) as u8 as u16;
    move_phantoon_orbit(v1 as isize, v2, &mut v5, ctx);
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballsB(k: u16, ctx: &mut Execution<'_>) {
    if ctx.enemy_ai_var_B(0) != 0 {
        let v1: i32 = k as i32 >> 1;
        {
            store(
                (0x869b41 & 0xffff) as u16,
                SlotField::PreAction,
                (v1) as isize,
                ctx,
            );
            store(180, SlotField::VelocityX, (v1) as isize, ctx);
            store(48, SlotField::VelocityY, (v1) as isize, ctx);
        }
    }
}

pub(super) fn EprojPreInstr_KraidRocks(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        ctx.set_slot(SlotField::Definition, (usize::from(k >> 1)) as isize, 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: u16 = add_word(SlotField::VelocityX, (v1 as usize) as isize, 8, ctx);
        if v2 as i32 - 256i32 & 0x8000 == 0 {
            v2 = -256i32 as u16;
        }
        store(v2, SlotField::VelocityX, (v1) as isize, ctx);
        add_word(SlotField::VelocityY, (v1 as usize) as isize, 64, ctx);
    };
}

pub(super) use self::EprojPreInstr_CrocomireBridgeCrumbling as EprojPreInstr_RocksFallingKraidCeiling;

pub(super) fn sub_869DA5(k: u16, ctx: &mut Execution<'_>) {
    store(0, SlotField::Graphics, (k >> 1) as isize, ctx);
}

pub(super) fn EprojPreInit_MiniKraidSpit(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        ctx.set_slot(SlotField::Definition, (usize::from(k >> 1)) as isize, 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: i16 = ((ctx.slot(SlotField::VelocityY, (v1) as isize)) as i16).wrapping_add(64);
        if v2 >= 0 && ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 - 960i32 & 0x8000 == 0 {
            v2 = 1024;
        }
        store(v2 as u16, SlotField::VelocityY, (v1) as isize, ctx);
    };
}

pub(super) fn EprojPreInstr_MiniKraidSpikes(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        ctx.set_slot(SlotField::Definition, (usize::from(k >> 1)) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_WalkingLavaSeahorseFireball(k: u16, ctx: &mut Execution<'_>) {
    let mut v2: u16 = 0;
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
        || EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
    {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    } else {
        let v1: i32 = k as i32 >> 1;
        if (ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000) == 0 {
            v2 = (ctx.slot(SlotField::VelocityX, (v1) as isize) as i32 - 64i32) as u16;
            store(v2, SlotField::VelocityX, (v1) as isize, ctx);
            if v2 as i32 - 512i32 & 0x8000 != 0 {
                v2 = 512;
            }
        } else {
            v2 = (ctx.slot(SlotField::VelocityX, (v1) as isize) as i32 + 64) as u16;
            store(v2, SlotField::VelocityX, (v1) as isize, ctx);
            if v2 as i32 + 512 & 0x8000 == 0 {
                v2 = -512i32 as u16;
            }
        }
        store(v2, SlotField::VelocityX, (v1) as isize, ctx);
    };
}

pub(super) fn EprojPreInstr_CeresElevatorPad(k: u16, ctx: &mut Execution<'_>) {
    let mut v3: bool = false;
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(SlotField::BehaviorA, (v1) as isize) == 0
        || {
            let mut v2: bool = ctx.slot(SlotField::BehaviorA, (v1) as isize) == 1;
            v3 = ((ctx.slot(SlotField::BehaviorA, (v1) as isize)) as i16).wrapping_sub(1) < 0;
            ctx.decrement(SlotField::BehaviorA, (v1) as isize);
            v2 as i32 != 0
        }
        || v3
    {
        store(
            ctx.player_y().wrapping_add(28),
            SlotField::Y,
            (v1) as isize,
            ctx,
        );
        let fresh113 = {
            let value = ctx.player_y().wrapping_add(1);
            ctx.set_player_y(value);
            value
        };
        if fresh113 as i32 - 73i32 & 0x8000 == 0 {
            {
                {
                    let value = 72;
                    ctx.set_player_y(value);
                };
                store(1, SlotField::Delay, (v1) as isize, ctx);
                store(
                    addr_off_86A28B as u16,
                    SlotField::Program,
                    (v1) as isize,
                    ctx,
                );
            }
            ctx.call_player_code(0xe);
        }
    }
}

pub(super) fn EprojPreInstr_CeresElevatorPlatform(k: u16, ctx: &mut Execution<'_>) {
    if ctx.player_y() == 72 {
        let v1: i32 = k as i32 >> 1;
        store(1, SlotField::Delay, (v1) as isize, ctx);
        store(
            addr_off_86A28B as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
    }
}

pub(super) fn EprojPreInstr_A977(k: u16, ctx: &mut Execution<'_>) {
    let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) != 0 || carry == 0 {
        let v2: u16 = (ctx.slot(SlotField::BehaviorB, (0) as isize))
            .wrapping_add(ctx.slot(SlotField::VelocityY, (v1) as isize));
        store(v2, SlotField::VelocityY, (v1) as isize, ctx);
        if (v2 & 0xf000) == 4096 {
            store(4096, SlotField::VelocityY, (v1) as isize, ctx);
        }
    } else {
        store(0xa918, SlotField::PreAction, (v1) as isize, ctx);
    };
}

pub(super) fn EprojPreInstr_BombTorizosChozoOrbs(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) != 0 {
        let v3: i32 = k as i32 >> 1;
        start_instruction((v3 as usize) as isize, addr_off_86AB25 as u16, ctx);
    } else {
        let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
        let v1: i32 = k as i32 >> 1;
        if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) != 0 || carry == 0 {
            let v2: u16 = add_word(SlotField::VelocityY, (v1 as usize) as isize, 18, ctx);
            if (v2 & 0xf000) == 4096 {
                store(0, SlotField::Definition, (v1) as isize, ctx);
            }
        } else {
            let v4: i32 = k as i32 >> 1;
            store(
                (ctx.slot(SlotField::Y, (k >> 1) as isize) & 0xfff0 | 8).wrapping_sub(2),
                SlotField::Y,
                (v4) as isize,
                ctx,
            );
            start_instruction((v4 as usize) as isize, addr_off_86AB41 as u16, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizosChozoOrbs(k: u16, ctx: &mut Execution<'_>) {
    let mut v1: i32 = 0;
    let mut v3: u16 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        store(
            -(ctx.slot(SlotField::VelocityX, (k >> 1) as isize) as i32) as u16,
            SlotField::VelocityX,
            (k >> 1) as isize,
            ctx,
        );
    }
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
        && {
            v1 = k as i32 >> 1;
            (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) == 0
        }
        && {
            let mut v2: i16 = ctx.slot(SlotField::VelocityX, (v1) as isize) as i16;
            if v2 >= 0 {
                v3 = (v2 as i32 - 64i32) as u16;
            } else {
                v3 = (v2 as i32 + 64) as u16;
            };
            store(v3, SlotField::VelocityX, (v1) as isize, ctx);
            let mut v4: u16 =
                -(ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 >> 1i32) as u16;
            store(v4, SlotField::VelocityY, (v1) as isize, ctx);
            (v4 & 0xff80) == 0xff80
        }
    {
        let v5: i32 = k as i32 >> 1;
        {
            store(
                ((ctx.slot(SlotField::Y, (v5) as isize) as i32 & 0xfff0 | 8) - 2i32) as u16,
                SlotField::Y,
                (v5) as isize,
                ctx,
            );
            store(
                addr_off_86AB41 as u16,
                SlotField::Program,
                (v5) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v5) as isize, ctx);
        }
    } else {
        add_word(SlotField::VelocityY, (k >> 1) as isize, 24, ctx);
    };
}

pub(super) fn EprojPreInstr_TorizoSonicBoom(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let v4: i32 = k as i32 >> 1;
        {
            store(
                addr_off_86ADE5 as u16,
                SlotField::Program,
                (v4) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v4) as isize, ctx);
            store(
                ctx.slot(SlotField::X, (v4) as isize),
                SlotField::BehaviorA,
                (v4) as isize,
                ctx,
            );
            store(
                ctx.slot(SlotField::Y, (v4) as isize),
                SlotField::BehaviorB,
                (v4) as isize,
                ctx,
            );
        }
    } else {
        let v1: i32 = k as i32 >> 1;
        {
            let index = (v1) as isize;
            let old = ctx.slot(SlotField::VelocityX, (index) as isize);
            let value = ((old as i32)
                + (if ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000 != 0 {
                    -16i32
                } else {
                    16
                })) as u16;
            ctx.set_slot(SlotField::VelocityX, (index) as isize, value);
        };
        if (ctx.slot(SlotField::VelocityX, (v1) as isize) & 0xf000) == 4096 {
            store(0, SlotField::Definition, (v1) as isize, ctx);
        }
    };
}

pub(super) fn sub_86B0B9(k: u16, ctx: &mut Execution<'_>) {
    let mut v2: i16 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        let v3: i32 = k as i32 >> 1;
        {
            store(0xb0dd, SlotField::PreAction, (v3) as isize, ctx);
            store(0, SlotField::VelocityY, (v3) as isize, ctx);
        }
    } else {
        let v1: i32 = k as i32 >> 1;
        if (ctx.slot(SlotField::BehaviorA, (v1) as isize) & 0x8000) != 0 {
            v2 = 48;
        } else {
            v2 = -48i16;
        }
        add_word(SlotField::VelocityX, (v1) as isize, v2 as u16, ctx);
    };
}

pub(super) fn sub_86B0DD(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        store(
            (if (ctx.slot(SlotField::BehaviorA, (v1) as isize) & 0x8000) != 0 {
                addr_off_86B1A8 as i32
            } else {
                addr_off_86B190 as i32
            }) as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    } else {
        add_word(SlotField::VelocityY, (k >> 1) as isize, 48, ctx);
    };
}

pub(super) fn EprojPreInstr_B237(k: u16, ctx: &mut Execution<'_>) {
    let mut v1: i32 = 0;
    if EprojBlockCollisition_Horiz(k, ctx) != 0 || {
        let mut carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
        v1 = k as i32 >> 1;
        (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) == 0 && carry != 0
    } {
        let v3: i32 = k as i32 >> 1;
        store(
            addr_off_86B2EF as u16,
            SlotField::Program,
            (v3) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v3) as isize, ctx);
    } else {
        let v2: u16 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 + 16) as u16;
        store(v2, SlotField::VelocityY, (v1) as isize, ctx);
        if (v2 & 0xf000) == 4096 {
            store(0, SlotField::Definition, (v1) as isize, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizoEyeBeam(k: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        store(
            addr_off_86B3CD as u16,
            SlotField::Program,
            (k >> 1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (k >> 1) as isize, ctx);
        return;
    }
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        {
            store(
                ((ctx.slot(SlotField::Y, (v1) as isize) as i32 & 0xfff0 | 8) - 2i32) as u16,
                SlotField::Y,
                (v1) as isize,
                ctx,
            );
            store(
                addr_off_86B3E5 as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v1) as isize, ctx);
        }
    }
}

pub(super) fn EprojPreInstr_EyeDoorProjectile(k: u16, ctx: &mut Execution<'_>) {
    let v3: i32 = k as i32 >> 1;
    if !(EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0)
    {
        let v1: i32 = ctx.slot(SlotField::BehaviorA, (k >> 1) as isize) as i32 >> 1;
        {
            let index = (v3) as isize;
            let old = ctx.slot(SlotField::VelocityX, (index) as isize);
            let value = old.wrapping_add((kSinCosTable8bit_Sext[(v1 + 64) as usize] >> 4) as u16);
            ctx.set_slot(SlotField::VelocityX, (index) as isize, value);
        };
        {
            let index = (v3) as isize;
            let old = ctx.slot(SlotField::VelocityY, (index) as isize);
            let value = ((old as i32) + (kSinCosTable8bit_Sext[v1 as usize] as i32 >> 4)) as u16;
            ctx.set_slot(SlotField::VelocityY, (index) as isize, value);
        };
        let t: i32 = ctx.prepare_bit_access(ctx.slot(SlotField::BehaviorB, (v3) as isize)) as i32;
        if ((ctx.bitmask() & ctx.door_bit_byte(t as isize) as u16) == 0) {
            return;
        }
    }
    start_instruction((v3 as usize) as isize, addr_off_86B5F3 as u16, ctx);
}

pub(super) fn EprojPreInstr_EyeDoorSweat(k: u16, ctx: &mut Execution<'_>) {
    EprojBlockCollisition_Horiz(k, ctx);
    let carry: u8 = EprojBlockCollisition_Vertical(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) != 0 || carry == 0 {
        add_word(SlotField::VelocityY, (v1 as usize) as isize, 12, ctx);
    } else {
        add_word(SlotField::Y, (v1 as usize) as isize, (-4i16) as u16, ctx);
        start_instruction((v1 as usize) as isize, addr_off_86B61D as u16, ctx);
    };
}

pub(super) fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(
    k: u16,
    ctx: &mut Execution<'_>,
) {
    store(
        (ctx.fx_y_pos() as i32 - 4i32) as u16,
        SlotField::Y,
        (k >> 1) as isize,
        ctx,
    );
}

pub(super) fn EprojPreInstr_TourianStatueStuff(k: u16, ctx: &mut Execution<'_>) {
    if ctx.tourian_entrance_statue_animstate() == 0 {
        {
            let old = ctx.tourian_entrance_statue_finished();
            let value = old | (0x8000);
            ctx.set_tourian_entrance_statue_finished(value);
        };
    }
    EprojPreInstr_BA42(k, ctx);
}

pub(super) fn EprojPreInstr_NorfairLavaquakeRocks_Inner(k: u16, ctx: &mut Execution<'_>) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, ctx) != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_NorfairLavaquakeRocks(k: u16, ctx: &mut Execution<'_>) {
    CallNorfairLavaquakeRocksFunc(
        (ctx.slot(SlotField::BehaviorA, (k >> 1) as isize) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, ctx);
}

pub(super) fn EprojPreInstr_MotherBrainRoomTurrets(k: u16, ctx: &mut Execution<'_>) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, ctx) & 1 != 0 {
        if ctx.mother_brain_turret_destroyed() != 0 {
            clear_unrounded_definition(k, ctx);
        }
    } else if ctx.mother_brain_turret_destroyed() != 0 {
        clear_unrounded_definition(k, ctx);
        let v5: i32 = k as i32 >> 1;
        ctx.state_mut().spawn.point = Point {
            x: ctx.slot(SlotField::X, (v5) as isize),
            y: ctx.slot(SlotField::Y, (v5) as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, ctx);
    } else {
        let v1: i32 = k as i32 >> 1;
        let fresh89 = ctx.decrement(SlotField::VelocityX, v1 as isize);
        let mut v2: bool = fresh89 == 1;
        if v2 {
            Eproj_SetXvelRandom(k, ctx);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, ctx);
            let v3: i32 = k as i32 >> 1;
            store(
                ctx.raw_turret_instruction(
                    ctx.slot_byte(SlotField::YFraction, v3 as isize * 2) as isize
                ),
                SlotField::Program,
                (v3) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v3) as isize, ctx);
        }
        let v4: i32 = k as i32 >> 1;
        let fresh93 = ctx.decrement(SlotField::VelocityY, v4 as isize);
        v2 = fresh93 == 1;
        if v2 {
            Eproj_SetYvelRandom(k, ctx);
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurretBullets as u16, k, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_MotherBrainBomb(k: u16, ctx: &mut Execution<'_>) {
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, ctx) != 0 {
        return;
    }
    let v1: i32 = k as i32 >> 1;

    let advance_bomb_phase: bool = if ctx.slot(SlotField::BehaviorB, (v1) as isize) != 0 {
        let v4: i32 = k as i32 >> 1;
        let v5: u16 = kMotherBrainsBomb_Yaccel
            [(ctx.slot(SlotField::BehaviorB, (v4) as isize) >> 1) as usize]
            as u16;
        if v5 == 0 {
            {
                store(0, SlotField::VelocityX, (v4) as isize, ctx);
                store(0, SlotField::VelocityY, (v4) as isize, ctx);
            }
            {
                let value = ctx.mother_brain_active_bombs().wrapping_sub(1);
                ctx.set_mother_brain_active_bombs(value);
            }
            store(0, SlotField::Definition, (v4) as isize, ctx);
            ctx.state_mut().spawn.point = Point {
                x: ctx.slot(SlotField::X, (v4) as isize),
                y: ctx.slot(SlotField::Y, (v4) as isize),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as u16,
                ctx.slot_byte(SlotField::XFraction, v4 as isize * 2) as u16,
                ctx,
            );
            ctx.state_mut().spawn.point = Point {
                x: ctx.slot(SlotField::X, (v4) as isize),
                y: ctx.slot(SlotField::Y, (v4) as isize),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, ctx);
            ctx.queue_sound(3, 6, 0x13);
            return;
        }
        (MoveMotherBrainBomb(k, v5, ctx) & 1 != 0)
    } else {
        let mut v2: i16 =
            (abs16(ctx.slot(SlotField::VelocityX, (v1) as isize)) as i32 - 2i32) as i16;
        if v2 < 0 {
            v2 = 0;
        }
        store(
            (if ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16,
            SlotField::VelocityX,
            (v1) as isize,
            ctx,
        );
        (MoveMotherBrainBomb(k, 7, ctx) & 1 != 0)
    };
    if advance_bomb_phase {
        let v3: i32 = k as i32 >> 1;
        add_word(SlotField::BehaviorB, (v3) as isize, 2, ctx);
    };
}

pub(super) fn EprojPreInstr_MotherBrainRainbowBeam(k: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = k as i32 >> 1;
    store(ctx.enemy_x(64), SlotField::X, (v2) as isize, ctx);
    store(ctx.enemy_y(64), SlotField::Y, (v2) as isize, ctx);
}

pub(super) fn EprojPreInit_MotherBrainsDeathExplosion_0(k: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = k as i32 >> 1;
    {
        store(
            (ctx.enemy_x(0) as i32 + ctx.slot(SlotField::VelocityX, (v2) as isize) as i32) as u16,
            SlotField::X,
            (v2) as isize,
            ctx,
        );
        store(
            (ctx.enemy_y(0) as i32 + ctx.slot(SlotField::VelocityY, (v2) as isize) as i32) as u16,
            SlotField::Y,
            (v2) as isize,
            ctx,
        );
    }
}

pub(super) fn EprojPreInstr_MotherBrainTubeFalling(k: u16, ctx: &mut Execution<'_>) {
    CallMotherBrainTubeFallingFunc(
        (ctx.slot(SlotField::BehaviorA, (k >> 1) as isize) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub(super) fn EprojPreInstr_MotherBrainGlassShatteringShard(k: u16, ctx: &mut Execution<'_>) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::X, SlotField::XFraction, ctx);
    let v6: u16 = ctx.slot(SlotField::YFraction, (v1) as isize);
    if (ctx.slot(SlotField::Y, (v1) as isize) & 0xff00) != 0 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    } else {
        add_word(SlotField::VelocityY, (v1) as isize, 32, ctx);
        if (ctx.next_random() & 0x420) == 0 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainGlassShatteringSparkle as u16, k, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_KagosBugs(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_KagosBugs_Func1(k, ctx);
    EprojPreInstr_KagosBugs_Func2(k, ctx);
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(SlotField::BehaviorC, (v1) as isize) != 0 {
        let fresh71 = add_word(SlotField::BehaviorC, (v1 as usize) as isize, u16::MAX, ctx);
    } else {
        start_instruction((v1 as usize) as isize, addr_word_86D052 as u16, ctx);
        store(
            ((0x86d0ebi32) as u16),
            SlotField::PreAction,
            (v1) as isize,
            ctx,
        );
    };
}

pub(super) fn EprojPreInstr_D0EC(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_KagosBugs_Func1(k, ctx);
    EprojPreInstr_KagosBugs_Func2(k, ctx);
    let reset_vertical_speed: bool = if EprojBlockCollisition_Horiz(k, ctx) != 0 {
        store(0, SlotField::VelocityX, (k >> 1) as isize, ctx);
        true
    } else if EprojBlockCollisition_Vertical(k, ctx) != 0 {
        true
    } else {
        let v2: u16 = ctx.slot(SlotField::VelocityY, (k >> 1) as isize);
        let v3: bool = ((word_86D082) as i16).wrapping_add((v2) as i16) < 0;
        store(
            (word_86D082).wrapping_add(v2),
            SlotField::VelocityY,
            (k >> 1) as isize,
            ctx,
        );
        if !v3 {
            false
        } else {
            return;
        }
    };
    if reset_vertical_speed {
        store(256, SlotField::VelocityY, (k >> 1) as isize, ctx);
    }
    store(
        ((0x86d128i32) as u16),
        SlotField::PreAction,
        (k >> 1) as isize,
        ctx,
    );
    start_instruction((usize::from(k >> 1)) as isize, addr_word_86D04A as u16, ctx);
}

pub(super) fn EprojPreInstr_N00bTubeShards(k: u16, ctx: &mut Execution<'_>) {
    let v1 = (k >> 1) as i32;
    integrate_shard(v1 as isize, SlotField::BehaviorB, SlotField::BehaviorA, ctx);
    Eproj_DeleteIfYposOutside(k, ctx);
}

pub(super) fn sub_86D83D(k: u16, ctx: &mut Execution<'_>) {
    let v2: i32 = (ctx.slot(SlotField::VelocityX, (k >> 1) as isize) as i32 & 0x17e | 0x80) >> 1;
    let v6: i32 = ctx.state().selected as i32 >> 1;
    let mut amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    ctx.add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v6 as isize,
        amt as u32,
    );
    add_word(SlotField::VelocityX, (v6) as isize, 2, ctx);
    amt = (ctx.slot(SlotField::VelocityY, (v6) as isize) as i16 as i32) << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v6 as isize, amt as u32);
    Eproj_DeleteIfYposOutside(ctx.state().selected, ctx);
}

pub(super) fn sub_86D89F(k: u16, ctx: &mut Execution<'_>) {
    let v1: u16 = (ctx.slot(SlotField::VelocityX, (k >> 1) as isize) & 0x17e | 0x80);
    let v2: i32 = v1 as i32 >> 1;
    let amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    let v5: i32 = ctx.state().selected as i32 >> 1;
    ctx.add_split(
        SlotField::BehaviorB,
        SlotField::BehaviorA,
        v5 as isize,
        amt as u32,
    );
    add_word(SlotField::VelocityX, (v5) as isize, 4, ctx);
    sub_86D8DF(ctx.state().selected, ctx);
}

pub(super) fn sub_86D8DF(k: u16, ctx: &mut Execution<'_>) {
    let v5: i32 = k as i32 >> 1;
    let amt: i32 = (ctx.slot(SlotField::VelocityY, (v5) as isize) as i16 as i32) << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v5 as isize, amt as u32);
    store(
        ctx.slot(SlotField::BehaviorB, (v5) as isize),
        SlotField::X,
        (v5) as isize,
        ctx,
    );
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, ctx);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes(k: u16, ctx: &mut Execution<'_>) {
    kEprojPreInstr_SpikeShootingPlantSpikes
        [(ctx.slot(SlotField::BehaviorA, (k >> 1) as isize) >> 1) as usize](k, ctx);
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, ctx);
}

pub(super) fn EprojPreInstr_DBF2_Func1(k: u16, ctx: &mut Execution<'_>) {
    if EprojPreInstrHelper_DBF2_Func2(k, ctx) != 0 {
        store(0, SlotField::Definition, (k >> 1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_DBF2(k: u16, ctx: &mut Execution<'_>) {
    dispatch(
        (ctx.slot(SlotField::BehaviorA, (k >> 1) as isize) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
    EprojPreInstr_DBF2_Func1(k, ctx);
}

pub(super) fn EprojPreInstr_SporeSpawners(k: u16, ctx: &mut Execution<'_>) {
    if ctx.kraid_unk9000() == 0 {
        let v1: i32 = k as i32 >> 1;
        if ctx.slot(SlotField::BehaviorB, (v1) as isize) == 0 {
            {
                store(
                    addr_word_86DC06 as u16,
                    SlotField::Program,
                    (v1) as isize,
                    ctx,
                );
                store(1, SlotField::Delay, (v1) as isize, ctx);
                store(
                    (ctx.next_random() & 0x1ff),
                    SlotField::BehaviorB,
                    (v1) as isize,
                    ctx,
                );
            }
        }
        ctx.decrement(SlotField::BehaviorB, (v1) as isize);
    }
}

pub(super) fn EprojPreInstr_BotwoonsBody(k: u16, ctx: &mut Execution<'_>) {
    if ctx.botwoon_death_phase() != 0 {
        let v1: i32 = k as i32 >> 1;
        if ctx.slot(SlotField::VelocityX, (v1) as isize) as i32 == 0x86ea98 & 0xffff {
            store(
                (0x86eaf4 & 0xffff) as u16,
                SlotField::VelocityX,
                (v1) as isize,
                ctx,
            );
        }
    }
    CallBotwoonEprojFunc(
        (ctx.slot(SlotField::VelocityX, (k >> 1) as isize) as i32 | 0x860000) as u32,
        k,
        ctx,
    );
}

pub(super) fn EprojPreInstr_BotwoonsSpit(k: u16, ctx: &mut Execution<'_>) {
    Eproj_FuncE73E_MoveXY(k, ctx);
    sub_86EC0C(k, ctx);
}

pub(super) fn sub_86A887(v0: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(v0, ctx) & 1 != 0 {
        let v6: i32 = v0 as i32 >> 1;
        {
            store(0xa48a, SlotField::Program, (v6) as isize, ctx);
            store(1, SlotField::Delay, (v6) as isize, ctx);
        }
    } else {
        let v1: i32 = v0 as i32 >> 1;
        let mut v2: i16 = ctx.slot(SlotField::VelocityX, (v1) as isize) as i16;
        if v2 >= 0 {
            let mut v4: i16 = (v2).wrapping_sub(4);
            if v4 < 0 {
                v4 = 3;
            }
            store(v4 as u16, SlotField::VelocityX, (v1) as isize, ctx);
        } else {
            let mut v3: i16 = (v2 as i32 + 4) as i16;
            if v3 >= 0 {
                v3 = 3;
            }
            store(v3 as u16, SlotField::VelocityX, (v1) as isize, ctx);
        }
        let carry: u8 = EprojBlockCollisition_Vertical(v0, ctx);
        if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) != 0 || carry == 0 {
            let v5: u16 = add_word(SlotField::VelocityY, (v1 as usize) as isize, 16, ctx);
            if (v5 & 0xf000) == 4096 {
                store(0, SlotField::Definition, (v1) as isize, ctx);
            }
        } else {
            let v7: i32 = v0 as i32 >> 1;
            add_word(SlotField::Y, (v7 as usize) as isize, (-3i16) as u16, ctx);
            start_instruction((v7 as usize) as isize, 0xa48e, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_D128(v0: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_KagosBugs_Func1(v0, ctx);
    EprojPreInstr_KagosBugs_Func2(v0, ctx);
    if EprojBlockCollisition_Horiz(v0, ctx) & 1 != 0 {
        store(0, SlotField::VelocityX, (v0 >> 1) as isize, ctx);
    } else if EprojBlockCollisition_Vertical(v0, ctx) & 1 != 0 {
        let v1: i32 = v0 as i32 >> 1;
        store(
            ((0x86d0ebi32) as u16),
            SlotField::PreAction,
            (v1) as isize,
            ctx,
        );
        start_instruction((v1 as usize) as isize, addr_word_86D03C as u16, ctx);
    } else {
        add_word(
            SlotField::VelocityY,
            (v0 >> 1) as isize,
            (word_86D082) as u16,
            ctx,
        );
    };
}

pub(super) fn EprojPreInstr_NamiFuneFireball(v0: u16, ctx: &mut Execution<'_>) {
    CallNamiFuneFireballFunc(
        (ctx.slot(SlotField::BehaviorA, (v0 >> 1) as isize) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    Eproj_NamiFuneFireball_After(v0, ctx);
}

pub(super) fn sub_86E049(v0: u16, ctx: &mut Execution<'_>) {
    CallLavamanFunc(
        (ctx.slot(SlotField::BehaviorA, (v0 >> 1) as isize) as i32 | 0x860000) as u32,
        v0,
        ctx,
    );
    sub_86E0A4(v0, ctx);
}

pub(super) fn EprojPreInstr_BA42(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    store(
        ctx.slot(SlotField::BehaviorA, (v1) as isize),
        SlotField::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.slot(SlotField::BehaviorB, (v1) as isize) as i32 + ctx.layer1_y_pos() as i32
            - ctx.expanding_square_origin() as i32) as u16,
        SlotField::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojInit_MotherBrainsDrool_Falling(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 12, ctx);
    if MoveEprojWithVelocityY(k, ctx) >= 0xd7 {
        add_word(SlotField::Y, (v1) as isize, (-4i16) as u16, ctx);
        store(
            addr_off_86C8E1 as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    }
}

pub(super) fn EprojInit_ShaktoolAttackFrontCircle(v1: u16, ctx: &mut Execution<'_>) {
    if EprojBlockCollisition_Horiz(v1, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(v1, ctx) & 1 != 0
    {
        store(0, SlotField::Definition, (v1 >> 1) as isize, ctx);
    }
}

pub(super) fn Eproj_PreInit_0x8aaf(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        ((ctx.slot(SlotField::VelocityY, (v1) as isize) as i16 as i32) << 8) as u32,
    );
    if ctx.slot(SlotField::Y, (v1) as isize) >= ctx.slot(SlotField::Counter, (v1) as isize) {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_CrocomireSpikeWallPieces(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = ctx.slot(SlotField::BehaviorA, (v1) as isize);
    if v2 != CrocomireSpikeWallPieces_Tab1[v1 as usize] {
        v2 = (v2 as i32 + CrocomireSpikeWallPieces_Tab2[v1 as usize] as i32) as u16;
        if v2 >= CrocomireSpikeWallPieces_Tab1[v1 as usize] {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    store(v2, SlotField::BehaviorA, (v1) as isize, ctx);
    let mut v3: i8 = ctx.slot_byte(SlotField::BehaviorA, k as isize + 1) as i8;
    let mut v4: bool =
        u8::overflowing_add(ctx.slot_byte(SlotField::VelocityX, k as isize), v3 as u8).1;
    {
        let index = k as isize;
        let old = ctx.slot_byte(SlotField::VelocityX, index);
        let value = (old as i32 + v3 as i32) as u8;
        ctx.set_slot_byte(SlotField::VelocityX, index, value);
    }
    let mut v5: u8 = (v4 as i32 + ctx.slot_byte(SlotField::VelocityX, k as isize + 1) as i32) as u8;
    if ((v5 as i32 - CrocomireSpikeWallPieces_Tab3[k as usize] as i32) as i8) >= 0 {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as u8;
    }
    ctx.set_slot_byte(SlotField::VelocityX, k as isize + 1, v5);
    let mut v6: i8 = ctx.slot_byte(SlotField::VelocityX, k as isize) as i8;
    v4 = u8::overflowing_add(
        ctx.slot_byte(SlotField::XFraction, k as isize + 1),
        v6 as u8,
    )
    .1;
    {
        let index = k as isize + 1;
        let old = ctx.slot_byte(SlotField::XFraction, index);
        let value = (old as i32 + v6 as i32) as u8;
        ctx.set_slot_byte(SlotField::XFraction, index, value);
    }
    let mut v7: i8 = ctx.slot_byte(SlotField::VelocityX, k as isize + 1) as i8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as u8, v7 as u8).1;
    let mut v8: i8 = (v9 as i32 + v7 as i32) as i8;
    v4 = (v4 as i32
        | u8::overflowing_add(ctx.slot_byte(SlotField::X, k as isize), v8 as u8).1 as i32)
        != 0;
    {
        let index = k as isize;
        let old = ctx.slot_byte(SlotField::X, index);
        let value = (old as i32 + v8 as i32) as u8;
        ctx.set_slot_byte(SlotField::X, index, value);
    }
    {
        let index = k as isize + 1;
        let old = ctx.slot_byte(SlotField::X, index);
        let value = (old as i32 + v4 as i32) as u8;
        ctx.set_slot_byte(SlotField::X, index, value);
    }
    ctx.add_split(
        SlotField::VelocityY,
        SlotField::BehaviorB,
        v1 as isize,
        0x3000,
    );
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        words!((
            ctx.slot(SlotField::VelocityY, (v1) as isize),
            ctx.slot(SlotField::BehaviorB, (v1) as isize)
        )),
    );
    if ctx.slot(SlotField::Y, (v1) as isize) >= 0xa8 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
        if (k & 2) == 0 {
            ctx.queue_sound(2, 6, 0x29);
        }
        let v12: i32 = k as i32 >> 1;
        ctx.state_mut().spawn.point = Point {
            x: ctx.slot(SlotField::X, (v12) as isize),
            y: ctx.slot(SlotField::Y, (v12) as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, ctx);
        ctx.queue_sound(2, 6, 0x25);
    }
}

pub(super) fn EprojPreInstr_9634(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(SlotField::BehaviorA, (v1) as isize) >= 8 {
        MoveEprojWithVelocity(k, ctx);
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            store(
                addr_off_869574 as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            add_word(SlotField::BehaviorA, (v1) as isize, 1, ctx);
            store(1, SlotField::Delay, (v1) as isize, ctx);
            {
                store(0, SlotField::VelocityX, (v1) as isize, ctx);
                store(0, SlotField::VelocityY, (v1) as isize, ctx);
            }
            store(2560, SlotField::Graphics, (v1) as isize, ctx);
            ctx.queue_sound(2, 6, 0x2b);
        }
    } else {
        add_word(SlotField::BehaviorA, (v1) as isize, 1, ctx);
    };
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 16, ctx);
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        store(
            (ctx.slot(SlotField::Properties, (v1) as isize) & 0xfff | 0x8000),
            SlotField::Properties,
            (v1) as isize,
            ctx,
        );
        store(
            (0x8699bf & 0xffff) as u16,
            SlotField::PreAction,
            (v1) as isize,
            ctx,
        );
        store(
            addr_word_86976C as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
        store(8, SlotField::BehaviorA, (v1) as isize, ctx);
        add_word(SlotField::Y, (v1) as isize, 8, ctx);
    }
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: bool = ctx.slot(SlotField::BehaviorA, (v1) as isize) == 1;
    let v3: bool =
        (add_word(SlotField::BehaviorA, (v1) as isize, (-1i16) as u16, ctx) & 0x8000) != 0;
    if v2 || v3 {
        {
            store(
                (0x869a01 & 0xffff) as u16,
                SlotField::PreAction,
                (v1) as isize,
                ctx,
            );
            store(
                addr_word_869772 as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v1) as isize, ctx);
        }
        add_word(SlotField::Y, (v1) as isize, (-8i16) as u16, ctx);
        {
            store(-768i32 as u16, SlotField::VelocityY, (v1) as isize, ctx);
            store(0, SlotField::BehaviorA, (v1) as isize, ctx);
        }
        if (ctx.nmi_frame() & 1) != 0 {
            store(-128i32 as u16, SlotField::VelocityX, (v1) as isize, ctx);
        } else {
            store(128, SlotField::VelocityX, (v1) as isize, ctx);
        }
    }
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballs3(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 16, ctx);
    let switch_fireball_phase: bool = if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        let v2: u16 = (ctx.slot(SlotField::BehaviorA, (v1) as isize) as i32 + 1) as u16;
        store(v2, SlotField::BehaviorA, (v1) as isize, ctx);
        if v2 as i32 - 3i32 & 0x8000 != 0 {
            store(
                word_869A3E[v2 as usize],
                SlotField::VelocityY,
                (v1) as isize,
                ctx,
            );
            return;
        }
        true
    } else {
        (EprojBlockCollisition_Horiz(k, ctx) & 1 != 0)
    };
    if switch_fireball_phase {
        {
            store(
                addr_word_869782 as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v1) as isize, ctx);
            store(
                (0x869a44 & 0xffff) as u16,
                SlotField::PreAction,
                (v1) as isize,
                ctx,
            );
        }
    };
}

pub(super) fn EprojPreInstr_PhantoonDestroyableFireballs_2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let apply_gravity: bool = (ctx.slot(SlotField::VelocityX, (v1) as isize) == 0)
        || ({
            let mut v2: bool = ctx.slot(SlotField::VelocityX, (v1) as isize) == 1;
            let mut v3: bool =
                (add_word(SlotField::VelocityX, (v1) as isize, (-1i16) as u16, ctx) & 0x8000) != 0;
            (v2 || v3)
                && ({
                    ctx.queue_sound(3, 6, 0x1d);
                    true
                })
        });
    if apply_gravity {
        add_word(SlotField::VelocityY, (v1) as isize, 16, ctx);
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            store(
                addr_word_8697AC as u16,
                SlotField::Program,
                (v1) as isize,
                ctx,
            );
            store(1, SlotField::Delay, (v1) as isize, ctx);
            add_word(SlotField::Y, (v1) as isize, 8, ctx);
            store(
                (0x869a44 & 0xffff) as u16,
                SlotField::PreAction,
                (v1) as isize,
                ctx,
            );
            ctx.queue_sound(3, 6, 0x1d);
        }
    };
}

pub(super) fn EprojPreInstr_PhantoonStartingFireballsB_2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = ctx.slot(SlotField::VelocityX, (v1) as isize);
    if v2 != 0 {
        store(
            (v2 as i32 - 1i32) as u16,
            SlotField::VelocityX,
            (v1) as isize,
            ctx,
        );
    } else if ((ctx.nmi_frame() & 1) != 0) {
        let v3: u16 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 - 1i32) as u16;
        store(v3, SlotField::VelocityY, (v1) as isize, ctx);
        if (v3 == 0) {
            {
                store(ctx.enemy_x(0), SlotField::X, (v1) as isize, ctx);
                store(
                    (ctx.enemy_y(0) as i32 + 16) as u16,
                    SlotField::Y,
                    (v1) as isize,
                    ctx,
                );
                store(1, SlotField::Delay, (v1) as isize, ctx);
                store(
                    addr_off_8697F8 as u16,
                    SlotField::Program,
                    (v1) as isize,
                    ctx,
                );
            }
            return;
        }
    }
    let v4: u8 = (ctx.slot(SlotField::BehaviorA, (v1) as isize) as i32 + 1) as u8;
    store(v4 as u16, SlotField::BehaviorA, (v1) as isize, ctx);
    let pt = Eproj_PhantomFireballs_Func1(v4 as u16, ctx.slot(SlotField::VelocityY, (v1) as isize));
    {
        store(
            (pt.x as i32 + ctx.enemy_x(0) as i32) as u16,
            SlotField::X,
            (v1) as isize,
            ctx,
        );
        store(
            (pt.y as i32 + ctx.enemy_y(0) as i32 + 16) as u16,
            SlotField::Y,
            (v1) as isize,
            ctx,
        );
    }
}

pub(super) fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::X, (v1 as usize) as isize, (-2i16) as u16, ctx);
    if (ctx.slot(SlotField::BehaviorA, (v1) as isize) & 0x8000) == 0 {
        add_word(SlotField::X, (v1) as isize, (-2i16) as u16, ctx);
    }
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        ctx.set_slot(SlotField::Definition, (v1 as usize) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::X, (v1 as usize) as isize, 2, ctx);
    if (ctx.slot(SlotField::BehaviorA, (v1) as isize) & 0x8000) == 0 {
        add_word(SlotField::X, (v1) as isize, 2, ctx);
    }
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        ctx.set_slot(SlotField::Definition, (v1 as usize) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_PirateClawThrownLeft(k: u16, ctx: &mut Execution<'_>) {
    advance_thrown_claw(k, false, ctx);
}

pub(super) fn EprojPreInstr_PirateClawThrownRight(k: u16, ctx: &mut Execution<'_>) {
    advance_thrown_claw(k, true, ctx);
}

pub(super) fn EprojPreInstr_AB07(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = ((8 * (ctx.player_x() as i32 - ctx.slot(SlotField::X, (v1) as isize) as i32))
        as i16 as i32)
        << 8;
    ctx.add_split(SlotField::X, SlotField::XFraction, v1 as isize, amt as u32);
    amt = ((8 * (ctx.player_y() as i32 - 36i32 - ctx.slot(SlotField::Y, (v1) as isize) as i32))
        as i16 as i32)
        << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v1 as isize, amt as u32);
}

pub(super) fn EprojPreInstr_GoldenTorizoEgg(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if (add_word(SlotField::BehaviorB, (v1) as isize, (-1i16) as u16, ctx) & 0x8000) != 0 {
        add_word(SlotField::Program, (v1) as isize, 2, ctx);
        store(1, SlotField::Delay, (v1) as isize, ctx);
        store(
            (if (ctx.slot(SlotField::BehaviorA, (v1) as isize) & 0x8000) != 0 {
                256
            } else {
                -256i32
            }) as u16,
            SlotField::VelocityX,
            (v1) as isize,
            ctx,
        );
    } else {
        if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
            store(
                -(ctx.slot(SlotField::VelocityX, (v1) as isize) as i32) as u16,
                SlotField::VelocityX,
                (v1) as isize,
                ctx,
            );
            {
                let index = (v1) as isize;
                let old = ctx.slot(SlotField::BehaviorA, (index) as isize);
                let value = old ^ (0x8000);
                ctx.set_slot(SlotField::BehaviorA, (index) as isize, value);
            };
        }
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
            && (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) == 0
        {
            {
                let index = (v1) as isize;
                let old = ctx.slot(SlotField::VelocityX, (index) as isize);
                let value = ((old as i32)
                    + (if ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000 != 0 {
                        32
                    } else {
                        -32i32
                    })) as u16;
                ctx.set_slot(SlotField::VelocityX, (index) as isize, value);
            };
            store(
                -(ctx.slot(SlotField::VelocityY, (v1) as isize) as i32) as u16,
                SlotField::VelocityY,
                (v1) as isize,
                ctx,
            );
        }
        add_word(SlotField::VelocityY, (v1) as isize, 48, ctx);
        if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0xf000) == 4096 {
            store(0, SlotField::Definition, (v1) as isize, ctx);
        }
    };
}

pub(super) fn EprojPreInstr_GoldenTorizoSuperMissile(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2 = ctx.slot(SlotField::BehaviorA, (v1) as isize);
    let v3: i32 = if (ctx.enemy_parameter_1(v2) & 0x8000) != 0 {
        32
    } else {
        -32i32
    };
    store(
        (ctx.enemy_x(v2) as i32 + v3) as u16,
        SlotField::X,
        (v1) as isize,
        ctx,
    );
    store(
        (ctx.enemy_y(v2) as i32 - 52i32) as u16,
        SlotField::Y,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn sub_86B535(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((ctx.slot(SlotField::VelocityX, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::X,
        SlotField::XFraction,
        (v1 as usize) as isize,
        split_amount,
    );
    let split_amount =
        ((ctx.slot(SlotField::VelocityY, (v1 as usize) as isize) as i16 as i32) << 8) as u32;
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        (v1 as usize) as isize,
        split_amount,
    );
    let v6: i16 = ctx.slot(SlotField::VelocityY, (v1 as usize) as isize) as i16;
    if v6 >= 0 {
        ctx.set_slot(
            SlotField::VelocityY,
            (v1 as usize) as isize,
            (v6 as u16).wrapping_add(32),
        );
        Eproj_DeleteIfYposOutside(k, ctx);
    } else {
        ctx.set_slot(
            SlotField::VelocityY,
            (v1 as usize) as isize,
            (v6 as u16).wrapping_add(32),
        );
        if (v6).wrapping_add(32) >= 0 {
            {
                let instruction: u16 =
                    (if (ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000) == 0 {
                        addr_word_86B4E3 as i32
                    } else {
                        addr_word_86B4D7 as i32
                    }) as u16;
                start_instruction((v1 as usize) as isize, instruction, ctx);
            }
        }
    };
}

pub(super) fn Eproj_DeleteIfYposOutside(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if ((ctx.slot(SlotField::Y, (v1) as isize)) as i16).wrapping_sub((ctx.layer1_y_pos()) as i16)
        >= 288
    {
        ctx.set_slot(SlotField::Definition, (v1 as usize) as isize, 0);
    }
}

pub(super) fn EprojPreInstr_TourianStatueUnlockingParticle(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = (ctx.slot(SlotField::VelocityX, (v1) as isize) as i16 as i32) << 8;
    ctx.add_split(SlotField::X, SlotField::XFraction, v1 as isize, amt as u32);
    let v7: u16 = (ctx.fx_y_pos() as i32 - ctx.slot(SlotField::Y, (v1) as isize) as i32) as u16;
    amt = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i16 as i32) << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v1 as isize, amt as u32);
    if ((v7 as i32 ^ ctx.fx_y_pos() as i32 - ctx.slot(SlotField::Y, (v1) as isize) as i32) & 0x8000)
        != 0
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as u16, k, ctx);
    }
    if (ctx.slot(SlotField::Y, (v1) as isize) & 0xff00) == 256 {
        store(
            addr_off_86B79F as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    } else {
        add_word(SlotField::VelocityY, (v1) as isize, 16, ctx);
    };
}

pub(super) fn EprojPreInstr_TourianStatueSoul(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i16 as i32) << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v1 as isize, amt as u32);
    if (ctx.slot(SlotField::Y, (v1) as isize) & 0x100) != 0 {
        store(
            addr_off_86B79F as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    }
    add_word(SlotField::VelocityY, (v1) as isize, (-128i16) as u16, ctx);
}

pub(super) fn Eproj_NorfairLavaquakeRocks_Func1(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 - 2i32) as i16;
    store(v2 as u16, SlotField::VelocityY, (v1) as isize, ctx);
    if v2 >= 0 {
        for n in (1i32..=2).rev() {
            let mut v3: i16 =
                (n + ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 - 1i32) as i16;
            if v3 < 0 {
                v3 = 0;
            }
            let t: i32 = 8 * v3 as i32 + 4 >> 1;
            apply_lavaquake_sample(v1 as isize, t, ctx);
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, ctx);
    } else {
        {
            store(0, SlotField::VelocityY, (v1) as isize, ctx);
            store(
                (0x86bc8f & 0xffff) as u16,
                SlotField::BehaviorA,
                (v1) as isize,
                ctx,
            );
        }
    };
}

pub(super) fn Eproj_NorfairLavaquakeRocks_Func2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 + 2) as u16;
    store(v2, SlotField::VelocityY, (v1) as isize, ctx);
    if v2 as i32 - 64i32 & 0x8000 == 0 {
        store(64, SlotField::VelocityY, (v1) as isize, ctx);
    }
    for n in (1i32..=2).rev() {
        let t: i32 = 8 * (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 - n + 1) >> 1;
        apply_lavaquake_sample(v1 as isize, t, ctx);
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, ctx);
}

pub(super) use self::EprojPreInstr_SpikeShootingPlantSpikes_MoveX2 as Eproj_NorfairLavaquakeRocks_Func3;

pub(super) fn EprojPreInstr_BE12(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(
        SlotField::Definition,
        (ctx.slot(SlotField::BehaviorA, v1 as isize) >> 1) as isize,
    ) != 0
    {
        EprojBlockCollisition_Horiz(k, ctx);
        EprojBlockCollisition_Vertical(k, ctx);
    } else {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    };
}

pub(super) fn EprojPreInstr_MotherBrainRoomTurretBullets(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = ctx.slot(SlotField::Properties, (index) as isize);
        let value = old ^ (0x8000);
        ctx.set_slot(SlotField::Properties, (index) as isize, value);
    };
    MoveEprojWithVelocity(k, ctx);
    if ctx.ridley_offscreen(
        ctx.slot(SlotField::X, (v1) as isize),
        ctx.slot(SlotField::Y, (v1) as isize),
    ) & 1
        != 0
    {
        {
            let index = (ctx.state().selected >> 1) as isize;
            ctx.set_slot(SlotField::Definition, index, 0);
        }
    }
}

pub(super) fn sub_86C320(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        store(
            (ctx.enemy_x(64) as i32 + 10) as u16,
            SlotField::X,
            (v1) as isize,
            ctx,
        );
        store(
            (ctx.enemy_y(64) as i32 + 16) as u16,
            SlotField::Y,
            (v1) as isize,
            ctx,
        );
    }
}

pub(super) fn Eproj_MoveToBlueRingSpawnPosition(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(SlotField::BehaviorA, (v1) as isize) != 0 {
        ctx.decrement(SlotField::BehaviorA, (v1) as isize);
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
            let mut v5: i16 = (ctx.enemy_health(v4) as i32 - 80i32) as i16;
            if v5 < 0 {
                v5 = 0;
            }
            ctx.set_enemy_health(v4, v5 as u16);
        } else if Eproj_CheckForCollisionWithSamus(k, ctx) & 1 != 0 {
            BlueRingContactExplosion(k, ctx);
            {
                let damage = ctx.reduce_suit_damage(0x50);
                ctx.damage_player(damage);
            }
            {
                let value = 96;
                ctx.set_player_invincibility(value);
            };
            {
                let value = 5;
                ctx.set_player_knockback(value);
            };
            {
                let value = (((ctx.player_x() as i32
                    - ctx.slot(SlotField::X, (k >> 1) as isize) as i32)
                    as i16)
                    >= 0) as u16;
                ctx.set_knockback_x_dir(value);
            };
        } else if CheckForBlueRingCollisionWithRoom(k, ctx) & 1 != 0 {
            Eproj_Earthqhake5(k, ctx);
        }
    };
}

pub(super) fn BlueRingContactExplosion(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    store(0, SlotField::Definition, (v1) as isize, ctx);
    ctx.state_mut().spawn.point = Point {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: ctx.slot(SlotField::Y, (v1) as isize),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, ctx);
    ctx.queue_sound(3, 6, 0x13);
    sub_86C42E(k, ctx);
}

pub(super) fn EprojPreInstr_C84D(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: i32 = (4 * ctx.slot(SlotField::BehaviorA, (v1) as isize) as i32) as u16 as i32 >> 1;
    {
        store(
            (ctx.enemy_x(64) as i32 + kEprojInit_MotherBrainsDrool[v2 as usize] as i32) as u16,
            SlotField::X,
            (v1) as isize,
            ctx,
        );
        store(
            (ctx.enemy_y(64) as i32 + kEprojInit_MotherBrainsDrool[(v2 + 1) as usize] as i32)
                as u16,
            SlotField::Y,
            (v1) as isize,
            ctx,
        );
    }
    {
        store(0, SlotField::VelocityX, (v1) as isize, ctx);
        store(0, SlotField::VelocityY, (v1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        store(
            (ctx.player_x() as i32 + ctx.slot(SlotField::VelocityX, (v1) as isize) as i32) as u16,
            SlotField::X,
            (v1) as isize,
            ctx,
        );
        store(
            (ctx.player_y() as i32 + ctx.slot(SlotField::VelocityY, (v1) as isize) as i32) as u16,
            SlotField::Y,
            (v1) as isize,
            ctx,
        );
    }
}

pub(super) fn EprojPreInstr_MotherBrainsExplodedDoorParticles(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = (abs16(ctx.slot(SlotField::VelocityX, (v1) as isize)) as i32 - 16i32) as i16;
    if v2 < 0 {
        v2 = 0;
    }
    store(
        (if ctx.slot(SlotField::VelocityX, (v1) as isize) & 0x8000 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16,
        SlotField::VelocityX,
        (v1) as isize,
        ctx,
    );
    add_word(SlotField::VelocityY, (v1) as isize, 32, ctx);
    MoveEprojWithVelocity(k, ctx);
    if (add_word(SlotField::BehaviorA, (v1) as isize, (-1i16) as u16, ctx) & 0x8000) != 0 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
        add_word(SlotField::Y, (v1) as isize, (-4i16) as u16, ctx);
        ctx.state_mut().spawn.point = Point {
            x: ctx.slot(SlotField::X, (v1) as isize),
            y: ctx.slot(SlotField::Y, (v1) as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    }
}

pub(super) fn EprojPreInstr_TimeBombSetJapaneseText(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        store(0, SlotField::VelocityX, (v1) as isize, ctx);
        store(0, SlotField::VelocityY, (v1) as isize, ctx);
    }
    {
        store(128, SlotField::X, (v1) as isize, ctx);
        store(192, SlotField::Y, (v1) as isize, ctx);
    }
}

pub(super) fn MotherBrainTubeFallingFunc_GenerateExplosion(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    ctx.state_mut().spawn.point = Point {
        x: ctx.slot(SlotField::X, (v1) as isize),
        y: (ctx.slot(SlotField::Y, (v1) as isize) as i32 + 8) as u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, ctx);
    store(
        (0x86cc08 & 0xffff) as u16,
        SlotField::BehaviorA,
        (v1) as isize,
        ctx,
    );
    MotherBrainTubeFallingFunc_Falling(k, ctx);
}

pub(super) fn MotherBrainTubeFallingFunc_Falling(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    add_word(SlotField::VelocityY, (v1) as isize, 6, ctx);
    let mut v2: i16 = MoveEprojWithVelocityY(k, ctx) as i16;
    if v2 as i32 - 208i32 & 0x8000 == 0 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
        ctx.state_mut().spawn.point = Point {
            x: ctx.slot(SlotField::X, (v1) as isize),
            y: ctx.slot(SlotField::Y, (v1) as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, ctx);
    }
}

pub(super) fn sub_86CFD5(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    store(
        (0x86cff8 & 0xffff) as u16,
        SlotField::PreAction,
        (v1) as isize,
        ctx,
    );
    add_word(SlotField::X, (v1) as isize, (-19i16) as u16, ctx);
}

pub(super) fn sub_86CFE6(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    store(
        (0x86cff8 & 0xffff) as u16,
        SlotField::PreAction,
        (v1) as isize,
        ctx,
    );
    add_word(SlotField::X, (v1) as isize, 19, ctx);
}

pub(super) fn sub_86CFF8(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
        store(
            addr_off_86CF56 as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    } else if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 {
        store(0, SlotField::VelocityX, (k >> 1) as isize, ctx);
    } else {
        let mut v2: u16 = (ctx.slot(SlotField::VelocityY, (v1) as isize) as i32 + 16) as u16;
        store(v2, SlotField::VelocityY, (v1) as isize, ctx);
        if v2 as i32 - 512i32 & 0x8000 == 0 {
            v2 = 512;
        }
        store(v2, SlotField::VelocityY, (v1) as isize, ctx);
    };
}

pub(super) fn EprojPreInstr_KagosBugs_Func1(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = ctx.slot(SlotField::BehaviorA, (v1 as usize) as isize);
    if v2 != 0 {
        let v3: u16 = v2.wrapping_sub(1);
        ctx.set_slot(SlotField::BehaviorA, (v1 as usize) as isize, v3);
        if v3 == 0 {
            ctx.queue_sound(2, 6, 0x6c);
        }
    }
}

pub(super) fn EprojPreInstr_KagosBugs_Func2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let enemy_index = ctx.slot(SlotField::BehaviorB, (v1) as isize);
    let v3: u16 =
        abs16((ctx.enemy_x(enemy_index)).wrapping_sub(ctx.slot(SlotField::X, (k >> 1) as isize)));
    if v3 as i32 - 23i32 & 0x8000 == 0 {
        set_property_bits((v1 as usize) as isize, 0x8000, ctx);
    }
}

pub(super) fn EprojPreInstr_MaridiaFloatersSpikes(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = ctx.slot(SlotField::VelocityX, (index) as isize);
        let value = ((old as i32)
            + (word_86D21A[ctx.slot(SlotField::BehaviorA, (v1) as isize) as usize] as i32))
            as u16;
        ctx.set_slot(SlotField::VelocityX, (index) as isize, value);
    };
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0 || {
        {
            let index = (v1) as isize;
            let old = ctx.slot(SlotField::VelocityY, (index) as isize);
            let value = ((old as i32)
                + (word_86D22A[ctx.slot(SlotField::BehaviorA, (v1) as isize) as usize] as i32))
                as u16;
            ctx.set_slot(SlotField::VelocityY, (index) as isize, value);
        };
        EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    } {
        store(
            addr_off_86D218 as u16,
            SlotField::Program,
            (v1) as isize,
            ctx,
        );
        store(1, SlotField::Delay, (v1) as isize, ctx);
    }
}

pub(super) fn EprojPreInstr_WreckedShipRobotLaser(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    store(0, SlotField::Graphics, (v1) as isize, ctx);
    if EprojBlockCollisition_Horiz(k, ctx) & 1 != 0
        || EprojBlockCollisition_Vertical(k, ctx) & 1 != 0
    {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    }
}

pub(super) fn sub_86D7BF(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if ctx.slot(SlotField::X, (v1) as isize) != 0xee00 {
        store(
            ctx.slot(SlotField::X, (v1) as isize),
            SlotField::BehaviorA,
            (v1) as isize,
            ctx,
        );
    }
    if ctx.nmi_frame() & 1 != 0 {
        store(-4608i32 as u16, SlotField::X, (v1) as isize, ctx);
    } else {
        store(
            ctx.slot(SlotField::BehaviorA, (v1) as isize),
            SlotField::X,
            (v1) as isize,
            ctx,
        );
    };
}

pub(super) fn sub_86D7DE(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (192i16 as i32) << 8;
    ctx.add_split(SlotField::Y, SlotField::YFraction, v1 as isize, amt as u32);
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        ((ctx.slot(SlotField::VelocityX, (v1) as isize) as i16 as i32) << 8) as u32,
    );
}

pub(super) use self::EprojPreInstr_SpikeShootingPlantSpikes_MoveX1 as EprojPreInstr_DBF2_MoveX1;

pub(super) fn EprojPreInstr_Spores(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = ctx.slot_byte(SlotField::BehaviorA, v1 as isize * 2) as u16;
    let mut r18: u16 = SignExtend8(
        ctx.raw_spore_delta(ctx.slot_byte(SlotField::BehaviorA, v1 as isize * 2) as isize) as u16,
    );
    if (ctx.slot(SlotField::BehaviorB, (v1) as isize) & 0x80) != 0 {
        r18 = r18.wrapping_neg();
    }
    add_word(SlotField::X, (v1) as isize, r18, ctx);
    r18 = SignExtend8(ctx.raw_spore_delta((v2 as i32 + 1) as isize) as u16);
    let v3: u16 = (r18 as i32 + ctx.slot(SlotField::Y, (v1) as isize) as i32 + r18 as i32) as u16;
    store(v3, SlotField::Y, (v1) as isize, ctx);
    if v3 as i32 - 768i32 & 0x8000 == 0 {
        store(0, SlotField::Definition, (v1) as isize, ctx);
    }
    store(
        (ctx.slot_byte(SlotField::BehaviorA, v1 as isize * 2) as i32 + 2) as u8 as u16,
        SlotField::BehaviorA,
        (v1) as isize,
        ctx,
    );
}

pub(super) fn EprojPreInstr_E605(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = (ctx.slot(SlotField::Counter, (v1) as isize))
        .wrapping_add(abs16(ctx.slot(SlotField::VelocityY, (v1) as isize)));
    if v2 >= 0x1000 {
        store(1, SlotField::Delay, (v1) as isize, ctx);
        let fresh60 = add_word(SlotField::Program, (v1 as usize) as isize, 1, ctx);
        let fresh61 = add_word(SlotField::Program, (v1 as usize) as isize, 1, ctx);
        v2 = 0;
    }
    store(v2, SlotField::Counter, (v1) as isize, ctx);
    {
        let index = (v1) as isize;
        let old = ctx.slot(SlotField::YFraction, (index) as isize);
        let value = ((old as i32)
            + ((ctx.slot_byte(SlotField::VelocityY, v1 as isize * 2) as i32) << 8))
            as u16;
        ctx.set_slot(SlotField::YFraction, (index) as isize, value);
    };
    {
        let index = (v1) as isize;
        let old = ctx.slot(SlotField::Y, (index) as isize);
        let value = old
            .wrapping_add((ctx.slot_byte(SlotField::VelocityY, v1 as isize * 2 + 1) as i8) as u16);
        ctx.set_slot(SlotField::Y, (index) as isize, value);
    };
}

pub(super) fn Eproj_FuncE73E_MoveXY(k: u16, ctx: &mut Execution<'_>) {
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

pub(super) fn Eproj_BotwoonsBody_Main(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = {
        let image = ctx.image();
        image.table_word(
            0x369f1,
            (ctx.slot(SlotField::BehaviorA, (v1) as isize) >> 1) as isize,
        )
    };
    if v2 != ctx.slot(SlotField::BehaviorB, (v1) as isize) {
        {
            store(v2, SlotField::Program, (v1) as isize, ctx);
            store(v2, SlotField::BehaviorB, (v1) as isize, ctx);
            store(1, SlotField::Delay, (v1) as isize, ctx);
        }
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, ctx);
}

pub(super) fn Eproj_BotwoonsBodyHurtFlashHandling1(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    {
        let index = (v1) as isize;
        let old = ctx.slot(SlotField::Graphics, (index) as isize);
        let value = old | (0xe00);
        ctx.set_slot(SlotField::Graphics, (index) as isize, value);
    };
    if ctx.enemy_flash_timer(0) != 0 && (ctx.random_enemy_counter() & 2) != 0 {
        {
            let index = (v1) as isize;
            let old = ctx.slot(SlotField::Graphics, (index) as isize);
            let value = old & (0xf1ff);
            ctx.set_slot(SlotField::Graphics, (index) as isize, value);
        };
    }
}

pub(super) fn EprojPreInstr_Sparks(k: u16, ctx: &mut Execution<'_>) {
    let v1: i32 = k as i32 >> 1;
    if (ctx.slot(SlotField::VelocityY, (v1) as isize) & 0x8000) == 0 {
        if EprojBlockCollisition_Vertical(k, ctx) & 1 != 0 {
            {
                store(0xf363, SlotField::Program, (v1) as isize, ctx);
                store(1, SlotField::Delay, (v1) as isize, ctx);
            }
            for _ in 0..2 {
                let high = (ctx.slot(SlotField::BehaviorB, v1 as isize) as i32 * 2
                    + (ctx.slot(SlotField::BehaviorA, v1 as isize) as i32 >> 15))
                    as u16;
                ctx.set_slot(SlotField::BehaviorB, v1 as isize, high);
                let low = ctx.slot(SlotField::BehaviorA, v1 as isize).wrapping_mul(2);
                ctx.set_slot(SlotField::BehaviorA, v1 as isize, low);
            }
            {
                store(0x8000, SlotField::VelocityX, (v1) as isize, ctx);
                store(-1i32 as u16, SlotField::VelocityY, (v1) as isize, ctx);
            }
            add_word(SlotField::Y, (v1) as isize, (-2i16) as u16, ctx);
            return;
        }
        let v2: u16 = ctx.slot(SlotField::VelocityX, (v1) as isize);
        store(
            (v2 as i32 + 0x4000) as u16,
            SlotField::VelocityX,
            (v1) as isize,
            ctx,
        );
        let v3: u16 = (u16::overflowing_add(v2, 0x4000u16).1 as i32
            + ctx.slot(SlotField::VelocityY, (v1) as isize) as i32) as u16;
        if v3 < 4 {
            store(v3, SlotField::VelocityY, (v1) as isize, ctx);
        }
    }
    ctx.add_split(
        SlotField::Y,
        SlotField::YFraction,
        v1 as isize,
        words!((
            ctx.slot(SlotField::VelocityY, (v1) as isize),
            ctx.slot(SlotField::VelocityX, (v1) as isize)
        )),
    );
    ctx.add_split(
        SlotField::X,
        SlotField::XFraction,
        v1 as isize,
        words!((
            ctx.slot(SlotField::BehaviorB, (v1) as isize),
            ctx.slot(SlotField::BehaviorA, (v1) as isize)
        )),
    );
    if (ctx.nmi_frame_counter_low() & 3) == 0 {
        ctx.create_sprite(
            ctx.slot(SlotField::X, (v1) as isize),
            ctx.slot(SlotField::Y, (v1) as isize),
            48,
            ctx.slot(SlotField::Graphics, (v1) as isize),
        );
    }
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, ctx)
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, ctx)
}

pub(super) fn EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k: u16, ctx: &mut Execution<'_>) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, ctx)
}

pub(super) use self::EprojPreInstr_96C0 as EprojPreInstr_96CE;

fn advance_thrown_claw(k: u16, initially_right: bool, ctx: &mut Execution<'_>) {
    let index = (k >> 1) as isize;
    let outbound = ctx.slot(SlotField::BehaviorB, index) != 0;
    let x = ctx.slot(SlotField::X, index);
    let speed = u16::from(ctx.slot_byte(SlotField::BehaviorA, index * 2 + 1));
    let right = if outbound {
        initially_right
    } else {
        !initially_right
    };
    ctx.set_slot(
        SlotField::X,
        index,
        if right {
            x.wrapping_add(speed)
        } else {
            x.wrapping_sub(speed)
        },
    );
    if outbound {
        let speed = ctx.slot(SlotField::BehaviorA, index).wrapping_sub(32);
        ctx.set_slot(SlotField::BehaviorA, index, speed);
        if speed == 0 {
            ctx.set_slot(SlotField::BehaviorB, index, 0);
        }
    } else {
        add_word(SlotField::BehaviorA, index, 32, ctx);
    }
    add_word(SlotField::Y, index, 1, ctx);
    if CheckIfEprojIsOffScreen(k, ctx) != 0 {
        ctx.set_slot(SlotField::Definition, index, 0);
    }
}

fn integrate_angle_axis(
    index: isize,
    high: SlotField,
    low: SlotField,
    velocity: SlotField,
    fraction: SlotField,
    phase: i32,
    ctx: &mut Execution<'_>,
) {
    // Y must reread Angle after the complete X integration.
    let negative = (ctx.slot(SlotField::Angle, index) as i32 + phase & 0x80) != 0;
    let amount = words!((ctx.slot(velocity, index), ctx.slot(fraction, index)));
    // Preserve the original signed-min negation panic in checked builds.
    let amount = if negative {
        -(amount as i32) as u32
    } else {
        amount
    };
    ctx.add_split(high, low, index, amount);
}

fn apply_lavaquake_sample(index: isize, table_index: i32, ctx: &mut Execution<'_>) {
    let amount = ctx.raw_lavaquake_delta((table_index >> 1) as isize);
    ctx.add_split(SlotField::Y, SlotField::YFraction, index, amount);
    let image = ctx.image();
    let value = image.table_word(0x104bc7, (table_index + 1) as isize);
    ctx.set_slot(SlotField::BehaviorB, index, value);
}
