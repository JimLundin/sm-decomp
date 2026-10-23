//! Station placement, access alignment and save-trigger activation.
#![forbid(unsafe_code)]
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
pub trait Context {
    type Block: Copy + Eq;
    type Program;
    fn block(&self, slot: usize) -> Self::Block;
    fn adjacent(&self, block: Self::Block, displacement: i16) -> Self::Block;
    fn clear_block(&mut self, slot: usize);
    fn release(&mut self, slot: usize);
    fn direction(&self) -> u16;
    fn pose(&self, pose: Pose) -> bool;
    fn facing(&self, side: Side) -> bool;
    fn needs_service(&self, service: Service) -> bool;
    fn selected(&self) -> usize;
    fn coordinates(&mut self, slot: usize);
    fn cannon_aligned(&self) -> bool;
    fn save_aligned(&self) -> bool;
    fn return_program(&self, slot: usize) -> Self::Program;
    fn set_program(&mut self, slot: usize, program: Self::Program);
    fn advance_one(&mut self, slot: usize);
    fn set_timer(&mut self, slot: usize);
    fn lock_player(&mut self);
    fn power_bomb_active(&self) -> bool;
    fn save_locked(&self) -> bool;
    fn solidify(&mut self, block: Self::Block);
    fn map_used(&self) -> bool;
    fn map_used_program(&self) -> Self::Program;
    fn install(&mut self, block: Self::Block, tile: StationBlock);
}
fn associated<C: Context>(c: &C, block: C::Block) -> Option<usize> {
    (0..40).rev().find(|&slot| c.block(slot) == block)
}
fn access(c: &mut impl Context, slot: usize, service: Service, side: Side) {
    let (direction, pose, displacement) = match side {
        Side::Right => (0, Pose::FacingLeftWall, -2),
        Side::Left => (
            1,
            Pose::FacingRightWall,
            if service == Service::Map { 4 } else { 2 },
        ),
    };
    if c.direction() & 15 == direction
        && c.pose(pose)
        && c.facing(side)
        && (service == Service::Map || c.needs_service(service))
    {
        let block = c.adjacent(c.block(slot), displacement);
        if let Some(found) = associated(c, block) {
            c.coordinates(c.selected());
            if c.cannon_aligned() {
                c.set_program(found, c.return_program(found));
                c.set_timer(found);
                c.lock_player();
                return;
            }
        }
    }
    c.release(slot);
}
fn place(c: &mut impl Context, slot: usize, service: Service) {
    c.solidify(c.block(slot));
    if service == Service::Map && c.map_used() {
        c.set_program(slot, c.map_used_program());
    } else {
        let displacement = if service == Service::Map { -4 } else { -2 };
        c.install(
            c.adjacent(c.block(slot), 2),
            StationBlock::Access(service, Side::Right),
        );
        c.install(
            c.adjacent(c.block(slot), displacement),
            StationBlock::Access(service, Side::Left),
        );
    }
}
fn save_trigger(c: &mut impl Context, slot: usize) {
    if !c.power_bomb_active()
        && (c.pose(Pose::StandingRight) || c.pose(Pose::StandingLeft))
        && !c.save_locked()
        && c.direction() & 15 == 3
    {
        c.coordinates(slot);
        if c.save_aligned() {
            let block = c.block(slot);
            c.clear_block(slot);
            c.release(slot);
            if let Some(found) = associated(c, block) {
                c.advance_one(found);
                c.advance_one(found);
                c.set_timer(found);
            }
        }
    }
}
pub fn setup(c: &mut impl Context, slot: usize, setup: Setup) -> u8 {
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
            c.install(c.block(slot), StationBlock::Save);
            0
        }
    }
}
