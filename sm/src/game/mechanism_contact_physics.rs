//! Ordered ice, sand and quicksand contact reactions.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Ice,
    Surface,
    Submerge,
    SlowFall,
    FastFall,
    StopVertical,
    CollisionSurface,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    YRadius,
    YPosition,
    Deceleration,
    Momentum,
    BoostCounter,
    EchoSound,
    ExtraRunFraction,
    ExtraRunSpeed,
    BaseSpeed,
    Equipment,
    ReactionPoint,
    YDirection,
    YSpeedFraction,
    YSpeed,
    YSpeedMiddle,
    ExternalYFraction,
    ExternalY,
    ExternalYMiddle,
    YAccelerationFraction,
    YAcceleration,
    AutoJump,
    CollisionDirection,
    ContactDamage,
    SandVisits,
}

pub trait Context {
    fn read(&self, field: Field) -> u16;
    fn write(&mut self, field: Field, value: u16);
    fn clear_base_fraction_sign(&mut self);
    fn has_displacement(&self) -> bool;
    fn displacement(&self) -> i32;
    fn set_displacement(&mut self, value: i32);
    fn missing_displacement(&mut self);
}

fn clear(c: &mut impl Context, fields: &[Field]) {
    for &field in fields {
        c.write(field, 0);
    }
}

fn surface(c: &mut impl Context) {
    use Field::*;
    clear(
        c,
        &[
            Momentum,
            BoostCounter,
            EchoSound,
            ExtraRunFraction,
            ExtraRunSpeed,
        ],
    );
    c.clear_base_fraction_sign();
    c.write(BaseSpeed, 0);
    let suited = c.read(Equipment) & 0x20 != 0;
    if c.read(ReactionPoint) != 0 {
        return;
    }
    let direction = c.read(YDirection) & 3;
    match direction {
        0 | 3 => clear(c, &[YSpeedFraction, YSpeed]),
        1 => {
            let limit = if suited { 0x380 } else { 0x280 };
            if limit < c.read(YSpeedMiddle) {
                clear(c, &[YSpeedFraction, YSpeed]);
                c.write(YSpeedMiddle, limit);
            }
        }
        _ => {}
    }
    clear(c, &[ExternalYFraction, ExternalY]);
    c.write(
        ExternalYMiddle,
        if direction == 0 || direction == 3 {
            if suited { 0x100 } else { 0x120 }
        } else {
            0x200
        },
    );
    if direction == 2 {
        c.write(AutoJump, 0);
    }
}

fn collision_surface(c: &mut impl Context) -> u8 {
    use Field::*;
    if c.read(CollisionDirection) & 2 == 0 {
        return 0;
    }
    if !c.has_displacement() {
        c.missing_displacement();
        return 0;
    }
    match c.read(YDirection) & 3 {
        0 | 3 => {
            if c.read(CollisionDirection) & 15 != 3 {
                return 0;
            }
            if c.read(ContactDamage) == 1 {
                c.set_displacement(0);
                return 1;
            }
            if (c.displacement() >> 8) as u16 > 0x30 {
                let value = ((c.displacement() as u32 & 0xff0000ff) | 0x3000) as i32;
                c.set_displacement(value);
            }
        }
        2 => {
            if c.read(ContactDamage) == 1 {
                c.set_displacement(0);
                return 1;
            }
        }
        _ => return 0,
    }
    let visits = c.read(SandVisits).wrapping_add(1);
    c.write(SandVisits, visits);
    0
}

pub fn setup(c: &mut impl Context, kind: Setup) -> u8 {
    use Field::*;
    match kind {
        Setup::Ice => {
            let radius = c.read(YRadius);
            let bottom = c.read(YPosition).wrapping_add(radius).wrapping_sub(1);
            if matches!(bottom & 15, 7 | 15) {
                c.write(Deceleration, 16);
            }
        }
        Setup::Surface => surface(c),
        Setup::Submerge | Setup::SlowFall | Setup::FastFall => {
            let fraction = match kind {
                Setup::Submerge => {
                    c.write(AutoJump, 0);
                    0x2000
                }
                Setup::SlowFall => 0x4000,
                _ => 0xc000,
            };
            c.write(ExternalYFraction, fraction);
            c.write(ExternalY, 1);
        }
        Setup::StopVertical => clear(
            c,
            &[YSpeedFraction, YSpeed, YAccelerationFraction, YAcceleration],
        ),
        Setup::CollisionSurface => return collision_surface(c),
    }
    0
}
