//! Effective-address helpers, one per 65816 addressing mode.
//! Generated code calls these; they are also what the fuzz crate verifies.
//!
//! Simplifications (native mode, as Super Metroid uses): no emulation-mode
//! direct-page page wrapping.

use crate::Machine;

#[inline]
fn bank_db(m: &Machine) -> u32 {
    (m.cpu.db as u32) << 16
}

pub fn dp(m: &Machine, o: u8) -> u32 {
    m.cpu.d.wrapping_add(o as u16) as u32
}

pub fn dp_x(m: &Machine, o: u8) -> u32 {
    m.cpu.d.wrapping_add(o as u16).wrapping_add(m.cpu.x) as u32
}

pub fn dp_y(m: &Machine, o: u8) -> u32 {
    m.cpu.d.wrapping_add(o as u16).wrapping_add(m.cpu.y) as u32
}

/// (d)
pub fn dp_ind(m: &mut Machine, o: u8) -> u32 {
    let p = dp(m, o) as u16;
    bank_db(m) | m.read16_bank0(p) as u32
}

/// (d,x)
pub fn dp_ind_x(m: &mut Machine, o: u8) -> u32 {
    let p = dp_x(m, o) as u16;
    bank_db(m) | m.read16_bank0(p) as u32
}

/// (d),y
pub fn dp_ind_y(m: &mut Machine, o: u8) -> u32 {
    let base = dp_ind(m, o);
    base.wrapping_add(m.cpu.y as u32) & 0xff_ffff
}

/// [d]
pub fn dp_ind_long(m: &mut Machine, o: u8) -> u32 {
    let p = dp(m, o) as u16;
    m.read24_bank0(p)
}

/// [d],y
pub fn dp_ind_long_y(m: &mut Machine, o: u8) -> u32 {
    dp_ind_long(m, o).wrapping_add(m.cpu.y as u32) & 0xff_ffff
}

pub fn abs(m: &Machine, o: u16) -> u32 {
    bank_db(m) | o as u32
}

pub fn abs_x(m: &Machine, o: u16) -> u32 {
    abs(m, o).wrapping_add(m.cpu.x as u32) & 0xff_ffff
}

pub fn abs_y(m: &Machine, o: u16) -> u32 {
    abs(m, o).wrapping_add(m.cpu.y as u32) & 0xff_ffff
}

pub fn long(o: u32) -> u32 {
    o & 0xff_ffff
}

pub fn long_x(m: &Machine, o: u32) -> u32 {
    o.wrapping_add(m.cpu.x as u32) & 0xff_ffff
}

/// d,s
pub fn sr(m: &Machine, o: u8) -> u32 {
    m.cpu.s.wrapping_add(o as u16) as u32
}

/// (d,s),y
pub fn sr_ind_y(m: &mut Machine, o: u8) -> u32 {
    let p = sr(m, o) as u16;
    let base = bank_db(m) | m.read16_bank0(p) as u32;
    base.wrapping_add(m.cpu.y as u32) & 0xff_ffff
}

/// JMP (a,x) / JSR (a,x): pointer is read from the *program* bank.
pub fn abs_ind_x_target(m: &mut Machine, o: u16) -> u32 {
    let ptr = m.cpu.pbr(o.wrapping_add(m.cpu.x));
    let lo = m.read8(ptr) as u32;
    let hi = m.read8(m.cpu.pbr(o.wrapping_add(m.cpu.x).wrapping_add(1))) as u32;
    ((m.cpu.pb as u32) << 16) | lo | hi << 8
}

/// JMP (a): pointer read from bank 0, target in program bank.
pub fn abs_ind_target(m: &mut Machine, o: u16) -> u32 {
    ((m.cpu.pb as u32) << 16) | m.read16_bank0(o) as u32
}

/// JML [a]: 24-bit pointer read from bank 0.
pub fn abs_ind_long_target(m: &mut Machine, o: u16) -> u32 {
    m.read24_bank0(o)
}
