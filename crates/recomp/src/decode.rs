use crate::opcodes::{Mode, OPCODES};
use crate::rom::Rom;

#[derive(Clone, Debug)]
pub struct Insn {
    pub addr: u32,
    pub op: u8,
    pub mnem: &'static str,
    pub mode: Mode,
    pub len: u32,
    /// Raw operand bytes, little-endian (for Block: low byte = dst bank, high = src bank).
    pub operand: u32,
    /// Widths in effect when this instruction executes.
    pub m8: bool,
    pub x8: bool,
}

impl Insn {
    /// Address of the next sequential instruction (same bank).
    pub fn next(&self) -> u32 {
        (self.addr & 0xff_0000) | ((self.addr + self.len) & 0xffff)
    }

    /// Branch target for Rel8/Rel16.
    pub fn rel_target(&self) -> u32 {
        let base = (self.addr + self.len) & 0xffff;
        let off = match self.mode {
            Mode::Rel8 => self.operand as u8 as i8 as i32,
            Mode::Rel16 => self.operand as u16 as i16 as i32,
            _ => 0,
        };
        (self.addr & 0xff_0000) | ((base as i32 + off) as u32 & 0xffff)
    }
}

/// Decode in a byte source (ROM, or a fuzz test's RAM).
pub fn decode_with(read: impl Fn(u32) -> Option<u8>, addr: u32, m8: bool, x8: bool) -> Option<Insn> {
    let op = read(addr)?;
    let (mnem, mode) = OPCODES[op as usize];
    let len = mode.len(m8, x8);
    let mut operand = 0u32;
    for i in 1..len {
        let a = (addr & 0xff_0000) | ((addr + i) & 0xffff);
        operand |= (read(a)? as u32) << (8 * (i - 1));
    }
    Some(Insn { addr, op, mnem, mode, len, operand, m8, x8 })
}

pub fn decode(rom: &Rom, addr: u32, m8: bool, x8: bool) -> Option<Insn> {
    decode_with(|a| rom.byte(a), addr, m8, x8)
}
