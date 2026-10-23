//! Ordered ice, sand and quicksand contact reactions.
#![forbid(unsafe_code)]
use super::super::fields::{Fields, Global};
use crate::game::mechanism_displacement::Displacement;

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

fn clear(c: &mut impl Fields, fields: &[Field]) {
    for &field in fields {
        c.set_global(Global::Contact(field), 0);
    }
}

fn surface(c: &mut impl Fields) {
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
    c.set_global(Global::Contact(BaseSpeed), 0);
    let suited = read(c, Equipment) & 0x20 != 0;
    if read(c, ReactionPoint) != 0 {
        return;
    }
    let direction = read(c, YDirection) & 3;
    match direction {
        0 | 3 => clear(c, &[YSpeedFraction, YSpeed]),
        1 => {
            let limit = if suited { 0x380 } else { 0x280 };
            if limit < read(c, YSpeedMiddle) {
                clear(c, &[YSpeedFraction, YSpeed]);
                c.set_global(Global::Contact(YSpeedMiddle), limit);
            }
        }
        _ => {}
    }
    clear(c, &[ExternalYFraction, ExternalY]);
    c.set_global(
        Global::Contact(ExternalYMiddle),
        if direction == 0 || direction == 3 {
            if suited { 0x100 } else { 0x120 }
        } else {
            0x200
        },
    );
    if direction == 2 {
        c.set_global(Global::Contact(AutoJump), 0);
    }
}

fn collision_surface(c: &mut impl Fields, displacement: &mut Displacement<'_>) -> u8 {
    use Field::*;
    if read(c, CollisionDirection) & 2 == 0 {
        return 0;
    }
    if !displacement.present() {
        c.missing_displacement();
        return 0;
    }
    match read(c, YDirection) & 3 {
        0 | 3 => {
            if read(c, CollisionDirection) & 15 != 3 {
                return 0;
            }
            if read(c, ContactDamage) == 1 {
                c.set_displacement(displacement, 0);
                return 1;
            }
            if (c.displacement(displacement) >> 8) as u16 > 0x30 {
                let value = ((c.displacement(displacement) as u32 & 0xff0000ff) | 0x3000) as i32;
                c.set_displacement(displacement, value);
            }
        }
        2 => {
            if read(c, ContactDamage) == 1 {
                c.set_displacement(displacement, 0);
                return 1;
            }
        }
        _ => return 0,
    }
    let visits = read(c, SandVisits).wrapping_add(1);
    c.set_global(Global::Contact(SandVisits), visits);
    0
}

pub fn setup(c: &mut impl Fields, displacement: &mut Displacement<'_>, kind: Setup) -> u8 {
    use Field::*;
    match kind {
        Setup::Ice => {
            let radius = read(c, YRadius);
            let bottom = read(c, YPosition).wrapping_add(radius).wrapping_sub(1);
            if matches!(bottom & 15, 7 | 15) {
                c.set_global(Global::Contact(Deceleration), 16);
            }
        }
        Setup::Surface => surface(c),
        Setup::Submerge | Setup::SlowFall | Setup::FastFall => {
            let fraction = match kind {
                Setup::Submerge => {
                    c.set_global(Global::Contact(AutoJump), 0);
                    0x2000
                }
                Setup::SlowFall => 0x4000,
                _ => 0xc000,
            };
            c.set_global(Global::Contact(ExternalYFraction), fraction);
            c.set_global(Global::Contact(ExternalY), 1);
        }
        Setup::StopVertical => clear(
            c,
            &[YSpeedFraction, YSpeed, YAccelerationFraction, YAcceleration],
        ),
        Setup::CollisionSurface => return collision_surface(c, displacement),
    }
    0
}

fn read(c: &impl Fields, field: Field) -> u16 {
    c.global(Global::Contact(field))
}
