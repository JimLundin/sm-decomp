use crate::cpu_infra::{CpuOpcodeHook, HookedFunctionRts};
use crate::game_state::Game;
use crate::host::Die;
use crate::snes::snes::{snes_cpuRead, snes_cpuWrite};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: usize) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: usize,
    ) -> *mut ::core::ffi::c_void;

    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
use crate::types::{Cpu, SaveLoadFunc, Snes};
static CYCLES_PER_OPCODE: [u8; 256] = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5, 2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5, 2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5, 2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
];
unsafe fn read_byte(cpu: *mut Cpu, adr: u32, game: &mut Game) -> u8 {
    return snes_cpuRead((*cpu).mem as *mut Snes, adr, game);
}
unsafe fn write_byte(cpu: *mut Cpu, adr: u32, val: u8, game: &mut Game) {
    snes_cpuWrite((*cpu).mem as *mut Snes, adr, val, game);
}
pub unsafe fn cpu_init(mem: *mut ::core::ffi::c_void, memType: ::core::ffi::c_int) -> *mut Cpu {
    let cpu: *mut Cpu = malloc(::core::mem::size_of::<Cpu>()) as *mut Cpu;
    memset(
        cpu as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<Cpu>(),
    );
    (*cpu).mem = mem;
    (*cpu).memType = memType as u8;
    return cpu;
}
pub unsafe fn cpu_free(cpu: *mut Cpu) {
    free(cpu as *mut ::core::ffi::c_void);
}
fn finish_reset(cpu: &mut Cpu) {
    cpu.dp = 0;
    cpu.k = 0;
    cpu.db = 0;
    cpu.c = false;
    cpu.z = false;
    cpu.v = false;
    cpu.n = false;
    cpu.i = true;
    cpu.d = false;
    cpu.xf = true;
    cpu.mf = true;
    cpu.e = true;
    cpu.irqWanted = false;
    cpu.nmiWanted = false;
    cpu.waiting = false;
    cpu.stopped = false;
    cpu.cyclesUsed = 0;
}
pub unsafe fn cpu_reset(cpu: *mut Cpu, game: &mut Game) {
    (*cpu).a = 0;
    (*cpu).x = 0;
    (*cpu).y = 0;
    (*cpu).sp = 0x100;
    (*cpu).pc = read_word(cpu, 0xfffc, 0xfffd, game);
    finish_reset(&mut *cpu);
}
pub unsafe fn cpu_saveload(
    cpu: *mut Cpu,
    func: Option<SaveLoadFunc>,
    ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*cpu).a as *mut ::core::ffi::c_void,
        27,
        game,
    );
    (*cpu).spBreakpoint = 0;
}
pub unsafe fn cpu_runOpcode(cpu: *mut Cpu, game: &mut Game) -> ::core::ffi::c_int {
    (*cpu).cyclesUsed = 0;
    if (*cpu).stopped {
        return 1;
    }
    if (*cpu).waiting {
        if !(*cpu).irqWanted && !(*cpu).nmiWanted {
            return 1;
        }
        // IRQ wakes WAI even when I prevents interrupt service.
        (*cpu).waiting = false;
    }
    if (!(*cpu).i && (*cpu).irqWanted) || (*cpu).nmiWanted {
        (*cpu).cyclesUsed = 7;
        if (*cpu).nmiWanted {
            (*cpu).nmiWanted = false;
            service_interrupt(cpu, false, game);
        } else {
            service_interrupt(cpu, true, game);
        }
    }
    let opcode = fetch_byte(cpu, game);
    (*cpu).cyclesUsed = (*cpu)
        .cyclesUsed
        .wrapping_add(CYCLES_PER_OPCODE[usize::from(opcode)]);
    dispatch_opcode(cpu, opcode, game);
    i32::from((*cpu).cyclesUsed)
}
fn next_program_address(cpu: &mut Cpu) -> u32 {
    let pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    u32::from(cpu.k) << 16 | u32::from(pc)
}
unsafe fn fetch_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    let address = next_program_address(&mut *cpu);
    read_byte(cpu, address, game)
}
unsafe fn fetch_word(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let low = fetch_byte(cpu, game);
    let high = fetch_byte(cpu, game);
    u16::from_le_bytes([low, high])
}
pub fn cpu_getFlags(cpu: &Cpu) -> u8 {
    (u8::from(cpu.n) << 7)
        | (u8::from(cpu.v) << 6)
        | (u8::from(cpu.mf) << 5)
        | (u8::from(cpu.xf) << 4)
        | (u8::from(cpu.d) << 3)
        | (u8::from(cpu.i) << 2)
        | (u8::from(cpu.z) << 1)
        | u8::from(cpu.c)
}
pub fn cpu_setFlags(val: u8, cpu: &mut Cpu) {
    cpu.n = val & 0x80 != 0;
    cpu.v = val & 0x40 != 0;
    cpu.mf = val & 0x20 != 0;
    cpu.xf = val & 0x10 != 0;
    cpu.d = val & 0x08 != 0;
    cpu.i = val & 0x04 != 0;
    cpu.z = val & 0x02 != 0;
    cpu.c = val & 0x01 != 0;
    if cpu.e {
        cpu.mf = true;
        cpu.xf = true;
        cpu.sp = (cpu.sp & 0xff) | 0x100;
    }
    if cpu.xf {
        cpu.x &= 0xff;
        cpu.y &= 0xff;
    }
}
fn exchange_carry_emulation(cpu: &mut Cpu) {
    std::mem::swap(&mut cpu.c, &mut cpu.e);
    cpu_setFlags(cpu_getFlags(cpu), cpu);
}
unsafe fn update_status_bits(cpu: *mut Cpu, set: bool, game: &mut Game) {
    let flags = cpu_getFlags(&*cpu);
    let mask = fetch_byte(cpu, game);
    cpu_setFlags(if set { flags | mask } else { flags & !mask }, &mut *cpu);
}
fn exchange_accumulator_bytes(cpu: &mut Cpu) {
    cpu.a = cpu.a.swap_bytes();
    set_zero_negative(cpu.a, true, cpu);
}
fn set_zero_negative(value: u16, byte: bool, cpu: &mut Cpu) {
    if byte {
        cpu.z = value & 0xff == 0;
        cpu.n = value & 0x80 != 0;
    } else {
        cpu.z = value == 0;
        cpu.n = value & 0x8000 != 0;
    }
}
fn subtract_for_compare(register: u16, value: u16, byte: bool, cpu: &mut Cpu) -> u16 {
    cpu.c = register >= value;
    let result = register.wrapping_sub(value);
    if byte {
        result & 0xff
    } else {
        result
    }
}
fn branch_relative(value: u8, check: bool, cpu: &mut Cpu) {
    if check {
        cpu.cyclesUsed = cpu.cyclesUsed.wrapping_add(1);
        cpu.pc = cpu.pc.wrapping_add_signed(value as i8 as i16);
    }
}
fn adjust_stack_pointer(delta: u16, cpu: &mut Cpu) {
    cpu.sp = cpu.sp.wrapping_add(delta);
    if cpu.e {
        cpu.sp = cpu.sp & 0xff | 0x100;
    }
}
unsafe fn pull_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    adjust_stack_pointer(1, &mut *cpu);
    read_byte(cpu, u32::from((*cpu).sp), game)
}
unsafe fn push_byte(cpu: *mut Cpu, value: u8, game: &mut Game) {
    write_byte(cpu, u32::from((*cpu).sp), value, game);
    adjust_stack_pointer(u16::MAX, &mut *cpu);
}
unsafe fn pull_word(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let low = pull_byte(cpu, game);
    let high = pull_byte(cpu, game);
    u16::from_le_bytes([low, high])
}
unsafe fn push_word(cpu: *mut Cpu, value: u16, game: &mut Game) {
    let [low, high] = value.to_le_bytes();
    push_byte(cpu, high, game);
    push_byte(cpu, low, game);
}
unsafe fn read_word(cpu: *mut Cpu, adrl: u32, adrh: u32, game: &mut Game) -> u16 {
    let low = read_byte(cpu, adrl, game);
    let high = read_byte(cpu, adrh, game);
    u16::from_le_bytes([low, high])
}
unsafe fn write_word(
    cpu: *mut Cpu,
    adrl: u32,
    adrh: u32,
    value: u16,
    reversed: bool,
    game: &mut Game,
) {
    let [low, high] = value.to_le_bytes();
    if reversed {
        write_byte(cpu, adrh, high, game);
        write_byte(cpu, adrl, low, game);
    } else {
        write_byte(cpu, adrl, low, game);
        write_byte(cpu, adrh, high, game);
    }
}
unsafe fn service_interrupt(cpu: *mut Cpu, irq: bool, game: &mut Game) {
    if !(*cpu).e {
        push_byte(cpu, (*cpu).k, game);
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    push_word(cpu, (*cpu).pc, game);
    let flags = cpu_getFlags(&*cpu);
    // Bit 4 is the cleared B flag on an emulation-mode hardware interrupt.
    push_byte(cpu, if (*cpu).e { flags & !0x10 } else { flags }, game);
    (*cpu).i = true;
    (*cpu).d = false;
    (*cpu).k = 0;
    let vector = match ((*cpu).e, irq) {
        (false, false) => 0xffea,
        (false, true) => 0xffee,
        (true, false) => 0xfffa,
        (true, true) => 0xfffe,
    };
    (*cpu).pc = read_word(cpu, vector, vector + 1, game);
}
fn address_immediate(cpu: &mut Cpu, register: CpuRegister) -> (u32, u32) {
    let byte = register.byte(cpu);
    let low = next_program_address(cpu);
    let high = if byte { 0 } else { next_program_address(cpu) };
    (low, high)
}
unsafe fn address_direct_page(
    cpu: *mut Cpu,
    index: Option<IndexRegister>,
    game: &mut Game,
) -> (u32, u32) {
    let offset = u16::from(fetch_byte(cpu, game));
    direct_page_address(offset, index, &mut *cpu)
}
#[derive(Clone, Copy)]
enum IndexRegister {
    X,
    Y,
}
impl IndexRegister {
    fn value(self, cpu: &Cpu) -> u16 {
        match self {
            Self::X => cpu.x,
            Self::Y => cpu.y,
        }
    }
    fn load(self, value: u16, cpu: &mut Cpu) {
        let value = if cpu.xf { value & 0xff } else { value };
        match self {
            Self::X => cpu.x = value,
            Self::Y => cpu.y = value,
        }
        set_zero_negative(value, cpu.xf, cpu);
    }
    fn update(self, delta: u16, cpu: &mut Cpu) {
        self.load(self.value(cpu).wrapping_add(delta), cpu);
    }
}
fn direct_page_address(offset: u16, index: Option<IndexRegister>, cpu: &mut Cpu) -> (u32, u32) {
    if cpu.dp & 0xff != 0 {
        cpu.cyclesUsed = cpu.cyclesUsed.wrapping_add(1);
    }
    let address = cpu.dp.wrapping_add(offset);
    let address = match index {
        Some(register) => address.wrapping_add(register.value(cpu)),
        None => address,
    };
    (u32::from(address), u32::from(address.wrapping_add(1)))
}
unsafe fn read_direct_page_pointer(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let (pointer_low, pointer_high) = address_direct_page(cpu, None, game);
    read_word(cpu, pointer_low, pointer_high, game)
}
unsafe fn address_direct_page_indirect(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let pointer = read_direct_page_pointer(cpu, game);
    data_bank_address(pointer, &*cpu)
}
unsafe fn address_direct_page_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let pointer = u32::from(read_word(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add((*cpu).x)),
        u32::from(
            (*cpu)
                .dp
                .wrapping_add(adr)
                .wrapping_add((*cpu).x)
                .wrapping_add(1),
        ),
        game,
    ));
    let low = (u32::from((*cpu).db) << 16) + pointer;
    (low, ((u32::from((*cpu).db) << 16) + pointer + 1) & 0xffffff)
}
unsafe fn address_direct_page_indirect_y(
    cpu: *mut Cpu,
    write: bool,
    game: &mut Game,
) -> (u32, u32) {
    let pointer = read_direct_page_pointer(cpu, game);
    absolute_indexed_address(u32::from(pointer), IndexRegister::Y, write, &mut *cpu)
}
unsafe fn read_direct_page_long_pointer(cpu: *mut Cpu, game: &mut Game) -> u32 {
    let adr = u16::from(fetch_byte(cpu, game));
    let (pointer_low, pointer_high) = direct_page_address(adr, None, &mut *cpu);
    let mut pointer = u32::from(read_word(cpu, pointer_low, pointer_high, game));
    pointer |= u32::from(read_byte(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(2)),
        game,
    )) << 16;
    pointer
}
unsafe fn address_direct_page_indirect_long(
    cpu: *mut Cpu,
    index: Option<IndexRegister>,
    game: &mut Game,
) -> (u32, u32) {
    let pointer = read_direct_page_long_pointer(cpu, game);
    long_address(pointer, index, &*cpu)
}
unsafe fn address_stack_relative(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    let low = u32::from((*cpu).sp.wrapping_add(adr));
    (low, u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1)))
}
unsafe fn address_stack_relative_indirect_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    let pointer = u32::from(read_word(
        cpu,
        u32::from((*cpu).sp.wrapping_add(adr)),
        u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    let low = ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y)) & 0xffffff;
    (
        low,
        ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y) + 1) & 0xffffff,
    )
}
unsafe fn address_absolute(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let offset = fetch_word(cpu, game);
    data_bank_address(offset, &*cpu)
}
fn data_bank_address(offset: u16, cpu: &Cpu) -> (u32, u32) {
    let low = (u32::from(cpu.db) << 16) | u32::from(offset);
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_indexed(
    cpu: *mut Cpu,
    index: IndexRegister,
    write: bool,
    game: &mut Game,
) -> (u32, u32) {
    let offset = u32::from(fetch_word(cpu, game));
    absolute_indexed_address(offset, index, write, &mut *cpu)
}
fn absolute_indexed_address(
    offset: u32,
    index: IndexRegister,
    write: bool,
    cpu: &mut Cpu,
) -> (u32, u32) {
    if write && (!cpu.xf || offset >> 8 != (offset + u32::from(index.value(cpu))) >> 8) {
        cpu.cyclesUsed = cpu.cyclesUsed.wrapping_add(1);
    }
    let low = ((u32::from(cpu.db) << 16) + offset + u32::from(index.value(cpu))) & 0xffffff;
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn fetch_long_address(cpu: *mut Cpu, game: &mut Game) -> u32 {
    let offset = fetch_word(cpu, game);
    let bank = fetch_byte(cpu, game);
    u32::from(bank) << 16 | u32::from(offset)
}
unsafe fn address_absolute_long(
    cpu: *mut Cpu,
    index: Option<IndexRegister>,
    game: &mut Game,
) -> (u32, u32) {
    let address = fetch_long_address(cpu, game);
    long_address(address, index, &*cpu)
}
fn long_address(address: u32, index: Option<IndexRegister>, cpu: &Cpu) -> (u32, u32) {
    let low = match index {
        Some(register) => address.wrapping_add(u32::from(register.value(cpu))) & 0xffffff,
        None => address,
    };
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let adr = fetch_word(cpu, game);
    let (low, high) = indexed_jump_pointer(adr, &*cpu);
    read_word(cpu, low, high, game)
}
fn indexed_jump_pointer(offset: u16, cpu: &Cpu) -> (u32, u32) {
    let pointer = offset.wrapping_add(cpu.x);
    let bank = u32::from(cpu.k) << 16;
    (
        bank | u32::from(pointer),
        bank | u32::from(pointer.wrapping_add(1)),
    )
}
#[derive(Clone, Copy)]
enum LogicOperation {
    And,
    Or,
    Xor,
}
fn apply_accumulator_logic(value: u16, byte: bool, operation: LogicOperation, cpu: &mut Cpu) {
    let result = match operation {
        LogicOperation::And => cpu.a & value,
        LogicOperation::Or => cpu.a | value,
        LogicOperation::Xor => cpu.a ^ value,
    };
    cpu.a = if byte {
        cpu.a & 0xff00 | result & 0xff
    } else {
        result
    };
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
unsafe fn read_operand(cpu: *mut Cpu, low: u32, high: u32, byte: bool, game: &mut Game) -> u16 {
    if byte {
        u16::from(read_byte(cpu, low, game))
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        read_word(cpu, low, high, game)
    }
}
unsafe fn logical_accumulator(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    operation: LogicOperation,
    game: &mut Game,
) {
    let byte = (*cpu).mf;
    let value = read_operand(cpu, low, high, byte, game);
    apply_accumulator_logic(value, byte, operation, &mut *cpu);
}
fn addition_overflow(register: u16, value: u16, result: i32, byte: bool) -> bool {
    let sign = if byte { 0x80 } else { 0x8000 };
    i32::from(register) & sign == i32::from(value) & sign
        && i32::from(value) & sign != result & sign
}
fn add_to_accumulator(value: u16, byte: bool, cpu: &mut Cpu) {
    if byte {
        let mut result = if cpu.d {
            let mut result = (i32::from(cpu.a) & 0xf) + (i32::from(value) & 0xf) + i32::from(cpu.c);
            if result > 0x9 {
                result = (result + 0x6 & 0xf) + 0x10;
            }
            (i32::from(cpu.a) & 0xf0) + (i32::from(value) & 0xf0) + result
        } else {
            (i32::from(cpu.a) & 0xff) + i32::from(value) + i32::from(cpu.c)
        };
        cpu.v = addition_overflow(cpu.a, value, result, true);
        if cpu.d && result > 0x9f {
            result += 0x60;
        }
        cpu.c = result > 0xff;
        cpu.a = (i32::from(cpu.a) & 0xff00 | result & 0xff) as u16;
    } else {
        let mut result = if cpu.d {
            let mut result = (i32::from(cpu.a) & 0xf) + (i32::from(value) & 0xf) + i32::from(cpu.c);
            if result > 0x9 {
                result = (result + 0x6 & 0xf) + 0x10;
            }
            result = (i32::from(cpu.a) & 0xf0) + (i32::from(value) & 0xf0) + result;
            if result > 0x9f {
                result = (result + 0x60 & 0xff) + 0x100;
            }
            result = (i32::from(cpu.a) & 0xf00) + (i32::from(value) & 0xf00) + result;
            if result > 0x9ff {
                result = (result + 0x600 & 0xfff) + 0x1000;
            }
            (i32::from(cpu.a) & 0xf000) + (i32::from(value) & 0xf000) + result
        } else {
            i32::from(cpu.a) + i32::from(value) + i32::from(cpu.c)
        };
        cpu.v = addition_overflow(cpu.a, value, result, false);
        if cpu.d && result > 0x9fff {
            result += 0x6000;
        }
        cpu.c = result > 0xffff;
        cpu.a = result as u16;
    }
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
unsafe fn add_with_carry(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    let byte = (*cpu).mf;
    let value = read_operand(cpu, low, high, byte, game);
    add_to_accumulator(value, byte, &mut *cpu);
}
fn subtract_from_accumulator(value: u16, byte: bool, cpu: &mut Cpu) {
    if byte {
        let mut result = if cpu.d {
            let mut result = (i32::from(cpu.a) & 0xf) + (i32::from(value) & 0xf) + i32::from(cpu.c);
            if result < 0x10 {
                result = result - 0x6 & (if (result - 0x6) < 0 { 0xf } else { 0x1f });
            }
            (i32::from(cpu.a) & 0xf0) + (i32::from(value) & 0xf0) + result
        } else {
            (i32::from(cpu.a) & 0xff) + i32::from(value) + i32::from(cpu.c)
        };
        cpu.v = addition_overflow(cpu.a, value, result, true);
        if cpu.d && result < 0x100 {
            result -= 0x60;
        }
        cpu.c = result > 0xff;
        cpu.a = (i32::from(cpu.a) & 0xff00 | result & 0xff) as u16;
    } else {
        let mut result = if cpu.d {
            let mut result = (i32::from(cpu.a) & 0xf) + (i32::from(value) & 0xf) + i32::from(cpu.c);
            if result < 0x10 {
                result = result - 0x6 & (if (result - 0x6) < 0 { 0xf } else { 0x1f });
            }
            result = (i32::from(cpu.a) & 0xf0) + (i32::from(value) & 0xf0) + result;
            if result < 0x100 {
                result = result - 0x60 & (if (result - 0x60) < 0 { 0xff } else { 0x1ff });
            }
            result = (i32::from(cpu.a) & 0xf00) + (i32::from(value) & 0xf00) + result;
            if result < 0x1000 {
                result = result - 0x600 & (if (result - 0x600) < 0 { 0xfff } else { 0x1fff });
            }
            (i32::from(cpu.a) & 0xf000) + (i32::from(value) & 0xf000) + result
        } else {
            i32::from(cpu.a) + i32::from(value) + i32::from(cpu.c)
        };
        cpu.v = addition_overflow(cpu.a, value, result, false);
        if cpu.d && result < 0x10000 {
            result -= 0x6000;
        }
        cpu.c = result > 0xffff;
        cpu.a = result as u16;
    }
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
unsafe fn subtract_with_carry(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    let byte = (*cpu).mf;
    let value = read_operand(cpu, low, high, byte, game);
    let value = if byte { value ^ 0xff } else { !value };
    subtract_from_accumulator(value, byte, &mut *cpu);
}
#[derive(Clone, Copy)]
enum CpuRegister {
    Accumulator,
    X,
    Y,
}
impl CpuRegister {
    fn byte(self, cpu: &Cpu) -> bool {
        match self {
            Self::Accumulator => cpu.mf,
            Self::X | Self::Y => cpu.xf,
        }
    }
    fn value(self, cpu: &Cpu) -> u16 {
        match self {
            Self::Accumulator => cpu.a,
            Self::X => cpu.x,
            Self::Y => cpu.y,
        }
    }
    fn load_upper(self, byte: bool, cpu: &Cpu) -> u16 {
        match (self, byte) {
            (Self::Accumulator, true) => cpu.a & 0xff00,
            _ => 0,
        }
    }
    fn load(self, value: u16, cpu: &mut Cpu) {
        match self {
            Self::Accumulator => cpu.a = value,
            Self::X => cpu.x = value,
            Self::Y => cpu.y = value,
        }
        set_zero_negative(value, self.byte(cpu), cpu);
    }
    fn compare(self, value: u16, byte: bool, cpu: &mut Cpu) {
        let lhs = self.value(cpu);
        let lhs = if byte { lhs & 0xff } else { lhs };
        let result = subtract_for_compare(lhs, value, byte, cpu);
        set_zero_negative(result, self.byte(cpu), cpu);
    }
}
unsafe fn compare_register(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    register: CpuRegister,
    game: &mut Game,
) {
    let byte = register.byte(&*cpu);
    let value = read_operand(cpu, low, high, byte, game);
    register.compare(value, byte, &mut *cpu);
}
fn test_bits(value: u16, byte: bool, cpu: &mut Cpu) {
    cpu.z = cpu.a & value == 0;
    cpu.n = value & if byte { 0x80 } else { 0x8000 } != 0;
    cpu.v = value & if byte { 0x40 } else { 0x4000 } != 0;
}
unsafe fn test_accumulator_bits(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    let byte = (*cpu).mf;
    let value = read_operand(cpu, low, high, byte, game);
    test_bits(value, byte, &mut *cpu);
}
unsafe fn load_register(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    register: CpuRegister,
    game: &mut Game,
) {
    let byte = register.byte(&*cpu);
    let upper = register.load_upper(byte, &*cpu);
    let value = upper | read_operand(cpu, low, high, byte, game);
    register.load(value, &mut *cpu);
}
unsafe fn pull_register(cpu: *mut Cpu, register: CpuRegister, game: &mut Game) {
    let byte = register.byte(&*cpu);
    let upper = register.load_upper(byte, &*cpu);
    let value = upper
        | if byte {
            u16::from(pull_byte(cpu, game))
        } else {
            (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
            pull_word(cpu, game)
        };
    register.load(value, &mut *cpu);
}
unsafe fn push_register(cpu: *mut Cpu, register: CpuRegister, game: &mut Game) {
    if register.byte(&*cpu) {
        push_byte(cpu, register.value(&*cpu) as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        push_word(cpu, register.value(&*cpu), game);
    }
}
unsafe fn write_operand(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    value: u16,
    byte: bool,
    game: &mut Game,
) {
    if byte {
        write_byte(cpu, low, value as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        write_word(cpu, low, high, value, false, game);
    }
}
unsafe fn store_register(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    register: CpuRegister,
    game: &mut Game,
) {
    let byte = register.byte(&*cpu);
    let value = register.value(&*cpu);
    write_operand(cpu, low, high, value, byte, game);
}
unsafe fn store_zero(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    write_operand(cpu, low, high, 0, (*cpu).mf, game);
}
unsafe fn rotate_memory_right(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    let (result, carry) = if (*cpu).mf {
        let value = read_byte(cpu, low, game);
        let carry = value & 1 != 0;
        let result = u16::from(value >> 1) | (u16::from((*cpu).c) << 7);
        write_byte(cpu, low, result as u8, game);
        (result, carry)
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = read_word(cpu, low, high, game);
        let carry = value & 1 != 0;
        let result = value >> 1 | (u16::from((*cpu).c) << 15);
        write_word(cpu, low, high, result, true, game);
        (result, carry)
    };
    set_zero_negative(result, (*cpu).mf, &mut *cpu);
    (*cpu).c = carry;
}
fn shifted_left_result(value: u16, byte: bool, rotate: bool, cpu: &mut Cpu) -> u16 {
    let result = value << 1 | u16::from(rotate && cpu.c);
    cpu.c = value & if byte { 0x80 } else { 0x8000 } != 0;
    result
}
fn shifted_right_result(value: u16, byte: bool, cpu: &mut Cpu) -> u16 {
    cpu.c = value & 1 != 0;
    if byte {
        value >> 1 & 0x7f
    } else {
        value >> 1
    }
}
#[derive(Clone, Copy)]
enum MemoryOperation {
    ShiftLeft,
    RotateLeft,
    ShiftRight,
    Increment,
    Decrement,
}
impl MemoryOperation {
    fn apply(self, value: u16, byte: bool, cpu: &mut Cpu) -> u16 {
        match self {
            Self::ShiftLeft => shifted_left_result(value, byte, false, cpu),
            Self::RotateLeft => shifted_left_result(value, byte, true, cpu),
            Self::ShiftRight => shifted_right_result(value, byte, cpu),
            Self::Increment => value.wrapping_add(1),
            Self::Decrement => value.wrapping_sub(1),
        }
    }
}
unsafe fn modify_memory(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    operation: MemoryOperation,
    game: &mut Game,
) {
    let byte = (*cpu).mf;
    let value = if byte {
        u16::from(read_byte(cpu, low, game))
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        read_word(cpu, low, high, game)
    };
    let result = operation.apply(value, byte, &mut *cpu);
    if byte {
        write_byte(cpu, low, result as u8, game);
    } else {
        write_word(cpu, low, high, result, true, game);
    }
    set_zero_negative(result, (*cpu).mf, &mut *cpu);
}
fn tested_bit_result(value: u16, byte: bool, set: bool, cpu: &mut Cpu) -> u16 {
    let accumulator = if byte { cpu.a & 0xff } else { cpu.a };
    cpu.z = accumulator & value == 0;
    if set {
        value | accumulator
    } else {
        value & !accumulator
    }
}
unsafe fn test_and_update_bits(cpu: *mut Cpu, low: u32, high: u32, set: bool, game: &mut Game) {
    if (*cpu).mf {
        let value = read_byte(cpu, low, game);
        let result = tested_bit_result(u16::from(value), true, set, &mut *cpu);
        write_byte(cpu, low, result as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = read_word(cpu, low, high, game);
        let result = tested_bit_result(value, false, set, &mut *cpu);
        write_word(cpu, low, high, result, true, game);
    }
}

pub unsafe fn DumpCpuHistory(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        printf(
            b"PC history: 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            game.pc_hist[(game.pc_hist_ctr.wrapping_add(i as u32) & 7 as u32) as usize],
        );
        i += 1;
    }
}
fn shift_accumulator_left(rotate: bool, cpu: &mut Cpu) {
    let result = shifted_left_result(cpu.a, cpu.mf, rotate, cpu);
    load_accumulator(result, cpu);
}
fn shift_accumulator_right(cpu: &mut Cpu) {
    let result = shifted_right_result(cpu.a, cpu.mf, cpu);
    load_accumulator(result, cpu);
}
fn update_accumulator(delta: u16, cpu: &mut Cpu) {
    let result = cpu.a.wrapping_add(delta);
    load_accumulator(result, cpu);
}
fn load_accumulator(value: u16, cpu: &mut Cpu) {
    cpu.a = if cpu.mf {
        cpu.a & 0xff00 | value & 0xff
    } else {
        value
    };
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
fn record_opcode_history(bank: u8, pc: u16, game: &mut Game) {
    // Preserve the inherited history value of 0xffffffff when PC is zero.
    let cur_pc = (i32::from(bank) << 16 | (i32::from(pc) - 1)) as u32;
    game.pc_hist[game.pc_hist_ctr as usize] = cur_pc;
    game.pc_hist_ctr = game.pc_hist_ctr.wrapping_add(1) & 7;
}
unsafe fn dispatch_opcode(cpu: *mut Cpu, mut opcode: u8, game: &mut Game) {
    record_opcode_history((*cpu).k, (*cpu).pc, game);
    if opcode == 0 {
        let addr = (u32::from((*cpu).k) << 16) | u32::from((*cpu).pc) | 0x800000;
        opcode = CpuOpcodeHook(addr.wrapping_sub(1), game) as u8;
        match opcode {
            0x00 => {
                return;
            }
            0x01 => {
                (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                return;
            }
            0x02 => {
                (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                (*cpu).k = pull_byte(cpu, game);
                return;
            }
            0xe5 | 0xe9 | 0xed => {
                (*cpu).c = true;
            }
            0x65 | 0x6d | 0x69 | 0x7d | 0x7f => {
                (*cpu).c = false;
            }
            _ => {}
        }
    }
    match opcode {
        0x01 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x02 => {
            fetch_byte(cpu, game);
            push_byte(cpu, (*cpu).k, game);
            push_word(cpu, (*cpu).pc, game);
            push_byte(cpu, cpu_getFlags(&*cpu), game);
            (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
            (*cpu).i = true;
            (*cpu).d = false;
            (*cpu).k = 0 as u8;
            (*cpu).pc = read_word(cpu, 0xffe4 as u32, 0xffe5 as u32, game);
        }
        0x03 => {
            let (low, high) = address_stack_relative(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x04 => {
            let (low, high) = address_direct_page(cpu, None, game);
            test_and_update_bits(cpu, low, high, true, game);
        }
        0x05 => {
            let (low, high) = address_direct_page(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x06 => {
            let (low, high) = address_direct_page(cpu, None, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftLeft, game);
        }
        0x07 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x08 => {
            push_byte(cpu, cpu_getFlags(&*cpu), game);
        }
        0x09 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x0a => {
            shift_accumulator_left(false, &mut *cpu);
        }
        0x0b => {
            push_word(cpu, (*cpu).dp, game);
        }
        0x0c => {
            let (low, high) = address_absolute(cpu, game);
            test_and_update_bits(cpu, low, high, true, game);
        }
        0x0d => {
            let (low, high) = address_absolute(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x0e => {
            let (low, high) = address_absolute(cpu, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftLeft, game);
        }
        0x0f => {
            let (low, high) = address_absolute_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x10 => {
            branch_relative(fetch_byte(cpu, game), !(*cpu).n, &mut *cpu);
        }
        0x11 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x12 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x13 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x14 => {
            let (low, high) = address_direct_page(cpu, None, game);
            test_and_update_bits(cpu, low, high, false, game);
        }
        0x15 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x16 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftLeft, game);
        }
        0x17 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x18 => {
            (*cpu).c = false;
        }
        0x19 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x1a => {
            update_accumulator(1, &mut *cpu);
        }
        0x1b => {
            (*cpu).sp = (*cpu).a;
        }
        0x1c => {
            let (low, high) = address_absolute(cpu, game);
            test_and_update_bits(cpu, low, high, false, game);
        }
        0x1d => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x1e => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftLeft, game);
        }
        0x1f => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::Or, game);
        }
        0x20 => {
            let value = fetch_word(cpu, game);
            push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
            (*cpu).pc = value;
        }
        0x21 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x22 => {
            let address = fetch_long_address(cpu, game);
            let target = address as u16;
            let newK = (address >> 16) as u8;
            push_byte(cpu, (*cpu).k, game);
            push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
            (*cpu).pc = target;
            (*cpu).k = newK;
        }
        0x23 => {
            let (low, high) = address_stack_relative(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x24 => {
            let (low, high) = address_direct_page(cpu, None, game);
            test_accumulator_bits(cpu, low, high, game);
        }
        0x25 => {
            let (low, high) = address_direct_page(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x26 => {
            let (low, high) = address_direct_page(cpu, None, game);
            modify_memory(cpu, low, high, MemoryOperation::RotateLeft, game);
        }
        0x27 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x28 => {
            cpu_setFlags(pull_byte(cpu, game), &mut *cpu);
        }
        0x29 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x2a => {
            shift_accumulator_left(true, &mut *cpu);
        }
        0x2b => {
            (*cpu).dp = pull_word(cpu, game);
            set_zero_negative((*cpu).dp, false, &mut *cpu);
        }
        0x2c => {
            let (low, high) = address_absolute(cpu, game);
            test_accumulator_bits(cpu, low, high, game);
        }
        0x2d => {
            let (low, high) = address_absolute(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x2e => {
            let (low, high) = address_absolute(cpu, game);
            modify_memory(cpu, low, high, MemoryOperation::RotateLeft, game);
        }
        0x2f => {
            let (low, high) = address_absolute_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x30 => {
            branch_relative(fetch_byte(cpu, game), (*cpu).n, &mut *cpu);
        }
        0x31 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x32 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x33 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x34 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            test_accumulator_bits(cpu, low, high, game);
        }
        0x35 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x36 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            modify_memory(cpu, low, high, MemoryOperation::RotateLeft, game);
        }
        0x37 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x38 => {
            (*cpu).c = true;
        }
        0x39 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x3a => {
            update_accumulator(u16::MAX, &mut *cpu);
        }
        0x3b => {
            (*cpu).a = (*cpu).sp;
            set_zero_negative((*cpu).a, false, &mut *cpu);
        }
        0x3c => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            test_accumulator_bits(cpu, low, high, game);
        }
        0x3d => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x3e => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            modify_memory(cpu, low, high, MemoryOperation::RotateLeft, game);
        }
        0x3f => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::And, game);
        }
        0x40 => {
            cpu_setFlags(pull_byte(cpu, game), &mut *cpu);
            (*cpu).pc = pull_word(cpu, game);
            if !(*cpu).e {
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).k = pull_byte(cpu, game);
            }
        }
        0x41 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x42 => {
            fetch_byte(cpu, game);
        }
        0x43 => {
            let (low, high) = address_stack_relative(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x44 => {
            let dest = fetch_byte(cpu, game);
            let src = fetch_byte(cpu, game);
            (*cpu).db = dest;
            write_byte(
                cpu,
                u32::from(dest) << 16 | u32::from((*cpu).y),
                read_byte(cpu, u32::from(src) << 16 | u32::from((*cpu).x), game),
                game,
            );
            (*cpu).a = (*cpu).a.wrapping_sub(1);
            (*cpu).x = (*cpu).x.wrapping_sub(1);
            (*cpu).y = (*cpu).y.wrapping_sub(1);
            if (*cpu).a != 0xffff {
                (*cpu).pc = (*cpu).pc.wrapping_sub(3);
            }
            if (*cpu).xf {
                (*cpu).x &= 0xff;
                (*cpu).y &= 0xff;
            }
        }
        0x45 => {
            let (low, high) = address_direct_page(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x46 => {
            let (low, high) = address_direct_page(cpu, None, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftRight, game);
        }
        0x47 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x48 => {
            push_register(cpu, CpuRegister::Accumulator, game);
        }
        0x49 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x4a => {
            shift_accumulator_right(&mut *cpu);
        }
        0x4b => {
            push_byte(cpu, (*cpu).k, game);
        }
        0x4c => {
            (*cpu).pc = fetch_word(cpu, game);
        }
        0x4d => {
            let (low, high) = address_absolute(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x4e => {
            let (low, high) = address_absolute(cpu, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftRight, game);
        }
        0x4f => {
            let (low, high) = address_absolute_long(cpu, None, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x50 => {
            branch_relative(fetch_byte(cpu, game), !(*cpu).v, &mut *cpu);
        }
        0x51 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x52 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x53 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x54 => {
            let dest_0 = fetch_byte(cpu, game);
            let src_0 = fetch_byte(cpu, game);
            (*cpu).db = dest_0;
            write_byte(
                cpu,
                u32::from(dest_0) << 16 | u32::from((*cpu).y),
                read_byte(cpu, u32::from(src_0) << 16 | u32::from((*cpu).x), game),
                game,
            );
            (*cpu).a = (*cpu).a.wrapping_sub(1);
            (*cpu).x = (*cpu).x.wrapping_add(1);
            (*cpu).y = (*cpu).y.wrapping_add(1);
            if (*cpu).a != 0xffff {
                (*cpu).pc = (*cpu).pc.wrapping_sub(3);
            }
            if (*cpu).xf {
                (*cpu).x &= 0xff;
                (*cpu).y &= 0xff;
            }
        }
        0x55 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x56 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftRight, game);
        }
        0x57 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x58 => {
            (*cpu).i = false;
        }
        0x59 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x5a => {
            push_register(cpu, CpuRegister::Y, game);
        }
        0x5b => {
            (*cpu).dp = (*cpu).a;
            set_zero_negative((*cpu).dp, false, &mut *cpu);
        }
        0x5c => {
            let address = fetch_long_address(cpu, game);
            let target = address as u16;
            let target_bank = (address >> 16) as u8;
            if target_bank == 0x80 && target == 0x8573 {
                printf(
                    b"Current PC = 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).pc as ::core::ffi::c_int,
                );
                DumpCpuHistory(game);
                Die(b"The game has crashed!\n\0" as *const u8 as *const ::core::ffi::c_char);
            }
            (*cpu).k = target_bank;
            (*cpu).pc = target;
        }
        0x5d => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x5e => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            modify_memory(cpu, low, high, MemoryOperation::ShiftRight, game);
        }
        0x5f => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
        }
        0x60 => {
            if (*cpu).sp >= (*cpu).spBreakpoint && (*cpu).spBreakpoint != 0 {
                let delta = (*cpu).sp - (*cpu).spBreakpoint;
                if delta != 0 && delta != 2 {
                    __assert_fail(
                        b"delta == 0 || delta == 2\0" as *const u8 as *const ::core::ffi::c_char,
                        b"src/snes/cpu.c\0" as *const u8 as *const ::core::ffi::c_char,
                        1386 as ::core::ffi::c_uint,
                        b"void cpu_doOpcode(Cpu *, uint8_t)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                (*cpu).spBreakpoint = 0;
                if HookedFunctionRts(i32::from(delta == 2), game) {
                    return;
                }
            }
            (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
        }
        0x61 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            add_with_carry(cpu, low, high, game);
        }
        0x62 => {
            let displacement = fetch_word(cpu, game) as i16;
            push_word(cpu, (*cpu).pc.wrapping_add_signed(displacement), game);
        }
        0x63 => {
            let (low, high) = address_stack_relative(cpu, game);
            add_with_carry(cpu, low, high, game);
        }
        0x64 => {
            let (low, high) = address_direct_page(cpu, None, game);
            store_zero(cpu, low, high, game);
        }
        0x65 => {
            let (low, high) = address_direct_page(cpu, None, game);
            add_with_carry(cpu, low, high, game);
        }
        0x66 => {
            let (low, high) = address_direct_page(cpu, None, game);
            rotate_memory_right(cpu, low, high, game);
        }
        0x67 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            add_with_carry(cpu, low, high, game);
        }
        0x68 => {
            pull_register(cpu, CpuRegister::Accumulator, game);
        }
        0x69 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            add_with_carry(cpu, low, high, game);
        }
        0x6a => {
            let carry = (*cpu).a & 1 != 0;
            if (*cpu).mf {
                (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a >> 1 & 0x7f) | (u16::from((*cpu).c) << 7);
            } else {
                (*cpu).a = (*cpu).a >> 1 | (u16::from((*cpu).c) << 15);
            }
            (*cpu).c = carry;
            set_zero_negative((*cpu).a, (*cpu).mf, &mut *cpu);
        }
        0x6b => {
            if (*cpu).sp >= (*cpu).spBreakpoint && (*cpu).spBreakpoint != 0 {
                if (*cpu).sp != (*cpu).spBreakpoint {
                    __assert_fail(
                        b"cpu->sp == cpu->spBreakpoint\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"src/snes/cpu.c\0" as *const u8 as *const ::core::ffi::c_char,
                        1464 as ::core::ffi::c_uint,
                        b"void cpu_doOpcode(Cpu *, uint8_t)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                (*cpu).spBreakpoint = 0;
                if HookedFunctionRts(0, game) {
                    return;
                }
            }
            (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
            (*cpu).k = pull_byte(cpu, game);
        }
        0x6c => {
            let adr = fetch_word(cpu, game);
            (*cpu).pc = read_word(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
        }
        0x6d => {
            let (low, high) = address_absolute(cpu, game);
            add_with_carry(cpu, low, high, game);
        }
        0x6e => {
            let (low, high) = address_absolute(cpu, game);
            rotate_memory_right(cpu, low, high, game);
        }
        0x6f => {
            let (low, high) = address_absolute_long(cpu, None, game);
            add_with_carry(cpu, low, high, game);
        }
        0x70 => {
            branch_relative(fetch_byte(cpu, game), (*cpu).v, &mut *cpu);
        }
        0x71 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            add_with_carry(cpu, low, high, game);
        }
        0x72 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            add_with_carry(cpu, low, high, game);
        }
        0x73 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            add_with_carry(cpu, low, high, game);
        }
        0x74 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            store_zero(cpu, low, high, game);
        }
        0x75 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            add_with_carry(cpu, low, high, game);
        }
        0x76 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            rotate_memory_right(cpu, low, high, game);
        }
        0x77 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            add_with_carry(cpu, low, high, game);
        }
        0x78 => {
            (*cpu).i = true;
        }
        0x79 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            add_with_carry(cpu, low, high, game);
        }
        0x7a => {
            pull_register(cpu, CpuRegister::Y, game);
        }
        0x7b => {
            (*cpu).a = (*cpu).dp;
            set_zero_negative((*cpu).a, false, &mut *cpu);
        }
        0x7c => {
            (*cpu).pc = address_absolute_indexed_indirect(cpu, game);
        }
        0x7d => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            add_with_carry(cpu, low, high, game);
        }
        0x7e => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            rotate_memory_right(cpu, low, high, game);
        }
        0x7f => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            add_with_carry(cpu, low, high, game);
        }
        0x80 => {
            // Relative branches start after the displacement operand.
            let displacement = fetch_byte(cpu, game) as i8;
            (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement as i16);
        }
        0x81 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x82 => {
            let displacement = fetch_word(cpu, game) as i16;
            (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
        }
        0x83 => {
            let (low, high) = address_stack_relative(cpu, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x84 => {
            let (low, high) = address_direct_page(cpu, None, game);
            store_register(cpu, low, high, CpuRegister::Y, game);
        }
        0x85 => {
            let (low, high) = address_direct_page(cpu, None, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x86 => {
            let (low, high) = address_direct_page(cpu, None, game);
            store_register(cpu, low, high, CpuRegister::X, game);
        }
        0x87 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x88 => {
            IndexRegister::Y.update(u16::MAX, &mut *cpu);
        }
        0x89 => {
            if (*cpu).mf {
                let result = (*cpu).a as u8 & fetch_byte(cpu, game);
                (*cpu).z = result == 0;
            } else {
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                let result = (*cpu).a & fetch_word(cpu, game);
                (*cpu).z = result == 0;
            }
        }
        0x8a => {
            load_accumulator((*cpu).x, &mut *cpu);
        }
        0x8b => {
            push_byte(cpu, (*cpu).db, game);
        }
        0x8c => {
            let (low, high) = address_absolute(cpu, game);
            store_register(cpu, low, high, CpuRegister::Y, game);
        }
        0x8d => {
            let (low, high) = address_absolute(cpu, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x8e => {
            let (low, high) = address_absolute(cpu, game);
            store_register(cpu, low, high, CpuRegister::X, game);
        }
        0x8f => {
            let (low, high) = address_absolute_long(cpu, None, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x90 => {
            branch_relative(fetch_byte(cpu, game), !(*cpu).c, &mut *cpu);
        }
        0x91 => {
            let (low, high) = address_direct_page_indirect_y(cpu, true, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x92 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x93 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x94 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            store_register(cpu, low, high, CpuRegister::Y, game);
        }
        0x95 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x96 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::Y), game);
            store_register(cpu, low, high, CpuRegister::X, game);
        }
        0x97 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x98 => {
            load_accumulator((*cpu).y, &mut *cpu);
        }
        0x99 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, true, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x9a => {
            (*cpu).sp = (*cpu).x;
        }
        0x9b => {
            IndexRegister::Y.load((*cpu).x, &mut *cpu);
        }
        0x9c => {
            let (low, high) = address_absolute(cpu, game);
            store_zero(cpu, low, high, game);
        }
        0x9d => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0x9e => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            store_zero(cpu, low, high, game);
        }
        0x9f => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            store_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xa0 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Y);
            load_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xa1 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xa2 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::X);
            load_register(cpu, low, high, CpuRegister::X, game);
        }
        0xa3 => {
            let (low, high) = address_stack_relative(cpu, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xa4 => {
            let (low, high) = address_direct_page(cpu, None, game);
            load_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xa5 => {
            let (low, high) = address_direct_page(cpu, None, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xa6 => {
            let (low, high) = address_direct_page(cpu, None, game);
            load_register(cpu, low, high, CpuRegister::X, game);
        }
        0xa7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xa8 => {
            IndexRegister::Y.load((*cpu).a, &mut *cpu);
        }
        0xa9 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xaa => {
            IndexRegister::X.load((*cpu).a, &mut *cpu);
        }
        0xab => {
            (*cpu).db = pull_byte(cpu, game);
            set_zero_negative(u16::from((*cpu).db), true, &mut *cpu);
        }
        0xac => {
            let (low, high) = address_absolute(cpu, game);
            load_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xad => {
            let (low, high) = address_absolute(cpu, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xae => {
            let (low, high) = address_absolute(cpu, game);
            load_register(cpu, low, high, CpuRegister::X, game);
        }
        0xaf => {
            let (low, high) = address_absolute_long(cpu, None, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb0 => {
            branch_relative(fetch_byte(cpu, game), (*cpu).c, &mut *cpu);
        }
        0xb1 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb2 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb3 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb4 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            load_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xb5 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb6 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::Y), game);
            load_register(cpu, low, high, CpuRegister::X, game);
        }
        0xb7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xb8 => {
            (*cpu).v = false;
        }
        0xb9 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xba => {
            if (*cpu).xf {
                (*cpu).x = (*cpu).sp & 0xff;
            } else {
                (*cpu).x = (*cpu).sp;
            }
            set_zero_negative((*cpu).x, (*cpu).xf, &mut *cpu);
        }
        0xbb => {
            IndexRegister::X.load((*cpu).y, &mut *cpu);
        }
        0xbc => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            load_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xbd => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xbe => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            load_register(cpu, low, high, CpuRegister::X, game);
        }
        0xbf => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            load_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xc0 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Y);
            compare_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xc1 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xc2 => {
            update_status_bits(cpu, false, game);
        }
        0xc3 => {
            let (low, high) = address_stack_relative(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xc4 => {
            let (low, high) = address_direct_page(cpu, None, game);
            compare_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xc5 => {
            let (low, high) = address_direct_page(cpu, None, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xc6 => {
            let (low, high) = address_direct_page(cpu, None, game);
            modify_memory(cpu, low, high, MemoryOperation::Decrement, game);
        }
        0xc7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xc8 => {
            IndexRegister::Y.update(1, &mut *cpu);
        }
        0xc9 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xca => {
            IndexRegister::X.update(u16::MAX, &mut *cpu);
        }
        0xcb => {
            (*cpu).waiting = true;
        }
        0xcc => {
            let (low, high) = address_absolute(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Y, game);
        }
        0xcd => {
            let (low, high) = address_absolute(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xce => {
            let (low, high) = address_absolute(cpu, game);
            modify_memory(cpu, low, high, MemoryOperation::Decrement, game);
        }
        0xcf => {
            let (low, high) = address_absolute_long(cpu, None, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd0 => {
            branch_relative(fetch_byte(cpu, game), !(*cpu).z, &mut *cpu);
        }
        0xd1 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd2 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd3 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd4 => {
            let (low, high) = address_direct_page(cpu, None, game);
            push_word(cpu, read_word(cpu, low, high, game), game);
        }
        0xd5 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd6 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            modify_memory(cpu, low, high, MemoryOperation::Decrement, game);
        }
        0xd7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xd8 => {
            (*cpu).d = false;
        }
        0xd9 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xda => {
            push_register(cpu, CpuRegister::X, game);
        }
        0xdb => {
            (*cpu).stopped = true;
        }
        0xdc => {
            let adr = fetch_word(cpu, game);
            (*cpu).pc = read_word(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
            (*cpu).k = read_byte(cpu, u32::from(adr.wrapping_add(2)), game);
        }
        0xdd => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xde => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            modify_memory(cpu, low, high, MemoryOperation::Decrement, game);
        }
        0xdf => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            compare_register(cpu, low, high, CpuRegister::Accumulator, game);
        }
        0xe0 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::X);
            compare_register(cpu, low, high, CpuRegister::X, game);
        }
        0xe1 => {
            let (low, high) = address_direct_page_indexed_indirect(cpu, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xe2 => {
            update_status_bits(cpu, true, game);
        }
        0xe3 => {
            let (low, high) = address_stack_relative(cpu, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xe4 => {
            let (low, high) = address_direct_page(cpu, None, game);
            compare_register(cpu, low, high, CpuRegister::X, game);
        }
        0xe5 => {
            let (low, high) = address_direct_page(cpu, None, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xe6 => {
            let (low, high) = address_direct_page(cpu, None, game);
            modify_memory(cpu, low, high, MemoryOperation::Increment, game);
        }
        0xe7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, None, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xe8 => {
            IndexRegister::X.update(1, &mut *cpu);
        }
        0xe9 => {
            let (low, high) = address_immediate(&mut *cpu, CpuRegister::Accumulator);
            subtract_with_carry(cpu, low, high, game);
        }
        0xeb => {
            exchange_accumulator_bytes(&mut *cpu);
        }
        0xec => {
            let (low, high) = address_absolute(cpu, game);
            compare_register(cpu, low, high, CpuRegister::X, game);
        }
        0xed => {
            let (low, high) = address_absolute(cpu, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xee => {
            let (low, high) = address_absolute(cpu, game);
            modify_memory(cpu, low, high, MemoryOperation::Increment, game);
        }
        0xef => {
            let (low, high) = address_absolute_long(cpu, None, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf0 => {
            branch_relative(fetch_byte(cpu, game), (*cpu).z, &mut *cpu);
        }
        0xf1 => {
            let (low, high) = address_direct_page_indirect_y(cpu, false, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf2 => {
            let (low, high) = address_direct_page_indirect(cpu, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf3 => {
            let (low, high) = address_stack_relative_indirect_y(cpu, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf4 => {
            let w: u16 = fetch_word(cpu, game);
            if w == 0xc2ae {
                (*cpu).y = 0;
            }
            push_word(cpu, w, game);
        }
        0xf5 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf6 => {
            let (low, high) = address_direct_page(cpu, Some(IndexRegister::X), game);
            modify_memory(cpu, low, high, MemoryOperation::Increment, game);
        }
        0xf7 => {
            let (low, high) = address_direct_page_indirect_long(cpu, Some(IndexRegister::Y), game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xf8 => {
            (*cpu).d = true;
        }
        0xf9 => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::Y, false, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xfa => {
            pull_register(cpu, CpuRegister::X, game);
        }
        0xfb => {
            exchange_carry_emulation(&mut *cpu);
        }
        0xfc => {
            let target = address_absolute_indexed_indirect(cpu, game);
            push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
            (*cpu).pc = target;
        }
        0xfd => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, false, game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xfe => {
            let (low, high) = address_absolute_indexed(cpu, IndexRegister::X, true, game);
            modify_memory(cpu, low, high, MemoryOperation::Increment, game);
        }
        0xff => {
            let (low, high) = address_absolute_long(cpu, Some(IndexRegister::X), game);
            subtract_with_carry(cpu, low, high, game);
        }
        0xea | _ => {}
    }
}
