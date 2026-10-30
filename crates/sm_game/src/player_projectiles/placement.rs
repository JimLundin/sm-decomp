//! Aim consumption and muzzle placement with one captured optional image.
use super::host::Host;
use crate::shot;
pub fn place(h: &mut impl Host, raw: u16) -> bool {
    let image = h.loaded_image();
    let pose_byte = |pose: u16, offset: usize| {
        image
            .as_ref()
            .expect("ROM must be loaded")
            .read(0x8b629 + 8 * usize::from(pose) + offset)
    };
    let mut pose = h.session().player.pose;
    let pending = h.session().projectiles.controls.pending_aim;
    let direction = if pending != 0 {
        h.session_mut().projectiles.controls.pending_aim = 0;
        pending as u8
    } else {
        let mut direction = pose_byte(pose, 3);
        if direction & 0xf0 != 0 {
            if direction == 0x10 {
                direction = pose_byte(h.session().player.history.last_different.pose, 3);
            }
            if direction & 0xf0 != 0 {
                h.session_mut().projectiles.count = h.session().projectiles.count.wrapping_sub(1);
                return false;
            }
            pose = h.session().player.pose;
        }
        direction
    };
    shot!(h, raw, direction = u16::from(direction));
    let height = pose_byte(pose, 4);
    let direction = usize::from(shot!(h, raw, direction) & 15);
    let (xs, ys) = if h.session().player.pose == 0x75
        || h.session().player.pose == 0x76
        || h.session().player.movement_and_facing >> 8 == 1
    {
        (&RUNNING_X, &RUNNING_Y)
    } else {
        (&ORDINARY_X, &ORDINARY_Y)
    };
    let x = h.session().player.x;
    let Some(&offset) = xs.get(direction) else {
        panic!("index {direction} outside 10");
    };
    shot!(h, raw, x = x.wrapping_add_signed(offset));
    let y = h
        .session()
        .player
        .y
        .wrapping_add_signed(ys[direction])
        .wrapping_sub(u16::from(height));
    shot!(h, raw, y = y);
    true
}
const ORDINARY_X: [i16; 10] = [2, 13, 11, 13, 2, -5, -14, -11, -19, -2];
const ORDINARY_Y: [i16; 10] = [-8, -13, 1, 4, 13, 13, 4, 1, -19, -8];
const RUNNING_X: [i16; 10] = [2, 15, 15, 13, 2, -5, -13, -13, -15, -2];
const RUNNING_Y: [i16; 10] = [-8, -16, -2, 1, 13, 13, 1, -2, -16, -8];
