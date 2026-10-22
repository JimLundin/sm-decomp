//! Shared geometry and projectile eligibility for contact scans.
#![forbid(unsafe_code)]
use crate::rom::RomRecord;
use crate::types::Hitbox;
use core::mem::offset_of;
pub(crate) struct Bounds {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}
impl Bounds {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            left: x.wrapping_sub(width),
            right: x.wrapping_add(width),
            top: y.wrapping_sub(height),
            bottom: y.wrapping_add(height),
        }
    }
    pub fn hits(&self, x: u16, y: u16, record: &RomRecord, at: usize, touch: bool) -> bool {
        let edge = |field, origin: u16| record.word(at + field).wrapping_add(origin);
        if touch {
            (edge(offset_of!(Hitbox, left), x).wrapping_sub(self.right) as i16) < 0
                && (edge(offset_of!(Hitbox, right), x).wrapping_sub(self.left) as i16) >= 0
                && (edge(offset_of!(Hitbox, top), y).wrapping_sub(self.bottom) as i16) < 0
                && (edge(offset_of!(Hitbox, bottom), y).wrapping_sub(self.top) as i16) >= 0
        } else {
            (self.right.wrapping_sub(edge(offset_of!(Hitbox, left), x)) as i16) >= 0
                && (self.left.wrapping_sub(edge(offset_of!(Hitbox, right), x)) as i16) < 0
                && (self.bottom.wrapping_sub(edge(offset_of!(Hitbox, top), y)) as i16) >= 0
                && (self.top.wrapping_sub(edge(offset_of!(Hitbox, bottom), y)) as i16) < 0
        }
    }
}
pub(crate) fn axis(a: u16, b: u16, radius: u16, enemy_radius: u16) -> bool {
    let distance = (a.wrapping_sub(b) as i16).wrapping_abs() as u16;
    i32::from(distance) - i32::from(radius) < i32::from(enemy_radius)
}

pub(crate) fn is_shot(kind: u16) -> bool {
    kind != 0 && kind & 0xf00 != 0x300 && kind & 0xf00 != 0x500 && kind & 0xf00 < 0x700
}
