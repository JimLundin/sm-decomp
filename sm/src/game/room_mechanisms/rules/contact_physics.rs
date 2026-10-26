//! Ordered ice, sand and quicksand contact reactions.
#![forbid(unsafe_code)]
use super::super::host::{Contact as Displacement, Diagnostic, Host};

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

fn surface(c: &mut impl Host) {
    {
        c.session_mut().player.contact.momentum = 0;
        c.session_mut().player.contact.speed_boost = 0;
        c.session_mut().projectiles.controls.echo_sound = 0;
        c.session_mut().player.contact.extra_run.fraction = 0;
        c.session_mut().player.contact.extra_run.whole = 0;
    };
    c.session_mut().player.contact.base_run.fraction &= 0x7fff;
    c.session_mut().player.contact.base_run.whole = 0;
    let suited = c.session().inventory.equipment.active & 0x20 != 0;
    if c.session().player.contact.reaction_point != 0 {
        return;
    }
    let direction = c.session().player.contact.y_direction & 3;
    match direction {
        0 | 3 => {
            c.session_mut().player.contact.y_speed.fraction = 0;
            c.session_mut().player.contact.y_speed.whole = 0;
        }
        1 => {
            let limit = if suited { 0x380 } else { 0x280 };
            if limit < c.session().player.contact.y_speed.middle() {
                {
                    c.session_mut().player.contact.y_speed.fraction = 0;
                    c.session_mut().player.contact.y_speed.whole = 0;
                };
                c.session_mut().player.contact.y_speed.set_middle(limit);
            }
        }
        _ => {}
    }
    {
        c.session_mut().player.contact.external_y.fraction = 0;
        c.session_mut().player.contact.external_y.whole = 0;
    };
    c.session_mut()
        .player
        .contact
        .external_y
        .set_middle(if direction == 0 || direction == 3 {
            if suited { 0x100 } else { 0x120 }
        } else {
            0x200
        });
    if direction == 2 {
        c.session_mut().player.contact.auto_jump = 0;
    }
}

fn collision_surface(c: &mut impl Host, displacement: &mut Displacement<'_>) -> u8 {
    if c.session().player.contact.collision_direction & 2 == 0 {
        return 0;
    }
    if matches!(displacement, Displacement::Absent) {
        c.diagnostic(Diagnostic::MissingDisplacement);
        return 0;
    }
    match c.session().player.contact.y_direction & 3 {
        0 | 3 => {
            if c.session().player.contact.collision_direction & 15 != 3 {
                return 0;
            }
            if c.session().player.contact.contact_attack == 1 {
                set_displacement(c, displacement, 0);
                return 1;
            }
            if (displacement_value(c, displacement) >> 8) as u16 > 0x30 {
                let value =
                    ((displacement_value(c, displacement) as u32 & 0xff0000ff) | 0x3000) as i32;
                set_displacement(c, displacement, value);
            }
        }
        2 => {
            if c.session().player.contact.contact_attack == 1 {
                set_displacement(c, displacement, 0);
                return 1;
            }
        }
        _ => return 0,
    }
    let visits = c.session().player.contact.sand_visits.wrapping_add(1);
    c.session_mut().player.contact.sand_visits = visits;
    0
}

pub fn setup(c: &mut impl Host, displacement: &mut Displacement<'_>, kind: Setup) -> u8 {
    match kind {
        Setup::Ice => {
            let radius = c.session().player.contact.radius;
            let bottom = c.session().player.y.wrapping_add(radius).wrapping_sub(1);
            if matches!(bottom & 15, 7 | 15) {
                c.session_mut().player.contact.deceleration = 16;
            }
        }
        Setup::Surface => surface(c),
        Setup::Submerge | Setup::SlowFall | Setup::FastFall => {
            let fraction = match kind {
                Setup::Submerge => {
                    c.session_mut().player.contact.auto_jump = 0;
                    0x2000
                }
                Setup::SlowFall => 0x4000,
                _ => 0xc000,
            };
            c.session_mut().player.contact.external_y.fraction = fraction;
            c.session_mut().player.contact.external_y.whole = 1;
        }
        Setup::StopVertical => {
            c.session_mut().player.contact.y_speed.fraction = 0;
            c.session_mut().player.contact.y_speed.whole = 0;
            c.session_mut().player.contact.y_acceleration.fraction = 0;
            c.session_mut().player.contact.y_acceleration.whole = 0;
        }
        Setup::CollisionSurface => return collision_surface(c, displacement),
    }
    0
}

fn displacement_value(c: &impl Host, displacement: &Displacement<'_>) -> i32 {
    match displacement {
        Displacement::Absent => panic!("missing contact displacement"),
        Displacement::Local(value) => **value,
        Displacement::Imported(token) => c.contact_displacement(token),
    }
}
fn set_displacement(c: &mut impl Host, displacement: &mut Displacement<'_>, value: i32) {
    match displacement {
        Displacement::Absent => panic!("missing contact displacement"),
        Displacement::Local(target) => **target = value,
        Displacement::Imported(token) => c.set_contact_displacement(token, value),
    }
}
