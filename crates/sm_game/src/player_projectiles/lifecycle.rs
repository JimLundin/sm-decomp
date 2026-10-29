//! Ordered impacts and retirement over the live projectile session.
#![forbid(unsafe_code)]
use super::{Host, content, motion};
use crate::shot;
const DIRECTIONS: [(i8, i8); 10] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
];
pub fn impact(h: &mut impl Host, raw: u16) {
    let ballistic = shot!(h, raw, kind) & 0xf00 != 0;
    if ballistic && h.inputs().cinematic == 0 {
        h.sound(2, 6, 7);
    }
    // Sound may replace kind and content; family retains its original decision.
    let kind = shot!(h, raw, kind);
    shot!(
        h,
        raw,
        kind = kind & 0xf0ff | if ballistic { 0x800 } else { 0x700 }
    );
    let heavy = ballistic && kind & 0x200 != 0;
    let address = if !ballistic {
        0x93867b
    } else if heavy {
        0x938693
    } else {
        0x93867f
    };
    let cursor = content::table(h, address, 0);
    shot!(h, raw, cursor = cursor);
    if ballistic {
        if heavy {
            h.session_mut().feedback.earthquake_kind = 20;
            h.session_mut().feedback.earthquake_timer = 30;
        }
        if h.session().projectiles.controls.cooldown.wrapping_sub(21) & 0x8000 == 0 {
            h.session_mut().projectiles.controls.cooldown = 20;
        }
    } else {
        h.sound(2, 6, 12);
    }
    shot!(h, raw, remaining = 1);
    shot!(h, raw, damage = 8);
}
pub fn retire(h: &mut impl Host, raw: u16) {
    match shot!(h, raw, kind) & 0xf00 {
        0 => {
            let (dx, dy) = DIRECTIONS[usize::from(shot!(h, raw, direction) & 15)];
            if dx != 0 {
                let position = shot!(h, raw, x);
                let radius = shot!(h, raw, x_radius);
                shot!(h,raw,x = if dx>0 {position.wrapping_add(radius)} else {position.wrapping_sub(radius)}; SliceLe);
            }
            if dy != 0 {
                let position = shot!(h, raw, y);
                let radius = shot!(h, raw, y_radius);
                shot!(h,raw,y = if dy>0 {position.wrapping_add(radius)} else {position.wrapping_sub(radius)}; SliceLe);
            }
        }
        0x100 | 0x200 => {}
        _ => {
            motion::clear(h, raw);
            return;
        }
    }
    impact(h, raw);
    shot!(h, raw, motion = 0xb169);
}
