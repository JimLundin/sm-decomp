use crate::game_state::Game;
use crate::snes::apu::{apu_cpuRead, apu_cpuWrite};
extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);

}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int8_t, size_t, uint16_t,
    uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, Dsp, DspChannel, DspRegWriteHistory,
    SaveLoadFunc, Spc, Timer,
};
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
static cyclesPerOpcode: [::core::ffi::c_int; 256] = [
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
unsafe fn spc_read(mut spc: *mut Spc, mut adr: uint16_t) -> uint8_t {
    apu_cpuRead((*spc).apu, adr)
}
unsafe fn spc_write(mut spc: *mut Spc, mut adr: uint16_t, mut val: uint8_t) {
    apu_cpuWrite((*spc).apu, adr, val);
}
pub unsafe fn spc_init(mut apu: *mut Apu) -> *mut Spc {
    let mut spc: *mut Spc = malloc(::core::mem::size_of::<Spc>() as size_t) as *mut Spc;
    (*spc).apu = apu;
    spc
}
pub unsafe fn spc_free(mut spc: *mut Spc) {
    free(spc as *mut ::core::ffi::c_void);
}
pub unsafe fn spc_reset(mut spc: *mut Spc) {
    (*spc).a = 0 as uint8_t;
    (*spc).x = 0 as uint8_t;
    (*spc).y = 0 as uint8_t;
    (*spc).sp = 0 as uint8_t;
    (*spc).pc = (spc_read(spc, 0xfffe as uint16_t) as ::core::ffi::c_int
        | (spc_read(spc, 0xffff as uint16_t) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
    (*spc).c = false_0 != 0;
    (*spc).z = false_0 != 0;
    (*spc).v = false_0 != 0;
    (*spc).n = false_0 != 0;
    (*spc).i = false_0 != 0;
    (*spc).h = false_0 != 0;
    (*spc).p = false_0 != 0;
    (*spc).b = false_0 != 0;
    (*spc).stopped = false_0 != 0;
    (*spc).cyclesUsed = 0 as uint8_t;
}
pub unsafe fn spc_saveload(
    mut spc: *mut Spc,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*spc).a as *mut ::core::ffi::c_void,
        (23 as size_t).wrapping_sub(8 as size_t),
        game,
    );
}
pub unsafe fn spc_runOpcode(mut spc: *mut Spc) -> ::core::ffi::c_int {
    (*spc).cyclesUsed = 0 as uint8_t;
    if (*spc).stopped {
        return 1 as ::core::ffi::c_int;
    }
    let mut opcode: uint8_t = spc_readOpcode(spc);
    (*spc).cyclesUsed = cyclesPerOpcode[opcode as usize] as uint8_t;
    spc_doOpcode(spc, opcode);
    (*spc).cyclesUsed as ::core::ffi::c_int
}
unsafe fn spc_readOpcode(mut spc: *mut Spc) -> uint8_t {
    let fresh0 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    spc_read(spc, fresh0)
}
unsafe fn spc_readOpcodeWord(mut spc: *mut Spc) -> uint16_t {
    let mut low: uint8_t = spc_readOpcode(spc);
    (low as ::core::ffi::c_int
        | (spc_readOpcode(spc) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_getFlags(spc: *mut Spc) -> u8 {
    (u8::from((*spc).n) << 7)
        | (u8::from((*spc).v) << 6)
        | (u8::from((*spc).p) << 5)
        | (u8::from((*spc).b) << 4)
        | (u8::from((*spc).h) << 3)
        | (u8::from((*spc).i) << 2)
        | (u8::from((*spc).z) << 1)
        | u8::from((*spc).c)
}
unsafe fn spc_setFlags(spc: *mut Spc, val: u8) {
    (*spc).n = val & 0x80 != 0;
    (*spc).v = val & 0x40 != 0;
    (*spc).p = val & 0x20 != 0;
    (*spc).b = val & 0x10 != 0;
    (*spc).h = val & 0x08 != 0;
    (*spc).i = val & 0x04 != 0;
    (*spc).z = val & 0x02 != 0;
    (*spc).c = val & 0x01 != 0;
}
unsafe fn spc_setZN(spc: *mut Spc, value: u8) {
    (*spc).z = value == 0;
    (*spc).n = value & 0x80 != 0;
}
unsafe fn spc_doBranch(mut spc: *mut Spc, mut value: uint8_t, mut check: bool) {
    if check {
        (*spc).cyclesUsed =
            ((*spc).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        (*spc).pc =
            ((*spc).pc as ::core::ffi::c_int + value as int8_t as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe fn spc_pullByte(spc: *mut Spc) -> u8 {
    (*spc).sp = (*spc).sp.wrapping_add(1);
    spc_read(spc, 0x100 | u16::from((*spc).sp))
}
unsafe fn spc_pushByte(spc: *mut Spc, value: u8) {
    spc_write(spc, 0x100 | u16::from((*spc).sp), value);
    (*spc).sp = (*spc).sp.wrapping_sub(1);
}
unsafe fn spc_pullWord(spc: *mut Spc) -> u16 {
    let low = spc_pullByte(spc);
    let high = spc_pullByte(spc);
    u16::from_le_bytes([low, high])
}
unsafe fn spc_pushWord(spc: *mut Spc, value: u16) {
    let [low, high] = value.to_le_bytes();
    spc_pushByte(spc, high);
    spc_pushByte(spc, low);
}
unsafe fn spc_readWord(spc: *mut Spc, adrl: u16, adrh: u16) -> u16 {
    let low = spc_read(spc, adrl);
    let high = spc_read(spc, adrh);
    u16::from_le_bytes([low, high])
}
unsafe fn spc_writeWord(spc: *mut Spc, adrl: u16, adrh: u16, value: u16) {
    let [low, high] = value.to_le_bytes();
    spc_write(spc, adrl, low);
    spc_write(spc, adrh, high);
}
unsafe fn spc_adrDp(mut spc: *mut Spc) -> uint16_t {
    (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrAbs(mut spc: *mut Spc) -> uint16_t {
    spc_readOpcodeWord(spc)
}
unsafe fn spc_adrInd(mut spc: *mut Spc) -> uint16_t {
    ((*spc).x as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t
}
unsafe fn spc_adrIdx(mut spc: *mut Spc) -> uint16_t {
    let mut pointer: uint8_t = spc_readOpcode(spc);
    spc_readWord(
        spc,
        (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
        (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
    )
}
unsafe fn spc_adrImm(mut spc: *mut Spc) -> uint16_t {
    let fresh1 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    fresh1
}
unsafe fn spc_adrDpx(mut spc: *mut Spc) -> uint16_t {
    (spc_readOpcode(spc) as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrDpy(mut spc: *mut Spc) -> uint16_t {
    (spc_readOpcode(spc) as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrAbx(mut spc: *mut Spc) -> uint16_t {
    (spc_readOpcodeWord(spc) as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrAby(mut spc: *mut Spc) -> uint16_t {
    (spc_readOpcodeWord(spc) as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrIdy(mut spc: *mut Spc) -> uint16_t {
    let mut pointer: uint8_t = spc_readOpcode(spc);
    let mut adr: uint16_t = spc_readWord(
        spc,
        (pointer as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
        (pointer as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
    );
    (adr as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
        as uint16_t
}
unsafe fn spc_adrDpDp(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    *src = (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
    (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrDpImm(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    let fresh3 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    *src = fresh3;
    (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrIndInd(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    *src = ((*spc).y as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
    ((*spc).x as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t
}
unsafe fn spc_adrAbsBit(mut spc: *mut Spc, mut adr: *mut uint16_t) -> uint8_t {
    let mut adrBit: uint16_t = spc_readOpcodeWord(spc);
    *adr = (adrBit as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int) as uint16_t;
    (adrBit as ::core::ffi::c_int >> 13 as ::core::ffi::c_int) as uint8_t
}
unsafe fn spc_adrDpWord(mut spc: *mut Spc, mut low: *mut uint16_t) -> uint16_t {
    let mut adr: uint8_t = spc_readOpcode(spc);
    *low = (adr as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
    (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t
}
unsafe fn spc_adrIndP(mut spc: *mut Spc) -> uint16_t {
    let fresh2 = (*spc).x;
    (*spc).x = (*spc).x.wrapping_add(1);
    (fresh2 as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t
}
// OR, AND and EOR share addressing forms; opcode bits 5..6 select the logic.
fn spc_logic_value(opcode: u8, left: u8, right: u8) -> u8 {
    match opcode & 0x60 {
        0x00 => left | right,
        0x20 => left & right,
        0x40 => left ^ right,
        _ => unreachable!("invalid SPC logical opcode"),
    }
}
unsafe fn spc_logic_accumulator(spc: *mut Spc, address: u16, opcode: u8) {
    let left = (*spc).a;
    let right = spc_read(spc, address);
    (*spc).a = spc_logic_value(opcode, left, right);
    spc_setZN(spc, (*spc).a);
}
unsafe fn spc_logic_memory(spc: *mut Spc, destination: u16, source: u16, opcode: u8) {
    let right = spc_read(spc, source);
    let left = spc_read(spc, destination);
    let result = spc_logic_value(opcode, left, right);
    spc_write(spc, destination, result);
    spc_setZN(spc, result);
}
// Subtraction passes the complemented right operand through the same adder.
unsafe fn spc_add_with_carry(spc: *mut Spc, left: u8, right: u8) -> u8 {
    let result = u16::from(left) + u16::from(right) + u16::from((*spc).c);
    (*spc).v = left & 0x80 == right & 0x80 && u16::from(right & 0x80) != result & 0x80;
    (*spc).h = u16::from(left & 0x0f) + u16::from(right & 0x0f) + u16::from((*spc).c) > 0x0f;
    (*spc).c = result > 0xff;
    result as u8
}
unsafe fn spc_arithmetic_accumulator(spc: *mut Spc, address: u16, subtract: bool) {
    let value = spc_read(spc, address);
    let right = if subtract { !value } else { value };
    let result = spc_add_with_carry(spc, (*spc).a, right);
    (*spc).a = result;
    spc_setZN(spc, (*spc).a);
}
unsafe fn spc_arithmetic_memory(spc: *mut Spc, destination: u16, source: u16, subtract: bool) {
    let value = spc_read(spc, source);
    let right = if subtract { !value } else { value };
    let left = spc_read(spc, destination);
    let result = spc_add_with_carry(spc, left, right);
    spc_write(spc, destination, result);
    spc_setZN(spc, result);
}
unsafe fn spc_compare_flags(spc: *mut Spc, left: u8, right: u8) {
    (*spc).c = left >= right;
    spc_setZN(spc, left.wrapping_sub(right));
}
unsafe fn spc_compare_register<const REGISTER: u8>(spc: *mut Spc, address: u16) {
    let right = spc_read(spc, address);
    let left = match REGISTER {
        0 => (*spc).a,
        1 => (*spc).x,
        2 => (*spc).y,
        _ => unreachable!("invalid SPC comparison register"),
    };
    spc_compare_flags(spc, left, right);
}
unsafe fn spc_compare_memory(spc: *mut Spc, destination: u16, source: u16) {
    let right = spc_read(spc, source);
    let left = spc_read(spc, destination);
    spc_compare_flags(spc, left, right);
}
// Loads and stores both read the operand address; stores retain the dummy read.
unsafe fn spc_move_memory<const REGISTER: u8>(spc: *mut Spc, address: u16, store: bool) {
    let value = spc_read(spc, address);
    if store {
        let register = match REGISTER {
            0 => (*spc).a,
            1 => (*spc).x,
            2 => (*spc).y,
            _ => unreachable!("invalid SPC move register"),
        };
        spc_write(spc, address, register);
    } else {
        match REGISTER {
            0 => (*spc).a = value,
            1 => (*spc).x = value,
            2 => (*spc).y = value,
            _ => unreachable!("invalid SPC move register"),
        }
        spc_setZN(spc, value);
    }
}
// Return the shifted byte and outgoing carry; callers retain write order.
fn spc_shift<const RIGHT: bool, const ROTATE: bool>(value: u8, carry: bool) -> (u8, bool) {
    let incoming = u8::from(ROTATE && carry);
    if RIGHT {
        ((value >> 1) | (incoming << 7), value & 1 != 0)
    } else {
        ((value << 1) | incoming, value & 0x80 != 0)
    }
}

unsafe fn spc_shift_memory<const RIGHT: bool, const ROTATE: bool>(spc: *mut Spc, address: u16) {
    let value = spc_read(spc, address);
    let (result, carry) = spc_shift::<RIGHT, ROTATE>(value, if ROTATE { (*spc).c } else { false });
    (*spc).c = carry;
    spc_write(spc, address, result);
    spc_setZN(spc, result);
}
unsafe fn spc_inc(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe fn spc_dec(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe fn spc_doOpcode(mut spc: *mut Spc, mut opcode: uint8_t) {
    match opcode as ::core::ffi::c_int {
        1 | 17 | 33 | 49 | 65 | 81 | 97 | 113 | 129 | 145 | 161 | 177 | 193 | 209 | 225 | 241 => {
            spc_pushWord(spc, (*spc).pc);
            let mut adr: uint16_t = (0xffde as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                    * (opcode as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                as uint16_t;
            (*spc).pc = spc_readWord(
                spc,
                adr,
                (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t,
            );
        }
        2 | 18 | 34 | 50 | 66 | 82 | 98 | 114 | 130 | 146 | 162 | 178 | 194 | 210 | 226 | 242 => {
            let address = spc_adrDp(spc);
            let value = spc_read(spc, address);
            let mask = 1u8 << (opcode >> 5);
            let result = if opcode & 0x10 == 0 {
                value | mask
            } else {
                value & !mask
            };
            spc_write(spc, address, result);
        }
        3 | 19 | 35 | 51 | 67 | 83 | 99 | 115 | 131 | 147 | 163 | 179 | 195 | 211 | 227 | 243 => {
            let value = spc_read(spc, spc_adrDp(spc));
            let displacement = spc_readOpcode(spc);
            let bit_set = value & (1u8 << (opcode >> 5)) != 0;
            // Bit 4 selects branch-if-clear instead of branch-if-set.
            spc_doBranch(spc, displacement, bit_set == (opcode & 0x10 == 0));
        }
        4 | 36 | 68 => {
            spc_logic_accumulator(spc, spc_adrDp(spc), opcode);
        }
        5 | 37 | 69 => {
            spc_logic_accumulator(spc, spc_adrAbs(spc), opcode);
        }
        6 | 38 | 70 => {
            spc_logic_accumulator(spc, spc_adrInd(spc), opcode);
        }
        7 | 39 | 71 => {
            spc_logic_accumulator(spc, spc_adrIdx(spc), opcode);
        }
        8 | 40 | 72 => {
            spc_logic_accumulator(spc, spc_adrImm(spc), opcode);
        }
        9 | 41 | 73 => {
            let mut src: uint16_t = 0 as uint16_t;
            let mut dst: uint16_t = spc_adrDpDp(spc, &raw mut src);
            spc_logic_memory(spc, dst, src, opcode);
        }
        10 => {
            let mut adr_2: uint16_t = 0 as uint16_t;
            let mut bit: uint8_t = spc_adrAbsBit(spc, &raw mut adr_2);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                | spc_read(spc, adr_2) as ::core::ffi::c_int >> bit as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        11 => {
            spc_shift_memory::<false, false>(spc, spc_adrDp(spc));
        }
        12 => {
            spc_shift_memory::<false, false>(spc, spc_adrAbs(spc));
        }
        13 => {
            spc_pushByte(spc, spc_getFlags(spc));
        }
        14 => {
            let mut adr_3: uint16_t = spc_adrAbs(spc);
            let mut val_1: uint8_t = spc_read(spc, adr_3);
            let mut result: uint8_t = ((*spc).a as ::core::ffi::c_int
                + (val_1 as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, result);
            spc_write(
                spc,
                adr_3,
                (val_1 as ::core::ffi::c_int | (*spc).a as ::core::ffi::c_int) as uint8_t,
            );
        }
        15 => {
            spc_pushWord(spc, (*spc).pc);
            spc_pushByte(spc, spc_getFlags(spc));
            (*spc).i = false_0 != 0;
            (*spc).b = true_0 != 0;
            (*spc).pc = spc_readWord(spc, 0xffde as uint16_t, 0xffdf as uint16_t);
        }
        16 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).n);
        }
        20 | 52 | 84 => {
            spc_logic_accumulator(spc, spc_adrDpx(spc), opcode);
        }
        21 | 53 | 85 => {
            spc_logic_accumulator(spc, spc_adrAbx(spc), opcode);
        }
        22 | 54 | 86 => {
            spc_logic_accumulator(spc, spc_adrAby(spc), opcode);
        }
        23 | 55 | 87 => {
            spc_logic_accumulator(spc, spc_adrIdy(spc), opcode);
        }
        24 | 56 | 88 => {
            let mut src_0: uint16_t = 0 as uint16_t;
            let mut dst_0: uint16_t = spc_adrDpImm(spc, &raw mut src_0);
            spc_logic_memory(spc, dst_0, src_0, opcode);
        }
        25 | 57 | 89 => {
            let mut src_1: uint16_t = 0 as uint16_t;
            let mut dst_1: uint16_t = spc_adrIndInd(spc, &raw mut src_1);
            spc_logic_memory(spc, dst_1, src_1, opcode);
        }
        26 => {
            let mut low: uint16_t = 0 as uint16_t;
            let mut high: uint16_t = spc_adrDpWord(spc, &raw mut low);
            let mut value: uint16_t = (spc_readWord(spc, low, high) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16_t;
            (*spc).z = value as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = value as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
            spc_writeWord(spc, low, high, value);
        }
        27 => {
            spc_shift_memory::<false, false>(spc, spc_adrDpx(spc));
        }
        28 => {
            let (result, carry) = spc_shift::<false, false>((*spc).a, false);
            (*spc).c = carry;
            (*spc).a = result;
            spc_setZN(spc, (*spc).a);
        }
        29 => {
            (*spc).x = (*spc).x.wrapping_sub(1);
            spc_setZN(spc, (*spc).x);
        }
        30 => {
            spc_compare_register::<1>(spc, spc_adrAbs(spc));
        }
        31 => {
            let mut pointer: uint16_t = spc_readOpcodeWord(spc);
            (*spc).pc = spc_readWord(
                spc,
                (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
                    & 0xffff as ::core::ffi::c_int) as uint16_t,
                (pointer as ::core::ffi::c_int
                    + (*spc).x as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    & 0xffff as ::core::ffi::c_int) as uint16_t,
            );
        }
        32 => {
            (*spc).p = false_0 != 0;
        }
        42 => {
            let mut adr_4: uint16_t = 0 as uint16_t;
            let mut bit_0: uint8_t = spc_adrAbsBit(spc, &raw mut adr_4);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                | !(spc_read(spc, adr_4) as ::core::ffi::c_int >> bit_0 as ::core::ffi::c_int)
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        43 => {
            spc_shift_memory::<false, true>(spc, spc_adrDp(spc));
        }
        44 => {
            spc_shift_memory::<false, true>(spc, spc_adrAbs(spc));
        }
        45 => {
            spc_pushByte(spc, (*spc).a);
        }
        46 => {
            let mut val_2: uint8_t = (spc_read(spc, spc_adrDp(spc)) as ::core::ffi::c_int
                ^ 0xff as ::core::ffi::c_int) as uint8_t;
            let mut result_0: uint8_t = ((*spc).a as ::core::ffi::c_int
                + val_2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        47 => {
            // BRA is relative to PC after consuming its displacement byte.
            let displacement = spc_readOpcode(spc) as i8;
            (*spc).pc = (*spc).pc.wrapping_add(displacement as u16);
        }
        48 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).n);
        }
        58 => {
            let mut low_0: uint16_t = 0 as uint16_t;
            let mut high_0: uint16_t = spc_adrDpWord(spc, &raw mut low_0);
            let mut value_0: uint16_t = (spc_readWord(spc, low_0, high_0) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16_t;
            (*spc).z = value_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
            spc_writeWord(spc, low_0, high_0, value_0);
        }
        59 => {
            spc_shift_memory::<false, true>(spc, spc_adrDpx(spc));
        }
        60 => {
            let (result, carry) = spc_shift::<false, true>((*spc).a, (*spc).c);
            (*spc).a = result;
            (*spc).c = carry;
            spc_setZN(spc, (*spc).a);
        }
        61 => {
            (*spc).x = (*spc).x.wrapping_add(1);
            spc_setZN(spc, (*spc).x);
        }
        62 => {
            spc_compare_register::<1>(spc, spc_adrDp(spc));
        }
        63 => {
            let mut dst_5: uint16_t = spc_readOpcodeWord(spc);
            spc_pushWord(spc, (*spc).pc);
            (*spc).pc = dst_5;
        }
        64 => {
            (*spc).p = true_0 != 0;
        }
        74 => {
            let mut adr_5: uint16_t = 0 as uint16_t;
            let mut bit_1: uint8_t = spc_adrAbsBit(spc, &raw mut adr_5);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                & (spc_read(spc, adr_5) as ::core::ffi::c_int >> bit_1 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int)
                != 0;
        }
        75 => {
            spc_shift_memory::<true, false>(spc, spc_adrDp(spc));
        }
        76 => {
            spc_shift_memory::<true, false>(spc, spc_adrAbs(spc));
        }
        77 => {
            spc_pushByte(spc, (*spc).x);
        }
        78 => {
            let mut adr_6: uint16_t = spc_adrAbs(spc);
            let mut val_3: uint8_t = spc_read(spc, adr_6);
            let mut result_1: uint8_t = ((*spc).a as ::core::ffi::c_int
                + (val_3 as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, result_1);
            spc_write(
                spc,
                adr_6,
                (val_3 as ::core::ffi::c_int & !((*spc).a as ::core::ffi::c_int)) as uint8_t,
            );
        }
        79 => {
            let mut dst_7: uint8_t = spc_readOpcode(spc);
            spc_pushWord(spc, (*spc).pc);
            (*spc).pc = (0xff00 as ::core::ffi::c_int | dst_7 as ::core::ffi::c_int) as uint16_t;
        }
        80 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).v);
        }
        90 => {
            let mut low_1: uint16_t = 0 as uint16_t;
            let mut high_1: uint16_t = spc_adrDpWord(spc, &raw mut low_1);
            let mut value_1: uint16_t = (spc_readWord(spc, low_1, high_1) as ::core::ffi::c_int
                ^ 0xffff as ::core::ffi::c_int) as uint16_t;
            let mut ya: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_2: ::core::ffi::c_int =
                ya as ::core::ffi::c_int + value_1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            (*spc).c = result_2 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_2 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_2 & 0x8000 as ::core::ffi::c_int != 0;
        }
        91 => {
            spc_shift_memory::<true, false>(spc, spc_adrDpx(spc));
        }
        92 => {
            let (result, carry) = spc_shift::<true, false>((*spc).a, false);
            (*spc).c = carry;
            (*spc).a = result;
            spc_setZN(spc, (*spc).a);
        }
        93 => {
            (*spc).x = (*spc).a;
            spc_setZN(spc, (*spc).x);
        }
        94 => {
            spc_compare_register::<2>(spc, spc_adrAbs(spc));
        }
        95 => {
            (*spc).pc = spc_readOpcodeWord(spc);
        }
        96 => {
            (*spc).c = false_0 != 0;
        }
        100 => {
            spc_compare_register::<0>(spc, spc_adrDp(spc));
        }
        101 => {
            spc_compare_register::<0>(spc, spc_adrAbs(spc));
        }
        102 => {
            spc_compare_register::<0>(spc, spc_adrInd(spc));
        }
        103 => {
            spc_compare_register::<0>(spc, spc_adrIdx(spc));
        }
        104 => {
            spc_compare_register::<0>(spc, spc_adrImm(spc));
        }
        105 => {
            let mut src_8: uint16_t = 0 as uint16_t;
            let mut dst_10: uint16_t = spc_adrDpDp(spc, &raw mut src_8);
            spc_compare_memory(spc, dst_10, src_8);
        }
        106 => {
            let mut adr_7: uint16_t = 0 as uint16_t;
            let mut bit_2: uint8_t = spc_adrAbsBit(spc, &raw mut adr_7);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                & (!(spc_read(spc, adr_7) as ::core::ffi::c_int >> bit_2 as ::core::ffi::c_int)
                    & 1 as ::core::ffi::c_int)
                != 0;
        }
        107 => {
            spc_shift_memory::<true, true>(spc, spc_adrDp(spc));
        }
        108 => {
            spc_shift_memory::<true, true>(spc, spc_adrAbs(spc));
        }
        109 => {
            spc_pushByte(spc, (*spc).y);
        }
        110 => {
            let mut adr_8: uint16_t = spc_adrDp(spc);
            let mut result_3: uint8_t =
                (spc_read(spc, adr_8) as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8_t;
            spc_write(spc, adr_8, result_3);
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_3 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        111 => {
            (*spc).pc = spc_pullWord(spc);
        }
        112 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).v);
        }
        116 => {
            spc_compare_register::<0>(spc, spc_adrDpx(spc));
        }
        117 => {
            spc_compare_register::<0>(spc, spc_adrAbx(spc));
        }
        118 => {
            spc_compare_register::<0>(spc, spc_adrAby(spc));
        }
        119 => {
            spc_compare_register::<0>(spc, spc_adrIdy(spc));
        }
        120 => {
            let mut src_9: uint16_t = 0 as uint16_t;
            let mut dst_11: uint16_t = spc_adrDpImm(spc, &raw mut src_9);
            spc_compare_memory(spc, dst_11, src_9);
        }
        121 => {
            let mut src_10: uint16_t = 0 as uint16_t;
            let mut dst_12: uint16_t = spc_adrIndInd(spc, &raw mut src_10);
            spc_compare_memory(spc, dst_12, src_10);
        }
        122 => {
            let mut low_2: uint16_t = 0 as uint16_t;
            let mut high_2: uint16_t = spc_adrDpWord(spc, &raw mut low_2);
            let mut value_2: uint16_t = spc_readWord(spc, low_2, high_2);
            let mut ya_0: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_4: ::core::ffi::c_int =
                ya_0 as ::core::ffi::c_int + value_2 as ::core::ffi::c_int;
            (*spc).v = ya_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == value_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                && value_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != result_4 & 0x8000 as ::core::ffi::c_int;
            (*spc).h = (ya_0 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + (value_2 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int
                > 0xfff as ::core::ffi::c_int;
            (*spc).c = result_4 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_4 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_4 & 0x8000 as ::core::ffi::c_int != 0;
            (*spc).a = (result_4 & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_4 >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        123 => {
            spc_shift_memory::<true, true>(spc, spc_adrDpx(spc));
        }
        124 => {
            let (result, carry) = spc_shift::<true, true>((*spc).a, (*spc).c);
            (*spc).a = result;
            (*spc).c = carry;
            spc_setZN(spc, (*spc).a);
        }
        125 => {
            (*spc).a = (*spc).x;
            spc_setZN(spc, (*spc).a);
        }
        126 => {
            spc_compare_register::<2>(spc, spc_adrDp(spc));
        }
        127 => {
            spc_setFlags(spc, spc_pullByte(spc));
            (*spc).pc = spc_pullWord(spc);
        }
        128 => {
            (*spc).c = true_0 != 0;
        }
        132 | 164 => {
            spc_arithmetic_accumulator(spc, spc_adrDp(spc), opcode & 0x20 != 0);
        }
        133 | 165 => {
            spc_arithmetic_accumulator(spc, spc_adrAbs(spc), opcode & 0x20 != 0);
        }
        134 | 166 => {
            spc_arithmetic_accumulator(spc, spc_adrInd(spc), opcode & 0x20 != 0);
        }
        135 | 167 => {
            spc_arithmetic_accumulator(spc, spc_adrIdx(spc), opcode & 0x20 != 0);
        }
        136 | 168 => {
            spc_arithmetic_accumulator(spc, spc_adrImm(spc), opcode & 0x20 != 0);
        }
        137 | 169 => {
            let mut src_11: uint16_t = 0 as uint16_t;
            let mut dst_13: uint16_t = spc_adrDpDp(spc, &raw mut src_11);
            spc_arithmetic_memory(spc, dst_13, src_11, opcode & 0x20 != 0);
        }
        138 => {
            let mut adr_9: uint16_t = 0 as uint16_t;
            let mut bit_3: uint8_t = spc_adrAbsBit(spc, &raw mut adr_9);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                ^ spc_read(spc, adr_9) as ::core::ffi::c_int >> bit_3 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        139 => {
            spc_dec(spc, spc_adrDp(spc));
        }
        140 => {
            spc_dec(spc, spc_adrAbs(spc));
        }
        141 => {
            spc_move_memory::<2>(spc, spc_adrImm(spc), false);
        }
        142 => {
            spc_setFlags(spc, spc_pullByte(spc));
        }
        143 => {
            let mut src_12: uint16_t = 0 as uint16_t;
            let mut dst_14: uint16_t = spc_adrDpImm(spc, &raw mut src_12);
            let mut val_4: uint8_t = spc_read(spc, src_12);
            spc_read(spc, dst_14);
            spc_write(spc, dst_14, val_4);
        }
        144 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).c);
        }
        148 | 180 => {
            spc_arithmetic_accumulator(spc, spc_adrDpx(spc), opcode & 0x20 != 0);
        }
        149 | 181 => {
            spc_arithmetic_accumulator(spc, spc_adrAbx(spc), opcode & 0x20 != 0);
        }
        150 | 182 => {
            spc_arithmetic_accumulator(spc, spc_adrAby(spc), opcode & 0x20 != 0);
        }
        151 | 183 => {
            spc_arithmetic_accumulator(spc, spc_adrIdy(spc), opcode & 0x20 != 0);
        }
        152 | 184 => {
            let mut src_13: uint16_t = 0 as uint16_t;
            let mut dst_15: uint16_t = spc_adrDpImm(spc, &raw mut src_13);
            spc_arithmetic_memory(spc, dst_15, src_13, opcode & 0x20 != 0);
        }
        153 | 185 => {
            let mut src_14: uint16_t = 0 as uint16_t;
            let mut dst_16: uint16_t = spc_adrIndInd(spc, &raw mut src_14);
            spc_arithmetic_memory(spc, dst_16, src_14, opcode & 0x20 != 0);
        }
        154 => {
            let mut low_3: uint16_t = 0 as uint16_t;
            let mut high_3: uint16_t = spc_adrDpWord(spc, &raw mut low_3);
            let mut value_3: uint16_t = (spc_readWord(spc, low_3, high_3) as ::core::ffi::c_int
                ^ 0xffff as ::core::ffi::c_int) as uint16_t;
            let mut ya_1: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_5: ::core::ffi::c_int = ya_1 as ::core::ffi::c_int
                + value_3 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
            (*spc).v = ya_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == value_3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                && value_3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != result_5 & 0x8000 as ::core::ffi::c_int;
            (*spc).h = (ya_1 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + (value_3 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int
                > 0xfff as ::core::ffi::c_int;
            (*spc).c = result_5 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_5 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_5 & 0x8000 as ::core::ffi::c_int != 0;
            (*spc).a = (result_5 & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_5 >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        155 => {
            spc_dec(spc, spc_adrDpx(spc));
        }
        156 => {
            (*spc).a = (*spc).a.wrapping_sub(1);
            spc_setZN(spc, (*spc).a);
        }
        157 => {
            (*spc).x = (*spc).sp;
            spc_setZN(spc, (*spc).x);
        }
        158 => {
            let mut value_4: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_6: ::core::ffi::c_int = 0xffff as ::core::ffi::c_int;
            let mut mod_0: ::core::ffi::c_int = (*spc).a as ::core::ffi::c_int;
            if (*spc).x as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                result_6 = value_4 as ::core::ffi::c_int / (*spc).x as ::core::ffi::c_int;
                mod_0 = value_4 as ::core::ffi::c_int % (*spc).x as ::core::ffi::c_int;
            }
            (*spc).v = result_6 > 0xff as ::core::ffi::c_int;
            (*spc).h = (*spc).x as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                <= (*spc).y as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
            (*spc).a = result_6 as uint8_t;
            (*spc).y = mod_0 as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        159 => {
            (*spc).a = ((*spc).a as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                | ((*spc).a as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
                as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        160 => {
            (*spc).i = true_0 != 0;
        }
        170 => {
            let mut adr_10: uint16_t = 0 as uint16_t;
            let mut bit_4: uint8_t = spc_adrAbsBit(spc, &raw mut adr_10);
            (*spc).c = spc_read(spc, adr_10) as ::core::ffi::c_int >> bit_4 as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0;
        }
        171 => {
            spc_inc(spc, spc_adrDp(spc));
        }
        172 => {
            spc_inc(spc, spc_adrAbs(spc));
        }
        173 => {
            spc_compare_register::<2>(spc, spc_adrImm(spc));
        }
        174 => {
            (*spc).a = spc_pullByte(spc);
        }
        175 => {
            let mut adr_11: uint16_t = spc_adrIndP(spc);
            spc_write(spc, adr_11, (*spc).a);
        }
        176 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).c);
        }
        186 => {
            let mut low_4: uint16_t = 0 as uint16_t;
            let mut high_4: uint16_t = spc_adrDpWord(spc, &raw mut low_4);
            let mut val_5: uint16_t = spc_readWord(spc, low_4, high_4);
            (*spc).a = (val_5 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (val_5 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
            (*spc).z = val_5 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = val_5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
        }
        187 => {
            spc_inc(spc, spc_adrDpx(spc));
        }
        188 => {
            (*spc).a = (*spc).a.wrapping_add(1);
            spc_setZN(spc, (*spc).a);
        }
        189 => {
            (*spc).sp = (*spc).x;
        }
        190 => {
            if (*spc).a as ::core::ffi::c_int > 0x99 as ::core::ffi::c_int || !(*spc).c {
                (*spc).a = ((*spc).a as ::core::ffi::c_int - 0x60 as ::core::ffi::c_int) as uint8_t;
                (*spc).c = false_0 != 0;
            }
            if (*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int > 9 as ::core::ffi::c_int
                || !(*spc).h
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint8_t;
            }
            spc_setZN(spc, (*spc).a);
        }
        191 => {
            let mut adr_12: uint16_t = spc_adrIndP(spc);
            spc_move_memory::<0>(spc, adr_12, false);
        }
        192 => {
            (*spc).i = false_0 != 0;
        }
        196 | 228 => {
            spc_move_memory::<0>(spc, spc_adrDp(spc), opcode & 0x20 == 0);
        }
        197 | 229 => {
            spc_move_memory::<0>(spc, spc_adrAbs(spc), opcode & 0x20 == 0);
        }
        198 | 230 => {
            spc_move_memory::<0>(spc, spc_adrInd(spc), opcode & 0x20 == 0);
        }
        199 | 231 => {
            spc_move_memory::<0>(spc, spc_adrIdx(spc), opcode & 0x20 == 0);
        }
        200 => {
            spc_compare_register::<1>(spc, spc_adrImm(spc));
        }
        201 | 233 => {
            spc_move_memory::<1>(spc, spc_adrAbs(spc), opcode & 0x20 == 0);
        }
        202 => {
            let mut adr_13: uint16_t = 0 as uint16_t;
            let mut bit_5: uint8_t = spc_adrAbsBit(spc, &raw mut adr_13);
            let mut result_7: uint8_t = (spc_read(spc, adr_13) as ::core::ffi::c_int
                & !((1 as ::core::ffi::c_int) << bit_5 as ::core::ffi::c_int)
                | ((*spc).c as ::core::ffi::c_int) << bit_5 as ::core::ffi::c_int)
                as uint8_t;
            spc_write(spc, adr_13, result_7);
        }
        203 | 235 => {
            spc_move_memory::<2>(spc, spc_adrDp(spc), opcode & 0x20 == 0);
        }
        204 | 236 => {
            spc_move_memory::<2>(spc, spc_adrAbs(spc), opcode & 0x20 == 0);
        }
        205 => {
            spc_move_memory::<1>(spc, spc_adrImm(spc), false);
        }
        206 => {
            (*spc).x = spc_pullByte(spc);
        }
        207 => {
            let mut result_8: uint16_t =
                ((*spc).a as ::core::ffi::c_int * (*spc).y as ::core::ffi::c_int) as uint16_t;
            (*spc).a = (result_8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_8 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, (*spc).y);
        }
        208 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).z);
        }
        212 | 244 => {
            spc_move_memory::<0>(spc, spc_adrDpx(spc), opcode & 0x20 == 0);
        }
        213 | 245 => {
            spc_move_memory::<0>(spc, spc_adrAbx(spc), opcode & 0x20 == 0);
        }
        214 | 246 => {
            spc_move_memory::<0>(spc, spc_adrAby(spc), opcode & 0x20 == 0);
        }
        215 | 247 => {
            spc_move_memory::<0>(spc, spc_adrIdy(spc), opcode & 0x20 == 0);
        }
        216 | 248 => {
            spc_move_memory::<1>(spc, spc_adrDp(spc), opcode & 0x20 == 0);
        }
        217 | 249 => {
            spc_move_memory::<1>(spc, spc_adrDpy(spc), opcode & 0x20 == 0);
        }
        218 => {
            let mut low_5: uint16_t = 0 as uint16_t;
            let mut high_5: uint16_t = spc_adrDpWord(spc, &raw mut low_5);
            spc_read(spc, low_5);
            spc_write(spc, low_5, (*spc).a);
            spc_write(spc, high_5, (*spc).y);
        }
        219 | 251 => {
            spc_move_memory::<2>(spc, spc_adrDpx(spc), opcode & 0x20 == 0);
        }
        220 => {
            (*spc).y = (*spc).y.wrapping_sub(1);
            spc_setZN(spc, (*spc).y);
        }
        221 => {
            (*spc).a = (*spc).y;
            spc_setZN(spc, (*spc).a);
        }
        222 => {
            let mut val_6: uint8_t = (spc_read(spc, spc_adrDpx(spc)) as ::core::ffi::c_int
                ^ 0xff as ::core::ffi::c_int) as uint8_t;
            let mut result_9: uint8_t = ((*spc).a as ::core::ffi::c_int
                + val_6 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_9 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        223 => {
            if (*spc).a as ::core::ffi::c_int > 0x99 as ::core::ffi::c_int
                || (*spc).c as ::core::ffi::c_int != 0
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int + 0x60 as ::core::ffi::c_int) as uint8_t;
                (*spc).c = true_0 != 0;
            }
            if (*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int > 9 as ::core::ffi::c_int
                || (*spc).h as ::core::ffi::c_int != 0
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint8_t;
            }
            spc_setZN(spc, (*spc).a);
        }
        224 => {
            (*spc).v = false_0 != 0;
            (*spc).h = false_0 != 0;
        }
        232 => {
            spc_move_memory::<0>(spc, spc_adrImm(spc), false);
        }
        234 => {
            let mut adr_14: uint16_t = 0 as uint16_t;
            let mut bit_6: uint8_t = spc_adrAbsBit(spc, &raw mut adr_14);
            let mut result_10: uint8_t = (spc_read(spc, adr_14) as ::core::ffi::c_int
                ^ (1 as ::core::ffi::c_int) << bit_6 as ::core::ffi::c_int)
                as uint8_t;
            spc_write(spc, adr_14, result_10);
        }
        237 => {
            (*spc).c = !(*spc).c;
        }
        238 => {
            (*spc).y = spc_pullByte(spc);
        }
        239 => {
            (*spc).stopped = true_0 != 0;
        }
        240 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).z);
        }
        250 => {
            let mut src_18: uint16_t = 0 as uint16_t;
            let mut dst_20: uint16_t = spc_adrDpDp(spc, &raw mut src_18);
            let mut val_7: uint8_t = spc_read(spc, src_18);
            spc_write(spc, dst_20, val_7);
        }
        252 => {
            (*spc).y = (*spc).y.wrapping_add(1);
            spc_setZN(spc, (*spc).y);
        }
        253 => {
            (*spc).y = (*spc).a;
            spc_setZN(spc, (*spc).y);
        }
        254 => {
            (*spc).y = (*spc).y.wrapping_sub(1);
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                (*spc).y as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        255 => {
            (*spc).stopped = true_0 != 0;
        }
        0 | _ => {}
    };
}
