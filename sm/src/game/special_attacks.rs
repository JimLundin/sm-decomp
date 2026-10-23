//! Complete special-beam creation, motion and retirement over an explicit owner.
#![forbid(unsafe_code)]
use crate::game::projectile_initialization::Initialization;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Wave,
    Ice,
    IceReleased,
    Plasma,
    Spazer,
    SpazerReleased,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Attack {
    Wave,
    Ice,
    Spazer,
    Plasma,
}
#[derive(Clone, Copy)]
pub enum Field {
    Type,
    Direction,
    X,
    Y,
    XFraction,
    YFraction,
    XSpeed,
    YSpeed,
    Variable,
    Timer,
    Auxiliary,
}
#[derive(Clone, Copy)]
pub enum Control {
    Selected,
    Hud,
    AutoCancel,
    Count,
    Cooldown,
    Charge,
    OrbitStep,
}
#[derive(Clone, Copy)]
pub enum Input {
    SamusX,
    SamusY,
    CameraX,
    CameraY,
    Beams,
    Facing,
    PowerBombs,
}
pub trait Context<O> {
    fn read(&self, owner: &O, slot: u16, field: Field) -> u16;
    fn low_byte(&self, owner: &O, slot: u16, field: Field) -> u8;
    /// Legacy integration words: native RAM lanes, or logical words in an owned model.
    fn native_word(&self, owner: &O, slot: u16, field: Field) -> u16;
    fn write_native(&mut self, owner: &mut O, slot: u16, field: Field, value: u16);
    fn write(&mut self, owner: &mut O, slot: u16, field: Field, value: u16);
    fn motion_is(&self, owner: &O, slot: u16, kind: Kind) -> bool;
    fn motion(&mut self, owner: &mut O, slot: u16, kind: Kind);
    fn control(&self, owner: &O, field: Control) -> u16;
    fn set_control(&mut self, owner: &mut O, field: Control, value: u16);
    fn input(&self, owner: &O, field: Input) -> u16;
    fn initialize(&mut self, owner: O, slot: u16, kind: Initialization) -> O;
    fn trail(&mut self, owner: O, slot: u16) -> O;
    fn clear(&mut self, owner: O, slot: u16) -> O;
    fn sound(&mut self, owner: O, sound: u16) -> O;
    fn power_bombs(&mut self, owner: O, value: u16) -> O;
    fn cooldown(&mut self, owner: O, index: usize) -> (O, u16);
    fn point(&mut self, owner: O, angle: u16, radius: u16) -> (O, (u16, u16));
    fn invalid(&mut self, owner: O, index: usize, length: usize) -> !;
    fn invalid_plasma_phase(&mut self, owner: O) -> !;
    fn resume(&mut self, owner: O, selected: u16, kind: Kind, original: u16, previous_x: u16) -> O
    where
        Self: Sized,
    {
        resume(owner, selected, kind, original, previous_x, self)
    }
}
pub fn fire<O>(mut o: O, c: &mut impl Context<O>) -> (O, u8) {
    if c.control(&o, Control::Hud) != 3 {
        return (o, 0);
    }
    let selector = usize::from(c.input(&o, Input::Beams) & 15);
    let costs = [0i16, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0];
    let Some(&cost) = costs.get(selector) else {
        c.invalid(o, selector, costs.len());
    };
    let remaining = (c.input(&o, Input::PowerBombs) as i16)
        .wrapping_sub(cost)
        .max(0) as u16;
    o = c.power_bombs(o, remaining);
    let attack = match selector {
        1 => Some(Attack::Wave),
        2 => Some(Attack::Ice),
        4 => Some(Attack::Spazer),
        8 => Some(Attack::Plasma),
        _ => None,
    };
    let (mut o, result) = if let Some(attack) = attack {
        create(o, attack, c)
    } else {
        (o, 0)
    };
    if c.input(&o, Input::PowerBombs) == 0 {
        c.set_control(&mut o, Control::Hud, 0);
        c.set_control(&mut o, Control::AutoCancel, 0);
    }
    (o, result)
}
pub fn create<O>(mut o: O, attack: Attack, c: &mut impl Context<O>) -> (O, u8) {
    use Field::*;
    if (attack == Attack::Ice
        && (c.motion_is(&o, 0, Kind::IceReleased) || c.motion_is(&o, 0, Kind::Ice)))
        || (attack == Attack::Plasma && c.motion_is(&o, 0, Kind::Plasma))
    {
        return (o, 0);
    }
    for index in (0..4).rev() {
        let slot = 2 * index as u16;
        let initialization = match attack {
            Attack::Wave => {
                c.write(&mut o, slot, Timer, 4);
                let weapon = c.input(&o, Input::Beams) & 0x100f | 0x8010;
                c.write(&mut o, slot, Type, weapon);
                c.write(&mut o, slot, Direction, 0);
                c.motion(&mut o, slot, Kind::Wave);
                c.write(&mut o, slot, YSpeed, 600);
                for field in [XSpeed, Variable, XFraction, YFraction] {
                    c.write(&mut o, slot, field, 0);
                }
                let x = c
                    .input(&o, Input::SamusX)
                    .wrapping_add([128, 128, 0xff80, 0xff80][index]);
                c.write(&mut o, slot, X, x);
                let y = c
                    .input(&o, Input::SamusY)
                    .wrapping_add([128, 0xff80, 0xff80, 128][index]);
                c.write(&mut o, slot, Y, y);
                Initialization::SpecialAttack
            }
            Attack::Ice => {
                c.write(&mut o, slot, Timer, 4);
                let weapon = c.input(&o, Input::Beams) & 0x100f | 0x8010;
                c.write(&mut o, slot, Type, weapon);
                c.write(&mut o, slot, Direction, 0);
                c.motion(&mut o, slot, Kind::Ice);
                c.write(&mut o, slot, Variable, 64 * index as u16);
                c.write(&mut o, slot, YSpeed, 600);
                Initialization::Projectile
            }
            Attack::Spazer => {
                c.write(&mut o, slot, Timer, [0, 0, 4, 4][index]);
                c.write(&mut o, slot, Direction, 5);
                c.motion(&mut o, slot, Kind::Spazer);
                c.write(&mut o, slot, XSpeed, 40);
                c.write(&mut o, slot, YSpeed, [4, 0xfffc, 4, 0xfffc][index]);
                for field in [Variable, Auxiliary, XFraction, YFraction] {
                    c.write(&mut o, slot, field, 0);
                }
                if index >= 2 {
                    c.write(&mut o, slot, Type, 0x8024);
                    Initialization::Echo
                } else {
                    let weapon = c.input(&o, Input::Beams) & 0x100f | 0x8010;
                    c.write(&mut o, slot, Type, weapon);
                    Initialization::SpecialAttack
                }
            }
            Attack::Plasma => {
                let weapon = c.input(&o, Input::Beams) & 0x100f | 0x8010;
                c.write(&mut o, slot, Type, weapon);
                c.write(&mut o, slot, Direction, 0);
                c.motion(&mut o, slot, Kind::Plasma);
                c.write(&mut o, slot, Variable, 64 * index as u16);
                c.write(&mut o, slot, XSpeed, 40);
                c.write(&mut o, slot, YSpeed, 0);
                Initialization::SpecialAttack
            }
        };
        o = c.initialize(o, slot, initialization);
    }
    c.set_control(&mut o, Control::Count, 4);
    let index = usize::from(c.read(&o, 0, Type) & 0x3f);
    let cooldown;
    (o, cooldown) = c.cooldown(o, index);
    c.set_control(&mut o, Control::Cooldown, cooldown);
    let step = match attack {
        Attack::Wave => 4,
        Attack::Spazer => 0,
        _ => {
            if c.input(&o, Input::Facing) == 4 {
                0xfffc
            } else {
                4
            }
        }
    };
    c.set_control(&mut o, Control::OrbitStep, step);
    o = c.sound(
        o,
        match attack {
            Attack::Wave => 0x28,
            Attack::Ice => 0x23,
            Attack::Spazer => 0x25,
            Attack::Plasma => 0x27,
        },
    );
    (o, 1)
}
pub fn update<O>(mut o: O, mut selected: u16, kind: Kind, c: &mut impl Context<O>) -> O {
    use Field::*;
    let slot = selected & !1;
    if c.read(&o, slot, Direction) & 0xf0 != 0 {
        if matches!(kind, Kind::Wave | Kind::Ice) {
            o = c.sound(o, if kind == Kind::Wave { 0x29 } else { 0x24 });
        }
        if kind == Kind::Spazer {
            let paired = [4, 6, 0, 2].get(usize::from(slot >> 1)).copied();
            let paired = match paired {
                Some(v) => v,
                None => c.invalid(o, usize::from(slot >> 1), 4),
            };
            o = c.clear(o, selected);
            return c.clear(o, paired);
        }
        return c.clear(o, selected);
    }
    if kind == Kind::Plasma {
        hold(&mut o, c);
        let angle = c.read(&o, slot, Variable);
        let radius = c.read(&o, slot, XSpeed);
        let point;
        (o, point) = c.point(o, angle, radius);
        let x = point.0.wrapping_add(c.input(&o, Input::SamusX));
        c.write(&mut o, slot, X, x);
        let y = point.1.wrapping_add(c.input(&o, Input::SamusY));
        c.write(&mut o, slot, Y, y);
        orbit_step(&mut o, slot, c);
        return match c.read(&o, slot, YSpeed).wrapping_mul(2) {
            0 => plasma_radius(o, selected, false, c),
            2 => plasma_radius(o, selected, true, c),
            4 => depart_plasma(o, selected, c),
            _ => c.invalid_plasma_phase(o),
        };
    }
    if kind == Kind::Wave {
        let previous = c.read(&o, slot, YSpeed);
        c.write(&mut o, slot, YSpeed, previous.wrapping_sub(1));
        if previous == 1 || (previous as i16).wrapping_sub(1) < 0 {
            o = c.sound(o, 0x29);
            return c.clear(o, selected);
        }
    }
    let previous_x = c.read(&o, slot, XSpeed);
    let previous = c.read(&o, slot, Timer);
    c.write(&mut o, slot, Timer, previous.wrapping_sub(1));
    if previous == 1 {
        c.write(&mut o, slot, Timer, 4);
        o = c.trail(o, selected);
        selected = c.control(&o, Control::Selected);
    }
    c.resume(o, selected, kind, slot, previous_x)
}
pub fn resume<O>(
    mut o: O,
    selected: u16,
    kind: Kind,
    original: u16,
    previous_x: u16,
    c: &mut impl Context<O>,
) -> O {
    use Field::*;
    if kind == Kind::Wave {
        let target = c.input(&o, Input::SamusX);
        pursue(&mut o, original, X, XFraction, XSpeed, target, c);
        let target = c.input(&o, Input::SamusY);
        pursue(&mut o, original, Y, YFraction, Variable, target, c);
        if selected == 6 && (c.read(&o, 6, XSpeed) ^ previous_x) & 0x8000 != 0 {
            o = c.sound(o, 0x28);
        }
    } else if kind == Kind::SpazerReleased {
        let y = c.read(&o, selected, Y).wrapping_add(8);
        c.write(&mut o, selected, Y, y);
        if y.wrapping_sub(c.input(&o, Input::CameraY))
            .wrapping_sub(248)
            & 0x8000
            == 0
        {
            return c.clear(o, selected);
        }
    } else {
        let radius = if kind == Kind::Ice {
            32
        } else {
            c.read(&o, selected, XSpeed)
        };
        let angle = c.read(&o, selected, Variable);
        let point;
        (o, point) = c.point(o, angle, radius);
        let x = point.0.wrapping_add(c.input(&o, Input::SamusX));
        c.write(&mut o, selected, X, x);
        if kind == Kind::IceReleased
            && outside(
                c.read(&o, selected, X),
                c.input(&o, Input::CameraX),
                32,
                288,
            )
        {
            return c.clear(o, selected);
        }
        if kind == Kind::Spazer {
            o = advance_spazer(o, selected, point.1, c);
        } else {
            let y = point.1.wrapping_add(c.input(&o, Input::SamusY));
            c.write(&mut o, selected, Y, y);
            if kind == Kind::IceReleased
                && outside(
                    c.read(&o, selected, Y),
                    c.input(&o, Input::CameraY),
                    0xfff0,
                    256,
                )
            {
                return c.clear(o, selected);
            }
            orbit_step(&mut o, selected, c);
            if kind == Kind::IceReleased {
                let radius = u16::from((c.read(&o, selected, XSpeed) as u8).wrapping_add(8));
                c.write(&mut o, selected, XSpeed, radius);
            } else {
                let previous = c.read(&o, selected, YSpeed);
                c.write(&mut o, selected, YSpeed, previous.wrapping_sub(1));
                if previous == 1 {
                    c.motion(&mut o, selected, Kind::IceReleased);
                    c.write(&mut o, selected, XSpeed, 40);
                    o = c.sound(o, 0x24);
                }
            }
        }
    }
    hold(&mut o, c);
    o
}
pub fn plasma_radius<O>(mut o: O, selected: u16, contracting: bool, c: &mut impl Context<O>) -> O {
    let radius = c.read(&o, selected, Field::XSpeed) as u8;
    let radius = if contracting {
        radius.wrapping_sub(4)
    } else {
        radius.wrapping_add(4)
    };
    c.write(&mut o, selected, Field::XSpeed, u16::from(radius));
    if if contracting {
        radius < 45
    } else {
        radius >= 192
    } {
        c.write(
            &mut o,
            selected,
            Field::YSpeed,
            if contracting { 2 } else { 1 },
        );
    }
    o
}
pub fn depart_plasma<O>(mut o: O, selected: u16, c: &mut impl Context<O>) -> O {
    if outside(
        c.read(&o, selected, Field::X),
        c.input(&o, Input::CameraX),
        32,
        288,
    ) || outside(
        c.read(&o, selected, Field::Y),
        c.input(&o, Input::CameraY),
        0xfff0,
        256,
    ) {
        c.clear(o, selected)
    } else {
        let radius = u16::from((c.read(&o, selected, Field::XSpeed) as u8).wrapping_add(4));
        c.write(&mut o, selected, Field::XSpeed, radius);
        o
    }
}
fn advance_spazer<O>(mut o: O, selected: u16, y: u16, c: &mut impl Context<O>) -> O {
    use Field::*;
    let index = usize::from(selected >> 1);
    let phase = usize::from(c.read(&o, selected, Auxiliary) >> 1);
    if phase >= 3 {
        c.invalid(o, phase, 3);
    }
    let y = y
        .wrapping_add(c.input(&o, Input::SamusY))
        .wrapping_sub(if phase == 0 { 0 } else { 114 });
    c.write(&mut o, selected, Y, y);
    if phase != 0 && y.wrapping_sub(c.input(&o, Input::CameraY)).wrapping_sub(16) & 0x8000 != 0 {
        return finish_spazer(o, selected, c);
    }
    let step = c.low_byte(&o, selected, YSpeed);
    let angle = c.low_byte(&o, selected, Variable).wrapping_add(step);
    c.write(&mut o, selected, Variable, u16::from(angle));
    if phase == 0 {
        if angle == 128 {
            c.write(&mut o, selected, XSpeed, 160);
            let Some(&speed) = [2, 0xfffe, 2, 0xfffe].get(index) else {
                c.invalid(o, index, 4);
            };
            c.write(&mut o, selected, YSpeed, speed);
            c.write(&mut o, selected, Direction, 0);
            c.write(&mut o, selected, Auxiliary, 2);
        }
    } else if phase == 1 {
        let radius = c.read(&o, selected, XSpeed).wrapping_sub(5);
        c.write(&mut o, selected, XSpeed, radius);
        if radius == 0 {
            let Some(&speed) = [0xfffe, 2, 0xfffe, 2].get(index) else {
                c.invalid(o, index, 4);
            };
            c.write(&mut o, selected, YSpeed, speed);
            let angle = u16::from((c.read(&o, selected, Variable) as u8).wrapping_add(128));
            c.write(&mut o, selected, Variable, angle);
            c.write(&mut o, selected, Auxiliary, 4);
            if selected == 0 {
                o = c.sound(o, 0x26);
            }
        }
    } else {
        let radius = c.read(&o, selected, XSpeed).wrapping_add(5);
        c.write(&mut o, selected, XSpeed, radius);
        if radius.wrapping_sub(96) & 0x8000 == 0 {
            return finish_spazer(o, selected, c);
        }
    }
    o
}
fn finish_spazer<O>(mut o: O, selected: u16, c: &mut impl Context<O>) -> O {
    use Field::*;
    let index = usize::from(selected >> 1);
    // The compound assignment reads X before demanding the table entry.
    let x = c.read(&o, selected, X);
    let Some(&offset) = [16, 16, 0xfff0, 0xfff0].get(index) else {
        c.invalid(o, index, 4);
    };
    c.write(&mut o, selected, X, x.wrapping_add(offset));
    c.write(&mut o, selected, Direction, 5);
    c.write(&mut o, selected, Timer, 4);
    c.motion(&mut o, selected, Kind::SpazerReleased);
    if (selected as i16).wrapping_sub(4) < 0 {
        c.write(&mut o, selected, Type, 0x8024);
        o = c.initialize(o, selected, Initialization::Echo);
    }
    o
}

fn hold<O>(o: &mut O, c: &mut impl Context<O>) {
    c.set_control(o, Control::Cooldown, 2);
    c.set_control(o, Control::Charge, 0);
}
fn orbit_step<O>(o: &mut O, slot: u16, c: &mut impl Context<O>) {
    let angle =
        (c.control(o, Control::OrbitStep) as u8)
            .wrapping_add(c.read(o, slot, Field::Variable) as u8);
    c.write(o, slot, Field::Variable, u16::from(angle));
}
fn outside(position: u16, camera: u16, lower: u16, upper: u16) -> bool {
    let relative = position.wrapping_sub(camera);
    relative.wrapping_add(lower) & 0x8000 != 0 || relative.wrapping_sub(upper) & 0x8000 == 0
}
fn pursue<O>(
    o: &mut O,
    slot: u16,
    position: Field,
    fraction: Field,
    speed: Field,
    target: u16,
    c: &mut impl Context<O>,
) {
    let previous = c.read(o, slot, speed);
    if (target as i16).wrapping_sub(c.read(o, slot, position) as i16) < 0 {
        if previous.wrapping_add(2047) & 0x8000 == 0 {
            c.write(o, slot, speed, previous.wrapping_sub(64));
        }
    } else if previous.wrapping_sub(2048) & 0x8000 != 0 {
        c.write(o, slot, speed, previous.wrapping_add(64));
    }
    let speed = c.read(o, slot, speed);
    let low = u32::from(c.native_word(o, slot, fraction));
    let high = u32::from(c.native_word(o, slot, position));
    let next = (low | (high << 16)).wrapping_add((i32::from(speed as i16) << 8) as u32);
    c.write_native(o, slot, fraction, next as u16);
    c.write_native(o, slot, position, (next >> 16) as u16);
}
