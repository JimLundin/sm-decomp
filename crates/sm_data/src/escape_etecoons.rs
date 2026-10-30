//! The escape room Etecoons' initial state at $B3:E718, indexed by enemy
//! parameter 1. Five parallel word tables, one entry per Etecoon.

/// LoROM address of the X position table; the other four follow it.
pub const ADDRESS: u32 = 0xb3_e718;
pub const ETECOONS: usize = 3;
pub const BYTES: usize = 5 * 2 * ETECOONS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscapeEtecoon {
    pub x_position: u16,
    pub y_position: u16,
    /// Bank $B3 function.
    pub function: u16,
    /// Bank $B3 instruction list.
    pub instruction_list: u16,
    pub x_velocity: u16,
}

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscapeEtecoons {
    etecoons: [EscapeEtecoon; ETECOONS],
}

impl EscapeEtecoons {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let column = |table: usize, etecoon: usize| {
            let at = 2 * (ETECOONS * table + etecoon);
            u16::from_le_bytes([bytes[at], bytes[at + 1]])
        };
        Ok(Self {
            etecoons: core::array::from_fn(|etecoon| EscapeEtecoon {
                x_position: column(0, etecoon),
                y_position: column(1, etecoon),
                function: column(2, etecoon),
                instruction_list: column(3, etecoon),
                x_velocity: column(4, etecoon),
            }),
        })
    }

    pub fn etecoon(&self, index: usize) -> Option<EscapeEtecoon> {
        self.etecoons.get(index).copied()
    }
}
