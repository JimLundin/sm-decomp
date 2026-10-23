//! Item selection and cancellation over an explicit live owner.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    Beam,
    Missiles,
    SuperMissiles,
    PowerBombs,
    Grapple,
    Xray,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Selection {
    Accepted,
    Unavailable,
}
#[derive(Clone, Copy)]
pub enum Input {
    CancelButton,
    SwitchButton,
    Pressed,
    Held,
}
#[derive(Clone, Copy)]
pub enum Field {
    Selected,
    Automatic,
    Changed,
    Charge,
}
#[derive(Clone, Copy, Debug)]
pub enum Table {
    Selection,
    Movement,
    Normal,
    Crouch,
}
pub trait Context<O> {
    fn input(&self, owner: &O, input: Input) -> u16;
    fn field(&self, owner: &O, field: Field) -> u16;
    fn write(&mut self, owner: &mut O, field: Field, value: u16);
    fn available(&self, owner: &O, choice: Choice) -> bool;
    fn grapple_inactive(&self, owner: &O) -> bool;
    fn clear_animation(&mut self, owner: &mut O);
    fn palette(&mut self, owner: O) -> O;
    fn return_grapple_inactive(&mut self, owner: O) -> O;
    fn invalid(&mut self, owner: O, table: Table, index: usize, length: usize) -> !;
}
pub fn select<O>(mut o: O, choice: Choice, c: &mut impl Context<O>) -> (O, Selection) {
    if !c.available(&o, choice) {
        return (o, Selection::Unavailable);
    }
    if choice == Choice::Grapple {
        if c.grapple_inactive(&o) {
            o = c.palette(o);
            c.write(&mut o, Field::Charge, 0);
            c.clear_animation(&mut o);
            o = c.return_grapple_inactive(o);
        }
    } else {
        c.write(&mut o, Field::Charge, 0);
        c.clear_animation(&mut o);
        o = c.palette(o);
    }
    (o, Selection::Accepted)
}
pub fn switch<O>(mut o: O, c: &mut impl Context<O>) -> O {
    use Field::*;
    let previous = c.field(&o, Selected);
    let mut automatic = false;
    let selection = if c.input(&o, Input::CancelButton) & c.input(&o, Input::Pressed) != 0 {
        c.write(&mut o, Automatic, 0);
        Some(0)
    } else {
        automatic = c.input(&o, Input::CancelButton) & c.input(&o, Input::Held) != 0;
        if c.input(&o, Input::SwitchButton) & c.input(&o, Input::Pressed) == 0 {
            None
        } else {
            let next = c.field(&o, Selected).wrapping_add(1);
            Some(if c.field(&o, Selected).wrapping_sub(5) & 0x8000 == 0 {
                0
            } else {
                next
            })
        }
    };
    if let Some(mut selected) = selection {
        c.write(&mut o, Selected, selected);
        loop {
            let choices = [
                Choice::Beam,
                Choice::Missiles,
                Choice::SuperMissiles,
                Choice::PowerBombs,
                Choice::Grapple,
                Choice::Xray,
            ];
            let Some(&choice) = choices.get(usize::from(selected)) else {
                c.invalid(o, Table::Selection, usize::from(selected), choices.len())
            };
            let (next, result) = select(o, choice, c);
            o = next;
            if result == Selection::Accepted {
                break;
            }
            selected = c.field(&o, Selected).wrapping_add(1);
            c.write(&mut o, Selected, selected);
            if selected.wrapping_sub(6) & 0x8000 == 0 {
                selected = 0;
                c.write(&mut o, Selected, 0);
            }
        }
        let value = if automatic { c.field(&o, Selected) } else { 0 };
        c.write(&mut o, Automatic, value);
    }
    let changed = if c.field(&o, Selected) == previous {
        let next = c.field(&o, Changed).wrapping_add(1);
        if c.field(&o, Changed).wrapping_sub(2) & 0x8000 == 0 {
            2
        } else {
            next
        }
    } else {
        1
    };
    c.write(&mut o, Changed, changed);
    o
}
/// Shared by generic native owners as well as the current RAM coordinator.
pub fn cooldown_value(value: u16, frozen: bool) -> u16 {
    if frozen {
        32
    } else if value == 0 || value & 0x8000 != 0 {
        0
    } else {
        let next = value.wrapping_sub(1);
        if next & 0x8000 != 0 { 0 } else { next }
    }
}
