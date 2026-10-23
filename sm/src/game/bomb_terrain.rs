//! Explosion cross/perimeter and spread contact traversal with operation-owned bounds.
#![forbid(unsafe_code)]
use crate::game::bomb_lifecycle::{Field, Fields};
use crate::game::projectile_room::Axis;
#[derive(Clone, Copy)]
pub enum Input {
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
    Radius,
    OriginX,
    OriginY,
}
pub trait Context<O>: Fields<O> {
    type Header;
    fn input(&self, owner: &O, field: Input) -> u16;
    fn cursor(&self, owner: &O) -> u16;
    fn select(&mut self, owner: &mut O, block: u16);
    fn level(&mut self, owner: O) -> (O, u16);
    fn bts(&mut self, owner: O) -> (O, u8);
    fn slope(&mut self, owner: O) -> (O, bool);
    fn mechanism(&mut self, owner: O, bombable: bool, solid: bool) -> (O, Option<Self::Header>);
    fn special(&mut self, owner: O) -> (O, Self::Header);
    fn spawn(&mut self, owner: O, header: Self::Header) -> O;
}
fn locate<O>(owner: &mut O, x: u16, y: u16, c: &mut impl Context<O>) {
    c.select(owner, 0);
    let block = if x < 4096 && y < 4096 {
        (x >> 4) + (y >> 4) * u16::from(c.input(owner, Input::Width) as u8)
    } else {
        0xffff
    };
    c.select(owner, block);
}
fn redirect<O>(mut owner: O, axis: Axis, c: &mut impl Context<O>) -> (O, bool) {
    let bts;
    (owner, bts) = c.bts(owner);
    let offset = i16::from(bts as i8);
    if offset == 0 {
        return (owner, false);
    }
    let delta = if axis == Axis::X {
        offset as u16
    } else {
        (offset as u16).wrapping_mul(c.input(&owner, Input::Width))
    };
    let block = c.cursor(&owner).wrapping_add(delta);
    c.select(&mut owner, block);
    (owner, true)
}
fn react<O>(mut owner: O, offset: u16, c: &mut impl Context<O>) -> O {
    c.select(&mut owner, offset >> 1);
    loop {
        let level;
        (owner, level) = c.level(owner);
        let kind = level >> 12;
        match kind {
            5 | 13 => {
                let retry;
                (owner, retry) = redirect(owner, if kind == 5 { Axis::X } else { Axis::Y }, c);
                if retry {
                    continue;
                }
            }
            4 | 7 | 12 | 15 => {
                let header;
                (owner, header) =
                    c.mechanism(owner, matches!(kind, 7 | 15), matches!(kind, 12 | 15));
                if let Some(header) = header {
                    owner = c.spawn(owner, header);
                }
            }
            11 => {
                let header;
                (owner, header) = c.special(owner);
                owner = c.spawn(owner, header);
            }
            _ => {}
        }
        return owner;
    }
}
fn cross<O>(mut owner: O, slot: u16, c: &mut impl Context<O>) -> O {
    if c.read(&owner, slot, Field::Variable) != 0 {
        return owner;
    }
    let kind = c.read(&owner, slot, Field::Type);
    if kind & 1 != 0 {
        return owner;
    }
    c.write(&mut owner, slot, Field::Type, kind | 1);
    if c.cursor(&owner) == 0xffff {
        return owner;
    }
    let center = c.cursor(&owner).wrapping_mul(2);
    let live = c.cursor(&owner).wrapping_mul(2);
    owner = react(owner, live, c);
    let width = c.input(&owner, Input::Width);
    let width2 = c.input(&owner, Input::Width);
    let mut at = center.wrapping_sub(width).wrapping_sub(width2);
    owner = react(owner, at, c);
    at = at
        .wrapping_add(c.input(&owner, Input::Width).wrapping_mul(2))
        .wrapping_add(2);
    owner = react(owner, at, c);
    at = at.wrapping_sub(4);
    owner = react(owner, at, c);
    at = at
        .wrapping_add(c.input(&owner, Input::Width).wrapping_mul(2))
        .wrapping_add(2);
    react(owner, at, c)
}
fn edge<O>(mut owner: O, start: u16, count: u16, vertical: bool, c: &mut impl Context<O>) -> O {
    let product = u16::from(start as u8) * u16::from(c.input(&owner, Input::Width) as u8);
    let mut at = product.wrapping_add(start >> 8).wrapping_mul(2);
    let mut remaining = count as i16;
    loop {
        owner = react(owner, at, c);
        at = at.wrapping_add(if vertical {
            c.input(&owner, Input::Width).wrapping_mul(2)
        } else {
            2
        });
        remaining = remaining.wrapping_sub(1);
        if remaining < 0 {
            return owner;
        }
    }
}
fn perimeter<O>(mut owner: O, slot: u16, c: &mut impl Context<O>) -> O {
    let variable = c.read(&owner, slot, Field::Variable);
    if variable != 0 {
        if variable & 0x8000 != 0 {
            c.write(&mut owner, slot, Field::Variable, 0);
        }
        return owner;
    }
    let horizontal = c.input(&owner, Input::Radius) >> 8;
    let vertical = (3 * (c.input(&owner, Input::Radius) >> 8)) >> 2;
    let left = c
        .input(&owner, Input::OriginX)
        .wrapping_sub(c.input(&owner, Input::Radius) >> 8) as i16;
    let left = (left.max(0) as u16) >> 4;
    let mut right = horizontal.wrapping_add(c.input(&owner, Input::OriginX)) >> 4;
    if right >= c.input(&owner, Input::Width) {
        right = c.input(&owner, Input::Width).wrapping_sub(1);
    }
    let top = c.input(&owner, Input::OriginY).wrapping_sub(vertical) as i16;
    let top = (top.max(0) as u16) >> 4;
    let mut bottom = vertical.wrapping_add(c.input(&owner, Input::OriginY)) >> 4;
    if bottom >= c.input(&owner, Input::Height) {
        bottom = c.input(&owner, Input::Height).wrapping_sub(1);
    }
    let top_left = top | (left << 8);
    owner = edge(owner, top_left, right.wrapping_sub(left), false, c);
    owner = edge(owner, top_left, bottom.wrapping_sub(top), true, c);
    owner = edge(
        owner,
        bottom | (left << 8),
        right.wrapping_sub(left),
        false,
        c,
    );
    edge(owner, top | (right << 8), bottom.wrapping_sub(top), true, c)
}
const BEAM_CLASS: [u8; 12] = [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
const SHOT_CLASS: [i8; 9] = [0, 0, 0, 3, -1, 2, -1, -1, -1];
pub fn explode<O>(mut owner: O, slot: u16, c: &mut impl Context<O>) -> O {
    let kind = c.read(&owner, slot, Field::Type);
    let class = if kind & 0xf00 != 0 {
        let index = usize::from((c.read(&owner, slot, Field::Type) >> 8) & 15);
        let Some(&value) = SHOT_CLASS.get(index) else {
            c.invalid(owner, 2 * index + 1, 18);
        };
        value as u16
    } else {
        let index = usize::from(c.read(&owner, slot, Field::Type) & 15);
        let Some(&value) = BEAM_CLASS.get(index) else {
            c.invalid(owner, 3 * index + 2, 36);
        };
        u16::from(value)
    };
    let x = c.read(&owner, slot, Field::X) as i16;
    if x < 0 {
        return owner;
    }
    let captured_x = c.read(&owner, slot, Field::X);
    if ((x as u16 >> 8) as i16).wrapping_sub(c.input(&owner, Input::ScrollWidth) as i16) >= 0 {
        return owner;
    }
    let y = c.read(&owner, slot, Field::Y) as i16;
    if y < 0 {
        return owner;
    }
    let captured_y = c.read(&owner, slot, Field::Y);
    if ((y as u16 >> 8) as i16).wrapping_sub(c.input(&owner, Input::ScrollHeight) as i16) >= 0 {
        return owner;
    }
    locate(&mut owner, captured_x, captured_y, c);
    if class == 2 {
        cross(owner, slot, c)
    } else {
        perimeter(owner, slot, c)
    }
}
pub fn spread<O>(mut owner: O, slot: u16, c: &mut impl Context<O>) -> (O, u8) {
    let x = c.read(&owner, slot, Field::X);
    let y = c.read(&owner, slot, Field::Y);
    locate(&mut owner, x, y, c);
    if c.read(&owner, slot, Field::Variable) == 0 {
        return (cross(owner, slot, c), 0);
    }
    if c.cursor(&owner) == 0xffff {
        return (owner, 1);
    }
    loop {
        let level;
        (owner, level) = c.level(owner);
        let kind = level >> 12;
        let blocked = match kind {
            0 | 2 | 3 | 6 | 7 => false,
            1 => {
                let bts;
                (owner, bts) = c.bts(owner);
                if bts & 31 < 5 {
                    true
                } else {
                    let hit;
                    (owner, hit) = c.slope(owner);
                    hit
                }
            }
            5 | 13 => {
                let retry;
                (owner, retry) = redirect(owner, if kind == 5 { Axis::X } else { Axis::Y }, c);
                if retry {
                    continue;
                }
                false
            }
            _ => true,
        };
        return (owner, u8::from(blocked));
    }
}
