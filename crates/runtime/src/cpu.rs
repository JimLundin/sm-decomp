use crate::Mx;

pub mod flag {
    pub const C: u8 = 0x01;
    pub const Z: u8 = 0x02;
    pub const I: u8 = 0x04;
    pub const D: u8 = 0x08;
    pub const X: u8 = 0x10;
    pub const M: u8 = 0x20;
    pub const V: u8 = 0x40;
    pub const N: u8 = 0x80;
}

/// 65816 register file. Native mode only (Super Metroid never leaves it after boot).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub pb: u8,
    pub p: u8,
    pub e: bool,
}

impl Cpu {
    #[inline]
    pub fn flag(&self, f: u8) -> bool {
        self.p & f != 0
    }

    #[inline]
    pub fn set_flag(&mut self, f: u8, on: bool) {
        if on {
            self.p |= f
        } else {
            self.p &= !f
        }
    }

    /// True when the accumulator is 8-bit.
    #[inline]
    pub fn m8(&self) -> bool {
        self.flag(flag::M)
    }

    /// True when X/Y are 8-bit.
    #[inline]
    pub fn x8(&self) -> bool {
        self.flag(flag::X)
    }

    #[inline]
    pub fn mx(&self) -> Mx {
        ((self.m8() as u8) << 1) | self.x8() as u8
    }

    /// Write P, keeping the invariant that X/Y high bytes are zero in 8-bit index mode.
    pub fn set_p(&mut self, v: u8) {
        self.p = v;
        if self.e {
            self.p |= flag::M | flag::X;
        }
        if self.x8() {
            self.x &= 0xff;
            self.y &= 0xff;
        }
    }

    #[inline]
    pub fn set_nz(&mut self, v: u16, wide: bool) {
        if wide {
            self.set_flag(flag::Z, v == 0);
            self.set_flag(flag::N, v & 0x8000 != 0);
        } else {
            self.set_flag(flag::Z, v & 0xff == 0);
            self.set_flag(flag::N, v & 0x80 != 0);
        }
    }

    /// Program bank + 16-bit address → 24-bit address.
    #[inline]
    pub fn pbr(&self, addr: u16) -> u32 {
        ((self.pb as u32) << 16) | addr as u32
    }
}
