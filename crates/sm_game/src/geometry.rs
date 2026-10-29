//! 16-bit coordinates and 16.16 fixed-point values shared by gameplay state.

/// A 16.16 value stored as (fraction, whole), the RAM word order.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FixedMotion {
    pub fraction: u16,
    pub whole: u16,
}
impl FixedMotion {
    // Overlapping middle words are computed views, never separately stored.
    pub fn middle(&self) -> u16 {
        (self.fraction >> 8) | (self.whole << 8)
    }
    pub fn set_middle(&mut self, value: u16) {
        self.fraction = (self.fraction & 0xff) | (value << 8);
        self.whole = (self.whole & 0xff00) | (value >> 8);
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Position {
    pub whole: u16,
    pub fraction: u16,
}
impl Position {
    /// Add a signed 8.8 velocity to this 16.16 position, wrapping at 16 bits.
    pub fn advance_8_8(&mut self, velocity: i16) {
        self.advance(i32::from(velocity) << 8);
    }
    pub fn advance(&mut self, velocity: i32) {
        let position = (u32::from(self.whole) << 16) | u32::from(self.fraction);
        let next = position.wrapping_add(velocity as u32);
        self.fraction = next as u16;
        self.whole = (next >> 16) as u16;
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rectangle {
    pub origin: Point,
    pub extent: Point,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vector32 {
    pub x: u32,
    pub y: u32,
}
