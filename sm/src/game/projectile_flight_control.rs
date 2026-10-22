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

pub trait MissileContext<O> {
    type Record;
    fn activate(&mut self, owner: &mut O, slot: u16, variable: u16);
    fn acceleration(&mut self, owner: O, slot: u16) -> (O, Self::Record);
    fn add_acceleration(&mut self, owner: O, slot: u16, record: &Self::Record, axis: Axis) -> O;
    fn launch(&mut self, owner: O, slot: u16, speed: u16) -> O;
    fn heavy(&self, owner: &O, slot: u16) -> bool;
    fn spawn(&mut self, owner: O) -> O;
}
pub fn missile_acceleration_address(kind: u16, direction: u16) -> u16 {
    let table = if kind & 0x200 != 0 { 0xc32b } else { 0xc303 };
    table + 4 * (direction & 15)
}
pub fn accelerate_missile<O>(
    mut owner: O,
    slot: u16,
    variable: u16,
    context: &mut impl MissileContext<O>,
) -> O {
    if variable & 0xff00 != 0 {
        let (owner, record) = context.acceleration(owner, slot);
        let owner = context.add_acceleration(owner, slot, &record, Axis::X);
        context.add_acceleration(owner, slot, &record, Axis::Y)
    } else {
        let speed = variable + 0x100;
        context.activate(&mut owner, slot, speed);
        owner = context.launch(owner, slot, speed);
        if context.heavy(&owner, slot) {
            owner = context.spawn(owner);
        }
        owner
    }
}

pub trait ExtensionContext<O>: Context<O> {
    fn speed(&self, owner: &O, slot: u16, axis: Axis) -> i16;
    fn impacted(&self, owner: &O, slot: u16) -> bool;
    fn shift_extension(&mut self, owner: &mut O, parent: u16, child: u16, axis: Axis, delta: i16);
    fn select(&mut self, owner: &mut O, slot: u16);
}

pub trait CreationContext<O> {
    fn occupied(&self, owner: &O, slot: u16) -> bool;
    /// False means a compatibility call completed creation with the whole owner.
    fn prepare(&mut self, owner: O, slot: u16) -> (O, bool);
    fn place(&mut self, owner: O, slot: u16) -> O;
    fn initialize(&mut self, owner: O, slot: u16) -> O;
    fn arm(&mut self, owner: &mut O, slot: u16);
    fn link(&mut self, owner: O, slot: u16) -> O;
    fn count(&mut self, owner: &mut O);
}
pub fn spawn_extension<O>(owner: O, context: &mut impl CreationContext<O>) -> O {
    let Some(slot) = (0..5)
        .map(|i| 2 * i)
        .find(|&slot| !context.occupied(&owner, slot))
    else {
        return owner;
    };
    let (mut owner, prepared) = context.prepare(owner, slot);
    if prepared {
        // Rejected placement still initializes, links and counts the captured child.
        owner = context.place(owner, slot);
        owner = context.initialize(owner, slot);
        context.arm(&mut owner, slot);
        owner = context.link(owner, slot);
        context.count(&mut owner);
    }
    owner
}
/// Follow the parent at low speed; otherwise collide the linked child first.
pub fn contact_extension<O>(
    mut owner: O,
    parent: u16,
    child: u16,
    axis: Axis,
    context: &mut impl ExtensionContext<O>,
) -> O {
    let speed = context.speed(&owner, parent, axis);
    let magnitude = speed.unsigned_abs() & 0xff00;
    if magnitude.wrapping_sub(0xb00) & 0x8000 != 0 {
        if context.impacted(&owner, parent) {
            return context.release(owner, child);
        }
        context.shift_extension(&mut owner, parent, child, axis, 0);
    } else {
        let distance = (magnitude >> 8) as i16 - 10;
        let delta = if speed < 0 { distance } else { -distance };
        context.shift_extension(&mut owner, parent, child, axis, delta);
        context.select(&mut owner, child);
        owner = context.collide(owner, child, Flight::Missile, axis).0;
        // A failed collision keeps its own selector and writes; restore only on return.
        context.select(&mut owner, parent);
        if context.impacted(&owner, parent) {
            owner = context.release(owner, child);
        }
    }
    owner
}
