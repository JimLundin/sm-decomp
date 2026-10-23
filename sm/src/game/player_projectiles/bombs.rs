//! Bomb fuses, spread motion and contact over ordinary session fields.
#![forbid(unsafe_code)]
use super::{Host, bomb_terrain, content, motion};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Bomb,
    PowerBomb,
    Spread,
}
pub(super) const SPREAD_FRACTIONS: [u16; 5] = [0, 0, 0x8000, 0, 0];
pub(super) fn advance_fuse(h: &mut impl Host, power: bool) {
    let selected = h.session().projectiles.selected;
    let raw = h.session().projectiles.selected;
    let remaining = shot_native!(h, raw, variable);
    if remaining == 0 {
        if power && h.session().projectiles.power_bomb.flag == 0 {
            let live = h.session().projectiles.selected;
            motion::clear(h, live);
        }
        return;
    }
    let remaining = remaining.wrapping_sub(1);
    shot!(h,raw,variable = remaining; SliceNative);
    if remaining == 15 {
        let old = shot!(h, raw, cursor);
        shot!(h, raw, cursor = old.wrapping_add(28));
    } else if remaining == 0 {
        if power {
            h.session_mut().projectiles.power_bomb.x = shot!(h, raw, x);
            h.session_mut().projectiles.power_bomb.y = shot!(h, raw, y);
            h.enable_hdma();
            h.spawn_power_bomb();
            shot!(h,selected,variable = 0xffff; SliceNative);
        } else {
            h.sound(2, 6, 8);
            let cursor = content::table(h, 0x938683, 0);
            shot!(h, selected, cursor = cursor);
            shot!(h, selected, remaining = 1);
        }
    }
}
fn velocity(h: &impl Host, raw: u16) -> u32 {
    (u32::from(shot_native!(h, raw, y_speed)) << 16) | u32::from(shot_native!(h, raw, trail_time))
}
fn add_y(h: &mut impl Host, raw: u16, amount: u32) {
    let fraction = u32::from(shot_native!(h, raw, y_fraction));
    let high = u32::from(shot_native!(h, raw, y));
    let sum = (fraction | (high << 16)).wrapping_add(amount);
    shot!(h,raw,y_fraction = sum as u16; SliceNative);
    shot!(h,raw,y = (sum>>16) as u16; SliceNative);
}
fn horizontal(speed: u16) -> u32 {
    let magnitude = u32::from(speed & 0x7fff) << 8;
    if speed & 0x8000 != 0 {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }
}
fn add_x(h: &mut impl Host, raw: u16, amount: u32) {
    let fraction = u32::from(shot_native!(h, raw, x_fraction));
    let high = u32::from(shot_native!(h, raw, x));
    let sum = (fraction | (high << 16)).wrapping_add(amount);
    shot!(h,raw,x_fraction = sum as u16; SliceNative);
    shot!(h,raw,x = (sum>>16) as u16; SliceNative);
}
pub(super) fn bomb_update(h: &mut impl Host, raw: u16, kind: Kind) {
    if shot_native!(h, raw, direction) & 0xf0 != 0 {
        motion::clear(h, raw);
        return;
    }
    advance_fuse(h, kind == Kind::PowerBomb);
    if kind != Kind::Spread {
        bomb_terrain::explode(h, raw);
        return;
    }
    if shot_native!(h, raw, variable) != 0 {
        let gravity = (u32::from(h.session().player.contact.y_acceleration.whole) << 16)
            | u32::from(h.session().player.contact.y_acceleration.fraction);
        // The low speed lane is read before the high lane, then published first.
        let low = u32::from(shot_native!(h, raw, trail_time));
        let high = u32::from(shot_native!(h, raw, y_speed));
        let sum = (low | (high << 16)).wrapping_add(gravity);
        shot!(h,raw,trail_time = sum as u16; SliceNative);
        shot!(h,raw,y_speed = (sum>>16) as u16; SliceNative);
        let step = velocity(h, raw);
        add_y(h, raw, step);
        if bomb_terrain::spread(h, raw) & 1 != 0 {
            let reverse = velocity(h, raw).wrapping_neg();
            add_y(h, raw, reverse);
            if shot_native!(h, raw, y_speed) & 0x8000 != 0 {
                shot!(h,raw,y_speed = 0; SliceNative);
                shot!(h,raw,y_radius = 0; SliceNative);
            } else {
                let index = usize::from(raw >> 1).wrapping_sub(5);
                let fraction = SPREAD_FRACTIONS[index];
                shot!(h,raw,trail_time = fraction; SliceNative);
                let speed = shot_native!(h, raw, auxiliary);
                shot!(h,raw,y_speed = speed; SliceNative);
            }
            return;
        }
        let step = horizontal(shot_native!(h, raw, x_speed));
        add_x(h, raw, step);
    }
    if bomb_terrain::spread(h, raw) & 1 != 0 {
        let speed = shot_native!(h, raw, x_speed);
        let live = shot_native!(h, raw, x_speed) ^ 0x8000;
        shot!(h,raw,x_speed = live; SliceNative);
        add_x(h, raw, horizontal(speed).wrapping_neg());
    }
}
