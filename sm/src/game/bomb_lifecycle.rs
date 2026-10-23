//! Complete bomb fuse and motion transactions over an explicit projectile owner.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Bomb,
    PowerBomb,
    Spread,
}
#[derive(Clone, Copy)]
pub enum Field {
    Type,
    Direction,
    Variable,
    X,
    Y,
    XFraction,
    YFraction,
    XSpeed,
    YSpeed,
    SpeedFraction,
    YRadius,
    Auxiliary,
}
pub const SPREAD_FRACTIONS: [u16; 5] = [0, 0, 0x8000, 0, 0];
pub trait Fields<O> {
    fn selected(&self, owner: &O) -> u16;
    fn read(&self, owner: &O, slot: u16, field: Field) -> u16;
    fn write(&mut self, owner: &mut O, slot: u16, field: Field, value: u16);
    fn invalid(&mut self, owner: O, index: usize, length: usize) -> !;
}
pub trait Context<O>: Fields<O> {
    fn fuse(&mut self, owner: O, power: bool) -> O
    where
        Self: Sized,
    {
        advance_fuse(owner, power, self)
    }
    fn power_active(&self, owner: &O) -> bool;
    fn gravity(&self, owner: &O) -> u32;
    fn clear(&mut self, owner: O, slot: u16) -> O;
    /// Publish X before reading and publishing Y, including broad-index aliases.
    fn origin(&mut self, owner: O, slot: u16) -> O;
    fn enable_hdma(&mut self, owner: O) -> O;
    fn spawn_power_bomb(&mut self, owner: O) -> O;
    fn sound(&mut self, owner: O) -> O;
    fn flash_animation(&mut self, owner: O, slot: u16) -> O;
    fn explosion_animation(&mut self, owner: O, slot: u16) -> O;
    fn contact(&mut self, owner: O, slot: u16, spread: bool) -> (O, u8);
}
pub fn advance_fuse<O>(mut owner: O, power: bool, c: &mut impl Context<O>) -> O {
    use Field::*;
    let selected = c.selected(&owner);
    let slot = c.selected(&owner);
    let remaining = c.read(&owner, slot, Variable);
    if remaining == 0 {
        if power && !c.power_active(&owner) {
            let live = c.selected(&owner);
            owner = c.clear(owner, live);
        }
        return owner;
    }
    let remaining = remaining.wrapping_sub(1);
    c.write(&mut owner, slot, Variable, remaining);
    if remaining == 15 {
        owner = c.flash_animation(owner, slot);
    } else if remaining == 0 {
        if power {
            owner = c.origin(owner, slot);
            owner = c.enable_hdma(owner);
            owner = c.spawn_power_bomb(owner);
            c.write(&mut owner, selected, Variable, 0xffff);
        } else {
            owner = c.sound(owner);
            owner = c.explosion_animation(owner, selected);
        }
    }
    owner
}
fn pair<O>(owner: &O, slot: u16, high: Field, low: Field, c: &impl Fields<O>) -> u32 {
    let fraction = u32::from(c.read(owner, slot, low));
    fraction | (u32::from(c.read(owner, slot, high)) << 16)
}
fn velocity<O>(owner: &O, slot: u16, c: &impl Fields<O>) -> u32 {
    (u32::from(c.read(owner, slot, Field::YSpeed)) << 16)
        | u32::from(c.read(owner, slot, Field::SpeedFraction))
}
fn add<O>(owner: &mut O, slot: u16, high: Field, low: Field, delta: u32, c: &mut impl Fields<O>) {
    let sum = pair(owner, slot, high, low, c).wrapping_add(delta);
    c.write(owner, slot, low, sum as u16);
    c.write(owner, slot, high, (sum >> 16) as u16);
}
fn horizontal(speed: u16) -> u32 {
    let magnitude = u32::from(speed & 0x7fff) << 8;
    if speed & 0x8000 != 0 {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }
}
pub fn update<O>(mut owner: O, slot: u16, kind: Kind, c: &mut impl Context<O>) -> O {
    use Field::*;
    if c.read(&owner, slot, Direction) & 0xf0 != 0 {
        return c.clear(owner, slot);
    }
    owner = c.fuse(owner, kind == Kind::PowerBomb);
    if kind != Kind::Spread {
        return c.contact(owner, slot, false).0;
    }
    if c.read(&owner, slot, Variable) != 0 {
        let gravity = c.gravity(&owner);
        add(&mut owner, slot, YSpeed, SpeedFraction, gravity, c);
        let step = velocity(&owner, slot, c);
        add(&mut owner, slot, Y, YFraction, step, c);
        let hit;
        (owner, hit) = c.contact(owner, slot, true);
        if hit & 1 != 0 {
            let reverse = velocity(&owner, slot, c).wrapping_neg();
            add(&mut owner, slot, Y, YFraction, reverse, c);
            if c.read(&owner, slot, YSpeed) & 0x8000 != 0 {
                c.write(&mut owner, slot, YSpeed, 0);
                c.write(&mut owner, slot, YRadius, 0);
            } else {
                let index = usize::from(slot >> 1).wrapping_sub(5);
                let Some(&fraction) = SPREAD_FRACTIONS.get(index) else {
                    c.invalid(owner, index, SPREAD_FRACTIONS.len());
                };
                c.write(&mut owner, slot, SpeedFraction, fraction);
                let speed = c.read(&owner, slot, Auxiliary);
                c.write(&mut owner, slot, YSpeed, speed);
            }
            return owner;
        }
        let step = horizontal(c.read(&owner, slot, XSpeed));
        add(&mut owner, slot, X, XFraction, step, c);
    }
    let hit;
    (owner, hit) = c.contact(owner, slot, true);
    if hit & 1 != 0 {
        let speed = c.read(&owner, slot, XSpeed);
        let live = c.read(&owner, slot, XSpeed) ^ 0x8000;
        c.write(&mut owner, slot, XSpeed, live);
        add(
            &mut owner,
            slot,
            X,
            XFraction,
            horizontal(speed).wrapping_neg(),
            c,
        );
    }
    owner
}
