use crate::cpu_infra::{CpuOpcodeHook, HookedFunctionRts};
use crate::host::Die;
use crate::snes::snes::{snes_cpuRead, snes_cpuWrite};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int32_t, int8_t,
    size_t, uint16_t, uint32, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart,
    Cpu, Dma, DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, SaveLoadFunc, Snes, Spc, Timer, WindowLayer,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut cyclesPerOpcode: [::core::ffi::c_int; 256] = [
    7 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
];
unsafe fn cpu_read(mut cpu: *mut Cpu, mut adr: uint32_t) -> uint8_t {
    return snes_cpuRead((*cpu).mem as *mut Snes, adr);
}
unsafe fn cpu_write(mut cpu: *mut Cpu, mut adr: uint32_t, mut val: uint8_t) {
    snes_cpuWrite((*cpu).mem as *mut Snes, adr, val);
}
pub unsafe fn cpu_init(
    mut mem: *mut ::core::ffi::c_void,
    mut memType: ::core::ffi::c_int,
) -> *mut Cpu {
    let mut cpu: *mut Cpu = malloc(::core::mem::size_of::<Cpu>() as size_t) as *mut Cpu;
    memset(
        cpu as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<Cpu>() as size_t,
    );
    (*cpu).mem = mem;
    (*cpu).memType = memType as uint8_t;
    return cpu;
}
pub unsafe fn cpu_free(mut cpu: *mut Cpu) {
    free(cpu as *mut ::core::ffi::c_void);
}
pub unsafe fn cpu_reset(mut cpu: *mut Cpu) {
    (*cpu).a = 0 as uint16_t;
    (*cpu).x = 0 as uint16_t;
    (*cpu).y = 0 as uint16_t;
    (*cpu).sp = 0x100 as uint16_t;
    (*cpu).pc = (cpu_read(cpu, 0xfffc as uint32_t) as ::core::ffi::c_int
        | (cpu_read(cpu, 0xfffd as uint32_t) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
    (*cpu).dp = 0 as uint16_t;
    (*cpu).k = 0 as uint8_t;
    (*cpu).db = 0 as uint8_t;
    (*cpu).c = false_0 != 0;
    (*cpu).z = false_0 != 0;
    (*cpu).v = false_0 != 0;
    (*cpu).n = false_0 != 0;
    (*cpu).i = true_0 != 0;
    (*cpu).d = false_0 != 0;
    (*cpu).xf = true_0 != 0;
    (*cpu).mf = true_0 != 0;
    (*cpu).e = true_0 != 0;
    (*cpu).irqWanted = false_0 != 0;
    (*cpu).nmiWanted = false_0 != 0;
    (*cpu).waiting = false_0 != 0;
    (*cpu).stopped = false_0 != 0;
    (*cpu).cyclesUsed = 0 as uint8_t;
}
pub unsafe fn cpu_saveload(
    mut cpu: *mut Cpu,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*cpu).a as *mut ::core::ffi::c_void,
        (37 as size_t).wrapping_sub(10 as size_t),
    );
    (*cpu).spBreakpoint = 0 as uint16_t;
}
pub unsafe fn cpu_runOpcode(mut cpu: *mut Cpu) -> ::core::ffi::c_int {
    (*cpu).cyclesUsed = 0 as uint8_t;
    if (*cpu).stopped {
        return 1 as ::core::ffi::c_int;
    }
    if !(*cpu).i && (*cpu).irqWanted as ::core::ffi::c_int != 0
        || (*cpu).nmiWanted as ::core::ffi::c_int != 0
    {
        (*cpu).cyclesUsed = 7 as uint8_t;
        if (*cpu).nmiWanted {
            (*cpu).nmiWanted = false_0 != 0;
            cpu_doInterrupt(cpu, false_0 != 0);
        } else {
            cpu_doInterrupt(cpu, true_0 != 0);
        }
    }
    let mut opcode: uint8_t = cpu_readOpcode(cpu);
    (*cpu).cyclesUsed = cyclesPerOpcode[opcode as usize] as uint8_t;
    cpu_doOpcode(cpu, opcode);
    return (*cpu).cyclesUsed as ::core::ffi::c_int;
}
unsafe fn cpu_readOpcode(mut cpu: *mut Cpu) -> uint8_t {
    let fresh0 = (*cpu).pc;
    (*cpu).pc = (*cpu).pc.wrapping_add(1);
    return cpu_read(
        cpu,
        (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | fresh0 as ::core::ffi::c_int) as uint32_t,
    );
}
unsafe fn cpu_readOpcodeWord(mut cpu: *mut Cpu) -> uint16_t {
    let mut low: uint8_t = cpu_readOpcode(cpu);
    return (low as ::core::ffi::c_int
        | (cpu_readOpcode(cpu) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
pub unsafe fn cpu_getFlags(mut cpu: *mut Cpu) -> uint8_t {
    let mut val: uint8_t = (((*cpu).n as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).v as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).mf as ::core::ffi::c_int) << 5 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).xf as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).d as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).i as ::core::ffi::c_int) << 2 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*cpu).z as ::core::ffi::c_int) << 1 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | (*cpu).c as ::core::ffi::c_int) as uint8_t;
    return val;
}
pub unsafe fn cpu_setFlags(mut cpu: *mut Cpu, mut val: uint8_t) {
    (*cpu).n = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
    (*cpu).v = val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
    (*cpu).mf = val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
    (*cpu).xf = val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
    (*cpu).d = val as ::core::ffi::c_int & 8 as ::core::ffi::c_int != 0;
    (*cpu).i = val as ::core::ffi::c_int & 4 as ::core::ffi::c_int != 0;
    (*cpu).z = val as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0;
    (*cpu).c = val as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
    if (*cpu).e {
        (*cpu).mf = true_0 != 0;
        (*cpu).xf = true_0 != 0;
        (*cpu).sp = ((*cpu).sp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16_t;
    }
    if (*cpu).xf {
        (*cpu).x = ((*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
        (*cpu).y = ((*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe fn cpu_setZN(mut cpu: *mut Cpu, mut value: uint16_t, mut byte: bool) {
    if byte {
        (*cpu).z =
            value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        (*cpu).n = value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
    } else {
        (*cpu).z = value as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        (*cpu).n = value as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
    };
}
unsafe fn cpu_doBranch(mut cpu: *mut Cpu, mut value: uint8_t, mut check: bool) {
    if check {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).pc =
            ((*cpu).pc as ::core::ffi::c_int + value as int8_t as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe fn cpu_pullByte(mut cpu: *mut Cpu) -> uint8_t {
    (*cpu).sp = (*cpu).sp.wrapping_add(1);
    if (*cpu).e {
        (*cpu).sp = ((*cpu).sp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16_t;
    }
    return cpu_read(cpu, (*cpu).sp as uint32_t);
}
unsafe fn cpu_pushByte(mut cpu: *mut Cpu, mut value: uint8_t) {
    cpu_write(cpu, (*cpu).sp as uint32_t, value);
    (*cpu).sp = (*cpu).sp.wrapping_sub(1);
    if (*cpu).e {
        (*cpu).sp = ((*cpu).sp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe fn cpu_pullWord(mut cpu: *mut Cpu) -> uint16_t {
    let mut value: uint8_t = cpu_pullByte(cpu);
    return (value as ::core::ffi::c_int
        | (cpu_pullByte(cpu) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe fn cpu_pushWord(mut cpu: *mut Cpu, mut value: uint16_t) {
    cpu_pushByte(
        cpu,
        (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t,
    );
    cpu_pushByte(
        cpu,
        (value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
}
unsafe fn cpu_readWord(mut cpu: *mut Cpu, mut adrl: uint32_t, mut adrh: uint32_t) -> uint16_t {
    let mut value: uint8_t = cpu_read(cpu, adrl);
    return (value as ::core::ffi::c_int
        | (cpu_read(cpu, adrh) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe fn cpu_writeWord(
    mut cpu: *mut Cpu,
    mut adrl: uint32_t,
    mut adrh: uint32_t,
    mut value: uint16_t,
    mut reversed: bool,
) {
    if reversed {
        cpu_write(
            cpu,
            adrh,
            (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t,
        );
        cpu_write(
            cpu,
            adrl,
            (value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
        );
    } else {
        cpu_write(
            cpu,
            adrl,
            (value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
        );
        cpu_write(
            cpu,
            adrh,
            (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t,
        );
    };
}
unsafe fn cpu_doInterrupt(mut cpu: *mut Cpu, mut irq: bool) {
    cpu_pushByte(cpu, (*cpu).k);
    cpu_pushWord(cpu, (*cpu).pc);
    cpu_pushByte(cpu, cpu_getFlags(cpu));
    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    (*cpu).i = true_0 != 0;
    (*cpu).d = false_0 != 0;
    (*cpu).k = 0 as uint8_t;
    if irq {
        (*cpu).pc = cpu_readWord(cpu, 0xffee as uint32_t, 0xffef as uint32_t);
    } else {
        (*cpu).pc = cpu_readWord(cpu, 0xffea as uint32_t, 0xffeb as uint32_t);
    };
}
unsafe fn cpu_adrImm(mut cpu: *mut Cpu, mut low: *mut uint32_t, mut xFlag: bool) -> uint32_t {
    if xFlag as ::core::ffi::c_int != 0 && (*cpu).xf as ::core::ffi::c_int != 0
        || !xFlag && (*cpu).mf as ::core::ffi::c_int != 0
    {
        let fresh1 = (*cpu).pc;
        (*cpu).pc = (*cpu).pc.wrapping_add(1);
        *low = (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | fresh1 as ::core::ffi::c_int) as uint32_t;
        return 0 as uint32_t;
    } else {
        let fresh2 = (*cpu).pc;
        (*cpu).pc = (*cpu).pc.wrapping_add(1);
        *low = (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | fresh2 as ::core::ffi::c_int) as uint32_t;
        let fresh3 = (*cpu).pc;
        (*cpu).pc = (*cpu).pc.wrapping_add(1);
        return (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | fresh3 as ::core::ffi::c_int) as uint32_t;
    };
}
unsafe fn cpu_adrDp(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
    return ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrDpx(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((*cpu).dp as ::core::ffi::c_int
        + adr as ::core::ffi::c_int
        + (*cpu).x as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
    return ((*cpu).dp as ::core::ffi::c_int
        + adr as ::core::ffi::c_int
        + (*cpu).x as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrDpy(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((*cpu).dp as ::core::ffi::c_int
        + adr as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
    return ((*cpu).dp as ::core::ffi::c_int
        + adr as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrIdp(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer: uint16_t = cpu_readWord(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as uint32_t,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    );
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrIdx(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer: uint16_t = cpu_readWord(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int
            + adr as ::core::ffi::c_int
            + (*cpu).x as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
        ((*cpu).dp as ::core::ffi::c_int
            + adr as ::core::ffi::c_int
            + (*cpu).x as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    );
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrIdy(mut cpu: *mut Cpu, mut low: *mut uint32_t, mut write: bool) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer: uint16_t = cpu_readWord(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as uint32_t,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    );
    if write as ::core::ffi::c_int != 0
        && (!(*cpu).xf
            || pointer as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                != pointer as ::core::ffi::c_int + (*cpu).y as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)
    {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrIdl(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer: uint32_t = cpu_readWord(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as uint32_t,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    ) as uint32_t;
    pointer |= ((cpu_read(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 2 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    ) as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32_t;
    *low = pointer;
    return pointer.wrapping_add(1 as uint32_t) & 0xffffff as uint32_t;
}
unsafe fn cpu_adrIly(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    if (*cpu).dp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer: uint32_t = cpu_readWord(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as uint32_t,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    ) as uint32_t;
    pointer |= ((cpu_read(
        cpu,
        ((*cpu).dp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 2 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    ) as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32_t;
    *low = pointer.wrapping_add((*cpu).y as uint32_t) & 0xffffff as uint32_t;
    return pointer
        .wrapping_add((*cpu).y as uint32_t)
        .wrapping_add(1 as uint32_t)
        & 0xffffff as uint32_t;
}
unsafe fn cpu_adrSr(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    *low = ((*cpu).sp as ::core::ffi::c_int + adr as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
    return ((*cpu).sp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrIsy(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint8_t = cpu_readOpcode(cpu);
    let mut pointer: uint16_t = cpu_readWord(
        cpu,
        ((*cpu).sp as ::core::ffi::c_int + adr as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as uint32_t,
        ((*cpu).sp as ::core::ffi::c_int + adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xffff as ::core::ffi::c_int) as uint32_t,
    );
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + pointer as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrAbs(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint16_t = cpu_readOpcodeWord(cpu);
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrAbx(mut cpu: *mut Cpu, mut low: *mut uint32_t, mut write: bool) -> uint32_t {
    let mut adr: uint16_t = cpu_readOpcodeWord(cpu);
    if write as ::core::ffi::c_int != 0
        && (!(*cpu).xf
            || adr as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                != adr as ::core::ffi::c_int + (*cpu).x as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)
    {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int
        + (*cpu).x as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int
        + (*cpu).x as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrAby(mut cpu: *mut Cpu, mut low: *mut uint32_t, mut write: bool) -> uint32_t {
    let mut adr: uint16_t = cpu_readOpcodeWord(cpu);
    if write as ::core::ffi::c_int != 0
        && (!(*cpu).xf
            || adr as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                != adr as ::core::ffi::c_int + (*cpu).y as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)
    {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
    return ((((*cpu).db as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
        + adr as ::core::ffi::c_int
        + (*cpu).y as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        & 0xffffff as ::core::ffi::c_int) as uint32_t;
}
unsafe fn cpu_adrAbl(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint32_t = cpu_readOpcodeWord(cpu) as uint32_t;
    adr |= ((cpu_readOpcode(cpu) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32_t;
    *low = adr;
    return adr.wrapping_add(1 as uint32_t) & 0xffffff as uint32_t;
}
unsafe fn cpu_adrAlx(mut cpu: *mut Cpu, mut low: *mut uint32_t) -> uint32_t {
    let mut adr: uint32_t = cpu_readOpcodeWord(cpu) as uint32_t;
    adr |= ((cpu_readOpcode(cpu) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32_t;
    *low = adr.wrapping_add((*cpu).x as uint32_t) & 0xffffff as uint32_t;
    return adr
        .wrapping_add((*cpu).x as uint32_t)
        .wrapping_add(1 as uint32_t)
        & 0xffffff as uint32_t;
}
unsafe fn cpu_adrIax(mut cpu: *mut Cpu) -> uint16_t {
    let mut adr: uint16_t = cpu_readOpcodeWord(cpu);
    return cpu_readWord(
        cpu,
        (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | adr as ::core::ffi::c_int + (*cpu).x as ::core::ffi::c_int
                & 0xffff as ::core::ffi::c_int) as uint32_t,
        (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | adr as ::core::ffi::c_int + (*cpu).x as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                & 0xffff as ::core::ffi::c_int) as uint32_t,
    );
}
unsafe fn cpu_and(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | (*cpu).a as ::core::ffi::c_int
                & value as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & value_0 as ::core::ffi::c_int) as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_ora(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | ((*cpu).a as ::core::ffi::c_int | value as ::core::ffi::c_int)
                & 0xff as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int | value_0 as ::core::ffi::c_int) as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_eor(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | ((*cpu).a as ::core::ffi::c_int ^ value as ::core::ffi::c_int)
                & 0xff as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int ^ value_0 as ::core::ffi::c_int) as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_adc(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*cpu).d {
            result = ((*cpu).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (*cpu).c as ::core::ffi::c_int;
            if result > 0x9 as ::core::ffi::c_int {
                result = (result + 0x6 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                    + 0x10 as ::core::ffi::c_int;
            }
            result = ((*cpu).a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + (value as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + result;
        } else {
            result = ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                + value as ::core::ffi::c_int
                + (*cpu).c as ::core::ffi::c_int;
        }
        (*cpu).v = (*cpu).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                != result & 0x80 as ::core::ffi::c_int;
        if (*cpu).d as ::core::ffi::c_int != 0 && result > 0x9f as ::core::ffi::c_int {
            result += 0x60 as ::core::ffi::c_int;
        }
        (*cpu).c = result > 0xff as ::core::ffi::c_int;
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | result & 0xff as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        let mut result_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*cpu).d {
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (*cpu).c as ::core::ffi::c_int;
            if result_0 > 0x9 as ::core::ffi::c_int {
                result_0 = (result_0 + 0x6 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                    + 0x10 as ::core::ffi::c_int;
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + result_0;
            if result_0 > 0x9f as ::core::ffi::c_int {
                result_0 = (result_0 + 0x60 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                    + 0x100 as ::core::ffi::c_int;
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int)
                + result_0;
            if result_0 > 0x9ff as ::core::ffi::c_int {
                result_0 = (result_0 + 0x600 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                    + 0x1000 as ::core::ffi::c_int;
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int)
                + result_0;
        } else {
            result_0 = (*cpu).a as ::core::ffi::c_int
                + value_0 as ::core::ffi::c_int
                + (*cpu).c as ::core::ffi::c_int;
        }
        (*cpu).v = (*cpu).a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            && value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != result_0 & 0x8000 as ::core::ffi::c_int;
        if (*cpu).d as ::core::ffi::c_int != 0 && result_0 > 0x9fff as ::core::ffi::c_int {
            result_0 += 0x6000 as ::core::ffi::c_int;
        }
        (*cpu).c = result_0 > 0xffff as ::core::ffi::c_int;
        (*cpu).a = result_0 as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_sbc(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t =
            (cpu_read(cpu, low) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
        let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*cpu).d {
            result = ((*cpu).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (*cpu).c as ::core::ffi::c_int;
            if result < 0x10 as ::core::ffi::c_int {
                result = result - 0x6 as ::core::ffi::c_int
                    & (if (result - 0x6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                        0xf as ::core::ffi::c_int
                    } else {
                        0x1f as ::core::ffi::c_int
                    });
            }
            result = ((*cpu).a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + (value as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + result;
        } else {
            result = ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                + value as ::core::ffi::c_int
                + (*cpu).c as ::core::ffi::c_int;
        }
        (*cpu).v = (*cpu).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                != result & 0x80 as ::core::ffi::c_int;
        if (*cpu).d as ::core::ffi::c_int != 0 && result < 0x100 as ::core::ffi::c_int {
            result -= 0x60 as ::core::ffi::c_int;
        }
        (*cpu).c = result > 0xff as ::core::ffi::c_int;
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | result & 0xff as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int
            ^ 0xffff as ::core::ffi::c_int) as uint16_t;
        let mut result_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*cpu).d {
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                + (*cpu).c as ::core::ffi::c_int;
            if result_0 < 0x10 as ::core::ffi::c_int {
                result_0 = result_0 - 0x6 as ::core::ffi::c_int
                    & (if (result_0 - 0x6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                        0xf as ::core::ffi::c_int
                    } else {
                        0x1f as ::core::ffi::c_int
                    });
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
                + result_0;
            if result_0 < 0x100 as ::core::ffi::c_int {
                result_0 = result_0 - 0x60 as ::core::ffi::c_int
                    & (if (result_0 - 0x60 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                        0xff as ::core::ffi::c_int
                    } else {
                        0x1ff as ::core::ffi::c_int
                    });
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int)
                + result_0;
            if result_0 < 0x1000 as ::core::ffi::c_int {
                result_0 = result_0 - 0x600 as ::core::ffi::c_int
                    & (if (result_0 - 0x600 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                        0xfff as ::core::ffi::c_int
                    } else {
                        0x1fff as ::core::ffi::c_int
                    });
            }
            result_0 = ((*cpu).a as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int)
                + (value_0 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int)
                + result_0;
        } else {
            result_0 = (*cpu).a as ::core::ffi::c_int
                + value_0 as ::core::ffi::c_int
                + (*cpu).c as ::core::ffi::c_int;
        }
        (*cpu).v = (*cpu).a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            && value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != result_0 & 0x8000 as ::core::ffi::c_int;
        if (*cpu).d as ::core::ffi::c_int != 0 && result_0 < 0x10000 as ::core::ffi::c_int {
            result_0 -= 0x6000 as ::core::ffi::c_int;
        }
        (*cpu).c = result_0 > 0xffff as ::core::ffi::c_int;
        (*cpu).a = result_0 as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_cmp(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        let mut value: uint8_t =
            (cpu_read(cpu, low) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
        result = ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            + value as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xff as ::core::ffi::c_int;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int
            ^ 0xffff as ::core::ffi::c_int) as uint16_t;
        result = (*cpu).a as ::core::ffi::c_int
            + value_0 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xffff as ::core::ffi::c_int;
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_cpx(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).xf {
        let mut value: uint8_t =
            (cpu_read(cpu, low) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
        result = ((*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            + value as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xff as ::core::ffi::c_int;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int
            ^ 0xffff as ::core::ffi::c_int) as uint16_t;
        result = (*cpu).x as ::core::ffi::c_int
            + value_0 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xffff as ::core::ffi::c_int;
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).xf);
}
unsafe fn cpu_cpy(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).xf {
        let mut value: uint8_t =
            (cpu_read(cpu, low) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
        result = ((*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
            + value as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xff as ::core::ffi::c_int;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int
            ^ 0xffff as ::core::ffi::c_int) as uint16_t;
        result = (*cpu).y as ::core::ffi::c_int
            + value_0 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        (*cpu).c = result > 0xffff as ::core::ffi::c_int;
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).xf);
}
unsafe fn cpu_bit(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        let mut result: uint8_t = ((*cpu).a as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            & value as ::core::ffi::c_int) as uint8_t;
        (*cpu).z = result as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        (*cpu).n = value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
        (*cpu).v = value as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        let mut result_0: uint16_t =
            ((*cpu).a as ::core::ffi::c_int & value_0 as ::core::ffi::c_int) as uint16_t;
        (*cpu).z = result_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        (*cpu).n = value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
        (*cpu).v = value_0 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0;
    };
}
unsafe fn cpu_lda(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
            | cpu_read(cpu, low) as ::core::ffi::c_int) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).a = cpu_readWord(cpu, low, high);
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_ldx(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).xf {
        (*cpu).x = cpu_read(cpu, low) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).x = cpu_readWord(cpu, low, high);
    }
    cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
}
unsafe fn cpu_ldy(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).xf {
        (*cpu).y = cpu_read(cpu, low) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).y = cpu_readWord(cpu, low, high);
    }
    cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
}
unsafe fn cpu_sta(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        cpu_write(cpu, low, (*cpu).a as uint8_t);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).a, false_0 != 0);
    };
}
unsafe fn cpu_stx(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).xf {
        cpu_write(cpu, low, (*cpu).x as uint8_t);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).x, false_0 != 0);
    };
}
unsafe fn cpu_sty(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).xf {
        cpu_write(cpu, low, (*cpu).y as uint8_t);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).y, false_0 != 0);
    };
}
unsafe fn cpu_stz(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        cpu_write(cpu, low, 0 as uint8_t);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, 0 as uint16_t, false_0 != 0);
    };
}
unsafe fn cpu_ror(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut carry: bool = false_0 != 0;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        carry = value as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
        result = value as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
            | ((*cpu).c as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        carry = value_0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
        result = value_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
            | ((*cpu).c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
    (*cpu).c = carry;
}
unsafe fn cpu_rol(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        result = (cpu_read(cpu, low) as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (*cpu).c as ::core::ffi::c_int;
        (*cpu).c = result & 0x100 as ::core::ffi::c_int != 0;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        result = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (*cpu).c as ::core::ffi::c_int;
        (*cpu).c = result & 0x10000 as ::core::ffi::c_int != 0;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_lsr(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).c = value as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
        result = value as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).c = value_0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
        result = value_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_asl(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        result = (cpu_read(cpu, low) as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
        (*cpu).c = result & 0x100 as ::core::ffi::c_int != 0;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        result = (cpu_readWord(cpu, low, high) as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
        (*cpu).c = result & 0x10000 as ::core::ffi::c_int != 0;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_inc(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        result = cpu_read(cpu, low) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        result = cpu_readWord(cpu, low, high) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_dec(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cpu).mf {
        result = cpu_read(cpu, low) as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        cpu_write(cpu, low, result as uint8_t);
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        result = cpu_readWord(cpu, low, high) as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        cpu_writeWord(cpu, low, high, result as uint16_t, true_0 != 0);
    }
    cpu_setZN(cpu, result as uint16_t, (*cpu).mf);
}
unsafe fn cpu_tsb(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).z = (*cpu).a as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            & value as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int;
        cpu_write(
            cpu,
            low,
            (value as ::core::ffi::c_int
                | (*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                as uint8_t,
        );
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).z = (*cpu).a as ::core::ffi::c_int & value_0 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int;
        cpu_writeWord(
            cpu,
            low,
            high,
            (value_0 as ::core::ffi::c_int | (*cpu).a as ::core::ffi::c_int) as uint16_t,
            true_0 != 0,
        );
    };
}
unsafe fn cpu_trb(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low);
        (*cpu).z = (*cpu).a as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            & value as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int;
        cpu_write(
            cpu,
            low,
            (value as ::core::ffi::c_int
                & !((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int))
                as uint8_t,
        );
    } else {
        (*cpu).cyclesUsed =
            ((*cpu).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high);
        (*cpu).z = (*cpu).a as ::core::ffi::c_int & value_0 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int;
        cpu_writeWord(
            cpu,
            low,
            high,
            (value_0 as ::core::ffi::c_int & !((*cpu).a as ::core::ffi::c_int)) as uint16_t,
            true_0 != 0,
        );
    };
}
pub static mut pc_hist: [uint32_t; 8] = [0; 8];
pub static mut pc_hist_ctr: uint32_t = 0;
pub static mut pc_bp: uint32_t = 0 as uint32_t;
pub unsafe fn DumpCpuHistory() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        printf(
            b"PC history: 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            pc_hist[(pc_hist_ctr.wrapping_add(i as uint32_t) & 7 as uint32_t) as usize],
        );
        i += 1;
    }
}
unsafe fn cpu_doOpcode(mut cpu: *mut Cpu, mut opcode: uint8_t) {
    let mut cur_pc: uint32 = (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
        | (*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
        as uint32;
    pc_hist[pc_hist_ctr as usize] = cur_pc as uint32_t;
    pc_hist_ctr = pc_hist_ctr.wrapping_add(1 as uint32_t) & 7 as uint32_t;
    if cur_pc == pc_bp {
        opcode = (opcode as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint8_t;
    }
    loop {
        match opcode as ::core::ffi::c_int {
            0 => {
                let mut addr: uint32_t =
                    (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).pc as ::core::ffi::c_int
                        | 0x800000 as ::core::ffi::c_int) as uint32_t;
                opcode = CpuOpcodeHook((addr as uint32).wrapping_sub(1 as uint32)) as uint8_t;
                match opcode as ::core::ffi::c_int {
                    0 => {
                        break;
                    }
                    1 => {
                        (*cpu).pc = (cpu_pullWord(cpu) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int)
                            as uint16_t;
                        break;
                    }
                    2 => {
                        (*cpu).pc = (cpu_pullWord(cpu) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int)
                            as uint16_t;
                        (*cpu).k = cpu_pullByte(cpu);
                        break;
                    }
                    229 | 233 | 237 => {
                        (*cpu).c = 1 as ::core::ffi::c_int != 0;
                    }
                    101 | 109 | 105 | 125 | 127 => {
                        (*cpu).c = 0 as ::core::ffi::c_int != 0;
                    }
                    _ => {}
                }
            }
            1 => {
                let mut low: uint32_t = 0 as uint32_t;
                let mut high: uint32_t = cpu_adrIdx(cpu, &raw mut low);
                cpu_ora(cpu, low, high);
                break;
            }
            2 => {
                cpu_readOpcode(cpu);
                cpu_pushByte(cpu, (*cpu).k);
                cpu_pushWord(cpu, (*cpu).pc);
                cpu_pushByte(cpu, cpu_getFlags(cpu));
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).i = true_0 != 0;
                (*cpu).d = false_0 != 0;
                (*cpu).k = 0 as uint8_t;
                (*cpu).pc = cpu_readWord(cpu, 0xffe4 as uint32_t, 0xffe5 as uint32_t);
                break;
            }
            3 => {
                let mut low_0: uint32_t = 0 as uint32_t;
                let mut high_0: uint32_t = cpu_adrSr(cpu, &raw mut low_0);
                cpu_ora(cpu, low_0, high_0);
                break;
            }
            4 => {
                let mut low_1: uint32_t = 0 as uint32_t;
                let mut high_1: uint32_t = cpu_adrDp(cpu, &raw mut low_1);
                cpu_tsb(cpu, low_1, high_1);
                break;
            }
            5 => {
                let mut low_2: uint32_t = 0 as uint32_t;
                let mut high_2: uint32_t = cpu_adrDp(cpu, &raw mut low_2);
                cpu_ora(cpu, low_2, high_2);
                break;
            }
            6 => {
                let mut low_3: uint32_t = 0 as uint32_t;
                let mut high_3: uint32_t = cpu_adrDp(cpu, &raw mut low_3);
                cpu_asl(cpu, low_3, high_3);
                break;
            }
            7 => {
                let mut low_4: uint32_t = 0 as uint32_t;
                let mut high_4: uint32_t = cpu_adrIdl(cpu, &raw mut low_4);
                cpu_ora(cpu, low_4, high_4);
                break;
            }
            8 => {
                cpu_pushByte(cpu, cpu_getFlags(cpu));
                break;
            }
            9 => {
                let mut low_5: uint32_t = 0 as uint32_t;
                let mut high_5: uint32_t = cpu_adrImm(cpu, &raw mut low_5, false_0 != 0);
                cpu_ora(cpu, low_5, high_5);
                break;
            }
            10 => {
                if (*cpu).mf {
                    (*cpu).c = (*cpu).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | ((*cpu).a as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).c = (*cpu).a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
                    (*cpu).a =
                        (((*cpu).a as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint16_t;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            11 => {
                cpu_pushWord(cpu, (*cpu).dp);
                break;
            }
            12 => {
                let mut low_6: uint32_t = 0 as uint32_t;
                let mut high_6: uint32_t = cpu_adrAbs(cpu, &raw mut low_6);
                cpu_tsb(cpu, low_6, high_6);
                break;
            }
            13 => {
                let mut low_7: uint32_t = 0 as uint32_t;
                let mut high_7: uint32_t = cpu_adrAbs(cpu, &raw mut low_7);
                cpu_ora(cpu, low_7, high_7);
                break;
            }
            14 => {
                let mut low_8: uint32_t = 0 as uint32_t;
                let mut high_8: uint32_t = cpu_adrAbs(cpu, &raw mut low_8);
                cpu_asl(cpu, low_8, high_8);
                break;
            }
            15 => {
                let mut low_9: uint32_t = 0 as uint32_t;
                let mut high_9: uint32_t = cpu_adrAbl(cpu, &raw mut low_9);
                cpu_ora(cpu, low_9, high_9);
                break;
            }
            16 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), !(*cpu).n);
                break;
            }
            17 => {
                let mut low_10: uint32_t = 0 as uint32_t;
                let mut high_10: uint32_t = cpu_adrIdy(cpu, &raw mut low_10, false_0 != 0);
                cpu_ora(cpu, low_10, high_10);
                break;
            }
            18 => {
                let mut low_11: uint32_t = 0 as uint32_t;
                let mut high_11: uint32_t = cpu_adrIdp(cpu, &raw mut low_11);
                cpu_ora(cpu, low_11, high_11);
                break;
            }
            19 => {
                let mut low_12: uint32_t = 0 as uint32_t;
                let mut high_12: uint32_t = cpu_adrIsy(cpu, &raw mut low_12);
                cpu_ora(cpu, low_12, high_12);
                break;
            }
            20 => {
                let mut low_13: uint32_t = 0 as uint32_t;
                let mut high_13: uint32_t = cpu_adrDp(cpu, &raw mut low_13);
                cpu_trb(cpu, low_13, high_13);
                break;
            }
            21 => {
                let mut low_14: uint32_t = 0 as uint32_t;
                let mut high_14: uint32_t = cpu_adrDpx(cpu, &raw mut low_14);
                cpu_ora(cpu, low_14, high_14);
                break;
            }
            22 => {
                let mut low_15: uint32_t = 0 as uint32_t;
                let mut high_15: uint32_t = cpu_adrDpx(cpu, &raw mut low_15);
                cpu_asl(cpu, low_15, high_15);
                break;
            }
            23 => {
                let mut low_16: uint32_t = 0 as uint32_t;
                let mut high_16: uint32_t = cpu_adrIly(cpu, &raw mut low_16);
                cpu_ora(cpu, low_16, high_16);
                break;
            }
            24 => {
                (*cpu).c = false_0 != 0;
                break;
            }
            25 => {
                let mut low_17: uint32_t = 0 as uint32_t;
                let mut high_17: uint32_t = cpu_adrAby(cpu, &raw mut low_17, false_0 != 0);
                cpu_ora(cpu, low_17, high_17);
                break;
            }
            26 => {
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).a as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a = (*cpu).a.wrapping_add(1);
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            27 => {
                (*cpu).sp = (*cpu).a;
                break;
            }
            28 => {
                let mut low_18: uint32_t = 0 as uint32_t;
                let mut high_18: uint32_t = cpu_adrAbs(cpu, &raw mut low_18);
                cpu_trb(cpu, low_18, high_18);
                break;
            }
            29 => {
                let mut low_19: uint32_t = 0 as uint32_t;
                let mut high_19: uint32_t = cpu_adrAbx(cpu, &raw mut low_19, false_0 != 0);
                cpu_ora(cpu, low_19, high_19);
                break;
            }
            30 => {
                let mut low_20: uint32_t = 0 as uint32_t;
                let mut high_20: uint32_t = cpu_adrAbx(cpu, &raw mut low_20, true_0 != 0);
                cpu_asl(cpu, low_20, high_20);
                break;
            }
            31 => {
                let mut low_21: uint32_t = 0 as uint32_t;
                let mut high_21: uint32_t = cpu_adrAlx(cpu, &raw mut low_21);
                cpu_ora(cpu, low_21, high_21);
                break;
            }
            32 => {
                let mut value: uint16_t = cpu_readOpcodeWord(cpu);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                );
                (*cpu).pc = value;
                break;
            }
            33 => {
                let mut low_22: uint32_t = 0 as uint32_t;
                let mut high_22: uint32_t = cpu_adrIdx(cpu, &raw mut low_22);
                cpu_and(cpu, low_22, high_22);
                break;
            }
            34 => {
                let mut value_0: uint16_t = cpu_readOpcodeWord(cpu);
                let mut newK: uint8_t = cpu_readOpcode(cpu);
                cpu_pushByte(cpu, (*cpu).k);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                );
                (*cpu).pc = value_0;
                (*cpu).k = newK;
                break;
            }
            35 => {
                let mut low_23: uint32_t = 0 as uint32_t;
                let mut high_23: uint32_t = cpu_adrSr(cpu, &raw mut low_23);
                cpu_and(cpu, low_23, high_23);
                break;
            }
            36 => {
                let mut low_24: uint32_t = 0 as uint32_t;
                let mut high_24: uint32_t = cpu_adrDp(cpu, &raw mut low_24);
                cpu_bit(cpu, low_24, high_24);
                break;
            }
            37 => {
                let mut low_25: uint32_t = 0 as uint32_t;
                let mut high_25: uint32_t = cpu_adrDp(cpu, &raw mut low_25);
                cpu_and(cpu, low_25, high_25);
                break;
            }
            38 => {
                let mut low_26: uint32_t = 0 as uint32_t;
                let mut high_26: uint32_t = cpu_adrDp(cpu, &raw mut low_26);
                cpu_rol(cpu, low_26, high_26);
                break;
            }
            39 => {
                let mut low_27: uint32_t = 0 as uint32_t;
                let mut high_27: uint32_t = cpu_adrIdl(cpu, &raw mut low_27);
                cpu_and(cpu, low_27, high_27);
                break;
            }
            40 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu));
                break;
            }
            41 => {
                let mut low_28: uint32_t = 0 as uint32_t;
                let mut high_28: uint32_t = cpu_adrImm(cpu, &raw mut low_28, false_0 != 0);
                cpu_and(cpu, low_28, high_28);
                break;
            }
            42 => {
                let mut result: ::core::ffi::c_int = ((*cpu).a as ::core::ffi::c_int)
                    << 1 as ::core::ffi::c_int
                    | (*cpu).c as ::core::ffi::c_int;
                if (*cpu).mf {
                    (*cpu).c = result & 0x100 as ::core::ffi::c_int != 0;
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | result & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).c = result & 0x10000 as ::core::ffi::c_int != 0;
                    (*cpu).a = result as uint16_t;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            43 => {
                (*cpu).dp = cpu_pullWord(cpu);
                cpu_setZN(cpu, (*cpu).dp, false_0 != 0);
                break;
            }
            44 => {
                let mut low_29: uint32_t = 0 as uint32_t;
                let mut high_29: uint32_t = cpu_adrAbs(cpu, &raw mut low_29);
                cpu_bit(cpu, low_29, high_29);
                break;
            }
            45 => {
                let mut low_30: uint32_t = 0 as uint32_t;
                let mut high_30: uint32_t = cpu_adrAbs(cpu, &raw mut low_30);
                cpu_and(cpu, low_30, high_30);
                break;
            }
            46 => {
                let mut low_31: uint32_t = 0 as uint32_t;
                let mut high_31: uint32_t = cpu_adrAbs(cpu, &raw mut low_31);
                cpu_rol(cpu, low_31, high_31);
                break;
            }
            47 => {
                let mut low_32: uint32_t = 0 as uint32_t;
                let mut high_32: uint32_t = cpu_adrAbl(cpu, &raw mut low_32);
                cpu_and(cpu, low_32, high_32);
                break;
            }
            48 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), (*cpu).n);
                break;
            }
            49 => {
                let mut low_33: uint32_t = 0 as uint32_t;
                let mut high_33: uint32_t = cpu_adrIdy(cpu, &raw mut low_33, false_0 != 0);
                cpu_and(cpu, low_33, high_33);
                break;
            }
            50 => {
                let mut low_34: uint32_t = 0 as uint32_t;
                let mut high_34: uint32_t = cpu_adrIdp(cpu, &raw mut low_34);
                cpu_and(cpu, low_34, high_34);
                break;
            }
            51 => {
                let mut low_35: uint32_t = 0 as uint32_t;
                let mut high_35: uint32_t = cpu_adrIsy(cpu, &raw mut low_35);
                cpu_and(cpu, low_35, high_35);
                break;
            }
            52 => {
                let mut low_36: uint32_t = 0 as uint32_t;
                let mut high_36: uint32_t = cpu_adrDpx(cpu, &raw mut low_36);
                cpu_bit(cpu, low_36, high_36);
                break;
            }
            53 => {
                let mut low_37: uint32_t = 0 as uint32_t;
                let mut high_37: uint32_t = cpu_adrDpx(cpu, &raw mut low_37);
                cpu_and(cpu, low_37, high_37);
                break;
            }
            54 => {
                let mut low_38: uint32_t = 0 as uint32_t;
                let mut high_38: uint32_t = cpu_adrDpx(cpu, &raw mut low_38);
                cpu_rol(cpu, low_38, high_38);
                break;
            }
            55 => {
                let mut low_39: uint32_t = 0 as uint32_t;
                let mut high_39: uint32_t = cpu_adrIly(cpu, &raw mut low_39);
                cpu_and(cpu, low_39, high_39);
                break;
            }
            56 => {
                (*cpu).c = true_0 != 0;
                break;
            }
            57 => {
                let mut low_40: uint32_t = 0 as uint32_t;
                let mut high_40: uint32_t = cpu_adrAby(cpu, &raw mut low_40, false_0 != 0);
                cpu_and(cpu, low_40, high_40);
                break;
            }
            58 => {
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).a as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a = (*cpu).a.wrapping_sub(1);
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            59 => {
                (*cpu).a = (*cpu).sp;
                cpu_setZN(cpu, (*cpu).a, false_0 != 0);
                break;
            }
            60 => {
                let mut low_41: uint32_t = 0 as uint32_t;
                let mut high_41: uint32_t = cpu_adrAbx(cpu, &raw mut low_41, false_0 != 0);
                cpu_bit(cpu, low_41, high_41);
                break;
            }
            61 => {
                let mut low_42: uint32_t = 0 as uint32_t;
                let mut high_42: uint32_t = cpu_adrAbx(cpu, &raw mut low_42, false_0 != 0);
                cpu_and(cpu, low_42, high_42);
                break;
            }
            62 => {
                let mut low_43: uint32_t = 0 as uint32_t;
                let mut high_43: uint32_t = cpu_adrAbx(cpu, &raw mut low_43, true_0 != 0);
                cpu_rol(cpu, low_43, high_43);
                break;
            }
            63 => {
                let mut low_44: uint32_t = 0 as uint32_t;
                let mut high_44: uint32_t = cpu_adrAlx(cpu, &raw mut low_44);
                cpu_and(cpu, low_44, high_44);
                break;
            }
            64 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu));
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).pc = cpu_pullWord(cpu);
                (*cpu).k = cpu_pullByte(cpu);
                break;
            }
            65 => {
                let mut low_45: uint32_t = 0 as uint32_t;
                let mut high_45: uint32_t = cpu_adrIdx(cpu, &raw mut low_45);
                cpu_eor(cpu, low_45, high_45);
                break;
            }
            66 => {
                cpu_readOpcode(cpu);
                break;
            }
            67 => {
                let mut low_46: uint32_t = 0 as uint32_t;
                let mut high_46: uint32_t = cpu_adrSr(cpu, &raw mut low_46);
                cpu_eor(cpu, low_46, high_46);
                break;
            }
            68 => {
                let mut dest: uint8_t = cpu_readOpcode(cpu);
                let mut src: uint8_t = cpu_readOpcode(cpu);
                (*cpu).db = dest;
                cpu_write(
                    cpu,
                    ((dest as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).y as ::core::ffi::c_int) as uint32_t,
                    cpu_read(
                        cpu,
                        ((src as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).x as ::core::ffi::c_int) as uint32_t,
                    ),
                );
                (*cpu).a = (*cpu).a.wrapping_sub(1);
                (*cpu).x = (*cpu).x.wrapping_sub(1);
                (*cpu).y = (*cpu).y.wrapping_sub(1);
                if (*cpu).a as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
                    (*cpu).pc =
                        ((*cpu).pc as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16_t;
                }
                if (*cpu).xf {
                    (*cpu).x =
                        ((*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                    (*cpu).y =
                        ((*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                }
                break;
            }
            69 => {
                let mut low_47: uint32_t = 0 as uint32_t;
                let mut high_47: uint32_t = cpu_adrDp(cpu, &raw mut low_47);
                cpu_eor(cpu, low_47, high_47);
                break;
            }
            70 => {
                let mut low_48: uint32_t = 0 as uint32_t;
                let mut high_48: uint32_t = cpu_adrDp(cpu, &raw mut low_48);
                cpu_lsr(cpu, low_48, high_48);
                break;
            }
            71 => {
                let mut low_49: uint32_t = 0 as uint32_t;
                let mut high_49: uint32_t = cpu_adrIdl(cpu, &raw mut low_49);
                cpu_eor(cpu, low_49, high_49);
                break;
            }
            72 => {
                if (*cpu).mf {
                    cpu_pushByte(cpu, (*cpu).a as uint8_t);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).a);
                }
                break;
            }
            73 => {
                let mut low_50: uint32_t = 0 as uint32_t;
                let mut high_50: uint32_t = cpu_adrImm(cpu, &raw mut low_50, false_0 != 0);
                cpu_eor(cpu, low_50, high_50);
                break;
            }
            74 => {
                (*cpu).c = (*cpu).a as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                            & 0x7f as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a =
                        ((*cpu).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16_t;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            75 => {
                cpu_pushByte(cpu, (*cpu).k);
                break;
            }
            76 => {
                (*cpu).pc = cpu_readOpcodeWord(cpu);
                break;
            }
            77 => {
                let mut low_51: uint32_t = 0 as uint32_t;
                let mut high_51: uint32_t = cpu_adrAbs(cpu, &raw mut low_51);
                cpu_eor(cpu, low_51, high_51);
                break;
            }
            78 => {
                let mut low_52: uint32_t = 0 as uint32_t;
                let mut high_52: uint32_t = cpu_adrAbs(cpu, &raw mut low_52);
                cpu_lsr(cpu, low_52, high_52);
                break;
            }
            79 => {
                let mut low_53: uint32_t = 0 as uint32_t;
                let mut high_53: uint32_t = cpu_adrAbl(cpu, &raw mut low_53);
                cpu_eor(cpu, low_53, high_53);
                break;
            }
            80 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), !(*cpu).v);
                break;
            }
            81 => {
                let mut low_54: uint32_t = 0 as uint32_t;
                let mut high_54: uint32_t = cpu_adrIdy(cpu, &raw mut low_54, false_0 != 0);
                cpu_eor(cpu, low_54, high_54);
                break;
            }
            82 => {
                let mut low_55: uint32_t = 0 as uint32_t;
                let mut high_55: uint32_t = cpu_adrIdp(cpu, &raw mut low_55);
                cpu_eor(cpu, low_55, high_55);
                break;
            }
            83 => {
                let mut low_56: uint32_t = 0 as uint32_t;
                let mut high_56: uint32_t = cpu_adrIsy(cpu, &raw mut low_56);
                cpu_eor(cpu, low_56, high_56);
                break;
            }
            84 => {
                let mut dest_0: uint8_t = cpu_readOpcode(cpu);
                let mut src_0: uint8_t = cpu_readOpcode(cpu);
                (*cpu).db = dest_0;
                cpu_write(
                    cpu,
                    ((dest_0 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).y as ::core::ffi::c_int) as uint32_t,
                    cpu_read(
                        cpu,
                        ((src_0 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).x as ::core::ffi::c_int) as uint32_t,
                    ),
                );
                (*cpu).a = (*cpu).a.wrapping_sub(1);
                (*cpu).x = (*cpu).x.wrapping_add(1);
                (*cpu).y = (*cpu).y.wrapping_add(1);
                if (*cpu).a as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
                    (*cpu).pc =
                        ((*cpu).pc as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16_t;
                }
                if (*cpu).xf {
                    (*cpu).x =
                        ((*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                    (*cpu).y =
                        ((*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                }
                break;
            }
            85 => {
                let mut low_57: uint32_t = 0 as uint32_t;
                let mut high_57: uint32_t = cpu_adrDpx(cpu, &raw mut low_57);
                cpu_eor(cpu, low_57, high_57);
                break;
            }
            86 => {
                let mut low_58: uint32_t = 0 as uint32_t;
                let mut high_58: uint32_t = cpu_adrDpx(cpu, &raw mut low_58);
                cpu_lsr(cpu, low_58, high_58);
                break;
            }
            87 => {
                let mut low_59: uint32_t = 0 as uint32_t;
                let mut high_59: uint32_t = cpu_adrIly(cpu, &raw mut low_59);
                cpu_eor(cpu, low_59, high_59);
                break;
            }
            88 => {
                (*cpu).i = false_0 != 0;
                break;
            }
            89 => {
                let mut low_60: uint32_t = 0 as uint32_t;
                let mut high_60: uint32_t = cpu_adrAby(cpu, &raw mut low_60, false_0 != 0);
                cpu_eor(cpu, low_60, high_60);
                break;
            }
            90 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).y as uint8_t);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).y);
                }
                break;
            }
            91 => {
                (*cpu).dp = (*cpu).a;
                cpu_setZN(cpu, (*cpu).dp, false_0 != 0);
                break;
            }
            92 => {
                let mut value_1: uint16_t = cpu_readOpcodeWord(cpu);
                let mut new_k: uint8_t = cpu_readOpcode(cpu);
                if new_k as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int
                    && value_1 as ::core::ffi::c_int == 0x8573 as ::core::ffi::c_int
                {
                    printf(
                        b"Current PC = 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
                        ((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).pc as ::core::ffi::c_int,
                    );
                    DumpCpuHistory();
                    Die(b"The game has crashed!\n\0" as *const u8 as *const ::core::ffi::c_char);
                }
                (*cpu).k = new_k;
                (*cpu).pc = value_1;
                break;
            }
            93 => {
                let mut low_61: uint32_t = 0 as uint32_t;
                let mut high_61: uint32_t = cpu_adrAbx(cpu, &raw mut low_61, false_0 != 0);
                cpu_eor(cpu, low_61, high_61);
                break;
            }
            94 => {
                let mut low_62: uint32_t = 0 as uint32_t;
                let mut high_62: uint32_t = cpu_adrAbx(cpu, &raw mut low_62, true_0 != 0);
                cpu_lsr(cpu, low_62, high_62);
                break;
            }
            95 => {
                let mut low_63: uint32_t = 0 as uint32_t;
                let mut high_63: uint32_t = cpu_adrAlx(cpu, &raw mut low_63);
                cpu_eor(cpu, low_63, high_63);
                break;
            }
            96 => {
                if (*cpu).sp as ::core::ffi::c_int >= (*cpu).spBreakpoint as ::core::ffi::c_int
                    && (*cpu).spBreakpoint as ::core::ffi::c_int != 0
                {
                    let mut delta: ::core::ffi::c_int =
                        (*cpu).sp as ::core::ffi::c_int - (*cpu).spBreakpoint as ::core::ffi::c_int;
                    '_c2rust_label: {
                        if delta == 0 as ::core::ffi::c_int || delta == 2 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"delta == 0 || delta == 2\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"src/snes/cpu.c\0" as *const u8 as *const ::core::ffi::c_char,
                                1386 as ::core::ffi::c_uint,
                                b"void cpu_doOpcode(Cpu *, uint8_t)\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    (*cpu).spBreakpoint = 0 as uint16_t;
                    if HookedFunctionRts((delta == 2 as ::core::ffi::c_int) as ::core::ffi::c_int) {
                        return;
                    }
                }
                (*cpu).pc =
                    (cpu_pullWord(cpu) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t;
                break;
            }
            97 => {
                let mut low_64: uint32_t = 0 as uint32_t;
                let mut high_64: uint32_t = cpu_adrIdx(cpu, &raw mut low_64);
                cpu_adc(cpu, low_64, high_64);
                break;
            }
            98 => {
                let mut value_2: uint16_t = cpu_readOpcodeWord(cpu);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int + value_2 as int16_t as ::core::ffi::c_int)
                        as uint16_t,
                );
                break;
            }
            99 => {
                let mut low_65: uint32_t = 0 as uint32_t;
                let mut high_65: uint32_t = cpu_adrSr(cpu, &raw mut low_65);
                cpu_adc(cpu, low_65, high_65);
                break;
            }
            100 => {
                let mut low_66: uint32_t = 0 as uint32_t;
                let mut high_66: uint32_t = cpu_adrDp(cpu, &raw mut low_66);
                cpu_stz(cpu, low_66, high_66);
                break;
            }
            101 => {
                let mut low_67: uint32_t = 0 as uint32_t;
                let mut high_67: uint32_t = cpu_adrDp(cpu, &raw mut low_67);
                cpu_adc(cpu, low_67, high_67);
                break;
            }
            102 => {
                let mut low_68: uint32_t = 0 as uint32_t;
                let mut high_68: uint32_t = cpu_adrDp(cpu, &raw mut low_68);
                cpu_ror(cpu, low_68, high_68);
                break;
            }
            103 => {
                let mut low_69: uint32_t = 0 as uint32_t;
                let mut high_69: uint32_t = cpu_adrIdl(cpu, &raw mut low_69);
                cpu_adc(cpu, low_69, high_69);
                break;
            }
            104 => {
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | cpu_pullByte(cpu) as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).a = cpu_pullWord(cpu);
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            105 => {
                let mut low_70: uint32_t = 0 as uint32_t;
                let mut high_70: uint32_t = cpu_adrImm(cpu, &raw mut low_70, false_0 != 0);
                cpu_adc(cpu, low_70, high_70);
                break;
            }
            106 => {
                let mut carry: bool = (*cpu).a as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                            & 0x7f as ::core::ffi::c_int
                        | ((*cpu).c as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                        | ((*cpu).c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int)
                        as uint16_t;
                }
                (*cpu).c = carry;
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            107 => {
                if (*cpu).sp as ::core::ffi::c_int >= (*cpu).spBreakpoint as ::core::ffi::c_int
                    && (*cpu).spBreakpoint as ::core::ffi::c_int != 0
                {
                    '_c2rust_label_0: {
                        if (*cpu).sp as ::core::ffi::c_int
                            == (*cpu).spBreakpoint as ::core::ffi::c_int
                        {
                        } else {
                            __assert_fail(
                                b"cpu->sp == cpu->spBreakpoint\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"src/snes/cpu.c\0" as *const u8 as *const ::core::ffi::c_char,
                                1464 as ::core::ffi::c_uint,
                                b"void cpu_doOpcode(Cpu *, uint8_t)\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    (*cpu).spBreakpoint = 0 as uint16_t;
                    if HookedFunctionRts(0 as ::core::ffi::c_int) {
                        return;
                    }
                }
                (*cpu).pc =
                    (cpu_pullWord(cpu) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t;
                (*cpu).k = cpu_pullByte(cpu);
                break;
            }
            108 => {
                let mut adr: uint16_t = cpu_readOpcodeWord(cpu);
                (*cpu).pc = cpu_readWord(
                    cpu,
                    adr as uint32_t,
                    (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                );
                break;
            }
            109 => {
                let mut low_71: uint32_t = 0 as uint32_t;
                let mut high_71: uint32_t = cpu_adrAbs(cpu, &raw mut low_71);
                cpu_adc(cpu, low_71, high_71);
                break;
            }
            110 => {
                let mut low_72: uint32_t = 0 as uint32_t;
                let mut high_72: uint32_t = cpu_adrAbs(cpu, &raw mut low_72);
                cpu_ror(cpu, low_72, high_72);
                break;
            }
            111 => {
                let mut low_73: uint32_t = 0 as uint32_t;
                let mut high_73: uint32_t = cpu_adrAbl(cpu, &raw mut low_73);
                cpu_adc(cpu, low_73, high_73);
                break;
            }
            112 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), (*cpu).v);
                break;
            }
            113 => {
                let mut low_74: uint32_t = 0 as uint32_t;
                let mut high_74: uint32_t = cpu_adrIdy(cpu, &raw mut low_74, false_0 != 0);
                cpu_adc(cpu, low_74, high_74);
                break;
            }
            114 => {
                let mut low_75: uint32_t = 0 as uint32_t;
                let mut high_75: uint32_t = cpu_adrIdp(cpu, &raw mut low_75);
                cpu_adc(cpu, low_75, high_75);
                break;
            }
            115 => {
                let mut low_76: uint32_t = 0 as uint32_t;
                let mut high_76: uint32_t = cpu_adrIsy(cpu, &raw mut low_76);
                cpu_adc(cpu, low_76, high_76);
                break;
            }
            116 => {
                let mut low_77: uint32_t = 0 as uint32_t;
                let mut high_77: uint32_t = cpu_adrDpx(cpu, &raw mut low_77);
                cpu_stz(cpu, low_77, high_77);
                break;
            }
            117 => {
                let mut low_78: uint32_t = 0 as uint32_t;
                let mut high_78: uint32_t = cpu_adrDpx(cpu, &raw mut low_78);
                cpu_adc(cpu, low_78, high_78);
                break;
            }
            118 => {
                let mut low_79: uint32_t = 0 as uint32_t;
                let mut high_79: uint32_t = cpu_adrDpx(cpu, &raw mut low_79);
                cpu_ror(cpu, low_79, high_79);
                break;
            }
            119 => {
                let mut low_80: uint32_t = 0 as uint32_t;
                let mut high_80: uint32_t = cpu_adrIly(cpu, &raw mut low_80);
                cpu_adc(cpu, low_80, high_80);
                break;
            }
            120 => {
                (*cpu).i = true_0 != 0;
                break;
            }
            121 => {
                let mut low_81: uint32_t = 0 as uint32_t;
                let mut high_81: uint32_t = cpu_adrAby(cpu, &raw mut low_81, false_0 != 0);
                cpu_adc(cpu, low_81, high_81);
                break;
            }
            122 => {
                if (*cpu).xf {
                    (*cpu).y = cpu_pullByte(cpu) as uint16_t;
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).y = cpu_pullWord(cpu);
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            123 => {
                (*cpu).a = (*cpu).dp;
                cpu_setZN(cpu, (*cpu).a, false_0 != 0);
                break;
            }
            124 => {
                (*cpu).pc = cpu_adrIax(cpu);
                break;
            }
            125 => {
                let mut low_82: uint32_t = 0 as uint32_t;
                let mut high_82: uint32_t = cpu_adrAbx(cpu, &raw mut low_82, false_0 != 0);
                cpu_adc(cpu, low_82, high_82);
                break;
            }
            126 => {
                let mut low_83: uint32_t = 0 as uint32_t;
                let mut high_83: uint32_t = cpu_adrAbx(cpu, &raw mut low_83, true_0 != 0);
                cpu_ror(cpu, low_83, high_83);
                break;
            }
            127 => {
                let mut low_84: uint32_t = 0 as uint32_t;
                let mut high_84: uint32_t = cpu_adrAlx(cpu, &raw mut low_84);
                cpu_adc(cpu, low_84, high_84);
                break;
            }
            128 => {
                // Relative branches start after the displacement operand.
                let displacement = cpu_readOpcode(cpu) as int8_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement as i16);
                break;
            }
            129 => {
                let mut low_85: uint32_t = 0 as uint32_t;
                let mut high_85: uint32_t = cpu_adrIdx(cpu, &raw mut low_85);
                cpu_sta(cpu, low_85, high_85);
                break;
            }
            130 => {
                let displacement = cpu_readOpcodeWord(cpu) as int16_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
                break;
            }
            131 => {
                let mut low_86: uint32_t = 0 as uint32_t;
                let mut high_86: uint32_t = cpu_adrSr(cpu, &raw mut low_86);
                cpu_sta(cpu, low_86, high_86);
                break;
            }
            132 => {
                let mut low_87: uint32_t = 0 as uint32_t;
                let mut high_87: uint32_t = cpu_adrDp(cpu, &raw mut low_87);
                cpu_sty(cpu, low_87, high_87);
                break;
            }
            133 => {
                let mut low_88: uint32_t = 0 as uint32_t;
                let mut high_88: uint32_t = cpu_adrDp(cpu, &raw mut low_88);
                cpu_sta(cpu, low_88, high_88);
                break;
            }
            134 => {
                let mut low_89: uint32_t = 0 as uint32_t;
                let mut high_89: uint32_t = cpu_adrDp(cpu, &raw mut low_89);
                cpu_stx(cpu, low_89, high_89);
                break;
            }
            135 => {
                let mut low_90: uint32_t = 0 as uint32_t;
                let mut high_90: uint32_t = cpu_adrIdl(cpu, &raw mut low_90);
                cpu_sta(cpu, low_90, high_90);
                break;
            }
            136 => {
                if (*cpu).xf {
                    (*cpu).y = ((*cpu).y as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).y = (*cpu).y.wrapping_sub(1);
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            137 => {
                if (*cpu).mf {
                    let mut result_0: uint8_t = ((*cpu).a as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int
                        & cpu_readOpcode(cpu) as ::core::ffi::c_int)
                        as uint8_t;
                    (*cpu).z = result_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    let mut result_1: uint16_t = ((*cpu).a as ::core::ffi::c_int
                        & cpu_readOpcodeWord(cpu) as ::core::ffi::c_int)
                        as uint16_t;
                    (*cpu).z = result_1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
                }
                break;
            }
            138 => {
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a = (*cpu).x;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            139 => {
                cpu_pushByte(cpu, (*cpu).db);
                break;
            }
            140 => {
                let mut low_91: uint32_t = 0 as uint32_t;
                let mut high_91: uint32_t = cpu_adrAbs(cpu, &raw mut low_91);
                cpu_sty(cpu, low_91, high_91);
                break;
            }
            141 => {
                let mut low_92: uint32_t = 0 as uint32_t;
                let mut high_92: uint32_t = cpu_adrAbs(cpu, &raw mut low_92);
                cpu_sta(cpu, low_92, high_92);
                break;
            }
            142 => {
                let mut low_93: uint32_t = 0 as uint32_t;
                let mut high_93: uint32_t = cpu_adrAbs(cpu, &raw mut low_93);
                cpu_stx(cpu, low_93, high_93);
                break;
            }
            143 => {
                let mut low_94: uint32_t = 0 as uint32_t;
                let mut high_94: uint32_t = cpu_adrAbl(cpu, &raw mut low_94);
                cpu_sta(cpu, low_94, high_94);
                break;
            }
            144 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), !(*cpu).c);
                break;
            }
            145 => {
                let mut low_95: uint32_t = 0 as uint32_t;
                let mut high_95: uint32_t = cpu_adrIdy(cpu, &raw mut low_95, true_0 != 0);
                cpu_sta(cpu, low_95, high_95);
                break;
            }
            146 => {
                let mut low_96: uint32_t = 0 as uint32_t;
                let mut high_96: uint32_t = cpu_adrIdp(cpu, &raw mut low_96);
                cpu_sta(cpu, low_96, high_96);
                break;
            }
            147 => {
                let mut low_97: uint32_t = 0 as uint32_t;
                let mut high_97: uint32_t = cpu_adrIsy(cpu, &raw mut low_97);
                cpu_sta(cpu, low_97, high_97);
                break;
            }
            148 => {
                let mut low_98: uint32_t = 0 as uint32_t;
                let mut high_98: uint32_t = cpu_adrDpx(cpu, &raw mut low_98);
                cpu_sty(cpu, low_98, high_98);
                break;
            }
            149 => {
                let mut low_99: uint32_t = 0 as uint32_t;
                let mut high_99: uint32_t = cpu_adrDpx(cpu, &raw mut low_99);
                cpu_sta(cpu, low_99, high_99);
                break;
            }
            150 => {
                let mut low_100: uint32_t = 0 as uint32_t;
                let mut high_100: uint32_t = cpu_adrDpy(cpu, &raw mut low_100);
                cpu_stx(cpu, low_100, high_100);
                break;
            }
            151 => {
                let mut low_101: uint32_t = 0 as uint32_t;
                let mut high_101: uint32_t = cpu_adrIly(cpu, &raw mut low_101);
                cpu_sta(cpu, low_101, high_101);
                break;
            }
            152 => {
                if (*cpu).mf {
                    (*cpu).a = ((*cpu).a as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | (*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                        as uint16_t;
                } else {
                    (*cpu).a = (*cpu).y;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            153 => {
                let mut low_102: uint32_t = 0 as uint32_t;
                let mut high_102: uint32_t = cpu_adrAby(cpu, &raw mut low_102, true_0 != 0);
                cpu_sta(cpu, low_102, high_102);
                break;
            }
            154 => {
                (*cpu).sp = (*cpu).x;
                break;
            }
            155 => {
                if (*cpu).xf {
                    (*cpu).y =
                        ((*cpu).x as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).y = (*cpu).x;
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            156 => {
                let mut low_103: uint32_t = 0 as uint32_t;
                let mut high_103: uint32_t = cpu_adrAbs(cpu, &raw mut low_103);
                cpu_stz(cpu, low_103, high_103);
                break;
            }
            157 => {
                let mut low_104: uint32_t = 0 as uint32_t;
                let mut high_104: uint32_t = cpu_adrAbx(cpu, &raw mut low_104, true_0 != 0);
                cpu_sta(cpu, low_104, high_104);
                break;
            }
            158 => {
                let mut low_105: uint32_t = 0 as uint32_t;
                let mut high_105: uint32_t = cpu_adrAbx(cpu, &raw mut low_105, true_0 != 0);
                cpu_stz(cpu, low_105, high_105);
                break;
            }
            159 => {
                let mut low_106: uint32_t = 0 as uint32_t;
                let mut high_106: uint32_t = cpu_adrAlx(cpu, &raw mut low_106);
                cpu_sta(cpu, low_106, high_106);
                break;
            }
            160 => {
                let mut low_107: uint32_t = 0 as uint32_t;
                let mut high_107: uint32_t = cpu_adrImm(cpu, &raw mut low_107, true_0 != 0);
                cpu_ldy(cpu, low_107, high_107);
                break;
            }
            161 => {
                let mut low_108: uint32_t = 0 as uint32_t;
                let mut high_108: uint32_t = cpu_adrIdx(cpu, &raw mut low_108);
                cpu_lda(cpu, low_108, high_108);
                break;
            }
            162 => {
                let mut low_109: uint32_t = 0 as uint32_t;
                let mut high_109: uint32_t = cpu_adrImm(cpu, &raw mut low_109, true_0 != 0);
                cpu_ldx(cpu, low_109, high_109);
                break;
            }
            163 => {
                let mut low_110: uint32_t = 0 as uint32_t;
                let mut high_110: uint32_t = cpu_adrSr(cpu, &raw mut low_110);
                cpu_lda(cpu, low_110, high_110);
                break;
            }
            164 => {
                let mut low_111: uint32_t = 0 as uint32_t;
                let mut high_111: uint32_t = cpu_adrDp(cpu, &raw mut low_111);
                cpu_ldy(cpu, low_111, high_111);
                break;
            }
            165 => {
                let mut low_112: uint32_t = 0 as uint32_t;
                let mut high_112: uint32_t = cpu_adrDp(cpu, &raw mut low_112);
                cpu_lda(cpu, low_112, high_112);
                break;
            }
            166 => {
                let mut low_113: uint32_t = 0 as uint32_t;
                let mut high_113: uint32_t = cpu_adrDp(cpu, &raw mut low_113);
                cpu_ldx(cpu, low_113, high_113);
                break;
            }
            167 => {
                let mut low_114: uint32_t = 0 as uint32_t;
                let mut high_114: uint32_t = cpu_adrIdl(cpu, &raw mut low_114);
                cpu_lda(cpu, low_114, high_114);
                break;
            }
            168 => {
                if (*cpu).xf {
                    (*cpu).y =
                        ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).y = (*cpu).a;
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            169 => {
                let mut low_115: uint32_t = 0 as uint32_t;
                let mut high_115: uint32_t = cpu_adrImm(cpu, &raw mut low_115, false_0 != 0);
                cpu_lda(cpu, low_115, high_115);
                break;
            }
            170 => {
                if (*cpu).xf {
                    (*cpu).x =
                        ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).x = (*cpu).a;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            171 => {
                (*cpu).db = cpu_pullByte(cpu);
                cpu_setZN(cpu, (*cpu).db as uint16_t, true_0 != 0);
                break;
            }
            172 => {
                let mut low_116: uint32_t = 0 as uint32_t;
                let mut high_116: uint32_t = cpu_adrAbs(cpu, &raw mut low_116);
                cpu_ldy(cpu, low_116, high_116);
                break;
            }
            173 => {
                let mut low_117: uint32_t = 0 as uint32_t;
                let mut high_117: uint32_t = cpu_adrAbs(cpu, &raw mut low_117);
                cpu_lda(cpu, low_117, high_117);
                break;
            }
            174 => {
                let mut low_118: uint32_t = 0 as uint32_t;
                let mut high_118: uint32_t = cpu_adrAbs(cpu, &raw mut low_118);
                cpu_ldx(cpu, low_118, high_118);
                break;
            }
            175 => {
                let mut low_119: uint32_t = 0 as uint32_t;
                let mut high_119: uint32_t = cpu_adrAbl(cpu, &raw mut low_119);
                cpu_lda(cpu, low_119, high_119);
                break;
            }
            176 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), (*cpu).c);
                break;
            }
            177 => {
                let mut low_120: uint32_t = 0 as uint32_t;
                let mut high_120: uint32_t = cpu_adrIdy(cpu, &raw mut low_120, false_0 != 0);
                cpu_lda(cpu, low_120, high_120);
                break;
            }
            178 => {
                let mut low_121: uint32_t = 0 as uint32_t;
                let mut high_121: uint32_t = cpu_adrIdp(cpu, &raw mut low_121);
                cpu_lda(cpu, low_121, high_121);
                break;
            }
            179 => {
                let mut low_122: uint32_t = 0 as uint32_t;
                let mut high_122: uint32_t = cpu_adrIsy(cpu, &raw mut low_122);
                cpu_lda(cpu, low_122, high_122);
                break;
            }
            180 => {
                let mut low_123: uint32_t = 0 as uint32_t;
                let mut high_123: uint32_t = cpu_adrDpx(cpu, &raw mut low_123);
                cpu_ldy(cpu, low_123, high_123);
                break;
            }
            181 => {
                let mut low_124: uint32_t = 0 as uint32_t;
                let mut high_124: uint32_t = cpu_adrDpx(cpu, &raw mut low_124);
                cpu_lda(cpu, low_124, high_124);
                break;
            }
            182 => {
                let mut low_125: uint32_t = 0 as uint32_t;
                let mut high_125: uint32_t = cpu_adrDpy(cpu, &raw mut low_125);
                cpu_ldx(cpu, low_125, high_125);
                break;
            }
            183 => {
                let mut low_126: uint32_t = 0 as uint32_t;
                let mut high_126: uint32_t = cpu_adrIly(cpu, &raw mut low_126);
                cpu_lda(cpu, low_126, high_126);
                break;
            }
            184 => {
                (*cpu).v = false_0 != 0;
                break;
            }
            185 => {
                let mut low_127: uint32_t = 0 as uint32_t;
                let mut high_127: uint32_t = cpu_adrAby(cpu, &raw mut low_127, false_0 != 0);
                cpu_lda(cpu, low_127, high_127);
                break;
            }
            186 => {
                if (*cpu).xf {
                    (*cpu).x =
                        ((*cpu).sp as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).x = (*cpu).sp;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            187 => {
                if (*cpu).xf {
                    (*cpu).x =
                        ((*cpu).y as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).x = (*cpu).y;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            188 => {
                let mut low_128: uint32_t = 0 as uint32_t;
                let mut high_128: uint32_t = cpu_adrAbx(cpu, &raw mut low_128, false_0 != 0);
                cpu_ldy(cpu, low_128, high_128);
                break;
            }
            189 => {
                let mut low_129: uint32_t = 0 as uint32_t;
                let mut high_129: uint32_t = cpu_adrAbx(cpu, &raw mut low_129, false_0 != 0);
                cpu_lda(cpu, low_129, high_129);
                break;
            }
            190 => {
                let mut low_130: uint32_t = 0 as uint32_t;
                let mut high_130: uint32_t = cpu_adrAby(cpu, &raw mut low_130, false_0 != 0);
                cpu_ldx(cpu, low_130, high_130);
                break;
            }
            191 => {
                let mut low_131: uint32_t = 0 as uint32_t;
                let mut high_131: uint32_t = cpu_adrAlx(cpu, &raw mut low_131);
                cpu_lda(cpu, low_131, high_131);
                break;
            }
            192 => {
                let mut low_132: uint32_t = 0 as uint32_t;
                let mut high_132: uint32_t = cpu_adrImm(cpu, &raw mut low_132, true_0 != 0);
                cpu_cpy(cpu, low_132, high_132);
                break;
            }
            193 => {
                let mut low_133: uint32_t = 0 as uint32_t;
                let mut high_133: uint32_t = cpu_adrIdx(cpu, &raw mut low_133);
                cpu_cmp(cpu, low_133, high_133);
                break;
            }
            194 => {
                cpu_setFlags(
                    cpu,
                    (cpu_getFlags(cpu) as ::core::ffi::c_int
                        & !(cpu_readOpcode(cpu) as ::core::ffi::c_int))
                        as uint8_t,
                );
                break;
            }
            195 => {
                let mut low_134: uint32_t = 0 as uint32_t;
                let mut high_134: uint32_t = cpu_adrSr(cpu, &raw mut low_134);
                cpu_cmp(cpu, low_134, high_134);
                break;
            }
            196 => {
                let mut low_135: uint32_t = 0 as uint32_t;
                let mut high_135: uint32_t = cpu_adrDp(cpu, &raw mut low_135);
                cpu_cpy(cpu, low_135, high_135);
                break;
            }
            197 => {
                let mut low_136: uint32_t = 0 as uint32_t;
                let mut high_136: uint32_t = cpu_adrDp(cpu, &raw mut low_136);
                cpu_cmp(cpu, low_136, high_136);
                break;
            }
            198 => {
                let mut low_137: uint32_t = 0 as uint32_t;
                let mut high_137: uint32_t = cpu_adrDp(cpu, &raw mut low_137);
                cpu_dec(cpu, low_137, high_137);
                break;
            }
            199 => {
                let mut low_138: uint32_t = 0 as uint32_t;
                let mut high_138: uint32_t = cpu_adrIdl(cpu, &raw mut low_138);
                cpu_cmp(cpu, low_138, high_138);
                break;
            }
            200 => {
                if (*cpu).xf {
                    (*cpu).y = ((*cpu).y as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).y = (*cpu).y.wrapping_add(1);
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            201 => {
                let mut low_139: uint32_t = 0 as uint32_t;
                let mut high_139: uint32_t = cpu_adrImm(cpu, &raw mut low_139, false_0 != 0);
                cpu_cmp(cpu, low_139, high_139);
                break;
            }
            202 => {
                if (*cpu).xf {
                    (*cpu).x = ((*cpu).x as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).x = (*cpu).x.wrapping_sub(1);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            203 => {
                '_c2rust_label_1: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        b"src/snes/cpu.c\0" as *const u8 as *const ::core::ffi::c_char,
                        2059 as ::core::ffi::c_uint,
                        b"void cpu_doOpcode(Cpu *, uint8_t)\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                break;
            }
            204 => {
                let mut low_140: uint32_t = 0 as uint32_t;
                let mut high_140: uint32_t = cpu_adrAbs(cpu, &raw mut low_140);
                cpu_cpy(cpu, low_140, high_140);
                break;
            }
            205 => {
                let mut low_141: uint32_t = 0 as uint32_t;
                let mut high_141: uint32_t = cpu_adrAbs(cpu, &raw mut low_141);
                cpu_cmp(cpu, low_141, high_141);
                break;
            }
            206 => {
                let mut low_142: uint32_t = 0 as uint32_t;
                let mut high_142: uint32_t = cpu_adrAbs(cpu, &raw mut low_142);
                cpu_dec(cpu, low_142, high_142);
                break;
            }
            207 => {
                let mut low_143: uint32_t = 0 as uint32_t;
                let mut high_143: uint32_t = cpu_adrAbl(cpu, &raw mut low_143);
                cpu_cmp(cpu, low_143, high_143);
                break;
            }
            208 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), !(*cpu).z);
                break;
            }
            209 => {
                let mut low_144: uint32_t = 0 as uint32_t;
                let mut high_144: uint32_t = cpu_adrIdy(cpu, &raw mut low_144, false_0 != 0);
                cpu_cmp(cpu, low_144, high_144);
                break;
            }
            210 => {
                let mut low_145: uint32_t = 0 as uint32_t;
                let mut high_145: uint32_t = cpu_adrIdp(cpu, &raw mut low_145);
                cpu_cmp(cpu, low_145, high_145);
                break;
            }
            211 => {
                let mut low_146: uint32_t = 0 as uint32_t;
                let mut high_146: uint32_t = cpu_adrIsy(cpu, &raw mut low_146);
                cpu_cmp(cpu, low_146, high_146);
                break;
            }
            212 => {
                let mut low_147: uint32_t = 0 as uint32_t;
                let mut high_147: uint32_t = cpu_adrDp(cpu, &raw mut low_147);
                cpu_pushWord(cpu, cpu_readWord(cpu, low_147, high_147));
                break;
            }
            213 => {
                let mut low_148: uint32_t = 0 as uint32_t;
                let mut high_148: uint32_t = cpu_adrDpx(cpu, &raw mut low_148);
                cpu_cmp(cpu, low_148, high_148);
                break;
            }
            214 => {
                let mut low_149: uint32_t = 0 as uint32_t;
                let mut high_149: uint32_t = cpu_adrDpx(cpu, &raw mut low_149);
                cpu_dec(cpu, low_149, high_149);
                break;
            }
            215 => {
                let mut low_150: uint32_t = 0 as uint32_t;
                let mut high_150: uint32_t = cpu_adrIly(cpu, &raw mut low_150);
                cpu_cmp(cpu, low_150, high_150);
                break;
            }
            216 => {
                (*cpu).d = false_0 != 0;
                break;
            }
            217 => {
                let mut low_151: uint32_t = 0 as uint32_t;
                let mut high_151: uint32_t = cpu_adrAby(cpu, &raw mut low_151, false_0 != 0);
                cpu_cmp(cpu, low_151, high_151);
                break;
            }
            218 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).x as uint8_t);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).x);
                }
                break;
            }
            219 => {
                (*cpu).stopped = true_0 != 0;
                break;
            }
            220 => {
                let mut adr_0: uint16_t = cpu_readOpcodeWord(cpu);
                (*cpu).pc = cpu_readWord(
                    cpu,
                    adr_0 as uint32_t,
                    (adr_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                );
                (*cpu).k = cpu_read(
                    cpu,
                    (adr_0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                );
                break;
            }
            221 => {
                let mut low_152: uint32_t = 0 as uint32_t;
                let mut high_152: uint32_t = cpu_adrAbx(cpu, &raw mut low_152, false_0 != 0);
                cpu_cmp(cpu, low_152, high_152);
                break;
            }
            222 => {
                let mut low_153: uint32_t = 0 as uint32_t;
                let mut high_153: uint32_t = cpu_adrAbx(cpu, &raw mut low_153, true_0 != 0);
                cpu_dec(cpu, low_153, high_153);
                break;
            }
            223 => {
                let mut low_154: uint32_t = 0 as uint32_t;
                let mut high_154: uint32_t = cpu_adrAlx(cpu, &raw mut low_154);
                cpu_cmp(cpu, low_154, high_154);
                break;
            }
            224 => {
                let mut low_155: uint32_t = 0 as uint32_t;
                let mut high_155: uint32_t = cpu_adrImm(cpu, &raw mut low_155, true_0 != 0);
                cpu_cpx(cpu, low_155, high_155);
                break;
            }
            225 => {
                let mut low_156: uint32_t = 0 as uint32_t;
                let mut high_156: uint32_t = cpu_adrIdx(cpu, &raw mut low_156);
                cpu_sbc(cpu, low_156, high_156);
                break;
            }
            226 => {
                cpu_setFlags(
                    cpu,
                    (cpu_getFlags(cpu) as ::core::ffi::c_int
                        | cpu_readOpcode(cpu) as ::core::ffi::c_int) as uint8_t,
                );
                break;
            }
            227 => {
                let mut low_157: uint32_t = 0 as uint32_t;
                let mut high_157: uint32_t = cpu_adrSr(cpu, &raw mut low_157);
                cpu_sbc(cpu, low_157, high_157);
                break;
            }
            228 => {
                let mut low_158: uint32_t = 0 as uint32_t;
                let mut high_158: uint32_t = cpu_adrDp(cpu, &raw mut low_158);
                cpu_cpx(cpu, low_158, high_158);
                break;
            }
            229 => {
                let mut low_159: uint32_t = 0 as uint32_t;
                let mut high_159: uint32_t = cpu_adrDp(cpu, &raw mut low_159);
                cpu_sbc(cpu, low_159, high_159);
                break;
            }
            230 => {
                let mut low_160: uint32_t = 0 as uint32_t;
                let mut high_160: uint32_t = cpu_adrDp(cpu, &raw mut low_160);
                cpu_inc(cpu, low_160, high_160);
                break;
            }
            231 => {
                let mut low_161: uint32_t = 0 as uint32_t;
                let mut high_161: uint32_t = cpu_adrIdl(cpu, &raw mut low_161);
                cpu_sbc(cpu, low_161, high_161);
                break;
            }
            232 => {
                if (*cpu).xf {
                    (*cpu).x = ((*cpu).x as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as uint16_t;
                } else {
                    (*cpu).x = (*cpu).x.wrapping_add(1);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            233 => {
                let mut low_162: uint32_t = 0 as uint32_t;
                let mut high_162: uint32_t = cpu_adrImm(cpu, &raw mut low_162, false_0 != 0);
                cpu_sbc(cpu, low_162, high_162);
                break;
            }
            235 => {
                let mut low_163: uint8_t =
                    ((*cpu).a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
                let mut high_163: uint8_t =
                    ((*cpu).a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
                (*cpu).a = ((low_163 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                    | high_163 as ::core::ffi::c_int) as uint16_t;
                cpu_setZN(cpu, high_163 as uint16_t, true_0 != 0);
                break;
            }
            236 => {
                let mut low_164: uint32_t = 0 as uint32_t;
                let mut high_164: uint32_t = cpu_adrAbs(cpu, &raw mut low_164);
                cpu_cpx(cpu, low_164, high_164);
                break;
            }
            237 => {
                let mut low_165: uint32_t = 0 as uint32_t;
                let mut high_165: uint32_t = cpu_adrAbs(cpu, &raw mut low_165);
                cpu_sbc(cpu, low_165, high_165);
                break;
            }
            238 => {
                let mut low_166: uint32_t = 0 as uint32_t;
                let mut high_166: uint32_t = cpu_adrAbs(cpu, &raw mut low_166);
                cpu_inc(cpu, low_166, high_166);
                break;
            }
            239 => {
                let mut low_167: uint32_t = 0 as uint32_t;
                let mut high_167: uint32_t = cpu_adrAbl(cpu, &raw mut low_167);
                cpu_sbc(cpu, low_167, high_167);
                break;
            }
            240 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu), (*cpu).z);
                break;
            }
            241 => {
                let mut low_168: uint32_t = 0 as uint32_t;
                let mut high_168: uint32_t = cpu_adrIdy(cpu, &raw mut low_168, false_0 != 0);
                cpu_sbc(cpu, low_168, high_168);
                break;
            }
            242 => {
                let mut low_169: uint32_t = 0 as uint32_t;
                let mut high_169: uint32_t = cpu_adrIdp(cpu, &raw mut low_169);
                cpu_sbc(cpu, low_169, high_169);
                break;
            }
            243 => {
                let mut low_170: uint32_t = 0 as uint32_t;
                let mut high_170: uint32_t = cpu_adrIsy(cpu, &raw mut low_170);
                cpu_sbc(cpu, low_170, high_170);
                break;
            }
            244 => {
                let mut w: uint16_t = cpu_readOpcodeWord(cpu);
                if w as ::core::ffi::c_int == 0xc2ae as ::core::ffi::c_int {
                    (*cpu).y = 0 as uint16_t;
                }
                cpu_pushWord(cpu, w);
                break;
            }
            245 => {
                let mut low_171: uint32_t = 0 as uint32_t;
                let mut high_171: uint32_t = cpu_adrDpx(cpu, &raw mut low_171);
                cpu_sbc(cpu, low_171, high_171);
                break;
            }
            246 => {
                let mut low_172: uint32_t = 0 as uint32_t;
                let mut high_172: uint32_t = cpu_adrDpx(cpu, &raw mut low_172);
                cpu_inc(cpu, low_172, high_172);
                break;
            }
            247 => {
                let mut low_173: uint32_t = 0 as uint32_t;
                let mut high_173: uint32_t = cpu_adrIly(cpu, &raw mut low_173);
                cpu_sbc(cpu, low_173, high_173);
                break;
            }
            248 => {
                (*cpu).d = true_0 != 0;
                break;
            }
            249 => {
                let mut low_174: uint32_t = 0 as uint32_t;
                let mut high_174: uint32_t = cpu_adrAby(cpu, &raw mut low_174, false_0 != 0);
                cpu_sbc(cpu, low_174, high_174);
                break;
            }
            250 => {
                if (*cpu).xf {
                    (*cpu).x = cpu_pullByte(cpu) as uint16_t;
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).x = cpu_pullWord(cpu);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            251 => {
                let mut temp: bool = (*cpu).c;
                (*cpu).c = (*cpu).e;
                (*cpu).e = temp;
                cpu_setFlags(cpu, cpu_getFlags(cpu));
                break;
            }
            252 => {
                let mut value_3: uint16_t = cpu_adrIax(cpu);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                );
                (*cpu).pc = value_3;
                break;
            }
            253 => {
                let mut low_175: uint32_t = 0 as uint32_t;
                let mut high_175: uint32_t = cpu_adrAbx(cpu, &raw mut low_175, false_0 != 0);
                cpu_sbc(cpu, low_175, high_175);
                break;
            }
            254 => {
                let mut low_176: uint32_t = 0 as uint32_t;
                let mut high_176: uint32_t = cpu_adrAbx(cpu, &raw mut low_176, true_0 != 0);
                cpu_inc(cpu, low_176, high_176);
                break;
            }
            255 => {
                let mut low_177: uint32_t = 0 as uint32_t;
                let mut high_177: uint32_t = cpu_adrAlx(cpu, &raw mut low_177);
                cpu_sbc(cpu, low_177, high_177);
                break;
            }
            234 | _ => {
                break;
            }
        }
    }
}
