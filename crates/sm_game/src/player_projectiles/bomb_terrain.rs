//! Bomb cross/perimeter scans and spread contact with live host observations.
#![forbid(unsafe_code)]
use super::{Host, collision::Room};
use crate::projectile_block_reactions::{self as reactions};
use crate::projectile_room::{Axis, Scan};
use crate::{shot, shot_native};
fn locate(h: &mut impl Host, x: u16, y: u16) {
    h.session_mut().room.collision_cursor = 0;
    let cell = if x < 4096 && y < 4096 {
        (x >> 4) + (y >> 4) * u16::from(h.session().room.width as u8)
    } else {
        0xffff
    };
    h.session_mut().room.collision_cursor = cell;
}
fn redirect(h: &mut impl Host, axis: Axis) -> bool {
    let offset = i16::from(super::access::room_bts(h, h.session().room.collision_cursor) as i8);
    if offset == 0 {
        return false;
    }
    let delta = if axis == Axis::X {
        offset as u16
    } else {
        (offset as u16).wrapping_mul(h.session().room.width)
    };
    h.session_mut().room.collision_cursor = h.session().room.collision_cursor.wrapping_add(delta);
    true
}
fn special_header(h: &mut impl Host) -> u16 {
    let bts = super::access::room_bts(h, h.session().room.collision_cursor);
    if bts & 0x80 != 0 {
        let table = h.record(0x94, 0x9e44);
        let pointer = table
            .image
            .table_word(table.base, isize::try_from(h.session().room.area).unwrap());
        let table = h.record(0x94, pointer);
        table.image.table_word(table.base, isize::from(bts & 0x7f))
    } else {
        let table = h.record(0x94, 0x9da4);
        table.image.table_word(table.base, isize::from(bts))
    }
}
fn react(h: &mut impl Host, offset: u16) {
    h.session_mut().room.collision_cursor = offset >> 1;
    loop {
        let kind = super::access::room_level(h, h.session().room.collision_cursor) >> 12;
        match kind {
            5 | 13 => {
                if redirect(h, if kind == 5 { Axis::X } else { Axis::Y }) {
                    continue;
                }
            }
            4 | 7 | 12 | 15 => {
                let header = reactions::mechanism(
                    matches!(kind, 7 | 15),
                    matches!(kind, 12 | 15),
                    &mut Room(h),
                )
                .map(|m| m.header);
                if let Some(header) = header {
                    h.spawn_mechanism(header);
                }
            }
            11 => {
                let header = special_header(h);
                h.spawn_mechanism(header);
            }
            _ => {}
        }
        return;
    }
}
fn cross(h: &mut impl Host, raw: u16) {
    if shot_native!(h, raw, variable) != 0 {
        return;
    }
    let kind = shot_native!(h, raw, kind);
    if kind & 1 != 0 {
        return;
    }
    shot!(h,raw,kind = kind|1; SliceNative);
    if h.session().room.collision_cursor == 0xffff {
        return;
    }
    let center = h.session().room.collision_cursor.wrapping_mul(2);
    let live = h.session().room.collision_cursor.wrapping_mul(2);
    react(h, live);
    let width = h.session().room.width;
    let width2 = h.session().room.width;
    let mut at = center.wrapping_sub(width).wrapping_sub(width2);
    react(h, at);
    at = at
        .wrapping_add(h.session().room.width.wrapping_mul(2))
        .wrapping_add(2);
    react(h, at);
    at = at.wrapping_sub(4);
    react(h, at);
    at = at
        .wrapping_add(h.session().room.width.wrapping_mul(2))
        .wrapping_add(2);
    react(h, at);
}
fn edge(h: &mut impl Host, start: u16, count: u16, vertical: bool) {
    let product = u16::from(start as u8) * u16::from(h.session().room.width as u8);
    let mut at = product.wrapping_add(start >> 8).wrapping_mul(2);
    let mut remaining = count as i16;
    loop {
        react(h, at);
        at = at.wrapping_add(if vertical {
            h.session().room.width.wrapping_mul(2)
        } else {
            2
        });
        remaining = remaining.wrapping_sub(1);
        if remaining < 0 {
            return;
        }
    }
}
fn perimeter(h: &mut impl Host, raw: u16) {
    let variable = shot_native!(h, raw, variable);
    if variable != 0 {
        if variable & 0x8000 != 0 {
            shot!(h,raw,variable = 0; SliceNative);
        }
        return;
    }
    let horizontal = h.session().room.power_bomb_radius >> 8;
    let vertical = (3 * (h.session().room.power_bomb_radius >> 8)) >> 2;
    let left = h
        .session()
        .projectiles
        .power_bomb
        .x
        .wrapping_sub(h.session().room.power_bomb_radius >> 8) as i16;
    let left = (left.max(0) as u16) >> 4;
    let mut right = horizontal.wrapping_add(h.session().projectiles.power_bomb.x) >> 4;
    if right >= h.session().room.width {
        right = h.session().room.width.wrapping_sub(1);
    }
    let top = h.session().projectiles.power_bomb.y.wrapping_sub(vertical) as i16;
    let top = (top.max(0) as u16) >> 4;
    let mut bottom = vertical.wrapping_add(h.session().projectiles.power_bomb.y) >> 4;
    if bottom >= h.session().room.height {
        bottom = h.session().room.height.wrapping_sub(1);
    }
    let top_left = top | (left << 8);
    edge(h, top_left, right.wrapping_sub(left), false);
    edge(h, top_left, bottom.wrapping_sub(top), true);
    edge(h, bottom | (left << 8), right.wrapping_sub(left), false);
    edge(h, top | (right << 8), bottom.wrapping_sub(top), true);
}
const BEAM_CLASS: [u8; 12] = [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
const SHOT_CLASS: [i8; 9] = [0, 0, 0, 3, -1, 2, -1, -1, -1];
pub fn explode(h: &mut impl Host, raw: u16) {
    let kind = shot_native!(h, raw, kind);
    let class = if kind & 0xf00 != 0 {
        let index = usize::from((shot_native!(h, raw, kind) >> 8) & 15);
        let Some(&value) = SHOT_CLASS.get(index) else {
            let index = 2 * index + 1;
            panic!("index out of bounds: the len is 18 but the index is {index}");
        };
        value as u16
    } else {
        let index = usize::from(shot_native!(h, raw, kind) & 15);
        let Some(&value) = BEAM_CLASS.get(index) else {
            let index = 3 * index + 2;
            panic!("index out of bounds: the len is 36 but the index is {index}");
        };
        u16::from(value)
    };
    let x = shot_native!(h, raw, x) as i16;
    if x < 0 {
        return;
    }
    let captured_x = shot_native!(h, raw, x);
    if ((x as u16 >> 8) as i16).wrapping_sub(h.session().room.scroll_width as i16) >= 0 {
        return;
    }
    let y = shot_native!(h, raw, y) as i16;
    if y < 0 {
        return;
    }
    let captured_y = shot_native!(h, raw, y);
    if ((y as u16 >> 8) as i16).wrapping_sub(h.session().room.scroll_height as i16) >= 0 {
        return;
    }
    locate(h, captured_x, captured_y);
    if class == 2 {
        cross(h, raw);
    } else {
        perimeter(h, raw);
    }
}
pub fn spread(h: &mut impl Host, raw: u16) -> u8 {
    let x = shot_native!(h, raw, x);
    let y = shot_native!(h, raw, y);
    locate(h, x, y);
    if shot_native!(h, raw, variable) == 0 {
        cross(h, raw);
        return 0;
    }
    if h.session().room.collision_cursor == 0xffff {
        return 1;
    }
    loop {
        let kind = super::access::room_level(h, h.session().room.collision_cursor) >> 12;
        let blocked = match kind {
            0 | 2 | 3 | 6 | 7 => false,
            1 => {
                let bts = super::access::room_bts(h, h.session().room.collision_cursor);
                if bts & 31 < 5 {
                    true
                } else {
                    reactions::surface(
                        Axis::X,
                        &mut Scan {
                            displacement: 0,
                            position: 0,
                            auxiliary: 0,
                            span: 0,
                            edge: 0,
                            missile: 0,
                            remaining: 0,
                            unblocked: 0,
                        },
                        &Room(h),
                    )
                }
            }
            5 | 13 => {
                if redirect(h, if kind == 5 { Axis::X } else { Axis::Y }) {
                    continue;
                }
                false
            }
            _ => true,
        };
        return u8::from(blocked);
    }
}
