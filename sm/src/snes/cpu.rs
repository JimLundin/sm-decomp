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
unsafe fn fetch_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    let pc = (*cpu).pc;
    (*cpu).pc = (*cpu).pc.wrapping_add(1);
    read_byte(cpu, u32::from((*cpu).k) << 16 | u32::from(pc), game)
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
unsafe fn pull_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    (*cpu).sp = (*cpu).sp.wrapping_add(1);
    if (*cpu).e {
        (*cpu).sp = (*cpu).sp & 0xff | 0x100;
    }
    read_byte(cpu, u32::from((*cpu).sp), game)
}
unsafe fn push_byte(cpu: *mut Cpu, value: u8, game: &mut Game) {
    write_byte(cpu, u32::from((*cpu).sp), value, game);
    (*cpu).sp = (*cpu).sp.wrapping_sub(1);
    if (*cpu).e {
        (*cpu).sp = (*cpu).sp & 0xff | 0x100;
    }
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
fn address_immediate(cpu: &mut Cpu, xFlag: bool) -> (u32, u32) {
    let byte = if xFlag { cpu.xf } else { cpu.mf };
    let pc = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    let low = u32::from(cpu.k) << 16 | u32::from(pc);
    let high = if byte {
        0
    } else {
        let pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(1);
        u32::from(cpu.k) << 16 | u32::from(pc)
    };
    (low, high)
}
unsafe fn address_direct_page(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    direct_page_address(adr, None, &mut *cpu)
}
unsafe fn address_direct_page_x(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    direct_page_address(adr, Some(CpuRegister::X), &mut *cpu)
}
unsafe fn address_direct_page_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    direct_page_address(adr, Some(CpuRegister::Y), &mut *cpu)
}
fn direct_page_address(offset: u16, index: Option<CpuRegister>, cpu: &mut Cpu) -> (u32, u32) {
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
unsafe fn address_direct_page_indirect(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    let (pointer_low, pointer_high) = direct_page_address(adr, None, &mut *cpu);
    let pointer = u32::from(read_word(cpu, pointer_low, pointer_high, game));
    let low = (u32::from((*cpu).db) << 16) + pointer;
    (low, low.wrapping_add(1) & 0xffffff)
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
    let adr = u16::from(fetch_byte(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let pointer = u32::from(read_word(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    if write && (!(*cpu).xf || pointer >> 8 != (pointer + u32::from((*cpu).y)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y)) & 0xffffff;
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_direct_page_indirect_long(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer = u32::from(read_word(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    pointer |= u32::from(read_byte(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(2)),
        game,
    )) << 16;
    let low = pointer;
    (low, pointer.wrapping_add(1) & 0xffffff)
}
unsafe fn address_direct_page_indirect_long_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(fetch_byte(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer = u32::from(read_word(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    pointer |= u32::from(read_byte(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(2)),
        game,
    )) << 16;
    let low = pointer.wrapping_add(u32::from((*cpu).y)) & 0xffffff;
    (low, low.wrapping_add(1) & 0xffffff)
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
    let adr = u32::from(fetch_word(cpu, game));
    let low = (u32::from((*cpu).db) << 16) + adr;
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_x(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
    let adr = u32::from(fetch_word(cpu, game));
    absolute_indexed_address(adr, CpuRegister::X, write, &mut *cpu)
}
unsafe fn address_absolute_y(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
    let adr = u32::from(fetch_word(cpu, game));
    absolute_indexed_address(adr, CpuRegister::Y, write, &mut *cpu)
}
fn absolute_indexed_address(
    offset: u32,
    index: CpuRegister,
    write: bool,
    cpu: &mut Cpu,
) -> (u32, u32) {
    if write && (!cpu.xf || offset >> 8 != (offset + u32::from(index.value(cpu))) >> 8) {
        cpu.cyclesUsed = cpu.cyclesUsed.wrapping_add(1);
    }
    let low = ((u32::from(cpu.db) << 16) + offset + u32::from(index.value(cpu))) & 0xffffff;
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_long(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let offset = fetch_word(cpu, game);
    let bank = fetch_byte(cpu, game);
    let low = u32::from(bank) << 16 | u32::from(offset);
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_long_x(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let offset = fetch_word(cpu, game);
    let bank = fetch_byte(cpu, game);
    let adr = u32::from(bank) << 16 | u32::from(offset);
    let low = adr.wrapping_add(u32::from((*cpu).x)) & 0xffffff;
    (low, low.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let adr = fetch_word(cpu, game);
    read_word(
        cpu,
        u32::from((*cpu).k) << 16 | u32::from(adr.wrapping_add((*cpu).x)),
        u32::from((*cpu).k) << 16 | u32::from(adr.wrapping_add((*cpu).x).wrapping_add(1)),
        game,
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
}
unsafe fn logical_accumulator(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    operation: LogicOperation,
    game: &mut Game,
) {
    let byte = (*cpu).mf;
    let value = if byte {
        u16::from(read_byte(cpu, low, game))
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        read_word(cpu, low, high, game)
    };
    apply_accumulator_logic(value, byte, operation, &mut *cpu);
    set_zero_negative((*cpu).a, (*cpu).mf, &mut *cpu);
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
}
unsafe fn add_with_carry(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(read_byte(cpu, low, game));
        add_to_accumulator(value, true, &mut *cpu);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = read_word(cpu, low, high, game);
        add_to_accumulator(value, false, &mut *cpu);
    }
    set_zero_negative((*cpu).a, (*cpu).mf, &mut *cpu);
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
}
unsafe fn subtract_with_carry(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(!read_byte(cpu, low, game));
        subtract_from_accumulator(value, true, &mut *cpu);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = !read_word(cpu, low, high, game);
        subtract_from_accumulator(value, false, &mut *cpu);
    }
    set_zero_negative((*cpu).a, (*cpu).mf, &mut *cpu);
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
    fn load(self, value: u16, cpu: &mut Cpu) {
        match self {
            Self::Accumulator => cpu.a = value,
            Self::X => cpu.x = value,
            Self::Y => cpu.y = value,
        }
        set_zero_negative(value, self.byte(cpu), cpu);
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
    let value = if byte {
        u16::from(read_byte(cpu, low, game))
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        read_word(cpu, low, high, game)
    };
    let lhs = register.value(&*cpu);
    let lhs = if byte { lhs & 0xff } else { lhs };
    let result = subtract_for_compare(lhs, value, byte, &mut *cpu);
    set_zero_negative(result, register.byte(&*cpu), &mut *cpu);
}
fn test_bits(value: u16, byte: bool, cpu: &mut Cpu) {
    cpu.z = cpu.a & value == 0;
    cpu.n = value & if byte { 0x80 } else { 0x8000 } != 0;
    cpu.v = value & if byte { 0x40 } else { 0x4000 } != 0;
}
unsafe fn test_accumulator_bits(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(read_byte(cpu, low, game));
        test_bits(value, true, &mut *cpu);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = read_word(cpu, low, high, game);
        test_bits(value, false, &mut *cpu);
    }
}
unsafe fn load_register(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    register: CpuRegister,
    game: &mut Game,
) {
    let value = if register.byte(&*cpu) {
        let upper = match register {
            CpuRegister::Accumulator => (*cpu).a & 0xff00,
            CpuRegister::X | CpuRegister::Y => 0,
        };
        upper | u16::from(read_byte(cpu, low, game))
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        read_word(cpu, low, high, game)
    };
    register.load(value, &mut *cpu);
}
unsafe fn pull_register(cpu: *mut Cpu, register: CpuRegister, game: &mut Game) {
    let value = if register.byte(&*cpu) {
        let upper = match register {
            CpuRegister::Accumulator => (*cpu).a & 0xff00,
            CpuRegister::X | CpuRegister::Y => 0,
        };
        upper | u16::from(pull_byte(cpu, game))
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
unsafe fn store_register(
    cpu: *mut Cpu,
    low: u32,
    high: u32,
    register: CpuRegister,
    game: &mut Game,
) {
    if register.byte(&*cpu) {
        write_byte(cpu, low, register.value(&*cpu) as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        write_word(cpu, low, high, register.value(&*cpu), false, game);
    }
}
unsafe fn store_zero(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    if (*cpu).mf {
        write_byte(cpu, low, 0, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        write_word(cpu, low, high, 0, false, game);
    }
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
unsafe fn shift_memory_left(cpu: *mut Cpu, low: u32, high: u32, rotate: bool, game: &mut Game) {
    let result = if (*cpu).mf {
        let value = read_byte(cpu, low, game);
        let result = (u16::from(value) << 1) | u16::from(rotate && (*cpu).c);
        (*cpu).c = result & 0x100 != 0;
        write_byte(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = read_word(cpu, low, high, game);
        let result = value << 1 | u16::from(rotate && (*cpu).c);
        (*cpu).c = value & 0x8000 != 0;
        write_word(cpu, low, high, result, true, game);
        result
    };
    set_zero_negative(result, (*cpu).mf, &mut *cpu);
}
unsafe fn shift_memory_right(cpu: *mut Cpu, low: u32, high: u32, game: &mut Game) {
    let result = if (*cpu).mf {
        let value = read_byte(cpu, low, game);
        (*cpu).c = value & 1 != 0;
        let result = value >> 1;
        write_byte(cpu, low, result, game);
        u16::from(result)
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = read_word(cpu, low, high, game);
        (*cpu).c = value & 1 != 0;
        let result = value >> 1;
        write_word(cpu, low, high, result, true, game);
        result
    };
    set_zero_negative(result, (*cpu).mf, &mut *cpu);
}
unsafe fn update_memory(cpu: *mut Cpu, low: u32, high: u32, delta: u16, game: &mut Game) {
    let result = if (*cpu).mf {
        let result = u16::from(read_byte(cpu, low, game)).wrapping_add(delta);
        write_byte(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let result = read_word(cpu, low, high, game).wrapping_add(delta);
        write_word(cpu, low, high, result, true, game);
        result
    };
    set_zero_negative(result, (*cpu).mf, &mut *cpu);
}
unsafe fn test_and_update_bits(cpu: *mut Cpu, low: u32, high: u32, set: bool, game: &mut Game) {
    if (*cpu).mf {
        let value = read_byte(cpu, low, game);
        (*cpu).z = (*cpu).a as u8 & value == 0;
        let accumulator = (*cpu).a as u8;
        let result = if set {
            value | accumulator
        } else {
            value & !accumulator
        };
        write_byte(cpu, low, result, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = read_word(cpu, low, high, game);
        (*cpu).z = (*cpu).a & value == 0;
        let accumulator = (*cpu).a;
        let result = if set {
            value | accumulator
        } else {
            value & !accumulator
        };
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
    let value = cpu.a;
    let result = value << 1 | u16::from(rotate && cpu.c);
    if cpu.mf {
        cpu.c = value & 0x80 != 0;
        cpu.a = value & 0xff00 | result & 0xff;
    } else {
        cpu.c = value & 0x8000 != 0;
        cpu.a = result;
    }
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
fn shift_accumulator_right(cpu: &mut Cpu) {
    cpu.c = cpu.a & 1 != 0;
    if cpu.mf {
        cpu.a = cpu.a & 0xff00 | (cpu.a >> 1 & 0x7f);
    } else {
        cpu.a >>= 1;
    }
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
fn update_accumulator(delta: u16, cpu: &mut Cpu) {
    let result = cpu.a.wrapping_add(delta);
    cpu.a = if cpu.mf {
        cpu.a & 0xff00 | result & 0xff
    } else {
        result
    };
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
fn update_x(delta: u16, cpu: &mut Cpu) {
    let result = cpu.x.wrapping_add(delta);
    cpu.x = if cpu.xf { result & 0xff } else { result };
    set_zero_negative(cpu.x, cpu.xf, cpu);
}
fn update_y(delta: u16, cpu: &mut Cpu) {
    let result = cpu.y.wrapping_add(delta);
    cpu.y = if cpu.xf { result & 0xff } else { result };
    set_zero_negative(cpu.y, cpu.xf, cpu);
}
fn transfer_to_accumulator(value: u16, cpu: &mut Cpu) {
    cpu.a = if cpu.mf {
        cpu.a & 0xff00 | value & 0xff
    } else {
        value
    };
    set_zero_negative(cpu.a, cpu.mf, cpu);
}
fn transfer_to_x(value: u16, cpu: &mut Cpu) {
    cpu.x = if cpu.xf { value & 0xff } else { value };
    set_zero_negative(cpu.x, cpu.xf, cpu);
}
fn transfer_to_y(value: u16, cpu: &mut Cpu) {
    cpu.y = if cpu.xf { value & 0xff } else { value };
    set_zero_negative(cpu.y, cpu.xf, cpu);
}
unsafe fn dispatch_opcode(cpu: *mut Cpu, mut opcode: u8, game: &mut Game) {
    // Preserve the inherited history value of 0xffffffff when PC is zero.
    let cur_pc = (i32::from((*cpu).k) << 16 | (i32::from((*cpu).pc) - 1)) as u32;
    game.pc_hist[game.pc_hist_ctr as usize] = cur_pc;
    game.pc_hist_ctr = game.pc_hist_ctr.wrapping_add(1) & 7;
    loop {
        match opcode {
            0x00 => {
                let addr = (u32::from((*cpu).k) << 16) | u32::from((*cpu).pc) | 0x800000;
                opcode = CpuOpcodeHook(addr.wrapping_sub(1), game) as u8;
                match opcode {
                    0x00 => {
                        break;
                    }
                    0x01 => {
                        (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                        break;
                    }
                    0x02 => {
                        (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                        (*cpu).k = pull_byte(cpu, game);
                        break;
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
            0x01 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
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
                break;
            }
            0x03 => {
                let (low, high) = address_stack_relative(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x04 => {
                let (low, high) = address_direct_page(cpu, game);
                test_and_update_bits(cpu, low, high, true, game);
                break;
            }
            0x05 => {
                let (low, high) = address_direct_page(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x06 => {
                let (low, high) = address_direct_page(cpu, game);
                shift_memory_left(cpu, low, high, false, game);
                break;
            }
            0x07 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x08 => {
                push_byte(cpu, cpu_getFlags(&*cpu), game);
                break;
            }
            0x09 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x0a => {
                shift_accumulator_left(false, &mut *cpu);
                break;
            }
            0x0b => {
                push_word(cpu, (*cpu).dp, game);
                break;
            }
            0x0c => {
                let (low, high) = address_absolute(cpu, game);
                test_and_update_bits(cpu, low, high, true, game);
                break;
            }
            0x0d => {
                let (low, high) = address_absolute(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x0e => {
                let (low, high) = address_absolute(cpu, game);
                shift_memory_left(cpu, low, high, false, game);
                break;
            }
            0x0f => {
                let (low, high) = address_absolute_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x10 => {
                branch_relative(fetch_byte(cpu, game), !(*cpu).n, &mut *cpu);
                break;
            }
            0x11 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x12 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x13 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x14 => {
                let (low, high) = address_direct_page(cpu, game);
                test_and_update_bits(cpu, low, high, false, game);
                break;
            }
            0x15 => {
                let (low, high) = address_direct_page_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x16 => {
                let (low, high) = address_direct_page_x(cpu, game);
                shift_memory_left(cpu, low, high, false, game);
                break;
            }
            0x17 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x18 => {
                (*cpu).c = false;
                break;
            }
            0x19 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x1a => {
                update_accumulator(1, &mut *cpu);
                break;
            }
            0x1b => {
                (*cpu).sp = (*cpu).a;
                break;
            }
            0x1c => {
                let (low, high) = address_absolute(cpu, game);
                test_and_update_bits(cpu, low, high, false, game);
                break;
            }
            0x1d => {
                let (low, high) = address_absolute_x(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x1e => {
                let (low, high) = address_absolute_x(cpu, true, game);
                shift_memory_left(cpu, low, high, false, game);
                break;
            }
            0x1f => {
                let (low, high) = address_absolute_long_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Or, game);
                break;
            }
            0x20 => {
                let value = fetch_word(cpu, game);
                push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value;
                break;
            }
            0x21 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x22 => {
                let target = fetch_word(cpu, game);
                let newK = fetch_byte(cpu, game);
                push_byte(cpu, (*cpu).k, game);
                push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = target;
                (*cpu).k = newK;
                break;
            }
            0x23 => {
                let (low, high) = address_stack_relative(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x24 => {
                let (low, high) = address_direct_page(cpu, game);
                test_accumulator_bits(cpu, low, high, game);
                break;
            }
            0x25 => {
                let (low, high) = address_direct_page(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x26 => {
                let (low, high) = address_direct_page(cpu, game);
                shift_memory_left(cpu, low, high, true, game);
                break;
            }
            0x27 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x28 => {
                cpu_setFlags(pull_byte(cpu, game), &mut *cpu);
                break;
            }
            0x29 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x2a => {
                shift_accumulator_left(true, &mut *cpu);
                break;
            }
            0x2b => {
                (*cpu).dp = pull_word(cpu, game);
                set_zero_negative((*cpu).dp, false, &mut *cpu);
                break;
            }
            0x2c => {
                let (low, high) = address_absolute(cpu, game);
                test_accumulator_bits(cpu, low, high, game);
                break;
            }
            0x2d => {
                let (low, high) = address_absolute(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x2e => {
                let (low, high) = address_absolute(cpu, game);
                shift_memory_left(cpu, low, high, true, game);
                break;
            }
            0x2f => {
                let (low, high) = address_absolute_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x30 => {
                branch_relative(fetch_byte(cpu, game), (*cpu).n, &mut *cpu);
                break;
            }
            0x31 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x32 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x33 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x34 => {
                let (low, high) = address_direct_page_x(cpu, game);
                test_accumulator_bits(cpu, low, high, game);
                break;
            }
            0x35 => {
                let (low, high) = address_direct_page_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x36 => {
                let (low, high) = address_direct_page_x(cpu, game);
                shift_memory_left(cpu, low, high, true, game);
                break;
            }
            0x37 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x38 => {
                (*cpu).c = true;
                break;
            }
            0x39 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x3a => {
                update_accumulator(u16::MAX, &mut *cpu);
                break;
            }
            0x3b => {
                (*cpu).a = (*cpu).sp;
                set_zero_negative((*cpu).a, false, &mut *cpu);
                break;
            }
            0x3c => {
                let (low, high) = address_absolute_x(cpu, false, game);
                test_accumulator_bits(cpu, low, high, game);
                break;
            }
            0x3d => {
                let (low, high) = address_absolute_x(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x3e => {
                let (low, high) = address_absolute_x(cpu, true, game);
                shift_memory_left(cpu, low, high, true, game);
                break;
            }
            0x3f => {
                let (low, high) = address_absolute_long_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::And, game);
                break;
            }
            0x40 => {
                cpu_setFlags(pull_byte(cpu, game), &mut *cpu);
                (*cpu).pc = pull_word(cpu, game);
                if !(*cpu).e {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).k = pull_byte(cpu, game);
                }
                break;
            }
            0x41 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x42 => {
                fetch_byte(cpu, game);
                break;
            }
            0x43 => {
                let (low, high) = address_stack_relative(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
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
                break;
            }
            0x45 => {
                let (low, high) = address_direct_page(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x46 => {
                let (low, high) = address_direct_page(cpu, game);
                shift_memory_right(cpu, low, high, game);
                break;
            }
            0x47 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x48 => {
                push_register(cpu, CpuRegister::Accumulator, game);
                break;
            }
            0x49 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x4a => {
                shift_accumulator_right(&mut *cpu);
                break;
            }
            0x4b => {
                push_byte(cpu, (*cpu).k, game);
                break;
            }
            0x4c => {
                (*cpu).pc = fetch_word(cpu, game);
                break;
            }
            0x4d => {
                let (low, high) = address_absolute(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x4e => {
                let (low, high) = address_absolute(cpu, game);
                shift_memory_right(cpu, low, high, game);
                break;
            }
            0x4f => {
                let (low, high) = address_absolute_long(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x50 => {
                branch_relative(fetch_byte(cpu, game), !(*cpu).v, &mut *cpu);
                break;
            }
            0x51 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x52 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x53 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
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
                break;
            }
            0x55 => {
                let (low, high) = address_direct_page_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x56 => {
                let (low, high) = address_direct_page_x(cpu, game);
                shift_memory_right(cpu, low, high, game);
                break;
            }
            0x57 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x58 => {
                (*cpu).i = false;
                break;
            }
            0x59 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x5a => {
                push_register(cpu, CpuRegister::Y, game);
                break;
            }
            0x5b => {
                (*cpu).dp = (*cpu).a;
                set_zero_negative((*cpu).dp, false, &mut *cpu);
                break;
            }
            0x5c => {
                let target: u16 = fetch_word(cpu, game);
                let target_bank: u8 = fetch_byte(cpu, game);
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
                break;
            }
            0x5d => {
                let (low, high) = address_absolute_x(cpu, false, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x5e => {
                let (low, high) = address_absolute_x(cpu, true, game);
                shift_memory_right(cpu, low, high, game);
                break;
            }
            0x5f => {
                let (low, high) = address_absolute_long_x(cpu, game);
                logical_accumulator(cpu, low, high, LogicOperation::Xor, game);
                break;
            }
            0x60 => {
                if (*cpu).sp >= (*cpu).spBreakpoint && (*cpu).spBreakpoint != 0 {
                    let delta = (*cpu).sp - (*cpu).spBreakpoint;
                    if delta != 0 && delta != 2 {
                        __assert_fail(
                            b"delta == 0 || delta == 2\0" as *const u8
                                as *const ::core::ffi::c_char,
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
                break;
            }
            0x61 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x62 => {
                let displacement = fetch_word(cpu, game) as i16;
                push_word(cpu, (*cpu).pc.wrapping_add_signed(displacement), game);
                break;
            }
            0x63 => {
                let (low, high) = address_stack_relative(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x64 => {
                let (low, high) = address_direct_page(cpu, game);
                store_zero(cpu, low, high, game);
                break;
            }
            0x65 => {
                let (low, high) = address_direct_page(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x66 => {
                let (low, high) = address_direct_page(cpu, game);
                rotate_memory_right(cpu, low, high, game);
                break;
            }
            0x67 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x68 => {
                pull_register(cpu, CpuRegister::Accumulator, game);
                break;
            }
            0x69 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x6a => {
                let carry = (*cpu).a & 1 != 0;
                if (*cpu).mf {
                    (*cpu).a =
                        (*cpu).a & 0xff00 | ((*cpu).a >> 1 & 0x7f) | (u16::from((*cpu).c) << 7);
                } else {
                    (*cpu).a = (*cpu).a >> 1 | (u16::from((*cpu).c) << 15);
                }
                (*cpu).c = carry;
                set_zero_negative((*cpu).a, (*cpu).mf, &mut *cpu);
                break;
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
                break;
            }
            0x6c => {
                let adr = fetch_word(cpu, game);
                (*cpu).pc = read_word(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
                break;
            }
            0x6d => {
                let (low, high) = address_absolute(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x6e => {
                let (low, high) = address_absolute(cpu, game);
                rotate_memory_right(cpu, low, high, game);
                break;
            }
            0x6f => {
                let (low, high) = address_absolute_long(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x70 => {
                branch_relative(fetch_byte(cpu, game), (*cpu).v, &mut *cpu);
                break;
            }
            0x71 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x72 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x73 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x74 => {
                let (low, high) = address_direct_page_x(cpu, game);
                store_zero(cpu, low, high, game);
                break;
            }
            0x75 => {
                let (low, high) = address_direct_page_x(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x76 => {
                let (low, high) = address_direct_page_x(cpu, game);
                rotate_memory_right(cpu, low, high, game);
                break;
            }
            0x77 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x78 => {
                (*cpu).i = true;
                break;
            }
            0x79 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x7a => {
                pull_register(cpu, CpuRegister::Y, game);
                break;
            }
            0x7b => {
                (*cpu).a = (*cpu).dp;
                set_zero_negative((*cpu).a, false, &mut *cpu);
                break;
            }
            0x7c => {
                (*cpu).pc = address_absolute_indexed_indirect(cpu, game);
                break;
            }
            0x7d => {
                let (low, high) = address_absolute_x(cpu, false, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x7e => {
                let (low, high) = address_absolute_x(cpu, true, game);
                rotate_memory_right(cpu, low, high, game);
                break;
            }
            0x7f => {
                let (low, high) = address_absolute_long_x(cpu, game);
                add_with_carry(cpu, low, high, game);
                break;
            }
            0x80 => {
                // Relative branches start after the displacement operand.
                let displacement = fetch_byte(cpu, game) as i8;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement as i16);
                break;
            }
            0x81 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x82 => {
                let displacement = fetch_word(cpu, game) as i16;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
                break;
            }
            0x83 => {
                let (low, high) = address_stack_relative(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x84 => {
                let (low, high) = address_direct_page(cpu, game);
                store_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0x85 => {
                let (low, high) = address_direct_page(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x86 => {
                let (low, high) = address_direct_page(cpu, game);
                store_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0x87 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x88 => {
                update_y(u16::MAX, &mut *cpu);
                break;
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
                break;
            }
            0x8a => {
                transfer_to_accumulator((*cpu).x, &mut *cpu);
                break;
            }
            0x8b => {
                push_byte(cpu, (*cpu).db, game);
                break;
            }
            0x8c => {
                let (low, high) = address_absolute(cpu, game);
                store_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0x8d => {
                let (low, high) = address_absolute(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x8e => {
                let (low, high) = address_absolute(cpu, game);
                store_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0x8f => {
                let (low, high) = address_absolute_long(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x90 => {
                branch_relative(fetch_byte(cpu, game), !(*cpu).c, &mut *cpu);
                break;
            }
            0x91 => {
                let (low, high) = address_direct_page_indirect_y(cpu, true, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x92 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x93 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x94 => {
                let (low, high) = address_direct_page_x(cpu, game);
                store_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0x95 => {
                let (low, high) = address_direct_page_x(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x96 => {
                let (low, high) = address_direct_page_y(cpu, game);
                store_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0x97 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x98 => {
                transfer_to_accumulator((*cpu).y, &mut *cpu);
                break;
            }
            0x99 => {
                let (low, high) = address_absolute_y(cpu, true, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x9a => {
                (*cpu).sp = (*cpu).x;
                break;
            }
            0x9b => {
                transfer_to_y((*cpu).x, &mut *cpu);
                break;
            }
            0x9c => {
                let (low, high) = address_absolute(cpu, game);
                store_zero(cpu, low, high, game);
                break;
            }
            0x9d => {
                let (low, high) = address_absolute_x(cpu, true, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0x9e => {
                let (low, high) = address_absolute_x(cpu, true, game);
                store_zero(cpu, low, high, game);
                break;
            }
            0x9f => {
                let (low, high) = address_absolute_long_x(cpu, game);
                store_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xa0 => {
                let (low, high) = address_immediate(&mut *cpu, true);
                load_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xa1 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xa2 => {
                let (low, high) = address_immediate(&mut *cpu, true);
                load_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xa3 => {
                let (low, high) = address_stack_relative(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xa4 => {
                let (low, high) = address_direct_page(cpu, game);
                load_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xa5 => {
                let (low, high) = address_direct_page(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xa6 => {
                let (low, high) = address_direct_page(cpu, game);
                load_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xa7 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xa8 => {
                transfer_to_y((*cpu).a, &mut *cpu);
                break;
            }
            0xa9 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xaa => {
                transfer_to_x((*cpu).a, &mut *cpu);
                break;
            }
            0xab => {
                (*cpu).db = pull_byte(cpu, game);
                set_zero_negative(u16::from((*cpu).db), true, &mut *cpu);
                break;
            }
            0xac => {
                let (low, high) = address_absolute(cpu, game);
                load_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xad => {
                let (low, high) = address_absolute(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xae => {
                let (low, high) = address_absolute(cpu, game);
                load_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xaf => {
                let (low, high) = address_absolute_long(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb0 => {
                branch_relative(fetch_byte(cpu, game), (*cpu).c, &mut *cpu);
                break;
            }
            0xb1 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb2 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb3 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb4 => {
                let (low, high) = address_direct_page_x(cpu, game);
                load_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xb5 => {
                let (low, high) = address_direct_page_x(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb6 => {
                let (low, high) = address_direct_page_y(cpu, game);
                load_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xb7 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xb8 => {
                (*cpu).v = false;
                break;
            }
            0xb9 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xba => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).sp & 0xff;
                } else {
                    (*cpu).x = (*cpu).sp;
                }
                set_zero_negative((*cpu).x, (*cpu).xf, &mut *cpu);
                break;
            }
            0xbb => {
                transfer_to_x((*cpu).y, &mut *cpu);
                break;
            }
            0xbc => {
                let (low, high) = address_absolute_x(cpu, false, game);
                load_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xbd => {
                let (low, high) = address_absolute_x(cpu, false, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xbe => {
                let (low, high) = address_absolute_y(cpu, false, game);
                load_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xbf => {
                let (low, high) = address_absolute_long_x(cpu, game);
                load_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xc0 => {
                let (low, high) = address_immediate(&mut *cpu, true);
                compare_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xc1 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xc2 => {
                cpu_setFlags(cpu_getFlags(&*cpu) & !fetch_byte(cpu, game), &mut *cpu);
                break;
            }
            0xc3 => {
                let (low, high) = address_stack_relative(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xc4 => {
                let (low, high) = address_direct_page(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xc5 => {
                let (low, high) = address_direct_page(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xc6 => {
                let (low, high) = address_direct_page(cpu, game);
                update_memory(cpu, low, high, u16::MAX, game);
                break;
            }
            0xc7 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xc8 => {
                update_y(1, &mut *cpu);
                break;
            }
            0xc9 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xca => {
                update_x(u16::MAX, &mut *cpu);
                break;
            }
            0xcb => {
                (*cpu).waiting = true;
                break;
            }
            0xcc => {
                let (low, high) = address_absolute(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Y, game);
                break;
            }
            0xcd => {
                let (low, high) = address_absolute(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xce => {
                let (low, high) = address_absolute(cpu, game);
                update_memory(cpu, low, high, u16::MAX, game);
                break;
            }
            0xcf => {
                let (low, high) = address_absolute_long(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd0 => {
                branch_relative(fetch_byte(cpu, game), !(*cpu).z, &mut *cpu);
                break;
            }
            0xd1 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd2 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd3 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd4 => {
                let (low, high) = address_direct_page(cpu, game);
                push_word(cpu, read_word(cpu, low, high, game), game);
                break;
            }
            0xd5 => {
                let (low, high) = address_direct_page_x(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd6 => {
                let (low, high) = address_direct_page_x(cpu, game);
                update_memory(cpu, low, high, u16::MAX, game);
                break;
            }
            0xd7 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xd8 => {
                (*cpu).d = false;
                break;
            }
            0xd9 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xda => {
                push_register(cpu, CpuRegister::X, game);
                break;
            }
            0xdb => {
                (*cpu).stopped = true;
                break;
            }
            0xdc => {
                let adr = fetch_word(cpu, game);
                (*cpu).pc = read_word(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
                (*cpu).k = read_byte(cpu, u32::from(adr.wrapping_add(2)), game);
                break;
            }
            0xdd => {
                let (low, high) = address_absolute_x(cpu, false, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xde => {
                let (low, high) = address_absolute_x(cpu, true, game);
                update_memory(cpu, low, high, u16::MAX, game);
                break;
            }
            0xdf => {
                let (low, high) = address_absolute_long_x(cpu, game);
                compare_register(cpu, low, high, CpuRegister::Accumulator, game);
                break;
            }
            0xe0 => {
                let (low, high) = address_immediate(&mut *cpu, true);
                compare_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xe1 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xe2 => {
                cpu_setFlags(cpu_getFlags(&*cpu) | fetch_byte(cpu, game), &mut *cpu);
                break;
            }
            0xe3 => {
                let (low, high) = address_stack_relative(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xe4 => {
                let (low, high) = address_direct_page(cpu, game);
                compare_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xe5 => {
                let (low, high) = address_direct_page(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xe6 => {
                let (low, high) = address_direct_page(cpu, game);
                update_memory(cpu, low, high, 1, game);
                break;
            }
            0xe7 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xe8 => {
                update_x(1, &mut *cpu);
                break;
            }
            0xe9 => {
                let (low, high) = address_immediate(&mut *cpu, false);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xeb => {
                exchange_accumulator_bytes(&mut *cpu);
                break;
            }
            0xec => {
                let (low, high) = address_absolute(cpu, game);
                compare_register(cpu, low, high, CpuRegister::X, game);
                break;
            }
            0xed => {
                let (low, high) = address_absolute(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xee => {
                let (low, high) = address_absolute(cpu, game);
                update_memory(cpu, low, high, 1, game);
                break;
            }
            0xef => {
                let (low, high) = address_absolute_long(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf0 => {
                branch_relative(fetch_byte(cpu, game), (*cpu).z, &mut *cpu);
                break;
            }
            0xf1 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf2 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf3 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf4 => {
                let w: u16 = fetch_word(cpu, game);
                if w == 0xc2ae {
                    (*cpu).y = 0;
                }
                push_word(cpu, w, game);
                break;
            }
            0xf5 => {
                let (low, high) = address_direct_page_x(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf6 => {
                let (low, high) = address_direct_page_x(cpu, game);
                update_memory(cpu, low, high, 1, game);
                break;
            }
            0xf7 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xf8 => {
                (*cpu).d = true;
                break;
            }
            0xf9 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xfa => {
                pull_register(cpu, CpuRegister::X, game);
                break;
            }
            0xfb => {
                exchange_carry_emulation(&mut *cpu);
                break;
            }
            0xfc => {
                let target = address_absolute_indexed_indirect(cpu, game);
                push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = target;
                break;
            }
            0xfd => {
                let (low, high) = address_absolute_x(cpu, false, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xfe => {
                let (low, high) = address_absolute_x(cpu, true, game);
                update_memory(cpu, low, high, 1, game);
                break;
            }
            0xff => {
                let (low, high) = address_absolute_long_x(cpu, game);
                subtract_with_carry(cpu, low, high, game);
                break;
            }
            0xea | _ => {
                break;
            }
        }
    }
}
