//! Station placement, access alignment and save-trigger activation.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    Map,
    Energy,
    Missile,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Access(Service, Side),
    Place(Service),
    SaveTrigger,
    Save,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationBlock {
    Access(Service, Side),
    Save,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    FacingLeftWall,
    FacingRightWall,
    StandingRight,
    StandingLeft,
}

fn associated<C: Fields>(c: &C, block: u16) -> Option<usize> {
    (0..40)
        .rev()
        .find(|&slot| c.slot(Column::Block, slot) == block)
}
fn access(c: &mut impl Fields, slot: usize, service: Service, side: Side) {
    let (direction, pose, displacement) = match side {
        Side::Right => (0, Pose::FacingLeftWall, -2),
        Side::Left => (
            1,
            Pose::FacingRightWall,
            if service == Service::Map { 4 } else { 2 },
        ),
    };
    if c.global(Global::CollisionDirection) & 15 == direction
        && pose_matches(c, pose)
        && c.global_byte(Global::Facing, 0) & if side == Side::Right { 4 } else { 8 } != 0
        && (service == Service::Map || needs_service(c, service))
    {
        let block = (c.slot(Column::Block, slot)).wrapping_add((displacement) as u16);
        if let Some(found) = associated(c, block) {
            c.coordinates(usize::from(c.global(Global::Selected) >> 1));
            if (16u16.wrapping_mul(c.global(Global::BlockY)) | 0xb) == c.global(Global::PlayerY) {
                c.set_slot(Column::Program, found, c.slot(Column::ReturnProgram, found));
                c.set_slot(Column::InstructionDelay, found, 1);
                c.player_control(6);
                return;
            }
        }
    }
    c.release(slot);
}
fn place(c: &mut impl Fields, slot: usize, service: Service) {
    solidify(c.slot(Column::Block, slot), c);
    if service == Service::Map && c.map_used() {
        c.set_slot(Column::Program, slot, 0xad76);
    } else {
        let displacement = if service == Service::Map { -4 } else { -2 };
        install(
            (c.slot(Column::Block, slot)).wrapping_add((2) as u16),
            StationBlock::Access(service, Side::Right),
            c,
        );
        install(
            (c.slot(Column::Block, slot)).wrapping_add((displacement) as u16),
            StationBlock::Access(service, Side::Left),
            c,
        );
    }
}
fn save_trigger(c: &mut impl Fields, slot: usize) {
    if c.global(Global::PowerBombActive) == 0
        && (pose_matches(c, Pose::StandingRight) || pose_matches(c, Pose::StandingLeft))
        && c.global(Global::SaveLocked) == 0
        && c.global(Global::CollisionDirection) & 15 == 3
    {
        c.coordinates(slot);
        if c.global(Global::PlayerX).wrapping_sub(8) >> 4 == c.global(Global::BlockX) {
            let block = c.slot(Column::Block, slot);
            c.set_slot(Column::Block, slot, 0);
            c.release(slot);
            if let Some(found) = associated(c, block) {
                c.advance(found, 1);
                c.advance(found, 1);
                c.set_slot(Column::InstructionDelay, found, 1);
            }
        }
    }
}
pub fn setup(c: &mut impl Fields, slot: usize, setup: Setup) -> u8 {
    match setup {
        Setup::Access(service, side) => {
            access(c, slot, service, side);
            1
        }
        Setup::Place(service) => {
            place(c, slot, service);
            0
        }
        Setup::SaveTrigger => {
            save_trigger(c, slot);
            1
        }
        Setup::Save => {
            install(c.slot(Column::Block, slot), StationBlock::Save, c);
            0
        }
    }
}

fn solidify(block: u16, c: &mut impl Fields) {
    c.set_level(block, c.level(block) & 0xfff | 0x8000);
}
fn pose_matches(c: &impl Fields, pose: Pose) -> bool {
    c.global(Global::Pose)
        == match pose {
            Pose::FacingLeftWall => 0x8a,
            Pose::FacingRightWall => 0x89,
            Pose::StandingRight => 1,
            Pose::StandingLeft => 2,
        }
}
fn needs_service(c: &impl Fields, service: Service) -> bool {
    match service {
        Service::Energy => c.global(Global::Health) != c.global(Global::MaxHealth),
        Service::Missile => c.global(Global::Missiles) != c.global(Global::MaxMissiles),
        Service::Map => unreachable!(),
    }
}
fn install(block: u16, tile: StationBlock, c: &mut impl Fields) {
    c.install(
        block,
        match tile {
            StationBlock::Access(Service::Map, Side::Right) => 0xb047,
            StationBlock::Access(Service::Map, Side::Left) => 0xb048,
            StationBlock::Access(Service::Energy, Side::Right) => 0xb049,
            StationBlock::Access(Service::Energy, Side::Left) => 0xb04a,
            StationBlock::Access(Service::Missile, Side::Right) => 0xb04b,
            StationBlock::Access(Service::Missile, Side::Left) => 0xb04c,
            StationBlock::Save => 0xb04d,
        },
    );
}
