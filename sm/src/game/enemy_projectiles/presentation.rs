//! Enemy projectile presentation selection; rendering stays an explicit edge.
#![forbid(unsafe_code)]
use super::access::enemy;
use super::host::Host;
use sm_game::geometry::Point;

pub enum Priority {
    Low,
    High,
}
pub(in crate::game) fn draw(priority: Priority, h: &mut impl Host) {
    let shake = screen_shake(h);
    let low = matches!(priority, Priority::Low);
    for slot in (0..18).rev() {
        if enemy!(h, slot, definition) != 0 && (enemy!(h, slot, properties) & 0x1000 != 0) == low {
            draw_one((slot * 2) as u16, shake, h);
        }
    }
}
pub(super) fn draw_one(handle: u16, shake: Point, h: &mut impl Host) {
    let slot = (handle >> 1) as isize;
    let map = enemy!(h, slot, spritemap);
    let tile = u16::from(enemy!(h, slot, graphics) as u8);
    let graphics = enemy!(h, slot, graphics) & 0xff00;
    let x = shake
        .x
        .wrapping_add(enemy!(h, slot, x))
        .wrapping_sub(h.inputs().camera_x);
    if (i32::from(x) + 128) & 0xfe00 == 0 {
        let y = shake
            .y
            .wrapping_add(enemy!(h, slot, y))
            .wrapping_sub(h.inputs().camera_y);
        if y & 0xff00 != 0 {
            if (i32::from(y) + 128) & 0xfe00 == 0 {
                h.draw_spritemap(true, 0x8d, map, x, y, tile, graphics);
            }
        } else {
            h.draw_spritemap(false, 0x8d, map, x, y, tile, graphics);
        }
    }
}
pub(super) fn screen_shake(h: &mut impl Host) -> Point {
    if h.session().feedback.earthquake_timer != 0
        && h.inputs().frozen == 0
        && (i32::from(h.session().feedback.earthquake_kind) - 36) & 0x8000 != 0
    {
        let index =
            isize::try_from(4u16.wrapping_mul(h.session().feedback.earthquake_kind) >> 1).unwrap();
        let negative = h.session().feedback.earthquake_timer & 2 != 0;
        let x = h.screen_shake_word(index);
        let y = h.screen_shake_word(index + 1);
        Point {
            x: if negative { x.wrapping_neg() } else { x },
            y: if negative { y.wrapping_neg() } else { y },
        }
    } else {
        Point { x: 0, y: 0 }
    }
}
