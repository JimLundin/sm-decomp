//! Station placement, access alignment and save-trigger activation.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::creation;
use super::super::host::*;
use super::terrain_setup as terrain;
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

fn associated<C: Host>(c: &C, block: u16) -> Option<usize> {
    (0..40)
        .rev()
        .find(|&slot| mechanism!(c, slot, block) == block)
}
fn access(c: &mut impl Host, slot: usize, service: Service, side: Side) {
    let (direction, pose, displacement) = match side {
        Side::Right => (0, Pose::FacingLeftWall, -2),
        Side::Left => (
            1,
            Pose::FacingRightWall,
            if service == Service::Map { 4 } else { 2 },
        ),
    };
    if c.session().player.contact.collision_direction & 15 == direction
        && pose_matches(c, pose)
        && (c.session().player.movement_and_facing.to_le_bytes()[0])
            & if side == Side::Right { 4 } else { 8 }
            != 0
        && (service == Service::Map || needs_service(c, service))
    {
        let block = (mechanism!(c, slot, block)).wrapping_add((displacement) as u16);
        if let Some(found) = associated(c, block) {
            let selected = usize::from(c.session().mechanisms.selected >> 1);
            creation::coordinates(c, selected);
            if (16u16.wrapping_mul(c.session().mechanisms.block_y) | 0xb) == c.session().player.y {
                mechanism!(c, found, program = mechanism!(c, found, return_program));
                mechanism!(c, found, instruction_delay = 1);
                c.player_control(6);
                return;
            }
        }
    }
    mechanism!(c, slot, header = 0);
}
fn place(c: &mut impl Host, slot: usize, service: Service) {
    solidify(mechanism!(c, slot, block), c);
    if service == Service::Map && c.map_used() {
        mechanism!(c, slot, program = 0xad76);
    } else {
        let displacement = if service == Service::Map { -4 } else { -2 };
        install(
            (mechanism!(c, slot, block)).wrapping_add(2_u16),
            StationBlock::Access(service, Side::Right),
            c,
        );
        install(
            (mechanism!(c, slot, block)).wrapping_add((displacement) as u16),
            StationBlock::Access(service, Side::Left),
            c,
        );
    }
}
fn save_trigger(c: &mut impl Host, slot: usize) {
    if c.inputs().power_bomb_active == 0
        && (pose_matches(c, Pose::StandingRight) || pose_matches(c, Pose::StandingLeft))
        && c.session().mechanisms.save_locked == 0
        && c.session().player.contact.collision_direction & 15 == 3
    {
        creation::coordinates(c, slot);
        if c.session().player.x.wrapping_sub(8) >> 4 == c.session().mechanisms.block_x {
            let block = mechanism!(c, slot, block);
            mechanism!(c, slot, block = 0);
            mechanism!(c, slot, header = 0);
            if let Some(found) = associated(c, block) {
                {
                    let index = found;
                    let value = mechanism!(c, index, program).wrapping_add(1);
                    mechanism!(c, index, program = value);
                };
                {
                    let index = found;
                    let value = mechanism!(c, index, program).wrapping_add(1);
                    mechanism!(c, index, program = value);
                };
                mechanism!(c, found, instruction_delay = 1);
            }
        }
    }
}
pub fn setup(c: &mut impl Host, slot: usize, setup: Setup) -> u8 {
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
            install(mechanism!(c, slot, block), StationBlock::Save, c);
            0
        }
    }
}

fn solidify(block: u16, c: &mut impl Host) {
    let level = terrain::level(c, block) & 0xfff | 0x8000;
    terrain::set_level(c, block, level);
}
fn pose_matches(c: &impl Host, pose: Pose) -> bool {
    c.session().player.pose
        == match pose {
            Pose::FacingLeftWall => 0x8a,
            Pose::FacingRightWall => 0x89,
            Pose::StandingRight => 1,
            Pose::StandingLeft => 2,
        }
}
fn needs_service(c: &impl Host, service: Service) -> bool {
    match service {
        Service::Energy => {
            c.session().inventory.health.current != c.session().inventory.health.maximum
        }
        Service::Missile => {
            c.session().inventory.ammunition[0].current
                != c.session().inventory.ammunition[0].maximum
        }
        Service::Map => unreachable!(),
    }
}
fn install(block: u16, tile: StationBlock, c: &mut impl Host) {
    terrain::install(
        c,
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
