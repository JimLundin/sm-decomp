//! Entrance conveyor installation and its conditional push on Samus.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use crate::progress as flags;
#[derive(Clone, Copy)]
pub enum Direction {
    Right,
    Left,
}

pub fn setup(c: &mut impl Host, slot: usize) -> u8 {
    let mut block = usize::from(mechanism!(c, slot, block));
    for _ in 0..56 {
        set_terrain_word(c, block, 0xff, ByteOrder::Little);
        block = usize::from((block as u16).wrapping_add(2));
    }
    0
}
pub fn activate(c: &mut impl Host, slot: usize, direction: Direction) {
    let origin = usize::from(mechanism!(c, slot, block));
    for index in 0..56 {
        let block = origin + 2 * index;
        set_terrain_word(c, block, 0x30ff, ByteOrder::Native);
        set_behavior_byte(
            c,
            (block) >> 1,
            match direction {
                Direction::Right => 8,
                Direction::Left => 9,
            },
        );
    }
}

pub fn carry(c: &mut impl Host, direction: Direction) {
    if (c.session().room.area != 3 || flags::boss_defeated(c, 1))
        && c.session().player.contact.y_speed.whole == 0
    {
        c.session_mut().player.contact.external_x.fraction = 0;
        c.session_mut().player.contact.external_x.whole = (match direction {
            Direction::Right => 2,
            Direction::Left => -2,
        }) as u16;
    }
    c.session_mut().player.contact.movement_speed_table = 0x9f49u16.wrapping_add(12);
}
