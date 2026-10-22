//! Ordered aim consumption and projectile placement, independent of state storage.
#![forbid(unsafe_code)]
pub use crate::game::projectile_room::Axis;

#[derive(Clone, Copy)]
pub enum Input {
    Pose,
    PreviousPose,
    X,
    Y,
    Running,
}
#[derive(Clone, Copy)]
pub enum PoseField {
    Direction,
    Height,
}
pub trait Context<O> {
    fn input(&self, owner: &O, field: Input) -> u16;
    fn pending(&mut self, owner: &mut O) -> Option<u8>;
    fn pose_byte(&mut self, owner: O, pose: u16, field: PoseField) -> (O, u8);
    fn reject(&mut self, owner: &mut O);
    fn set_direction(&mut self, owner: &mut O, slot: u16, direction: u8);
    fn direction(&self, owner: &O, slot: u16) -> u16;
    fn set_position(&mut self, owner: &mut O, slot: u16, axis: Axis, value: u16);
    fn invalid(&mut self, owner: O, index: usize) -> !;
}
pub fn place<O>(mut owner: O, slot: u16, context: &mut impl Context<O>) -> (O, bool) {
    let mut pose = context.input(&owner, Input::Pose);
    let direction = if let Some(direction) = context.pending(&mut owner) {
        direction
    } else {
        let (next, mut direction) = context.pose_byte(owner, pose, PoseField::Direction);
        owner = next;
        if direction & 0xf0 != 0 {
            if direction == 0x10 {
                let previous = context.input(&owner, Input::PreviousPose);
                (owner, direction) = context.pose_byte(owner, previous, PoseField::Direction);
            }
            if direction & 0xf0 != 0 {
                context.reject(&mut owner);
                return (owner, false);
            }
            pose = context.input(&owner, Input::Pose);
        }
        direction
    };
    context.set_direction(&mut owner, slot, direction);
    let (mut owner, height) = context.pose_byte(owner, pose, PoseField::Height);
    let direction = usize::from(context.direction(&owner, slot) & 15);
    let (xs, ys) = if context.input(&owner, Input::Pose) == 0x75
        || context.input(&owner, Input::Pose) == 0x76
        || context.input(&owner, Input::Running) == 1
    {
        (&RUNNING_X, &RUNNING_Y)
    } else {
        (&ORDINARY_X, &ORDINARY_Y)
    };
    let x = context.input(&owner, Input::X);
    let Some(&offset) = xs.get(direction) else {
        context.invalid(owner, direction);
    };
    context.set_position(&mut owner, slot, Axis::X, x.wrapping_add_signed(offset));
    let y = context.input(&owner, Input::Y);
    let y = y
        .wrapping_add_signed(ys[direction])
        .wrapping_sub(u16::from(height));
    context.set_position(&mut owner, slot, Axis::Y, y);
    (owner, true)
}

const ORDINARY_X: [i16; 10] = [2, 13, 11, 13, 2, -5, -14, -11, -19, -2];
const ORDINARY_Y: [i16; 10] = [-8, -13, 1, 4, 13, 13, 4, 1, -19, -8];
const RUNNING_X: [i16; 10] = [2, 15, 15, 13, 2, -5, -13, -13, -15, -2];
const RUNNING_Y: [i16; 10] = [-8, -16, -2, 1, 13, 13, 1, -2, -16, -8];
