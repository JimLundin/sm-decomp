//! Projectile terrain traversal on the existing session owner and reaction context.
#![forbid(unsafe_code)]
use super::{Host, motion::Flight};
use crate::projectile_block_reactions::{
    self as reactions, Definitions, Material, Progress, SlopeInputs,
};
use crate::projectile_room::{Axis, Scan};
use crate::{shot, shot_native};
use sm_data::cartridge::RomRecord;
// This is a borrow of the actual host, never a second owned state snapshot.
pub struct Room<'a, H: Host>(pub &'a mut H);
impl<H: Host> SlopeInputs for Room<'_, H> {
    fn block(&self) -> u16 {
        self.0.session().room.collision_cursor
    }
    fn selected(&self) -> u16 {
        self.0.session().projectiles.selected
    }
    fn width(&self) -> u16 {
        self.0.session().room.width
    }
    fn bts(&self, block: u16) -> u8 {
        super::access::room_bts(self.0, block)
    }
    fn position(&self, raw: u16, axis: Axis) -> u16 {
        let h = &*self.0;
        match axis {
            Axis::X => shot!(h, raw, x),
            Axis::Y => shot!(h, raw, y),
        }
    }
    fn radius(&self, raw: u16, axis: Axis) -> u16 {
        let h = &*self.0;
        match axis {
            Axis::X => shot!(h, raw, x_radius),
            Axis::Y => shot!(h, raw, y_radius),
        }
    }
    fn square_position(&self, raw: u16, axis: Axis) -> u16 {
        let h = &*self.0;
        match axis {
            Axis::X => shot_native!(h, raw, x),
            Axis::Y => shot_native!(h, raw, y),
        }
    }
    fn square_radius(&self, raw: u16, axis: Axis) -> u16 {
        let h = &*self.0;
        match axis {
            Axis::X => shot_native!(h, raw, x_radius),
            Axis::Y => shot_native!(h, raw, y_radius),
        }
    }
    fn square_probe(&self, axis: Axis) {
        self.0.probe(if axis == Axis::X {
            "square-slope.shot.horizontal"
        } else {
            "square-slope.shot.vertical"
        });
    }
    fn quadrant(&self, index: usize) -> bool {
        crate::projectile_geometry::SLOPE_QUADRANTS[index] != 0
    }
    fn height(&self, index: usize) -> u8 {
        crate::projectile_geometry::SLOPE_HEIGHTS[index]
    }
}
impl<H: Host> Room<'_, H> {
    fn move_by(&mut self, raw: u16, axis: Axis, amount: i32) {
        let h = &mut *self.0;
        match axis {
            Axis::X => {
                let low = u32::from(shot_native!(h, raw, x_fraction));
                let high = u32::from(shot_native!(h, raw, x));
                let sum = (low | (high << 16)).wrapping_add(amount as u32);
                shot!(h,raw,x_fraction = sum as u16; SliceNative);
                shot!(h,raw,x = (sum>>16) as u16; SliceNative);
            }
            Axis::Y => {
                let low = u32::from(shot_native!(h, raw, y_fraction));
                let high = u32::from(shot_native!(h, raw, y));
                let sum = (low | (high << 16)).wrapping_add(amount as u32);
                shot!(h,raw,y_fraction = sum as u16; SliceNative);
                shot!(h,raw,y = (sum>>16) as u16; SliceNative);
            }
        }
        if amount != 0 {
            h.probe("player_projectiles.motion.applied");
        }
    }
    fn scrolls(&self, axis: Axis) -> u16 {
        match axis {
            Axis::X => self.0.session().room.scroll_width,
            Axis::Y => self.0.session().room.scroll_height,
        }
    }
}
pub fn scan(h: &mut impl Host, raw: u16, flight: Flight, axis: Axis) -> u8 {
    let missile = matches!(flight, Flight::Missile | Flight::SuperMissile);
    let wave = flight == Flight::Wave;
    let amount = i32::from(match axis {
        Axis::X => shot_native!(h, raw, x_speed),
        Axis::Y => shot_native!(h, raw, y_speed),
    } as i16)
        << 8;
    let mut room = Room(h);
    let span = if missile {
        0
    } else {
        let other = axis.other();
        let first = room
            .square_position(raw, other)
            .wrapping_sub(room.square_radius(raw, other))
            & 0xfff0;
        room.square_radius(raw, other)
            .wrapping_add(room.square_position(raw, other))
            .wrapping_sub(1)
            .wrapping_sub(first)
            >> 4
    };
    let row = if axis == Axis::X {
        let y = room.square_position(raw, Axis::Y);
        let edge = if missile {
            y
        } else {
            y.wrapping_sub(room.square_radius(raw, Axis::Y))
        };
        u16::from((edge >> 4) as u8) * u16::from(room.0.session().room.width as u8)
    } else {
        0
    };
    // Fraction is published before position, before later projectile/room reads.
    room.move_by(raw, axis, amount);
    if missile && axis == Axis::X {
        let h = &*room.0;
        let _fraction = shot_native!(h, raw, x_fraction);
    }
    let position = room.square_position(raw, axis);
    let edge = if missile {
        position
    } else if amount < 0 {
        room.square_position(raw, axis)
            .wrapping_sub(room.square_radius(raw, axis))
    } else {
        room.square_radius(raw, axis)
            .wrapping_add(room.square_position(raw, axis))
            .wrapping_sub(1)
    };
    let mut block = if axis == Axis::X {
        let x = if missile {
            room.square_position(raw, Axis::X)
        } else {
            edge
        };
        row.wrapping_add(x >> 4).wrapping_mul(2)
    } else if missile {
        let row = u16::from((room.square_position(raw, Axis::Y) >> 4) as u8)
            * u16::from(room.0.session().room.width as u8);
        row.wrapping_add(room.square_position(raw, Axis::X) >> 4)
            .wrapping_mul(2)
    } else {
        let column = |room: &Room<'_, _>| {
            room.square_position(raw, Axis::X)
                .wrapping_sub(room.square_radius(raw, Axis::X))
                >> 4
        };
        // Non-wave reads the column before room width; wave reads it afterward.
        let before = if wave { 0 } else { column(&room) };
        let row = u16::from((edge >> 4) as u8) * u16::from(room.0.session().room.width as u8);
        row.wrapping_add(if wave { column(&room) } else { before })
            .wrapping_mul(2)
    };
    let outside = |high: u16, scrolls: u16| (high as i16).wrapping_sub(scrolls as i16) >= 0;
    if !missile && span.wrapping_sub(16) & 0x8000 == 0 {
        return 0;
    }
    if !wave && outside(u16::from((edge >> 8) as u8), room.scrolls(axis)) {
        return 0;
    }
    let mut scan = Scan {
        displacement: amount,
        position,
        span,
        edge,
        missile: u16::from(missile),
        auxiliary: 0,
        remaining: span,
        unblocked: span,
    };
    if wave {
        // Wave clipping reads stored high bytes, independently of native order.
        let high = room.position(raw, axis.other()) >> 8;
        if high & 0x80 != 0
            || outside(high, room.scrolls(axis.other()))
            || outside(u16::from((edge >> 8) as u8), room.scrolls(axis))
        {
            return 0;
        }
    }
    if missile {
        let (_, hit) = reactions::react(axis, block, scan, &mut room);
        if hit == 0 {
            return 0;
        }
    } else {
        loop {
            (scan, _) = reactions::react(axis, block, scan, &mut room);
            block = block.wrapping_add(if axis == Axis::X {
                room.0.session().room.width.wrapping_mul(2)
            } else {
                2
            });
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
        }
        if wave || scan.unblocked & 0x8000 == 0 {
            return 0;
        }
    }
    super::lifecycle::retire(room.0, raw);
    room.0.probe("player_projectiles.terrain.retired");
    1
}
impl<H: Host> Definitions for Room<'_, H> {
    type Table = RomRecord;
    fn resolve(&mut self, address: u32) -> RomRecord {
        self.0.record((address >> 16) as u8, address as u16)
    }
    fn bts(&self) -> u8 {
        super::access::room_bts(self.0, self.0.session().room.collision_cursor)
    }
    fn area(&self) -> u16 {
        self.0.session().room.area
    }
    fn spawn(&mut self, header: u16) {
        self.0.spawn_mechanism(header);
    }
}
impl<H: Host> reactions::Context for Room<'_, H> {
    fn byte_limit(&self) -> u16 {
        self.0.session().room.size_bytes
    }
    fn cursor(&self) -> u16 {
        self.0.session().room.collision_cursor
    }
    fn select(&mut self, cell: u16) {
        self.0.session_mut().room.collision_cursor = cell;
    }
    fn material(&self) -> Material {
        reactions::material(super::access::room_level(
            self.0,
            self.0.session().room.collision_cursor,
        ))
    }
    fn extension(&self) -> i16 {
        i16::from(Definitions::bts(self) as i8)
    }
    fn width(&self) -> u16 {
        self.0.session().room.width
    }
    fn slope(&self, axis: Axis, scan: &mut impl Progress) -> bool {
        reactions::slope(axis, scan, self)
    }
}
