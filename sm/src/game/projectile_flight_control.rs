//! Ordered beam and missile flight phases, independent of state storage.
#![forbid(unsafe_code)]
use crate::game::projectile_room::Axis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flight {
    NonWave,
    Wave,
    Missile,
    SuperMissile,
}
pub const AXES: [u8; 10] = [2, 3, 1, 3, 2, 2, 3, 1, 3, 2];
pub const X_ACCELERATION: [i16; 10] = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0];
pub const Y_ACCELERATION: [i16; 10] = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16];

pub trait Context<O> {
    type Direction: Copy;
    fn selected(&self, owner: &O) -> u16;
    fn retiring(&self, owner: &O, slot: u16) -> bool;
    fn extension(&self, owner: &O, slot: u16) -> bool;
    fn release(&mut self, owner: O, slot: u16) -> O;
    fn trail_ready(&mut self, owner: &mut O, slot: u16, interval: u16) -> bool;
    fn trail(&mut self, owner: O, slot: u16) -> O;
    fn advance(&mut self, owner: O, slot: u16, kind: Flight) -> O;
    fn direction(&self, owner: &O, slot: u16) -> Self::Direction;
    fn accelerate(&mut self, owner: O, slot: u16, direction: Self::Direction) -> O;
    fn axes(&mut self, owner: O, direction: Self::Direction) -> (O, u8);
    fn missile(&mut self, owner: O, slot: u16) -> O;
    fn collide(&mut self, owner: O, slot: u16, kind: Flight, axis: Axis) -> (O, u8);
    fn extension_collision(&mut self, owner: O, axis: Axis) -> O;
    fn cull(&mut self, owner: O) -> (O, bool);
}
pub fn clear_extensions<O>(mut owner: O, context: &mut impl Context<O>) -> O {
    for slot in (0..5).rev().map(|i| i * 2) {
        if context.extension(&owner, slot) {
            owner = context.release(owner, slot);
        }
    }
    owner
}
pub fn update<O>(
    mut owner: O,
    mut slot: u16,
    kind: Flight,
    interval: u16,
    context: &mut impl Context<O>,
) -> O {
    if context.retiring(&owner, slot) {
        owner = context.release(owner, slot);
        return if kind == Flight::SuperMissile {
            clear_extensions(owner, context)
        } else {
            owner
        };
    }
    if interval != 0 && context.trail_ready(&mut owner, slot, interval) {
        owner = context.trail(owner, slot);
        slot = context.selected(&owner);
    }
    context.advance(owner, slot, kind)
}
pub fn advance<O>(mut owner: O, slot: u16, kind: Flight, context: &mut impl Context<O>) -> O {
    let direction = context.direction(&owner, slot);
    if kind != Flight::SuperMissile {
        owner = context.accelerate(owner, slot, direction);
    }
    if matches!(kind, Flight::Missile | Flight::SuperMissile) {
        owner = context.missile(owner, slot);
    }
    let (owner, axes) = context.axes(owner, direction);
    let owner = collide(owner, axes, kind, context);
    let (owner, removed) = context.cull(owner);
    if kind == Flight::SuperMissile && removed {
        clear_extensions(owner, context)
    } else {
        owner
    }
}
pub fn collide<O>(mut owner: O, axes: u8, kind: Flight, context: &mut impl Context<O>) -> O {
    // Both axes retain the original selector across horizontal callbacks.
    let slot = context.selected(&owner);
    let collision = if kind == Flight::SuperMissile {
        Flight::Missile
    } else {
        kind
    };
    if axes & 1 != 0 {
        let (next, value) = context.collide(owner, slot, collision, Axis::X);
        owner = next;
        let hit = if matches!(kind, Flight::NonWave | Flight::Wave) {
            value & 1 != 0
        } else {
            value != 0
        };
        if kind == Flight::SuperMissile {
            owner = context.extension_collision(owner, Axis::X);
        }
        if hit || axes & 2 == 0 {
            return owner;
        }
    }
    // An empty mask still takes the original vertical path.
    owner = context.collide(owner, slot, collision, Axis::Y).0;
    if kind == Flight::SuperMissile {
        owner = context.extension_collision(owner, Axis::Y);
    }
    owner
}
pub fn update_extension<O>(mut owner: O, slot: u16, context: &mut impl Context<O>) -> O {
    if context.retiring(&owner, slot) {
        owner = context.release(owner, slot);
        owner = clear_extensions(owner, context);
    }
    owner
}
pub fn visible(position: u16, camera: u16) -> bool {
    let delta = position.wrapping_sub(camera);
    delta.wrapping_add(64) & 0x8000 == 0 && delta.wrapping_sub(320) & 0x8000 != 0
}
