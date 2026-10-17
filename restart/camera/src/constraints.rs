//! Selected bounded camera constraints from main's room.rs; no old dispatcher.
use crate::state::*;

fn product(a: u16, b: u16) -> u16 {
    u16::from(a as u8) * u16::from(b as u8)
}

fn neg(v: u16) -> bool {
    v & 0x8000 != 0
}

fn swap(v: u16) -> u16 {
    v.rotate_left(8)
}

pub(crate) fn horizontal(r: &mut State, right: bool) -> Result<(), &'static str> {
    let old = r.word(LAYER_X);
    let ideal = r.word(IDEAL_X);
    let mut pos = old;
    if neg(if right { ideal.wrapping_sub(pos) } else { pos.wrapping_sub(ideal) }) {
        pos = ideal;
        r.set_word(LAYER_X_FRACTION, 0);
    }
    let width = r.word(WIDTH);
    let base = product(r.word(LAYER_Y).wrapping_add(128) >> 8, width);
    if right {
        let max = swap(width.wrapping_sub(1));
        if max < pos {
            pos = max;
        } else if r.scroll(base.wrapping_add(1).wrapping_add(pos >> 8))? == 0 {
            let edge = pos & 0xff00;
            let moved = old.wrapping_sub(r.word(DISTANCE_X)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(edge)) { edge } else { moved };
        }
    } else if neg(pos) {
        pos = 0;
    } else if r.scroll(base.wrapping_add(pos >> 8))? == 0 {
        let edge = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(DISTANCE_X)).wrapping_add(2);
        pos = if moved >= edge { edge } else { moved };
    }
    r.set_word(LAYER_X, pos);
    Ok(())
}

pub(crate) fn vertical(r: &mut State, down: bool) -> Result<(), &'static str> {
    let old = r.word(LAYER_Y);
    let ideal = r.word(IDEAL_Y);
    let mut pos = old;
    let width = r.word(WIDTH);
    let row = product(pos >> 8, width).wrapping_add(r.word(LAYER_X).wrapping_add(128) >> 8);
    let margin = if down && r.scroll(row)? != 1 { 31 } else { 0 };
    if neg(if down { ideal.wrapping_sub(pos) } else { pos.wrapping_sub(ideal) }) {
        pos = ideal;
        r.set_word(LAYER_Y_FRACTION, 0);
    }
    if down {
        let max = margin + swap(r.word(HEIGHT).wrapping_sub(1));
        let mut bound = max;
        let clamp = if max < pos {
            true
        } else if r.scroll(width.wrapping_add(row))? == 0 {
            bound = margin + (pos & 0xff00);
            bound < pos
        } else {
            false
        };
        if clamp {
            let moved = old.wrapping_sub(r.word(DISTANCE_Y)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(bound)) { bound } else { moved };
        }
    } else if neg(pos) {
        pos = 0;
    } else {
        let row = product(pos >> 8, width).wrapping_add(r.word(LAYER_X).wrapping_add(128) >> 8);
        if r.scroll(row)? == 0 {
            let edge = (pos & 0xff00).wrapping_add(256);
            let moved = old.wrapping_add(r.word(DISTANCE_Y)).wrapping_add(2);
            pos = if moved >= edge { edge } else { moved };
        }
    }
    r.set_word(LAYER_Y, pos);
    Ok(())
}

pub(crate) fn automatic_x(r: &mut State) -> Result<(), &'static str> {
    if r.word(FROZEN) != 0 {
        return Ok(());
    }
    let old = r.word(LAYER_X);
    let width = r.word(WIDTH);
    let mut pos = if neg(old) { 0 } else { old };
    let max = swap(width.wrapping_sub(1));
    if max < pos {
        pos = max;
    }
    let base = product(r.word(LAYER_Y).wrapping_add(128) >> 8, width);
    let index = base.wrapping_add(pos >> 8);
    if r.scroll(index)? != 0 {
        if r.scroll(index.wrapping_add(1))? != 0 {
            r.set_word(LAYER_X, pos);
            return Ok(());
        }
        let boundary = pos & 0xff00;
        let moved = old.wrapping_sub(r.word(DISTANCE_X)).wrapping_sub(2);
        pos = if neg(moved.wrapping_sub(boundary)) {
            boundary
        } else if r.scroll(base.wrapping_add(moved >> 8))? != 0 {
            moved
        } else {
            (moved & 0xff00).wrapping_add(256)
        };
    } else {
        let boundary = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(DISTANCE_X)).wrapping_add(2);
        pos = if moved >= boundary {
            boundary
        } else if r.scroll(base.wrapping_add(u16::from(((moved >> 8) as u8).wrapping_add(1))))? != 0 {
            moved
        } else {
            moved & 0xff00
        };
    }
    r.set_word(LAYER_X, pos);
    Ok(())
}

pub(crate) fn automatic_y(r: &mut State) -> Result<(), &'static str> {
    if r.word(FROZEN) != 0 {
        return Ok(());
    }
    let old = r.word(LAYER_Y);
    let width = r.word(WIDTH);
    let x = r.word(LAYER_X).wrapping_add(128) >> 8;
    let initial = product(old >> 8, width).wrapping_add(x);
    let margin = if r.scroll(initial)? != 1 { 31 } else { 0 };
    let mut pos = if neg(old) { 0 } else { old };
    let max = margin + swap(r.word(HEIGHT).wrapping_sub(1));
    if max < pos {
        pos = max;
    }
    let index = product(pos >> 8, width).wrapping_add(x);
    if r.scroll(index)? == 0 {
        let boundary = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(DISTANCE_Y)).wrapping_add(2);
        pos = if moved >= boundary {
            boundary
        } else if r.scroll(product((moved >> 8).wrapping_add(1), width).wrapping_add(x))? != 0 {
            moved
        } else {
            moved & 0xff00
        };
    } else if r.scroll(width.wrapping_add(index))? == 0 {
        let boundary = margin + (pos & 0xff00);
        if boundary < pos {
            let moved = old.wrapping_sub(r.word(DISTANCE_Y)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(boundary)) {
                boundary
            } else if r.scroll(product(moved >> 8, width).wrapping_add(x))? != 0 {
                moved
            } else {
                (moved & 0xff00).wrapping_add(256)
            };
        }
    }
    r.set_word(LAYER_Y, pos);
    Ok(())
}
