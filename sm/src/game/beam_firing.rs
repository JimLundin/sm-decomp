//! Complete charge input and beam creation over an explicit owner.
#![forbid(unsafe_code)]
use crate::game::projectile_room::Axis;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Ordinary,
    Charged,
    Hyper,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue<S> {
    Fire(S),
    CancelCharge,
}
#[derive(Clone, Copy)]
pub enum Field {
    Damage,
    TrailTimer,
    XSpeed,
    YSpeed,
}
#[derive(Clone, Copy)]
pub enum Control {
    Count,
    Cooldown,
    Selected,
    PreviousCharge,
    Charge,
    Protection,
    ResumeSound,
    Glow,
}
#[derive(Clone, Copy)]
pub enum Input {
    Hyper,
    EquippedBeams,
    ShootButton,
    Held,
    Pressed,
    FilteredPressed,
    PendingAim,
}
pub trait Context<O> {
    type Sound;
    fn read(&self, o: &O, slot: u16, field: Field) -> u16;
    fn write(&mut self, o: &mut O, slot: u16, field: Field, value: u16);
    fn control(&self, o: &O, field: Control) -> u16;
    fn set_control(&mut self, o: &mut O, field: Control, value: u16);
    fn input(&self, o: &O, field: Input) -> u16;
    fn flare(&mut self, o: &mut O, index: usize, timer: bool, value: u16);
    fn wave(&self, o: &O, slot: u16) -> bool;
    fn is_beam(&self, o: &O, slot: u16) -> bool;
    fn place(&mut self, o: O, slot: u16) -> (O, bool);
    fn arm(&mut self, o: O, slot: u16, mode: Mode) -> (O, Self::Sound);
    fn sound(&mut self, o: O, cue: Cue<Self::Sound>) -> O;
    fn initialize(&mut self, o: O, slot: u16) -> O;
    fn cooldown(&mut self, o: O, slot: u16, mode: Mode) -> (O, u16);
    fn collide(&mut self, o: O, slot: u16, wave: bool) -> O;
    fn motion(&mut self, o: O, slot: u16, mode: Mode) -> O;
    fn launch(&mut self, o: O, slot: u16) -> O;
    fn special(&mut self, o: O) -> (O, u8);
    fn palette(&mut self, o: O) -> O;
    fn finish(&mut self, o: O, slot: u16, mode: Mode, wave: bool) -> O
    where
        Self: Sized,
    {
        finish(o, slot, mode, wave, self)
    }
    fn hyper(&mut self, o: O, slot: u16) -> O
    where
        Self: Sized,
    {
        prepare_hyper(o, slot, self)
    }
}
pub fn clear_animation<O>(o: &mut O, c: &mut impl Context<O>) {
    for timer in [false, true] {
        for i in 0..3 {
            c.flare(o, i, timer, 0);
        }
    }
}
pub fn update_input<O>(mut o: O, c: &mut impl Context<O>) -> O {
    use Control::*;
    let charge = c.control(&o, Charge);
    c.set_control(&mut o, PreviousCharge, charge);
    if c.input(&o, Input::Hyper) != 0 || c.input(&o, Input::EquippedBeams) & 0x1000 == 0 {
        if c.input(&o, Input::ShootButton) & c.input(&o, Input::Held) != 0 {
            o = fire(o, false, c);
        }
        return o;
    }
    if c.input(&o, Input::PendingAim) == 0 {
        if c.input(&o, Input::ShootButton) & c.input(&o, Input::Held) != 0 {
            if c.control(&o, Charge).wrapping_sub(120) & 0x8000 != 0 {
                let charge = c.control(&o, Charge).wrapping_add(1);
                c.set_control(&mut o, Charge, charge);
                if c.control(&o, Charge) == 1 {
                    clear_animation(&mut o, c);
                    o = fire(o, false, c);
                }
            } else {
                let (next, result) = c.special(o);
                o = next;
                if result & 1 != 0 {
                    c.set_control(&mut o, Charge, 0);
                    clear_animation(&mut o, c);
                    o = c.palette(o);
                }
            }
            return o;
        }
        if c.control(&o, Charge) == 0 {
            return o;
        }
    }
    let charged = c.control(&o, Charge).wrapping_sub(60) & 0x8000 == 0;
    c.set_control(&mut o, Charge, 0);
    clear_animation(&mut o, c);
    fire(o, charged, c)
}
pub fn fire<O>(o: O, charged: bool, c: &mut impl Context<O>) -> O {
    let mode = if charged {
        Mode::Charged
    } else if c.input(&o, Input::Hyper) != 0 {
        Mode::Hyper
    } else {
        Mode::Ordinary
    };
    fire_beam(o, mode, c)
}
pub fn fire_beam<O>(mut o: O, mode: Mode, c: &mut impl Context<O>) -> O {
    use Control::*;
    use Field::*;
    if c.control(&o, Count).wrapping_sub(5) & 0x8000 != 0 && c.control(&o, Cooldown) as u8 == 0 {
        c.set_control(&mut o, Cooldown, 1);
        let count = c.control(&o, Count).wrapping_add(1);
        c.set_control(&mut o, Count, count);
        let slot = (0..5)
            .map(|i| i * 2)
            .find(|&slot| c.read(&o, slot, Damage) == 0)
            .unwrap_or(8);
        let (next, placed) = c.place(o, slot);
        o = next;
        if placed {
            c.set_control(&mut o, Protection, 10);
            if mode != Mode::Hyper {
                c.write(&mut o, slot, TrailTimer, 4);
            }
            let (next, sound) = c.arm(o, slot, mode);
            o = c.sound(next, Cue::Fire(sound));
            c.set_control(&mut o, ResumeSound, 0);
            o = c.initialize(o, slot);
            if mode == Mode::Hyper {
                c.write(&mut o, slot, XSpeed, 0);
                c.write(&mut o, slot, YSpeed, 0);
                c.set_control(&mut o, Selected, slot);
                o = c.collide(o, slot, true);
                let selected = c.control(&o, Selected);
                o = c.hyper(o, selected);
                o = c.launch(o, selected);
                c.set_control(&mut o, Cooldown, 21);
                c.set_control(&mut o, Glow, 0x8014);
                for (i, value) in [29, 5, 5].into_iter().enumerate() {
                    c.flare(&mut o, i, false, value);
                }
                for i in 0..3 {
                    c.flare(&mut o, i, true, 3);
                }
                c.set_control(&mut o, Charge, 0x8000);
            } else {
                let wave = c.wave(&o, slot);
                let (next, cooldown) = c.cooldown(o, slot, mode);
                o = next;
                c.set_control(&mut o, Cooldown, cooldown);
                c.write(&mut o, slot, XSpeed, 0);
                c.write(&mut o, slot, YSpeed, 0);
                c.set_control(&mut o, Selected, slot);
                o = c.collide(o, slot, wave);
                let selected = if wave { slot } else { c.control(&o, Selected) };
                o = c.finish(o, selected, mode, wave);
                if mode == Mode::Charged {
                    c.set_control(&mut o, Glow, 4);
                }
            }
            return o;
        }
    }
    if mode != Mode::Hyper && c.control(&o, PreviousCharge).wrapping_sub(16) & 0x8000 == 0 {
        c.set_control(&mut o, ResumeSound, 0);
        o = c.sound(o, Cue::CancelCharge);
    }
    o
}
pub fn prepare_hyper<O>(mut o: O, slot: u16, c: &mut impl Context<O>) -> O {
    c.write(&mut o, slot, Field::Damage, 1000);
    c.motion(o, slot, Mode::Hyper)
}
pub fn finish<O>(o: O, slot: u16, mode: Mode, wave: bool, c: &mut impl Context<O>) -> O {
    if !wave && !c.is_beam(&o, slot) {
        return o;
    }
    let o = c.motion(o, slot, mode);
    c.launch(o, slot)
}
pub trait InitialCollision<O> {
    fn direction(&self, o: &O, caller: u16) -> u16;
    fn selected(&self, o: &O) -> u16;
    fn write_x_speed_native(&mut self, o: &mut O, slot: u16, value: u16);
    fn scan(&mut self, o: O, slot: u16, wave: bool, axis: Axis) -> (O, u8);
    fn invalid(&mut self, o: O, index: usize, length: usize) -> !;
}
pub fn initial_collision<O>(
    mut o: O,
    caller: u16,
    wave: bool,
    c: &mut impl InitialCollision<O>,
) -> O {
    let direction = c.direction(&o, caller) & 15;
    if direction >= 10 {
        c.invalid(o, usize::from(direction), 10);
    }
    let selected = c.selected(&o);
    match direction {
        0 | 4 | 5 | 9 => {
            o = c.scan(o, selected, wave, Axis::Y).0;
        }
        1 | 3 | 6 | 8 => {
            let (next, hit) = c.scan(o, selected, wave, Axis::X);
            o = next;
            if hit & 1 == 0 {
                o = c.scan(o, selected, wave, Axis::Y).0;
            }
        }
        2 => {
            o = c.scan(o, selected, wave, Axis::X).0;
        }
        7 => {
            c.write_x_speed_native(&mut o, selected, 0xffff);
            o = c.scan(o, selected, wave, Axis::X).0;
        }
        _ => unreachable!(),
    }
    o
}
