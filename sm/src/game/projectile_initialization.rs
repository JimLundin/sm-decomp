//! Ordered weapon entry initialization, independent of state and content storage.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Initialization {
    Projectile,
    SuperMissile,
    Bomb,
    SpecialAttack,
    Echo,
}
#[derive(Clone, Copy)]
pub enum Field {
    Damage,
    XRadius,
    YRadius,
    Timer,
}
/// A captured launch record observes edits to its original allocation.
pub trait Launch {
    fn damage(&self) -> u16;
}
pub trait Context<O> {
    type Weapon;
    type Launch: Launch;
    type Cursor: Copy;
    type Frame;
    fn weapon(&mut self, owner: O, weapon: Self::Weapon) -> (O, Self::Launch);
    fn set(&mut self, owner: &mut O, field: Field, value: u16);
    fn invalid_damage(&mut self, owner: O) -> O;
    fn entry(
        &mut self,
        owner: O,
        launch: &Self::Launch,
        direction: Option<usize>,
    ) -> (O, Self::Cursor);
    fn set_cursor(&mut self, owner: &mut O, cursor: Self::Cursor);
    fn frame(&mut self, owner: O, cursor: Self::Cursor) -> (O, Self::Frame);
    fn radius(&mut self, owner: O, frame: &Self::Frame, field: Field) -> (O, u16);
}
pub fn initialize<O, C: Context<O>>(
    owner: O,
    weapon: C::Weapon,
    kind: Initialization,
    direction: Option<usize>,
    context: &mut C,
) -> O {
    let (mut owner, launch) = context.weapon(owner, weapon);
    let damage = launch.damage();
    if kind != Initialization::Projectile {
        context.set(&mut owner, Field::Damage, damage);
    }
    // Echo checks the live record after its first damage store.
    let checked_damage = if kind == Initialization::Echo {
        launch.damage()
    } else {
        damage
    };
    if checked_damage & 0x8000 != 0 {
        owner = context.invalid_damage(owner);
    }
    if kind == Initialization::Projectile {
        context.set(&mut owner, Field::Damage, launch.damage());
    }
    let (mut owner, cursor) = context.entry(owner, &launch, direction);
    context.set_cursor(&mut owner, cursor);
    if kind == Initialization::Projectile {
        let (next, frame) = context.frame(owner, cursor);
        owner = next;
        for field in [Field::XRadius, Field::YRadius] {
            let (next, radius) = context.radius(owner, &frame, field);
            owner = next;
            context.set(&mut owner, field, radius);
        }
    }
    context.set(&mut owner, Field::Timer, 1);
    owner
}
