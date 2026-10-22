//! Ordered projectile impact and retirement, independent of state storage.
#![forbid(unsafe_code)]
use crate::game::projectile_room::Axis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Family {
    Beam,
    Ballistic,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Entry {
    BeamImpact,
    MissileImpact,
    HeavyImpact,
    BombExplosion,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Feedback {
    BeamImpact,
    MissileImpact,
    HeavyImpactShake,
}
pub trait Kind: Copy {
    fn impact_family(self) -> Family;
    fn uses_heavy_impact(self) -> bool;
    fn enter_impact(self, family: Family) -> Self;
}
pub trait State {
    type Kind: Kind;
    type Cursor: Copy;
    fn kind(&self, slot: usize) -> Self::Kind;
    fn set_kind(&mut self, slot: usize, kind: Self::Kind);
    fn cinematic(&self) -> bool;
    fn set_cursor(&mut self, slot: usize, cursor: Self::Cursor);
    fn cooldown(&self) -> u16;
    fn set_cooldown(&mut self, cooldown: u16);
    /// Set animation duration before damage, after all fallible lookups/effects.
    fn finish_impact(&mut self, slot: usize);
}
pub trait Definitions<O: State> {
    fn entry(&mut self, owner: O, entry: Entry) -> (O, O::Cursor);
    fn feedback(&mut self, owner: O, feedback: Feedback) -> O;
}
pub fn impact<O: State>(mut owner: O, slot: usize, definitions: &mut impl Definitions<O>) -> O {
    let family = owner.kind(slot).impact_family();
    if family == Family::Ballistic && !owner.cinematic() {
        owner = definitions.feedback(owner, Feedback::MissileImpact);
    }
    // Sound may replace the kind and cartridge. Sample the kind again before
    // publishing its transition, then resolve the entry against the live source.
    let kind = owner.kind(slot);
    owner.set_kind(slot, kind.enter_impact(family));
    let entry = match family {
        Family::Beam => Entry::BeamImpact,
        Family::Ballistic if kind.uses_heavy_impact() => Entry::HeavyImpact,
        Family::Ballistic => Entry::MissileImpact,
    };
    let (mut owner, cursor) = definitions.entry(owner, entry);
    owner.set_cursor(slot, cursor);
    match family {
        Family::Beam => owner = definitions.feedback(owner, Feedback::BeamImpact),
        Family::Ballistic => {
            if entry == Entry::HeavyImpact {
                owner = definitions.feedback(owner, Feedback::HeavyImpactShake);
            }
            if owner.cooldown().wrapping_sub(21) & 0x8000 == 0 {
                owner.set_cooldown(20);
            }
        }
    }
    owner.finish_impact(slot);
    owner
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Retirement {
    Beam,
    Missile,
    Release,
}
pub trait RetirementContext<O> {
    fn retirement(&self, owner: &O, slot: u16) -> Retirement;
    fn move_to_impact(&mut self, owner: O, slot: u16) -> O;
    fn impact(&mut self, owner: O, slot: u16) -> O;
    fn release(&mut self, owner: O, slot: u16) -> O;
    fn stop(&mut self, owner: &mut O, slot: u16);
}
pub fn retire<O>(mut owner: O, slot: u16, context: &mut impl RetirementContext<O>) -> O {
    match context.retirement(&owner, slot) {
        Retirement::Release => return context.release(owner, slot),
        Retirement::Beam => owner = context.move_to_impact(owner, slot),
        Retirement::Missile => {}
    }
    owner = context.impact(owner, slot);
    context.stop(&mut owner, slot);
    owner
}

pub const DIRECTIONS: [(i8, i8); 10] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
];
/// Diagonal impacts commit X before Y; untouched axes are never sampled.
pub fn impact_offsets((x, y): (i8, i8), mut shift: impl FnMut(Axis, i8)) {
    for (axis, step) in [(Axis::X, x), (Axis::Y, y)] {
        if step != 0 {
            shift(axis, step);
        }
    }
}
pub fn offset(position: u16, radius: u16, step: i8) -> u16 {
    if step > 0 {
        position.wrapping_add(radius)
    } else {
        position.wrapping_sub(radius)
    }
}
