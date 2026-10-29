//! Owned SPC700 execution with ordered operations through the APU bus.

use crate::initial_state::InitialState;
use crate::snes::apu::{apu_cpuRead, apu_cpuWrite};
use crate::types::Apu;
pub use crate::types::Spc;
use core::mem::{offset_of, size_of};

// Preserve the retained backend's live layout and legacy save span.
const _: () = {
    assert!(offset_of!(Spc, a) == size_of::<*mut Apu>());
    assert!(offset_of!(Spc, pc) == size_of::<*mut Apu>() + 4);
    assert!(offset_of!(Spc, c) == size_of::<*mut Apu>() + 6);
    assert!(offset_of!(Spc, cyclesUsed) == size_of::<*mut Apu>() + 15);
    assert!(size_of::<Spc>() == size_of::<*mut Apu>() + 16);
};

const CYCLES_PER_OPCODE: [u8; 256] = [
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 6, 8, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 4, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 5, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 3, 8,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 6, 6, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 4, 5, 2, 2, 4, 3,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 5, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3, 6,
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 2, 4, 5, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 12,
    5, 2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 2, 4, 4, 2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3,
    4, 2, 8, 4, 5, 4, 5, 4, 7, 2, 5, 6, 4, 5, 2, 4, 9, 2, 8, 4, 5, 5, 6, 6, 7, 4, 5, 5, 5, 2, 2, 6,
    3, 2, 8, 4, 5, 3, 4, 3, 6, 2, 4, 5, 3, 4, 3, 4, 3, 2, 8, 4, 5, 4, 5, 5, 6, 3, 4, 5, 4, 2, 2, 4,
    3,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusOperation {
    Read(u16),
    Write(u16, u8),
}

struct Core<'a, B: FnMut(BusOperation) -> u8> {
    cpu: &'a mut Spc,
    bus: B,
}

#[forbid(unsafe_code)]
impl<B: FnMut(BusOperation) -> u8> Core<'_, B> {
    fn read(&mut self, address: u16) -> u8 {
        (self.bus)(BusOperation::Read(address))
    }

    fn write(&mut self, address: u16, value: u8) {
        (self.bus)(BusOperation::Write(address, value));
    }

    fn fetch(&mut self) -> u8 {
        let address = self.cpu.pc;
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        self.read(address)
    }

    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch();
        u16::from_le_bytes([low, self.fetch()])
    }

    fn read_word(&mut self, low: u16, high: u16) -> u16 {
        let value = self.read(low);
        u16::from_le_bytes([value, self.read(high)])
    }

    fn write_word(&mut self, low: u16, high: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.write(low, bytes[0]);
        self.write(high, bytes[1]);
    }

    fn flags(&self) -> u8 {
        (u8::from(self.cpu.n) << 7)
            | (u8::from(self.cpu.v) << 6)
            | (u8::from(self.cpu.p) << 5)
            | (u8::from(self.cpu.b) << 4)
            | (u8::from(self.cpu.h) << 3)
            | (u8::from(self.cpu.i) << 2)
            | (u8::from(self.cpu.z) << 1)
            | u8::from(self.cpu.c)
    }

    fn set_flags(&mut self, value: u8) {
        self.cpu.n = value & 0x80 != 0;
        self.cpu.v = value & 0x40 != 0;
        self.cpu.p = value & 0x20 != 0;
        self.cpu.b = value & 0x10 != 0;
        self.cpu.h = value & 8 != 0;
        self.cpu.i = value & 4 != 0;
        self.cpu.z = value & 2 != 0;
        self.cpu.c = value & 1 != 0;
    }

    fn zn(&mut self, value: u8) {
        self.cpu.z = value == 0;
        self.cpu.n = value & 0x80 != 0;
    }

    fn branch(&mut self, offset: u8, condition: bool) {
        if condition {
            self.cpu.cyclesUsed = self.cpu.cyclesUsed.wrapping_add(2);
            self.cpu.pc = self.cpu.pc.wrapping_add_signed(i16::from(offset as i8));
        }
    }

    fn pull_byte(&mut self) -> u8 {
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        self.read(0x100 | u16::from(self.cpu.sp))
    }

    fn push_byte(&mut self, value: u8) {
        self.write(0x100 | u16::from(self.cpu.sp), value);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
    }

    fn pull_word(&mut self) -> u16 {
        let low = self.pull_byte();
        u16::from_le_bytes([low, self.pull_byte()])
    }

    fn push_word(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.push_byte(bytes[1]);
        self.push_byte(bytes[0]);
    }

    fn page(&self) -> u16 {
        u16::from(self.cpu.p) << 8
    }

    fn dp(&mut self) -> u16 {
        u16::from(self.fetch()) | self.page()
    }

    fn abs(&mut self) -> u16 {
        self.fetch_word()
    }
    fn ind(&self) -> u16 {
        u16::from(self.cpu.x) | self.page()
    }

    fn idx(&mut self) -> u16 {
        let pointer = self.fetch().wrapping_add(self.cpu.x);
        self.read_word(
            u16::from(pointer) | self.page(),
            u16::from(pointer.wrapping_add(1)) | self.page(),
        )
    }

    fn imm(&mut self) -> u16 {
        let address = self.cpu.pc;
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        address
    }

    fn dpx(&mut self) -> u16 {
        u16::from(self.fetch().wrapping_add(self.cpu.x)) | self.page()
    }
    fn dpy(&mut self) -> u16 {
        u16::from(self.fetch().wrapping_add(self.cpu.y)) | self.page()
    }
    fn abx(&mut self) -> u16 {
        self.fetch_word().wrapping_add(u16::from(self.cpu.x))
    }
    fn aby(&mut self) -> u16 {
        self.fetch_word().wrapping_add(u16::from(self.cpu.y))
    }

    fn idy(&mut self) -> u16 {
        let pointer = self.fetch();
        let address = self.read_word(
            u16::from(pointer) | self.page(),
            u16::from(pointer.wrapping_add(1)) | self.page(),
        );
        address.wrapping_add(u16::from(self.cpu.y))
    }

    fn dp_dp(&mut self) -> (u16, u16) {
        let source = self.dp();
        (self.dp(), source)
    }

    fn dp_imm(&mut self) -> (u16, u16) {
        let source = self.imm();
        (self.dp(), source)
    }

    fn ind_ind(&self) -> (u16, u16) {
        (self.ind(), u16::from(self.cpu.y) | self.page())
    }

    fn abs_bit(&mut self) -> (u16, u8) {
        let value = self.fetch_word();
        (value & 0x1fff, (value >> 13) as u8)
    }

    fn dp_word(&mut self) -> (u16, u16) {
        let low = self.fetch();
        (
            u16::from(low) | self.page(),
            u16::from(low.wrapping_add(1)) | self.page(),
        )
    }

    fn ind_post_increment(&mut self) -> u16 {
        let address = u16::from(self.cpu.x) | self.page();
        self.cpu.x = self.cpu.x.wrapping_add(1);
        address
    }
}

#[derive(Clone, Copy)]
enum Alu {
    Or,
    And,
    Eor,
    Cmp,
    Adc,
    Sbc,
}

#[forbid(unsafe_code)]
impl<B: FnMut(BusOperation) -> u8> Core<'_, B> {
    fn operand_address(&mut self, mode: u8) -> u16 {
        match mode {
            0x04 => self.dp(),
            0x05 => self.abs(),
            0x06 => self.ind(),
            0x07 => self.idx(),
            0x08 => self.imm(),
            0x14 => self.dpx(),
            0x15 => self.abx(),
            0x16 => self.aby(),
            0x17 => self.idy(),
            _ => unreachable!("SPC operand mode"),
        }
    }

    fn alu_value(&mut self, operation: Alu, left: u8, right: u8) -> u8 {
        match operation {
            Alu::Or => left | right,
            Alu::And => left & right,
            Alu::Eor => left ^ right,
            Alu::Cmp => {
                let result = u16::from(left) + u16::from(!right) + 1;
                self.cpu.c = result > 0xff;
                result as u8
            }
            Alu::Adc | Alu::Sbc => {
                let right = if matches!(operation, Alu::Sbc) {
                    !right
                } else {
                    right
                };
                let result = u16::from(left) + u16::from(right) + u16::from(self.cpu.c);
                self.cpu.v =
                    (left & 0x80) == (right & 0x80) && (right & 0x80) != (result as u8 & 0x80);
                self.cpu.h = (left & 0xf) + (right & 0xf) + u8::from(self.cpu.c) > 0xf;
                self.cpu.c = result > 0xff;
                result as u8
            }
        }
    }

    fn alu_opcode(&mut self, opcode: u8) {
        let operation = match opcode >> 5 {
            0 => Alu::Or,
            1 => Alu::And,
            2 => Alu::Eor,
            3 => Alu::Cmp,
            4 => Alu::Adc,
            5 => Alu::Sbc,
            _ => unreachable!("SPC ALU family"),
        };
        let mode = opcode & 0x1f;
        if matches!(mode, 0x09 | 0x18 | 0x19) {
            let (destination, source) = match mode {
                0x09 => self.dp_dp(),
                0x18 => self.dp_imm(),
                0x19 => self.ind_ind(),
                _ => unreachable!(),
            };
            let value = self.read(source);
            let left = self.read(destination);
            let result = self.alu_value(operation, left, value);
            if !matches!(operation, Alu::Cmp) {
                self.write(destination, result);
            }
            self.zn(result);
        } else {
            let address = self.operand_address(mode);
            let value = self.read(address);
            let result = self.alu_value(operation, self.cpu.a, value);
            if !matches!(operation, Alu::Cmp) {
                self.cpu.a = result;
            }
            self.zn(result);
        }
    }

    fn shift_value(&mut self, family: u8, value: u8) -> u8 {
        match family {
            0 => {
                self.cpu.c = value & 0x80 != 0;
                value.wrapping_shl(1)
            }
            1 => {
                let result = value.wrapping_shl(1) | u8::from(self.cpu.c);
                self.cpu.c = value & 0x80 != 0;
                result
            }
            2 => {
                self.cpu.c = value & 1 != 0;
                value >> 1
            }
            3 => {
                let result = (value >> 1) | (u8::from(self.cpu.c) << 7);
                self.cpu.c = value & 1 != 0;
                result
            }
            4 => value.wrapping_sub(1),
            5 => value.wrapping_add(1),
            _ => unreachable!("SPC shift family"),
        }
    }

    fn shift_opcode(&mut self, opcode: u8) {
        let address = match opcode & 0x1f {
            0x0b => self.dp(),
            0x0c => self.abs(),
            0x1b => self.dpx(),
            _ => unreachable!("SPC shift mode"),
        };
        let value = self.read(address);
        let result = self.shift_value(opcode >> 5, value);
        self.write(address, result);
        self.zn(result);
    }

    fn other_opcode(&mut self, opcode: u8) {
        match opcode {
            0x1c | 0x5c | 0x9c | 0xbc => {
                self.cpu.a = self.shift_value(opcode >> 5, self.cpu.a);
                self.zn(self.cpu.a);
            }
            0x3c | 0x7c => {
                let value = self.cpu.a;
                let carry = self.cpu.c;
                self.cpu.a = if opcode == 0x3c {
                    value.wrapping_shl(1) | u8::from(carry)
                } else {
                    (value >> 1) | (u8::from(carry) << 7)
                };
                self.cpu.c = value & (if opcode == 0x3c { 0x80 } else { 1 }) != 0;
                self.zn(self.cpu.a);
            }
            0x1d | 0x3d => {
                self.cpu.x = if opcode == 0x1d {
                    self.cpu.x.wrapping_sub(1)
                } else {
                    self.cpu.x.wrapping_add(1)
                };
                self.zn(self.cpu.x);
            }
            0xdc | 0xfc => {
                self.cpu.y = if opcode == 0xdc {
                    self.cpu.y.wrapping_sub(1)
                } else {
                    self.cpu.y.wrapping_add(1)
                };
                self.zn(self.cpu.y);
            }
            0x1e | 0x3e | 0xc8 | 0x5e | 0x7e | 0xad => {
                let address = match opcode {
                    0x1e | 0x5e => self.abs(),
                    0x3e | 0x7e => self.dp(),
                    0xc8 | 0xad => self.imm(),
                    _ => unreachable!(),
                };
                let value = self.read(address);
                let left = if matches!(opcode, 0x1e | 0x3e | 0xc8) {
                    self.cpu.x
                } else {
                    self.cpu.y
                };
                let result = self.alu_value(Alu::Cmp, left, value);
                self.zn(result);
            }
            0x5d | 0x7d | 0x9d | 0xdd | 0xfd => {
                let value = match opcode {
                    0x5d | 0xfd => self.cpu.a,
                    0x7d => self.cpu.x,
                    0x9d => self.cpu.sp,
                    0xdd => self.cpu.y,
                    _ => unreachable!(),
                };
                match opcode {
                    0x5d | 0x9d => self.cpu.x = value,
                    0x7d | 0xdd => self.cpu.a = value,
                    0xfd => self.cpu.y = value,
                    _ => unreachable!(),
                }
                self.zn(value);
            }
            0xbd => self.cpu.sp = self.cpu.x,
            0x8f | 0xfa => {
                let (destination, source) = if opcode == 0x8f {
                    self.dp_imm()
                } else {
                    self.dp_dp()
                };
                let value = self.read(source);
                if opcode == 0x8f {
                    self.read(destination);
                }
                self.write(destination, value);
            }
            0xaf | 0xbf => {
                let address = self.ind_post_increment();
                if opcode == 0xaf {
                    self.write(address, self.cpu.a);
                } else {
                    self.cpu.a = self.read(address);
                    self.zn(self.cpu.a);
                }
            }
            0xba => {
                let (low, high) = self.dp_word();
                let value = self.read_word(low, high);
                self.cpu.a = value as u8;
                self.cpu.y = (value >> 8) as u8;
                self.cpu.z = value == 0;
                self.cpu.n = value & 0x8000 != 0;
            }
            0xda => {
                let (low, high) = self.dp_word();
                self.read(low);
                self.write(low, self.cpu.a);
                self.write(high, self.cpu.y);
            }
            0xc4 | 0xc5 | 0xc6 | 0xc7 | 0xd4 | 0xd5 | 0xd6 | 0xd7 => {
                let address = self.operand_address(opcode & 0x1f);
                self.read(address);
                self.write(address, self.cpu.a);
            }
            0xe4 | 0xe5 | 0xe6 | 0xe7 | 0xe8 | 0xf4 | 0xf5 | 0xf6 | 0xf7 => {
                let address = self.operand_address(opcode & 0x1f);
                self.cpu.a = self.read(address);
                self.zn(self.cpu.a);
            }
            0xc9 | 0xd8 | 0xd9 | 0xcb | 0xcc | 0xdb => {
                let address = match opcode {
                    0xc9 | 0xcc => self.abs(),
                    0xd8 | 0xcb => self.dp(),
                    0xd9 => self.dpy(),
                    0xdb => self.dpx(),
                    _ => unreachable!(),
                };
                self.read(address);
                let value = if matches!(opcode, 0xc9 | 0xd8 | 0xd9) {
                    self.cpu.x
                } else {
                    self.cpu.y
                };
                self.write(address, value);
            }
            0xcd | 0xe9 | 0xf8 | 0xf9 | 0x8d | 0xeb | 0xec | 0xfb => {
                let address = match opcode {
                    0xcd | 0x8d => self.imm(),
                    0xe9 | 0xec => self.abs(),
                    0xf8 | 0xeb => self.dp(),
                    0xf9 => self.dpy(),
                    0xfb => self.dpx(),
                    _ => unreachable!(),
                };
                let value = self.read(address);
                if matches!(opcode, 0xcd | 0xe9 | 0xf8 | 0xf9) {
                    self.cpu.x = value;
                } else {
                    self.cpu.y = value;
                }
                self.zn(value);
            }
            _ => unreachable!("SPC opcode partition"),
        }
    }
}

#[forbid(unsafe_code)]
impl<'a, B: FnMut(BusOperation) -> u8> Core<'a, B> {
    fn control_opcode(&mut self, opcode: u8) {
        match opcode {
            0x00 => {}
            0x01 | 0x11 | 0x21 | 0x31 | 0x41 | 0x51 | 0x61 | 0x71 | 0x81 | 0x91 | 0xa1 | 0xb1
            | 0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let pc = self.cpu.pc;
                self.push_word(pc);
                let vector = 0xffde_u16.wrapping_sub(2 * u16::from(opcode >> 4));
                let pc = self.read_word(vector, vector.wrapping_add(1));
                self.cpu.pc = pc;
            }
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x82 | 0x92 | 0xa2 | 0xb2
            | 0xc2 | 0xd2 | 0xe2 | 0xf2 => {
                let address = self.dp();
                let value = self.read(address);
                let mask = 1_u8 << (opcode >> 5);
                let result = if opcode & 0x10 == 0 {
                    value | mask
                } else {
                    value & !mask
                };
                self.write(address, result);
            }
            0x03 | 0x13 | 0x23 | 0x33 | 0x43 | 0x53 | 0x63 | 0x73 | 0x83 | 0x93 | 0xa3 | 0xb3
            | 0xc3 | 0xd3 | 0xe3 | 0xf3 => {
                let address = self.dp();
                let value = self.read(address);
                let offset = self.fetch();
                let bit_set = value & (1_u8 << (opcode >> 5)) != 0;
                let condition = if opcode & 0x10 == 0 {
                    bit_set
                } else {
                    !bit_set
                };
                self.branch(offset, condition);
            }
            0x0a | 0x2a | 0x4a | 0x6a | 0x8a | 0xaa => {
                let (address, bit) = self.abs_bit();
                let value = self.read(address);
                let bit_set = value & (1_u8 << bit) != 0;
                self.cpu.c = match opcode {
                    0x0a => self.cpu.c | bit_set,
                    0x2a => self.cpu.c | !bit_set,
                    0x4a => self.cpu.c & bit_set,
                    0x6a => self.cpu.c & !bit_set,
                    0x8a => self.cpu.c ^ bit_set,
                    0xaa => bit_set,
                    _ => unreachable!(),
                };
            }
            0xca | 0xea => {
                let (address, bit) = self.abs_bit();
                let value = self.read(address);
                let mask = 1_u8 << bit;
                let result = if opcode == 0xca {
                    (value & !mask) | (u8::from(self.cpu.c) << bit)
                } else {
                    value ^ mask
                };
                self.write(address, result);
            }
            0x0e | 0x4e => {
                let address = self.abs();
                let value = self.read(address);
                let result = self.cpu.a.wrapping_sub(value);
                self.zn(result);
                let result = if opcode == 0x0e {
                    value | self.cpu.a
                } else {
                    value & !self.cpu.a
                };
                self.write(address, result);
            }
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xb0 | 0xd0 | 0xf0 => {
                let offset = self.fetch();
                let condition = match opcode {
                    0x10 => !self.cpu.n,
                    0x30 => self.cpu.n,
                    0x50 => !self.cpu.v,
                    0x70 => self.cpu.v,
                    0x90 => !self.cpu.c,
                    0xb0 => self.cpu.c,
                    0xd0 => !self.cpu.z,
                    0xf0 => self.cpu.z,
                    _ => unreachable!(),
                };
                self.branch(offset, condition);
            }
            0x2e | 0xde => {
                let address = if opcode == 0x2e {
                    self.dp()
                } else {
                    self.dpx()
                };
                let value = self.read(address);
                let result = self.cpu.a.wrapping_sub(value);
                let offset = self.fetch();
                self.branch(offset, result != 0);
            }
            0x6e => {
                let address = self.dp();
                let value = self.read(address);
                let result = value.wrapping_sub(1);
                self.write(address, result);
                let offset = self.fetch();
                self.branch(offset, result != 0);
            }
            0xfe => {
                self.cpu.y = self.cpu.y.wrapping_sub(1);
                let offset = self.fetch();
                self.branch(offset, self.cpu.y != 0);
            }
            0x2f => {
                let offset = self.fetch();
                let offset = i16::from(i8::from_le_bytes([offset]));
                self.cpu.pc = self.cpu.pc.wrapping_add_signed(offset);
            }
            0x0f => {
                let pc = self.cpu.pc;
                self.push_word(pc);
                let flags = self.flags();
                self.push_byte(flags);
                self.cpu.i = false;
                self.cpu.b = true;
                let pc = self.read_word(0xffde, 0xffdf);
                self.cpu.pc = pc;
            }
            0x1f => {
                let pointer = self.fetch_word();
                let low = pointer.wrapping_add(u16::from(self.cpu.x));
                let high = low.wrapping_add(1);
                let pc = self.read_word(low, high);
                self.cpu.pc = pc;
            }
            0x3f => {
                let destination = self.fetch_word();
                let pc = self.cpu.pc;
                self.push_word(pc);
                self.cpu.pc = destination;
            }
            0x4f => {
                let destination = self.fetch();
                let pc = self.cpu.pc;
                self.push_word(pc);
                self.cpu.pc = 0xff00 | u16::from(destination);
            }
            0x5f => {
                let pc = self.fetch_word();
                self.cpu.pc = pc;
            }
            0x6f => {
                let pc = self.pull_word();
                self.cpu.pc = pc;
            }
            0x7f => {
                let flags = self.pull_byte();
                self.set_flags(flags);
                let pc = self.pull_word();
                self.cpu.pc = pc;
            }
            0x0d => {
                let flags = self.flags();
                self.push_byte(flags);
            }
            0x2d => {
                let value = self.cpu.a;
                self.push_byte(value);
            }
            0x4d => {
                let value = self.cpu.x;
                self.push_byte(value);
            }
            0x6d => {
                let value = self.cpu.y;
                self.push_byte(value);
            }
            0x8e => {
                let flags = self.pull_byte();
                self.set_flags(flags);
            }
            0xae => {
                let value = self.pull_byte();
                self.cpu.a = value;
            }
            0xce => {
                let value = self.pull_byte();
                self.cpu.x = value;
            }
            0xee => {
                let value = self.pull_byte();
                self.cpu.y = value;
            }
            0x20 => self.cpu.p = false,
            0x40 => self.cpu.p = true,
            0x60 => self.cpu.c = false,
            0x80 => self.cpu.c = true,
            0xa0 => self.cpu.i = true,
            0xc0 => self.cpu.i = false,
            0xe0 => {
                self.cpu.v = false;
                self.cpu.h = false;
            }
            0xed => self.cpu.c = !self.cpu.c,
            0xef | 0xff => self.cpu.stopped = true,
            _ => unreachable!("opcode is outside the SPC control family"),
        }
    }
}

#[forbid(unsafe_code)]
impl<B: FnMut(BusOperation) -> u8> Core<'_, B> {
    fn word_opcode(&mut self, opcode: u8) {
        match opcode {
            0x1a | 0x3a => {
                let (low, high) = self.dp_word();
                let value = self.read_word(low, high);
                let value = if opcode == 0x1a {
                    value.wrapping_sub(1)
                } else {
                    value.wrapping_add(1)
                };
                self.cpu.z = value == 0;
                self.cpu.n = value & 0x8000 != 0;
                self.write_word(low, high, value);
            }
            0x5a => {
                let (low, high) = self.dp_word();
                let value = self.read_word(low, high) ^ 0xffff;
                let ya = u16::from(self.cpu.a) | (u16::from(self.cpu.y) << 8);
                let result = u32::from(ya) + u32::from(value) + 1;
                self.cpu.c = result > 0xffff;
                self.cpu.z = result & 0xffff == 0;
                self.cpu.n = result & 0x8000 != 0;
            }
            0x7a | 0x9a => {
                let (low, high) = self.dp_word();
                let mut value = self.read_word(low, high);
                let subtract = opcode == 0x9a;
                if subtract {
                    value ^= 0xffff;
                }
                let ya = u16::from(self.cpu.a) | (u16::from(self.cpu.y) << 8);
                let result = u32::from(ya) + u32::from(value) + u32::from(subtract);
                self.cpu.v = (ya & 0x8000) == (value & 0x8000)
                    && u32::from(value & 0x8000) != result & 0x8000;
                self.cpu.h = u32::from(ya & 0xfff) + u32::from(value & 0xfff) + 1 > 0xfff;
                self.cpu.c = result > 0xffff;
                self.cpu.z = result & 0xffff == 0;
                self.cpu.n = result & 0x8000 != 0;
                self.cpu.a = result as u8;
                self.cpu.y = (result >> 8) as u8;
            }
            0x9e => {
                let value = u16::from(self.cpu.a) | (u16::from(self.cpu.y) << 8);
                let divisor = u16::from(self.cpu.x);
                let mut result = 0xffffu16;
                let mut remainder = u16::from(self.cpu.a);
                if divisor != 0 {
                    result = value / divisor;
                    remainder = value % divisor;
                }
                self.cpu.v = result > 0xff;
                self.cpu.h = self.cpu.x & 0xf <= self.cpu.y & 0xf;
                self.cpu.a = result as u8;
                self.cpu.y = remainder as u8;
                self.zn(self.cpu.a);
            }
            0x9f => {
                self.cpu.a = self.cpu.a.rotate_left(4);
                self.zn(self.cpu.a);
            }
            0xbe => {
                if self.cpu.a > 0x99 || !self.cpu.c {
                    self.cpu.a = self.cpu.a.wrapping_sub(0x60);
                    self.cpu.c = false;
                }
                if self.cpu.a & 0xf > 9 || !self.cpu.h {
                    self.cpu.a = self.cpu.a.wrapping_sub(6);
                }
                self.zn(self.cpu.a);
            }
            0xcf => {
                let result = u16::from(self.cpu.a) * u16::from(self.cpu.y);
                self.cpu.a = result as u8;
                self.cpu.y = (result >> 8) as u8;
                self.zn(self.cpu.y);
            }
            0xdf => {
                if self.cpu.a > 0x99 || self.cpu.c {
                    self.cpu.a = self.cpu.a.wrapping_add(0x60);
                    self.cpu.c = true;
                }
                if self.cpu.a & 0xf > 9 || self.cpu.h {
                    self.cpu.a = self.cpu.a.wrapping_add(6);
                }
                self.zn(self.cpu.a);
            }
            _ => unreachable!("SPC opcode outside word family: {opcode:#04x}"),
        }
    }
}

#[forbid(unsafe_code)]
impl Spc {
    pub fn new() -> Self {
        Self::INITIAL
    }

    pub fn reset(&mut self, bus: impl FnMut(BusOperation) -> u8) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0;
        let mut core = Core { cpu: self, bus };
        core.cpu.pc = core.read_word(0xfffe, 0xffff);
        core.cpu.c = false;
        core.cpu.z = false;
        core.cpu.v = false;
        core.cpu.n = false;
        core.cpu.i = false;
        core.cpu.h = false;
        core.cpu.p = false;
        core.cpu.b = false;
        core.cpu.stopped = false;
        core.cpu.cyclesUsed = 0;
    }

    pub fn run_opcode(&mut self, bus: impl FnMut(BusOperation) -> u8) -> u8 {
        self.cyclesUsed = 0;
        if self.stopped {
            return 1;
        }
        let mut core = Core { cpu: self, bus };
        let opcode = core.fetch();
        core.cpu.cyclesUsed = CYCLES_PER_OPCODE[usize::from(opcode)];
        if matches!(opcode & 0xf, 1..=3)
            || matches!(
                opcode,
                0x00 | 0x0a
                    | 0x2a
                    | 0x4a
                    | 0x6a
                    | 0x8a
                    | 0xaa
                    | 0xca
                    | 0xea
                    | 0x0e
                    | 0x4e
                    | 0x10
                    | 0x30
                    | 0x50
                    | 0x70
                    | 0x90
                    | 0xb0
                    | 0xd0
                    | 0xf0
                    | 0x2e
                    | 0xde
                    | 0x6e
                    | 0xfe
                    | 0x2f
                    | 0x0f
                    | 0x1f
                    | 0x3f
                    | 0x4f
                    | 0x5f
                    | 0x6f
                    | 0x7f
                    | 0xef
                    | 0xff
                    | 0x0d
                    | 0x2d
                    | 0x4d
                    | 0x6d
                    | 0x8e
                    | 0xae
                    | 0xce
                    | 0xee
                    | 0x20
                    | 0x40
                    | 0x60
                    | 0x80
                    | 0xa0
                    | 0xc0
                    | 0xe0
                    | 0xed
            )
        {
            core.control_opcode(opcode);
        } else if matches!(
            opcode,
            0x1a | 0x3a | 0x5a | 0x7a | 0x9a | 0x9e | 0xcf | 0xbe | 0xdf | 0x9f
        ) {
            core.word_opcode(opcode);
        } else if opcode < 0xc0 && matches!(opcode & 0xf, 4..=9) {
            core.alu_opcode(opcode);
        } else if opcode < 0xc0 && matches!(opcode & 0x1f, 0x0b | 0x0c | 0x1b) {
            core.shift_opcode(opcode);
        } else {
            core.other_opcode(opcode);
        }
        core.cpu.cyclesUsed
    }
}

// The retained backend owns separate stable SPC/APU allocations on one thread.
// These adapters require a valid SPC with its live APU link. APU bus callbacks
// mutate RAM/registers/DSP only; they neither borrow nor reset the SPC registers.
pub unsafe fn spc_reset(spc: *mut Spc) {
    unsafe {
        let apu = (*spc).apu;
        (*spc).reset(|operation| match operation {
            BusOperation::Read(address) => apu_cpuRead(apu, address),
            BusOperation::Write(address, value) => {
                apu_cpuWrite(apu, address, value);
                0
            }
        });
    }
}

pub unsafe fn spc_runOpcode(spc: *mut Spc) -> i32 {
    unsafe {
        let apu = (*spc).apu;
        i32::from((*spc).run_opcode(|operation| match operation {
            BusOperation::Read(address) => apu_cpuRead(apu, address),
            BusOperation::Write(address, value) => {
                apu_cpuWrite(apu, address, value);
                0
            }
        }))
    }
}

pub unsafe fn spc_saveload(spc: *mut Spc, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        crate::rtl::snapshot_result(io.state(&mut *spc));
    }
}
