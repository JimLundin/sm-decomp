//! Per-direction grapple beam firing and connection records in bank $9B.

use crate::table::{LengthError, Rows};

pub const DIRECTIONS: usize = 10;

/// $9B:C0DB: the extension X velocities, then (at $C0EF) the Y velocities and,
/// after one padding byte, (at $C104) the beam end angles.
pub const EXTENSION_ADDRESS: u32 = 0x9b_c0db;
pub const EXTENSION_BYTES: usize = 0x3d;
const Y_VELOCITY: usize = 0x14;
const ANGLE: usize = 0x29;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Extension {
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub angle: u16,
}

impl Extension {
    /// Byte offset of `column` (0 to 2) from `EXTENSION_ADDRESS`.
    pub const fn column_offset(column: usize) -> usize {
        [0, Y_VELOCITY, ANGLE][column]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extensions([Extension; DIRECTIONS]);

impl Extensions {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != EXTENSION_BYTES {
            return Err(LengthError(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self(core::array::from_fn(|direction| Extension {
            x_velocity: word(2 * direction),
            y_velocity: word(Y_VELOCITY + 2 * direction),
            angle: word(ANGLE + 2 * direction),
        })))
    }

    pub fn direction(&self, direction: usize) -> Option<Extension> {
        self.0.get(direction).copied()
    }
}

/// $9B:C3C6: what a connected beam runs next, in three ten-direction sets.
pub const CONNECTION_ADDRESS: u32 = 0x9b_c3c6;
pub const CONNECTION_BYTES: usize = 3 * DIRECTIONS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionSet {
    Standard,
    Airborne,
    Crouching,
}

impl ConnectionSet {
    pub const fn address(self) -> u32 {
        CONNECTION_ADDRESS + (self as usize * DIRECTIONS * 4) as u32
    }
}

/// Bank $9B handler addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connection {
    pub beam_function: u16,
    pub handler: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connections(Rows<Connection, { 3 * DIRECTIONS }>);

impl Connections {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        Rows::import(bytes, |[beam_function, handler]| Connection {
            beam_function,
            handler,
        })
        .map(Self)
    }

    pub fn direction(&self, set: ConnectionSet, direction: usize) -> Option<Connection> {
        (direction < DIRECTIONS)
            .then(|| self.0.row(set as usize * DIRECTIONS + direction))
            .flatten()
    }
}
