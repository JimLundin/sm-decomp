//! Complete movement/HUD weapon coordination; consumers return the current owner.
#![forbid(unsafe_code)]
use crate::game::weapon_selection::{self, Table};
#[derive(Clone, Copy)]
pub enum Input {
    Pose,
    Movement,
    PendingAim,
    Frozen,
    RunButton,
    Held,
    Selected,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Normal,
    Jump,
    Bomb,
    Grapple,
    Turning,
    Crouch,
    Draygon,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    BeamInput,
    Missile,
    Bomb,
    Grapple,
    Xray,
    UpdateProjectiles,
}
pub trait Context<O> {
    fn input(&self, owner: &O, input: Input) -> u16;
    fn cooldown(&self, owner: &O) -> u16;
    fn set_cooldown(&mut self, owner: &mut O, value: u16);
    fn grapple_active(&self, owner: &O) -> bool;
    fn switch(&mut self, owner: O) -> O;
    fn cancel_grapple(&mut self, owner: &mut O);
    fn run(&mut self, owner: O, action: Action) -> O;
    fn invalid(&mut self, owner: O, table: Table, index: usize, length: usize) -> !;
}
pub fn coordinate<O>(mut o: O, c: &mut impl Context<O>) -> O {
    if c.input(&o, Input::Pose) == 0 || c.input(&o, Input::Pose) == 0x9b {
        return c.run(o, Action::UpdateProjectiles);
    }
    let cooldown = if c.input(&o, Input::Frozen) != 0 {
        weapon_selection::cooldown_value(0, true)
    } else {
        weapon_selection::cooldown_value(c.cooldown(&o), false)
    };
    c.set_cooldown(&mut o, cooldown);
    o = c.switch(o);
    use Mode::*;
    let modes = [
        Normal, Normal, Normal, Jump, Bomb, Normal, Normal, Bomb, Bomb, Bomb, Jump, Grapple,
        Grapple, Jump, Turning, Crouch, Normal, Bomb, Bomb, Bomb, Jump, Normal, Grapple, Turning,
        Turning, Jump, Draygon, Jump,
    ];
    let index = usize::from(c.input(&o, Input::Movement));
    let Some(&mode) = modes.get(index) else {
        c.invalid(o, Table::Movement, index, modes.len())
    };
    o = dispatch(o, mode, c);
    if c.input(&o, Input::Frozen) == 0 {
        o = c.run(o, Action::UpdateProjectiles);
    }
    o
}
fn normal<O>(o: O, c: &mut impl Context<O>) -> O {
    let index = if c.grapple_active(&o) {
        4
    } else if c.input(&o, Input::Frozen) != 0 {
        5
    } else {
        c.input(&o, Input::Selected).wrapping_mul(2) >> 1
    };
    match index {
        0 | 3 => c.run(o, Action::BeamInput),
        1 | 2 => c.run(o, Action::Missile),
        4 => c.run(o, Action::Grapple),
        5 => xray(o, c),
        6 => turning(o, c),
        _ => c.invalid(o, Table::Normal, usize::from(index), 7),
    }
}
fn xray<O>(o: O, c: &mut impl Context<O>) -> O {
    let action = if c.input(&o, Input::RunButton) & c.input(&o, Input::Held) != 0 {
        Action::Xray
    } else {
        Action::BeamInput
    };
    c.run(o, action)
}
fn turning<O>(mut o: O, c: &mut impl Context<O>) -> O {
    if c.input(&o, Input::PendingAim) != 0 {
        return normal(o, c);
    }
    if c.grapple_active(&o) {
        c.cancel_grapple(&mut o);
    }
    o
}
fn jump<O>(mut o: O, c: &mut impl Context<O>) -> O {
    if c.grapple_active(&o) {
        c.cancel_grapple(&mut o);
        normal(o, c)
    } else {
        o
    }
}
fn crouch<O>(o: O, c: &mut impl Context<O>) -> O {
    if c.input(&o, Input::Pose).wrapping_sub(0xf1) & 0x8000 == 0 {
        return normal(o, c);
    }
    if c.input(&o, Input::Pose).wrapping_sub(0xdb) & 0x8000 == 0 {
        return o;
    }
    let index = usize::from(c.input(&o, Input::Pose).wrapping_sub(53));
    let choices = [
        false, false, true, true, true, true, false, false, true, true, true, true,
    ];
    let Some(&cancel) = choices.get(index) else {
        c.invalid(o, Table::Crouch, index, choices.len())
    };
    if cancel { jump(o, c) } else { normal(o, c) }
}
pub fn dispatch<O>(o: O, mode: Mode, c: &mut impl Context<O>) -> O {
    match mode {
        Mode::Normal => normal(o, c),
        Mode::Jump => jump(o, c),
        Mode::Bomb => c.run(o, Action::Bomb),
        Mode::Grapple => c.run(o, Action::Grapple),
        Mode::Turning => turning(o, c),
        Mode::Crouch => crouch(o, c),
        Mode::Draygon => {
            if c.input(&o, Input::Pose) == 0xdf {
                c.run(o, Action::Bomb)
            } else {
                normal(o, c)
            }
        }
    }
}
