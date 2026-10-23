//! Complete missile creation with explicit owner and live selection decisions.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weapon {
    Missile,
    SuperMissile,
}
#[derive(Clone, Copy)]
pub enum Control {
    Count,
    Cooldown,
    Protection,
}
pub trait Context<O> {
    type SoundKey;
    type Kind: Copy;
    fn shoot(&self, owner: &O) -> bool;
    fn weapon(&self, owner: &O) -> Weapon;
    fn control(&self, owner: &O, field: Control) -> u16;
    fn set_control(&mut self, owner: &mut O, field: Control, value: u16);
    fn damage(&self, owner: &O, slot: u16) -> u16;
    fn ammo(&self, owner: &O, weapon: Weapon) -> bool;
    fn place(&mut self, owner: O, slot: u16) -> (O, bool);
    fn spend(&mut self, owner: O, weapon: Weapon) -> O;
    fn trail(&mut self, owner: &mut O, slot: u16, value: u16);
    fn arm(&mut self, owner: &mut O, slot: u16) -> Self::SoundKey;
    fn cinematic(&self, owner: &O) -> bool;
    fn sound(&mut self, owner: O, key: Self::SoundKey) -> O;
    fn speed(&mut self, owner: O, slot: u16) -> O;
    fn initialize(&mut self, owner: O, slot: u16) -> O;
    fn kind(&self, owner: &O, slot: u16) -> Self::Kind;
    fn super_motion(&self, kind: Self::Kind) -> bool;
    fn motion(&mut self, owner: &mut O, slot: u16, weapon: Weapon);
    fn cooldown(&mut self, owner: O, kind: Self::Kind) -> (O, u16);
    fn automatic(&self, owner: &O) -> bool;
    fn clear_selection(&mut self, owner: &mut O, automatic: bool);
}
pub fn fire<O>(mut owner: O, context: &mut impl Context<O>) -> O {
    use Control::*;
    if !context.shoot(&owner) {
        return owner;
    }
    let capacity = if context.weapon(&owner) == Weapon::SuperMissile {
        4
    } else {
        5
    };
    if context.control(&owner, Count).wrapping_sub(capacity) & 0x8000 == 0
        || context.control(&owner, Cooldown) as u8 != 0
    {
        return owner;
    }
    context.set_control(&mut owner, Cooldown, 1);
    let count = context.control(&owner, Count).wrapping_add(1);
    context.set_control(&mut owner, Count, count);
    let slot = if context.ammo(&owner, context.weapon(&owner)) {
        (0..5)
            .map(|i| 2 * i)
            .find(|&slot| context.damage(&owner, slot) == 0)
    } else {
        None
    };
    let Some(slot) = slot else {
        let count = context.control(&owner, Count).wrapping_sub(1);
        context.set_control(&mut owner, Count, count);
        return owner;
    };
    let (next, placed) = context.place(owner, slot);
    owner = next;
    if !placed {
        return owner;
    }
    context.set_control(&mut owner, Protection, 20);
    let weapon = context.weapon(&owner);
    owner = context.spend(owner, weapon);
    context.trail(&mut owner, slot, 4);
    let sound = context.arm(&mut owner, slot);
    if !context.cinematic(&owner) {
        owner = context.sound(owner, sound);
    }
    owner = context.speed(owner, slot);
    owner = context.initialize(owner, slot);
    let kind = context.kind(&owner, slot);
    let weapon = if context.super_motion(kind) {
        Weapon::SuperMissile
    } else {
        Weapon::Missile
    };
    context.motion(&mut owner, slot, weapon);
    let (next, cooldown) = context.cooldown(owner, kind);
    owner = next;
    context.set_control(&mut owner, Cooldown, cooldown);
    if context.automatic(&owner) {
        context.clear_selection(&mut owner, true);
    } else if !context.ammo(&owner, context.weapon(&owner)) {
        context.clear_selection(&mut owner, false);
    }
    owner
}
