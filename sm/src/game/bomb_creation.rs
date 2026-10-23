//! Complete morph-bomb creation with live owner reads at each consumer boundary.
#![forbid(unsafe_code)]
pub use crate::game::bomb_lifecycle::Kind;
#[derive(Clone, Copy)]
pub enum Input {
    Held,
    Pressed,
    FireButton,
    EquippedItems,
    PowerAmmo,
    PowerFlag,
    SamusX,
    SamusY,
}
#[derive(Clone, Copy)]
pub enum Control {
    Charge,
    BombCount,
    Cooldown,
    SpreadCharge,
    Hud,
    AutoHud,
}
#[derive(Clone, Copy)]
pub enum Field {
    Direction,
    X,
    Y,
    XFraction,
    YFraction,
    XSpeed,
    SpeedFraction,
    YSpeed,
    Auxiliary,
    Lifetime,
}
pub trait Context<O> {
    type CooldownKey: Copy;
    fn input(&self, o: &O, input: Input) -> u16;
    fn control(&self, o: &O, field: Control) -> u16;
    fn set_control(&mut self, o: &mut O, field: Control, value: u16);
    fn vacant(&self, o: &O, slot: u16) -> bool;
    fn is_spread(&self, o: &O, slot: u16) -> bool;
    fn write(&mut self, o: &mut O, slot: u16, field: Field, value: u16);
    fn ordinary_kind(&mut self, o: &mut O, slot: u16);
    fn spread_kind(&mut self, o: &mut O, slot: u16);
    fn power_kind(&mut self, o: &mut O, slot: u16) -> Self::CooldownKey;
    fn motion(&mut self, o: &mut O, slot: u16, kind: Kind);
    fn ordinary_cooldown(&self) -> Self::CooldownKey;
    fn spread_cooldown(&self, o: &O) -> Self::CooldownKey;
    fn lookup_cooldown(&mut self, o: O, key: Self::CooldownKey) -> (O, u16);
    fn clear_flare_animation(&mut self, o: &mut O);
    fn spend_power(&mut self, o: O) -> (O, bool);
    fn initialize(&mut self, o: O, slot: u16) -> O;
    fn cancel_sound(&mut self, o: O) -> O;
    fn palette(&mut self, o: O) -> O;
}
fn cancel_charge<O>(mut o: O, c: &mut impl Context<O>) -> O {
    if c.control(&o, Control::Charge) != 0 {
        o = c.cancel_sound(o);
        c.set_control(&mut o, Control::Charge, 0);
        c.clear_flare_animation(&mut o);
        o = c.palette(o);
    }
    o
}
pub fn reserve<O>(mut o: O, c: &mut impl Context<O>) -> (O, bool) {
    use Control::*;
    if c.input(&o, Input::FireButton) & c.input(&o, Input::Pressed) == 0
        || c.control(&o, BombCount) != 0
            && (c.control(&o, BombCount).wrapping_sub(5) & 0x8000 == 0
                || c.control(&o, Cooldown) as u8 != 0)
    {
        return (cancel_charge(o, c), false);
    }
    let value = c.control(&o, Cooldown).wrapping_add(1);
    c.set_control(&mut o, Cooldown, value);
    let value = c.control(&o, BombCount).wrapping_add(1);
    c.set_control(&mut o, BombCount, value);
    (o, true)
}
pub fn prepare<O>(mut o: O, c: &mut impl Context<O>) -> (O, bool) {
    use Control::*;
    if c.input(&o, Input::EquippedItems) & 0x1000 != 0 {
        if c.control(&o, Charge).wrapping_sub(60) & 0x8000 != 0 || c.control(&o, BombCount) != 0 {
            return reserve(o, c);
        }
        if c.input(&o, Input::Held) & 0x400 != 0 && c.control(&o, SpreadCharge) & 0xc0 != 0xc0 {
            let value = c.control(&o, SpreadCharge).wrapping_add(1);
            c.set_control(&mut o, SpreadCharge, value);
        } else {
            o = spread(o, c);
            o = c.palette(o);
            o = c.cancel_sound(o);
        }
    }
    (o, false)
}
fn apply_cooldown<O, C: Context<O>>(o: O, key: C::CooldownKey, c: &mut C) -> O {
    let (mut o, value) = c.lookup_cooldown(o, key);
    c.set_control(&mut o, Control::Cooldown, value);
    o
}
pub fn fire<O>(mut o: O, c: &mut impl Context<O>) -> O {
    use Control::*;
    if c.input(&o, Input::FireButton) & c.input(&o, Input::Held) == 0 {
        return cancel_charge(o, c);
    }
    let power = c.control(&o, Hud) == 3;
    if power && c.input(&o, Input::PowerFlag) & 0x8000 != 0 {
        return o;
    }
    let (next, reserved) = if power { reserve(o, c) } else { prepare(o, c) };
    o = next;
    if !reserved {
        return o;
    }
    if power {
        if c.input(&o, Input::PowerAmmo) == 0 {
            return o;
        }
        let (next, spent) = c.spend_power(o);
        o = next;
        if !spent {
            return o;
        }
    }
    let slot = (10..20)
        .step_by(2)
        .find(|&slot| c.vacant(&o, slot))
        .unwrap_or(18);
    let key = if power {
        c.power_kind(&mut o, slot)
    } else {
        c.ordinary_kind(&mut o, slot);
        c.ordinary_cooldown()
    };
    c.write(&mut o, slot, Field::Direction, 0);
    let x = c.input(&o, Input::SamusX);
    c.write(&mut o, slot, Field::X, x);
    let y = c.input(&o, Input::SamusY);
    c.write(&mut o, slot, Field::Y, y);
    c.write(&mut o, slot, Field::Lifetime, 60);
    o = c.initialize(o, slot);
    c.motion(
        &mut o,
        slot,
        if power { Kind::PowerBomb } else { Kind::Bomb },
    );
    o = apply_cooldown(o, key, c);
    if power {
        if c.control(&o, AutoHud) != 0 {
            c.set_control(&mut o, Hud, 0);
            c.set_control(&mut o, AutoHud, 0);
        } else if c.control(&o, Hud) == 3 && c.input(&o, Input::PowerAmmo) == 0 {
            c.set_control(&mut o, Hud, 0);
        }
    }
    o
}
const LIFETIMES: [u16; 5] = [120, 110, 100, 110, 120];
const X_SPEEDS: [u16; 5] = [0x8100, 0x8080, 0, 0x80, 0x100];
const Y_SPEEDS: [u16; 5] = [0, 1, 2, 1, 0];
pub fn spread<O>(mut o: O, c: &mut impl Context<O>) -> O {
    use Control::*;
    use Field::*;
    if c.is_spread(&o, 10) {
        return o;
    }
    for (pattern, slot) in (10..20).step_by(2).enumerate() {
        c.spread_kind(&mut o, slot);
        c.write(&mut o, slot, Direction, 0);
        c.motion(&mut o, slot, Kind::Spread);
        o = c.initialize(o, slot);
        let x = c.input(&o, Input::SamusX);
        c.write(&mut o, slot, X, x);
        c.write(&mut o, slot, XFraction, 0);
        let y = c.input(&o, Input::SamusY);
        c.write(&mut o, slot, Y, y);
        c.write(&mut o, slot, YFraction, 0);
        c.write(&mut o, slot, XSpeed, X_SPEEDS[pattern]);
        c.write(
            &mut o,
            slot,
            SpeedFraction,
            crate::game::bomb_lifecycle::SPREAD_FRACTIONS[pattern],
        );
        let speed = 0u16.wrapping_sub(Y_SPEEDS[pattern] + ((c.control(&o, SpreadCharge) >> 6) & 3));
        c.write(&mut o, slot, YSpeed, speed);
        c.write(&mut o, slot, Auxiliary, speed);
        c.write(&mut o, slot, Lifetime, LIFETIMES[pattern]);
    }
    c.set_control(&mut o, BombCount, 5);
    let key = c.spread_cooldown(&o);
    o = apply_cooldown(o, key, c);
    c.set_control(&mut o, SpreadCharge, 0);
    c.set_control(&mut o, Charge, 0);
    o
}
