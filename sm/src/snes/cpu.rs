use crate::cpu_infra::{CpuOpcodeHook, HookedFunctionRts};
use crate::game_state::Game;
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
static cyclesPerOpcode: [::core::ffi::c_int; 256] = [
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
unsafe fn cpu_read(mut cpu: *mut Cpu, mut adr: uint32_t, game: &mut Game) -> uint8_t {
    return snes_cpuRead((*cpu).mem as *mut Snes, adr, game);
}
unsafe fn cpu_write(mut cpu: *mut Cpu, mut adr: uint32_t, mut val: uint8_t, game: &mut Game) {
    snes_cpuWrite((*cpu).mem as *mut Snes, adr, val, game);
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
pub unsafe fn cpu_reset(mut cpu: *mut Cpu, game: &mut Game) {
    (*cpu).a = 0 as uint16_t;
    (*cpu).x = 0 as uint16_t;
    (*cpu).y = 0 as uint16_t;
    (*cpu).sp = 0x100 as uint16_t;
    (*cpu).pc = (cpu_read(cpu, 0xfffc as uint32_t, game) as ::core::ffi::c_int
        | (cpu_read(cpu, 0xfffd as uint32_t, game) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16_t;
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
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*cpu).a as *mut ::core::ffi::c_void,
        (37 as size_t).wrapping_sub(10 as size_t),
        game,
    );
    (*cpu).spBreakpoint = 0 as uint16_t;
}
pub unsafe fn cpu_runOpcode(mut cpu: *mut Cpu, game: &mut Game) -> ::core::ffi::c_int {
    (*cpu).cyclesUsed = 0 as uint8_t;
    if (*cpu).stopped {
        return 1 as ::core::ffi::c_int;
    }
    if (*cpu).waiting {
        if !(*cpu).irqWanted && !(*cpu).nmiWanted {
            return 1;
        }
        // IRQ wakes WAI even when I prevents interrupt service.
        (*cpu).waiting = false;
    }
    if !(*cpu).i && (*cpu).irqWanted as ::core::ffi::c_int != 0
        || (*cpu).nmiWanted as ::core::ffi::c_int != 0
    {
        (*cpu).cyclesUsed = 7 as uint8_t;
        if (*cpu).nmiWanted {
            (*cpu).nmiWanted = false_0 != 0;
            cpu_doInterrupt(cpu, false_0 != 0, game);
        } else {
            cpu_doInterrupt(cpu, true_0 != 0, game);
        }
    }
    let mut opcode: uint8_t = cpu_readOpcode(cpu, game);
    (*cpu).cyclesUsed = (*cpu)
        .cyclesUsed
        .wrapping_add(cyclesPerOpcode[opcode as usize] as uint8_t);
    cpu_doOpcode(cpu, opcode, game);
    return (*cpu).cyclesUsed as ::core::ffi::c_int;
}
unsafe fn cpu_readOpcode(cpu: *mut Cpu, game: &mut Game) -> uint8_t {
    let pc = (*cpu).pc;
    (*cpu).pc = (*cpu).pc.wrapping_add(1);
    cpu_read(cpu, u32::from((*cpu).k) << 16 | u32::from(pc), game)
}
unsafe fn cpu_readOpcodeWord(cpu: *mut Cpu, game: &mut Game) -> uint16_t {
    let low = cpu_readOpcode(cpu, game);
    let high = cpu_readOpcode(cpu, game);
    u16::from_le_bytes([low, high])
}
pub unsafe fn cpu_getFlags(cpu: *mut Cpu) -> uint8_t {
    (u8::from((*cpu).n) << 7)
        | (u8::from((*cpu).v) << 6)
        | (u8::from((*cpu).mf) << 5)
        | (u8::from((*cpu).xf) << 4)
        | (u8::from((*cpu).d) << 3)
        | (u8::from((*cpu).i) << 2)
        | (u8::from((*cpu).z) << 1)
        | u8::from((*cpu).c)
}
pub unsafe fn cpu_setFlags(cpu: *mut Cpu, val: uint8_t) {
    (*cpu).n = val & 0x80 != 0;
    (*cpu).v = val & 0x40 != 0;
    (*cpu).mf = val & 0x20 != 0;
    (*cpu).xf = val & 0x10 != 0;
    (*cpu).d = val & 0x08 != 0;
    (*cpu).i = val & 0x04 != 0;
    (*cpu).z = val & 0x02 != 0;
    (*cpu).c = val & 0x01 != 0;
    if (*cpu).e {
        (*cpu).mf = true;
        (*cpu).xf = true;
        (*cpu).sp = ((*cpu).sp & 0xff) | 0x100;
    }
    if (*cpu).xf {
        (*cpu).x &= 0xff;
        (*cpu).y &= 0xff;
    }
}
unsafe fn cpu_setZN(cpu: *mut Cpu, value: uint16_t, byte: bool) {
    if byte {
        (*cpu).z = value & 0xff == 0;
        (*cpu).n = value & 0x80 != 0;
    } else {
        (*cpu).z = value == 0;
        (*cpu).n = value & 0x8000 != 0;
    }
}
unsafe fn cpu_doBranch(cpu: *mut Cpu, value: uint8_t, check: bool) {
    if check {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).pc = (*cpu).pc.wrapping_add_signed(value as i8 as i16);
    }
}
unsafe fn cpu_pullByte(cpu: *mut Cpu, game: &mut Game) -> uint8_t {
    (*cpu).sp = (*cpu).sp.wrapping_add(1);
    if (*cpu).e {
        (*cpu).sp = (*cpu).sp & 0xff | 0x100;
    }
    cpu_read(cpu, u32::from((*cpu).sp), game)
}
unsafe fn cpu_pushByte(cpu: *mut Cpu, value: uint8_t, game: &mut Game) {
    cpu_write(cpu, u32::from((*cpu).sp), value, game);
    (*cpu).sp = (*cpu).sp.wrapping_sub(1);
    if (*cpu).e {
        (*cpu).sp = (*cpu).sp & 0xff | 0x100;
    }
}
unsafe fn cpu_pullWord(cpu: *mut Cpu, game: &mut Game) -> uint16_t {
    let low = cpu_pullByte(cpu, game);
    let high = cpu_pullByte(cpu, game);
    u16::from_le_bytes([low, high])
}
unsafe fn cpu_pushWord(cpu: *mut Cpu, value: uint16_t, game: &mut Game) {
    let [low, high] = value.to_le_bytes();
    cpu_pushByte(cpu, high, game);
    cpu_pushByte(cpu, low, game);
}
unsafe fn cpu_readWord(cpu: *mut Cpu, adrl: uint32_t, adrh: uint32_t, game: &mut Game) -> uint16_t {
    let low = cpu_read(cpu, adrl, game);
    let high = cpu_read(cpu, adrh, game);
    u16::from_le_bytes([low, high])
}
unsafe fn cpu_writeWord(
    cpu: *mut Cpu,
    adrl: uint32_t,
    adrh: uint32_t,
    value: uint16_t,
    reversed: bool,
    game: &mut Game,
) {
    let [low, high] = value.to_le_bytes();
    if reversed {
        cpu_write(cpu, adrh, high, game);
        cpu_write(cpu, adrl, low, game);
    } else {
        cpu_write(cpu, adrl, low, game);
        cpu_write(cpu, adrh, high, game);
    }
}
unsafe fn cpu_doInterrupt(mut cpu: *mut Cpu, mut irq: bool, game: &mut Game) {
    if !(*cpu).e {
        cpu_pushByte(cpu, (*cpu).k, game);
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    cpu_pushWord(cpu, (*cpu).pc, game);
    let flags = cpu_getFlags(cpu);
    // Bit 4 is the cleared B flag on an emulation-mode hardware interrupt.
    cpu_pushByte(cpu, if (*cpu).e { flags & !0x10 } else { flags }, game);
    (*cpu).i = true_0 != 0;
    (*cpu).d = false_0 != 0;
    (*cpu).k = 0 as uint8_t;
    let vector = if (*cpu).e {
        if irq {
            0xfffe
        } else {
            0xfffa
        }
    } else {
        if irq {
            0xffee
        } else {
            0xffea
        }
    };
    (*cpu).pc = cpu_readWord(cpu, vector, vector + 1, game);
}
unsafe fn cpu_adrImm(cpu: *mut Cpu, low: *mut uint32_t, xFlag: bool) -> uint32_t {
    let byte = if xFlag { (*cpu).xf } else { (*cpu).mf };
    let pc = (*cpu).pc;
    (*cpu).pc = (*cpu).pc.wrapping_add(1);
    *low = u32::from((*cpu).k) << 16 | u32::from(pc);
    if byte {
        0
    } else {
        let pc = (*cpu).pc;
        (*cpu).pc = (*cpu).pc.wrapping_add(1);
        u32::from((*cpu).k) << 16 | u32::from(pc)
    }
}
unsafe fn cpu_adrDp(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = u32::from((*cpu).dp.wrapping_add(adr));
    u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1))
}
unsafe fn cpu_adrDpx(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = u32::from((*cpu).dp.wrapping_add(adr).wrapping_add((*cpu).x));
    u32::from(
        (*cpu)
            .dp
            .wrapping_add(adr)
            .wrapping_add((*cpu).x)
            .wrapping_add(1),
    )
}
unsafe fn cpu_adrDpy(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = u32::from((*cpu).dp.wrapping_add(adr).wrapping_add((*cpu).y));
    u32::from(
        (*cpu)
            .dp
            .wrapping_add(adr)
            .wrapping_add((*cpu).y)
            .wrapping_add(1),
    )
}
unsafe fn cpu_adrIdp(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let pointer = u32::from(cpu_readWord(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    *low = (u32::from((*cpu).db) << 16) + pointer;
    ((u32::from((*cpu).db) << 16) + pointer + 1) & 0xffffff
}
unsafe fn cpu_adrIdx(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let pointer = u32::from(cpu_readWord(
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
    *low = (u32::from((*cpu).db) << 16) + pointer;
    ((u32::from((*cpu).db) << 16) + pointer + 1) & 0xffffff
}
unsafe fn cpu_adrIdy(cpu: *mut Cpu, low: *mut uint32_t, write: bool, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let pointer = u32::from(cpu_readWord(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    if write && (!(*cpu).xf || pointer >> 8 != (pointer + u32::from((*cpu).y)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y)) & 0xffffff;
    ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y) + 1) & 0xffffff
}
unsafe fn cpu_adrIdl(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer = u32::from(cpu_readWord(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    pointer |= u32::from(cpu_read(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(2)),
        game,
    )) << 16;
    *low = pointer;
    pointer.wrapping_add(1) & 0xffffff
}
unsafe fn cpu_adrIly(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let mut pointer = u32::from(cpu_readWord(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr)),
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    pointer |= u32::from(cpu_read(
        cpu,
        u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(2)),
        game,
    )) << 16;
    *low = pointer.wrapping_add(u32::from((*cpu).y)) & 0xffffff;
    pointer.wrapping_add(u32::from((*cpu).y)).wrapping_add(1) & 0xffffff
}
unsafe fn cpu_adrSr(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    *low = u32::from((*cpu).sp.wrapping_add(adr));
    u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1))
}
unsafe fn cpu_adrIsy(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    let pointer = u32::from(cpu_readWord(
        cpu,
        u32::from((*cpu).sp.wrapping_add(adr)),
        u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1)),
        game,
    ));
    *low = ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y)) & 0xffffff;
    ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y) + 1) & 0xffffff
}
unsafe fn cpu_adrAbs(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    *low = (u32::from((*cpu).db) << 16) + adr;
    ((u32::from((*cpu).db) << 16) + adr + 1) & 0xffffff
}
unsafe fn cpu_adrAbx(cpu: *mut Cpu, low: *mut uint32_t, write: bool, game: &mut Game) -> uint32_t {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    if write && (!(*cpu).xf || adr >> 8 != (adr + u32::from((*cpu).x)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).x)) & 0xffffff;
    ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).x) + 1) & 0xffffff
}
unsafe fn cpu_adrAby(cpu: *mut Cpu, low: *mut uint32_t, write: bool, game: &mut Game) -> uint32_t {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    if write && (!(*cpu).xf || adr >> 8 != (adr + u32::from((*cpu).y)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    *low = ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).y)) & 0xffffff;
    ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).y) + 1) & 0xffffff
}
unsafe fn cpu_adrAbl(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    *low = adr;
    adr.wrapping_add(1) & 0xffffff
}
unsafe fn cpu_adrAlx(cpu: *mut Cpu, low: *mut uint32_t, game: &mut Game) -> uint32_t {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    *low = adr.wrapping_add(u32::from((*cpu).x)) & 0xffffff;
    adr.wrapping_add(u32::from((*cpu).x)).wrapping_add(1) & 0xffffff
}
unsafe fn cpu_adrIax(cpu: *mut Cpu, game: &mut Game) -> uint16_t {
    let adr = cpu_readOpcodeWord(cpu, game);
    cpu_readWord(
        cpu,
        u32::from((*cpu).k) << 16 | u32::from(adr.wrapping_add((*cpu).x)),
        u32::from((*cpu).k) << 16 | u32::from(adr.wrapping_add((*cpu).x).wrapping_add(1)),
        game,
    )
}
unsafe fn cpu_and(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(cpu_read(cpu, low, game));
        (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a & value) & 0xff;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).a &= value;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_ora(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(cpu_read(cpu, low, game));
        (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a | value) & 0xff;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).a |= value;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_eor(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = u16::from(cpu_read(cpu, low, game));
        (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a ^ value) & 0xff;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).a ^= value;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_adc(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let mut value: uint8_t = cpu_read(cpu, low, game);
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
        let mut value_0: uint16_t = cpu_readWord(cpu, low, high, game);
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
unsafe fn cpu_sbc(mut cpu: *mut Cpu, mut low: uint32_t, mut high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let mut value: uint8_t = (cpu_read(cpu, low, game) as ::core::ffi::c_int
            ^ 0xff as ::core::ffi::c_int) as uint8_t;
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
        let mut value_0: uint16_t = (cpu_readWord(cpu, low, high, game) as ::core::ffi::c_int
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
unsafe fn cpu_cmp(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        let result = u32::from((*cpu).a & 0xff) + u32::from(!value) + 1;
        (*cpu).c = result > 0xff;
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        let result = u32::from((*cpu).a) + u32::from(!value) + 1;
        (*cpu).c = result > 0xffff;
        result
    };
    cpu_setZN(cpu, result as u16, (*cpu).mf);
}
unsafe fn cpu_cpx(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).xf {
        let value = cpu_read(cpu, low, game);
        let result = u32::from((*cpu).x & 0xff) + u32::from(!value) + 1;
        (*cpu).c = result > 0xff;
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        let result = u32::from((*cpu).x) + u32::from(!value) + 1;
        (*cpu).c = result > 0xffff;
        result
    };
    cpu_setZN(cpu, result as u16, (*cpu).xf);
}
unsafe fn cpu_cpy(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).xf {
        let value = cpu_read(cpu, low, game);
        let result = u32::from((*cpu).y & 0xff) + u32::from(!value) + 1;
        (*cpu).c = result > 0xff;
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        let result = u32::from((*cpu).y) + u32::from(!value) + 1;
        (*cpu).c = result > 0xffff;
        result
    };
    cpu_setZN(cpu, result as u16, (*cpu).xf);
}
unsafe fn cpu_bit(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        (*cpu).z = (*cpu).a as u8 & value == 0;
        (*cpu).n = value & 0x80 != 0;
        (*cpu).v = value & 0x40 != 0;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).z = (*cpu).a & value == 0;
        (*cpu).n = value & 0x8000 != 0;
        (*cpu).v = value & 0x4000 != 0;
    }
}
unsafe fn cpu_lda(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        (*cpu).a = (*cpu).a & 0xff00 | u16::from(cpu_read(cpu, low, game));
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).a = cpu_readWord(cpu, low, high, game);
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_ldx(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).xf {
        (*cpu).x = u16::from(cpu_read(cpu, low, game));
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).x = cpu_readWord(cpu, low, high, game);
    }
    cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
}
unsafe fn cpu_ldy(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).xf {
        (*cpu).y = u16::from(cpu_read(cpu, low, game));
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        (*cpu).y = cpu_readWord(cpu, low, high, game);
    }
    cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
}
unsafe fn cpu_sta(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        cpu_write(cpu, low, (*cpu).a as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).a, false, game);
    }
}
unsafe fn cpu_stx(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).xf {
        cpu_write(cpu, low, (*cpu).x as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).x, false, game);
    }
}
unsafe fn cpu_sty(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).xf {
        cpu_write(cpu, low, (*cpu).y as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, (*cpu).y, false, game);
    }
}
unsafe fn cpu_stz(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        cpu_write(cpu, low, 0, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        cpu_writeWord(cpu, low, high, 0, false, game);
    }
}
unsafe fn cpu_ror(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let (result, carry) = if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        let carry = value & 1 != 0;
        let result = u16::from(value >> 1) | (u16::from((*cpu).c) << 7);
        cpu_write(cpu, low, result as u8, game);
        (result, carry)
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = cpu_readWord(cpu, low, high, game);
        let carry = value & 1 != 0;
        let result = value >> 1 | (u16::from((*cpu).c) << 15);
        cpu_writeWord(cpu, low, high, result, true, game);
        (result, carry)
    };
    cpu_setZN(cpu, result, (*cpu).mf);
    (*cpu).c = carry;
}
unsafe fn cpu_rol(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let result = (u32::from(cpu_read(cpu, low, game)) << 1) | u32::from((*cpu).c);
        (*cpu).c = result & 0x100 != 0;
        cpu_write(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let result = (u32::from(cpu_readWord(cpu, low, high, game)) << 1) | u32::from((*cpu).c);
        (*cpu).c = result & 0x10000 != 0;
        cpu_writeWord(cpu, low, high, result as u16, true, game);
        result
    };
    cpu_setZN(cpu, result as u16, (*cpu).mf);
}
unsafe fn cpu_lsr(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        (*cpu).c = value & 1 != 0;
        let result = value >> 1;
        cpu_write(cpu, low, result, game);
        u16::from(result)
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).c = value & 1 != 0;
        let result = value >> 1;
        cpu_writeWord(cpu, low, high, result, true, game);
        result
    };
    cpu_setZN(cpu, result, (*cpu).mf);
}
unsafe fn cpu_asl(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let result = u32::from(cpu_read(cpu, low, game)) << 1;
        (*cpu).c = result & 0x100 != 0;
        cpu_write(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let result = u32::from(cpu_readWord(cpu, low, high, game)) << 1;
        (*cpu).c = result & 0x10000 != 0;
        cpu_writeWord(cpu, low, high, result as u16, true, game);
        result
    };
    cpu_setZN(cpu, result as u16, (*cpu).mf);
}
unsafe fn cpu_inc(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let result = u16::from(cpu_read(cpu, low, game)).wrapping_add(1);
        cpu_write(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let result = cpu_readWord(cpu, low, high, game).wrapping_add(1);
        cpu_writeWord(cpu, low, high, result, true, game);
        result
    };
    cpu_setZN(cpu, result, (*cpu).mf);
}
unsafe fn cpu_dec(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    let result = if (*cpu).mf {
        let result = u16::from(cpu_read(cpu, low, game)).wrapping_sub(1);
        cpu_write(cpu, low, result as u8, game);
        result
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let result = cpu_readWord(cpu, low, high, game).wrapping_sub(1);
        cpu_writeWord(cpu, low, high, result, true, game);
        result
    };
    cpu_setZN(cpu, result, (*cpu).mf);
}
unsafe fn cpu_tsb(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        (*cpu).z = (*cpu).a as u8 & value == 0;
        cpu_write(cpu, low, value | (*cpu).a as u8, game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).z = (*cpu).a & value == 0;
        cpu_writeWord(cpu, low, high, value | (*cpu).a, true, game);
    }
}
unsafe fn cpu_trb(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = cpu_read(cpu, low, game);
        (*cpu).z = (*cpu).a as u8 & value == 0;
        cpu_write(cpu, low, value & !((*cpu).a as u8), game);
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(2);
        let value = cpu_readWord(cpu, low, high, game);
        (*cpu).z = (*cpu).a & value == 0;
        cpu_writeWord(cpu, low, high, value & !(*cpu).a, true, game);
    }
}

pub unsafe fn DumpCpuHistory(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        printf(
            b"PC history: 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            game.pc_hist[(game.pc_hist_ctr.wrapping_add(i as uint32_t) & 7 as uint32_t) as usize],
        );
        i += 1;
    }
}
unsafe fn cpu_doOpcode(mut cpu: *mut Cpu, mut opcode: uint8_t, game: &mut Game) {
    let mut cur_pc: uint32 = (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
        | (*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
        as uint32;
    game.pc_hist[game.pc_hist_ctr as usize] = cur_pc as uint32_t;
    game.pc_hist_ctr = game.pc_hist_ctr.wrapping_add(1 as uint32_t) & 7 as uint32_t;
    if cur_pc == game.pc_bp {
        opcode = (opcode as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint8_t;
    }
    loop {
        match opcode as ::core::ffi::c_int {
            0 => {
                let mut addr: uint32_t =
                    (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).pc as ::core::ffi::c_int
                        | 0x800000 as ::core::ffi::c_int) as uint32_t;
                opcode = CpuOpcodeHook((addr as uint32).wrapping_sub(1 as uint32), game) as uint8_t;
                match opcode as ::core::ffi::c_int {
                    0 => {
                        break;
                    }
                    1 => {
                        (*cpu).pc = (cpu_pullWord(cpu, game) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int)
                            as uint16_t;
                        break;
                    }
                    2 => {
                        (*cpu).pc = (cpu_pullWord(cpu, game) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int)
                            as uint16_t;
                        (*cpu).k = cpu_pullByte(cpu, game);
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
                let mut high: uint32_t = cpu_adrIdx(cpu, &raw mut low, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            2 => {
                cpu_readOpcode(cpu, game);
                cpu_pushByte(cpu, (*cpu).k, game);
                cpu_pushWord(cpu, (*cpu).pc, game);
                cpu_pushByte(cpu, cpu_getFlags(cpu), game);
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).i = true_0 != 0;
                (*cpu).d = false_0 != 0;
                (*cpu).k = 0 as uint8_t;
                (*cpu).pc = cpu_readWord(cpu, 0xffe4 as uint32_t, 0xffe5 as uint32_t, game);
                break;
            }
            3 => {
                let mut low_0: uint32_t = 0 as uint32_t;
                let mut high_0: uint32_t = cpu_adrSr(cpu, &raw mut low_0, game);
                cpu_ora(cpu, low_0, high_0, game);
                break;
            }
            4 => {
                let mut low_1: uint32_t = 0 as uint32_t;
                let mut high_1: uint32_t = cpu_adrDp(cpu, &raw mut low_1, game);
                cpu_tsb(cpu, low_1, high_1, game);
                break;
            }
            5 => {
                let mut low_2: uint32_t = 0 as uint32_t;
                let mut high_2: uint32_t = cpu_adrDp(cpu, &raw mut low_2, game);
                cpu_ora(cpu, low_2, high_2, game);
                break;
            }
            6 => {
                let mut low_3: uint32_t = 0 as uint32_t;
                let mut high_3: uint32_t = cpu_adrDp(cpu, &raw mut low_3, game);
                cpu_asl(cpu, low_3, high_3, game);
                break;
            }
            7 => {
                let mut low_4: uint32_t = 0 as uint32_t;
                let mut high_4: uint32_t = cpu_adrIdl(cpu, &raw mut low_4, game);
                cpu_ora(cpu, low_4, high_4, game);
                break;
            }
            8 => {
                cpu_pushByte(cpu, cpu_getFlags(cpu), game);
                break;
            }
            9 => {
                let mut low_5: uint32_t = 0 as uint32_t;
                let mut high_5: uint32_t = cpu_adrImm(cpu, &raw mut low_5, false_0 != 0);
                cpu_ora(cpu, low_5, high_5, game);
                break;
            }
            10 => {
                if (*cpu).mf {
                    (*cpu).c = (*cpu).a & 0x80 != 0;
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a << 1 & 0xff);
                } else {
                    (*cpu).c = (*cpu).a & 0x8000 != 0;
                    (*cpu).a <<= 1;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            11 => {
                cpu_pushWord(cpu, (*cpu).dp, game);
                break;
            }
            12 => {
                let mut low_6: uint32_t = 0 as uint32_t;
                let mut high_6: uint32_t = cpu_adrAbs(cpu, &raw mut low_6, game);
                cpu_tsb(cpu, low_6, high_6, game);
                break;
            }
            13 => {
                let mut low_7: uint32_t = 0 as uint32_t;
                let mut high_7: uint32_t = cpu_adrAbs(cpu, &raw mut low_7, game);
                cpu_ora(cpu, low_7, high_7, game);
                break;
            }
            14 => {
                let mut low_8: uint32_t = 0 as uint32_t;
                let mut high_8: uint32_t = cpu_adrAbs(cpu, &raw mut low_8, game);
                cpu_asl(cpu, low_8, high_8, game);
                break;
            }
            15 => {
                let mut low_9: uint32_t = 0 as uint32_t;
                let mut high_9: uint32_t = cpu_adrAbl(cpu, &raw mut low_9, game);
                cpu_ora(cpu, low_9, high_9, game);
                break;
            }
            16 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).n);
                break;
            }
            17 => {
                let mut low_10: uint32_t = 0 as uint32_t;
                let mut high_10: uint32_t = cpu_adrIdy(cpu, &raw mut low_10, false_0 != 0, game);
                cpu_ora(cpu, low_10, high_10, game);
                break;
            }
            18 => {
                let mut low_11: uint32_t = 0 as uint32_t;
                let mut high_11: uint32_t = cpu_adrIdp(cpu, &raw mut low_11, game);
                cpu_ora(cpu, low_11, high_11, game);
                break;
            }
            19 => {
                let mut low_12: uint32_t = 0 as uint32_t;
                let mut high_12: uint32_t = cpu_adrIsy(cpu, &raw mut low_12, game);
                cpu_ora(cpu, low_12, high_12, game);
                break;
            }
            20 => {
                let mut low_13: uint32_t = 0 as uint32_t;
                let mut high_13: uint32_t = cpu_adrDp(cpu, &raw mut low_13, game);
                cpu_trb(cpu, low_13, high_13, game);
                break;
            }
            21 => {
                let mut low_14: uint32_t = 0 as uint32_t;
                let mut high_14: uint32_t = cpu_adrDpx(cpu, &raw mut low_14, game);
                cpu_ora(cpu, low_14, high_14, game);
                break;
            }
            22 => {
                let mut low_15: uint32_t = 0 as uint32_t;
                let mut high_15: uint32_t = cpu_adrDpx(cpu, &raw mut low_15, game);
                cpu_asl(cpu, low_15, high_15, game);
                break;
            }
            23 => {
                let mut low_16: uint32_t = 0 as uint32_t;
                let mut high_16: uint32_t = cpu_adrIly(cpu, &raw mut low_16, game);
                cpu_ora(cpu, low_16, high_16, game);
                break;
            }
            24 => {
                (*cpu).c = false;
                break;
            }
            25 => {
                let mut low_17: uint32_t = 0 as uint32_t;
                let mut high_17: uint32_t = cpu_adrAby(cpu, &raw mut low_17, false_0 != 0, game);
                cpu_ora(cpu, low_17, high_17, game);
                break;
            }
            26 => {
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a.wrapping_add(1) & 0xff);
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
                let mut high_18: uint32_t = cpu_adrAbs(cpu, &raw mut low_18, game);
                cpu_trb(cpu, low_18, high_18, game);
                break;
            }
            29 => {
                let mut low_19: uint32_t = 0 as uint32_t;
                let mut high_19: uint32_t = cpu_adrAbx(cpu, &raw mut low_19, false_0 != 0, game);
                cpu_ora(cpu, low_19, high_19, game);
                break;
            }
            30 => {
                let mut low_20: uint32_t = 0 as uint32_t;
                let mut high_20: uint32_t = cpu_adrAbx(cpu, &raw mut low_20, true_0 != 0, game);
                cpu_asl(cpu, low_20, high_20, game);
                break;
            }
            31 => {
                let mut low_21: uint32_t = 0 as uint32_t;
                let mut high_21: uint32_t = cpu_adrAlx(cpu, &raw mut low_21, game);
                cpu_ora(cpu, low_21, high_21, game);
                break;
            }
            32 => {
                let mut value: uint16_t = cpu_readOpcodeWord(cpu, game);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                    game,
                );
                (*cpu).pc = value;
                break;
            }
            33 => {
                let mut low_22: uint32_t = 0 as uint32_t;
                let mut high_22: uint32_t = cpu_adrIdx(cpu, &raw mut low_22, game);
                cpu_and(cpu, low_22, high_22, game);
                break;
            }
            34 => {
                let mut value_0: uint16_t = cpu_readOpcodeWord(cpu, game);
                let mut newK: uint8_t = cpu_readOpcode(cpu, game);
                cpu_pushByte(cpu, (*cpu).k, game);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                    game,
                );
                (*cpu).pc = value_0;
                (*cpu).k = newK;
                break;
            }
            35 => {
                let mut low_23: uint32_t = 0 as uint32_t;
                let mut high_23: uint32_t = cpu_adrSr(cpu, &raw mut low_23, game);
                cpu_and(cpu, low_23, high_23, game);
                break;
            }
            36 => {
                let mut low_24: uint32_t = 0 as uint32_t;
                let mut high_24: uint32_t = cpu_adrDp(cpu, &raw mut low_24, game);
                cpu_bit(cpu, low_24, high_24, game);
                break;
            }
            37 => {
                let mut low_25: uint32_t = 0 as uint32_t;
                let mut high_25: uint32_t = cpu_adrDp(cpu, &raw mut low_25, game);
                cpu_and(cpu, low_25, high_25, game);
                break;
            }
            38 => {
                let mut low_26: uint32_t = 0 as uint32_t;
                let mut high_26: uint32_t = cpu_adrDp(cpu, &raw mut low_26, game);
                cpu_rol(cpu, low_26, high_26, game);
                break;
            }
            39 => {
                let mut low_27: uint32_t = 0 as uint32_t;
                let mut high_27: uint32_t = cpu_adrIdl(cpu, &raw mut low_27, game);
                cpu_and(cpu, low_27, high_27, game);
                break;
            }
            40 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu, game));
                break;
            }
            41 => {
                let mut low_28: uint32_t = 0 as uint32_t;
                let mut high_28: uint32_t = cpu_adrImm(cpu, &raw mut low_28, false_0 != 0);
                cpu_and(cpu, low_28, high_28, game);
                break;
            }
            42 => {
                let result = (u32::from((*cpu).a) << 1) | u32::from((*cpu).c);
                if (*cpu).mf {
                    (*cpu).c = result & 0x100 != 0;
                    (*cpu).a = (*cpu).a & 0xff00 | (result as u16 & 0xff);
                } else {
                    (*cpu).c = result & 0x10000 != 0;
                    (*cpu).a = result as u16;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            43 => {
                (*cpu).dp = cpu_pullWord(cpu, game);
                cpu_setZN(cpu, (*cpu).dp, false);
                break;
            }
            44 => {
                let mut low_29: uint32_t = 0 as uint32_t;
                let mut high_29: uint32_t = cpu_adrAbs(cpu, &raw mut low_29, game);
                cpu_bit(cpu, low_29, high_29, game);
                break;
            }
            45 => {
                let mut low_30: uint32_t = 0 as uint32_t;
                let mut high_30: uint32_t = cpu_adrAbs(cpu, &raw mut low_30, game);
                cpu_and(cpu, low_30, high_30, game);
                break;
            }
            46 => {
                let mut low_31: uint32_t = 0 as uint32_t;
                let mut high_31: uint32_t = cpu_adrAbs(cpu, &raw mut low_31, game);
                cpu_rol(cpu, low_31, high_31, game);
                break;
            }
            47 => {
                let mut low_32: uint32_t = 0 as uint32_t;
                let mut high_32: uint32_t = cpu_adrAbl(cpu, &raw mut low_32, game);
                cpu_and(cpu, low_32, high_32, game);
                break;
            }
            48 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).n);
                break;
            }
            49 => {
                let mut low_33: uint32_t = 0 as uint32_t;
                let mut high_33: uint32_t = cpu_adrIdy(cpu, &raw mut low_33, false_0 != 0, game);
                cpu_and(cpu, low_33, high_33, game);
                break;
            }
            50 => {
                let mut low_34: uint32_t = 0 as uint32_t;
                let mut high_34: uint32_t = cpu_adrIdp(cpu, &raw mut low_34, game);
                cpu_and(cpu, low_34, high_34, game);
                break;
            }
            51 => {
                let mut low_35: uint32_t = 0 as uint32_t;
                let mut high_35: uint32_t = cpu_adrIsy(cpu, &raw mut low_35, game);
                cpu_and(cpu, low_35, high_35, game);
                break;
            }
            52 => {
                let mut low_36: uint32_t = 0 as uint32_t;
                let mut high_36: uint32_t = cpu_adrDpx(cpu, &raw mut low_36, game);
                cpu_bit(cpu, low_36, high_36, game);
                break;
            }
            53 => {
                let mut low_37: uint32_t = 0 as uint32_t;
                let mut high_37: uint32_t = cpu_adrDpx(cpu, &raw mut low_37, game);
                cpu_and(cpu, low_37, high_37, game);
                break;
            }
            54 => {
                let mut low_38: uint32_t = 0 as uint32_t;
                let mut high_38: uint32_t = cpu_adrDpx(cpu, &raw mut low_38, game);
                cpu_rol(cpu, low_38, high_38, game);
                break;
            }
            55 => {
                let mut low_39: uint32_t = 0 as uint32_t;
                let mut high_39: uint32_t = cpu_adrIly(cpu, &raw mut low_39, game);
                cpu_and(cpu, low_39, high_39, game);
                break;
            }
            56 => {
                (*cpu).c = true;
                break;
            }
            57 => {
                let mut low_40: uint32_t = 0 as uint32_t;
                let mut high_40: uint32_t = cpu_adrAby(cpu, &raw mut low_40, false_0 != 0, game);
                cpu_and(cpu, low_40, high_40, game);
                break;
            }
            58 => {
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a.wrapping_sub(1) & 0xff);
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
                let mut high_41: uint32_t = cpu_adrAbx(cpu, &raw mut low_41, false_0 != 0, game);
                cpu_bit(cpu, low_41, high_41, game);
                break;
            }
            61 => {
                let mut low_42: uint32_t = 0 as uint32_t;
                let mut high_42: uint32_t = cpu_adrAbx(cpu, &raw mut low_42, false_0 != 0, game);
                cpu_and(cpu, low_42, high_42, game);
                break;
            }
            62 => {
                let mut low_43: uint32_t = 0 as uint32_t;
                let mut high_43: uint32_t = cpu_adrAbx(cpu, &raw mut low_43, true_0 != 0, game);
                cpu_rol(cpu, low_43, high_43, game);
                break;
            }
            63 => {
                let mut low_44: uint32_t = 0 as uint32_t;
                let mut high_44: uint32_t = cpu_adrAlx(cpu, &raw mut low_44, game);
                cpu_and(cpu, low_44, high_44, game);
                break;
            }
            64 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu, game));
                (*cpu).pc = cpu_pullWord(cpu, game);
                if !(*cpu).e {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).k = cpu_pullByte(cpu, game);
                }
                break;
            }
            65 => {
                let mut low_45: uint32_t = 0 as uint32_t;
                let mut high_45: uint32_t = cpu_adrIdx(cpu, &raw mut low_45, game);
                cpu_eor(cpu, low_45, high_45, game);
                break;
            }
            66 => {
                cpu_readOpcode(cpu, game);
                break;
            }
            67 => {
                let mut low_46: uint32_t = 0 as uint32_t;
                let mut high_46: uint32_t = cpu_adrSr(cpu, &raw mut low_46, game);
                cpu_eor(cpu, low_46, high_46, game);
                break;
            }
            68 => {
                let mut dest: uint8_t = cpu_readOpcode(cpu, game);
                let mut src: uint8_t = cpu_readOpcode(cpu, game);
                (*cpu).db = dest;
                cpu_write(
                    cpu,
                    ((dest as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).y as ::core::ffi::c_int) as uint32_t,
                    cpu_read(
                        cpu,
                        ((src as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).x as ::core::ffi::c_int) as uint32_t,
                        game,
                    ),
                    game,
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
                let mut high_47: uint32_t = cpu_adrDp(cpu, &raw mut low_47, game);
                cpu_eor(cpu, low_47, high_47, game);
                break;
            }
            70 => {
                let mut low_48: uint32_t = 0 as uint32_t;
                let mut high_48: uint32_t = cpu_adrDp(cpu, &raw mut low_48, game);
                cpu_lsr(cpu, low_48, high_48, game);
                break;
            }
            71 => {
                let mut low_49: uint32_t = 0 as uint32_t;
                let mut high_49: uint32_t = cpu_adrIdl(cpu, &raw mut low_49, game);
                cpu_eor(cpu, low_49, high_49, game);
                break;
            }
            72 => {
                if (*cpu).mf {
                    cpu_pushByte(cpu, (*cpu).a as uint8_t, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).a, game);
                }
                break;
            }
            73 => {
                let mut low_50: uint32_t = 0 as uint32_t;
                let mut high_50: uint32_t = cpu_adrImm(cpu, &raw mut low_50, false_0 != 0);
                cpu_eor(cpu, low_50, high_50, game);
                break;
            }
            74 => {
                (*cpu).c = (*cpu).a & 1 != 0;
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).a >> 1 & 0x7f);
                } else {
                    (*cpu).a >>= 1;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            75 => {
                cpu_pushByte(cpu, (*cpu).k, game);
                break;
            }
            76 => {
                (*cpu).pc = cpu_readOpcodeWord(cpu, game);
                break;
            }
            77 => {
                let mut low_51: uint32_t = 0 as uint32_t;
                let mut high_51: uint32_t = cpu_adrAbs(cpu, &raw mut low_51, game);
                cpu_eor(cpu, low_51, high_51, game);
                break;
            }
            78 => {
                let mut low_52: uint32_t = 0 as uint32_t;
                let mut high_52: uint32_t = cpu_adrAbs(cpu, &raw mut low_52, game);
                cpu_lsr(cpu, low_52, high_52, game);
                break;
            }
            79 => {
                let mut low_53: uint32_t = 0 as uint32_t;
                let mut high_53: uint32_t = cpu_adrAbl(cpu, &raw mut low_53, game);
                cpu_eor(cpu, low_53, high_53, game);
                break;
            }
            80 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).v);
                break;
            }
            81 => {
                let mut low_54: uint32_t = 0 as uint32_t;
                let mut high_54: uint32_t = cpu_adrIdy(cpu, &raw mut low_54, false_0 != 0, game);
                cpu_eor(cpu, low_54, high_54, game);
                break;
            }
            82 => {
                let mut low_55: uint32_t = 0 as uint32_t;
                let mut high_55: uint32_t = cpu_adrIdp(cpu, &raw mut low_55, game);
                cpu_eor(cpu, low_55, high_55, game);
                break;
            }
            83 => {
                let mut low_56: uint32_t = 0 as uint32_t;
                let mut high_56: uint32_t = cpu_adrIsy(cpu, &raw mut low_56, game);
                cpu_eor(cpu, low_56, high_56, game);
                break;
            }
            84 => {
                let mut dest_0: uint8_t = cpu_readOpcode(cpu, game);
                let mut src_0: uint8_t = cpu_readOpcode(cpu, game);
                (*cpu).db = dest_0;
                cpu_write(
                    cpu,
                    ((dest_0 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).y as ::core::ffi::c_int) as uint32_t,
                    cpu_read(
                        cpu,
                        ((src_0 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).x as ::core::ffi::c_int) as uint32_t,
                        game,
                    ),
                    game,
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
                let mut high_57: uint32_t = cpu_adrDpx(cpu, &raw mut low_57, game);
                cpu_eor(cpu, low_57, high_57, game);
                break;
            }
            86 => {
                let mut low_58: uint32_t = 0 as uint32_t;
                let mut high_58: uint32_t = cpu_adrDpx(cpu, &raw mut low_58, game);
                cpu_lsr(cpu, low_58, high_58, game);
                break;
            }
            87 => {
                let mut low_59: uint32_t = 0 as uint32_t;
                let mut high_59: uint32_t = cpu_adrIly(cpu, &raw mut low_59, game);
                cpu_eor(cpu, low_59, high_59, game);
                break;
            }
            88 => {
                (*cpu).i = false;
                break;
            }
            89 => {
                let mut low_60: uint32_t = 0 as uint32_t;
                let mut high_60: uint32_t = cpu_adrAby(cpu, &raw mut low_60, false_0 != 0, game);
                cpu_eor(cpu, low_60, high_60, game);
                break;
            }
            90 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).y as uint8_t, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).y, game);
                }
                break;
            }
            91 => {
                (*cpu).dp = (*cpu).a;
                cpu_setZN(cpu, (*cpu).dp, false_0 != 0);
                break;
            }
            92 => {
                let mut value_1: uint16_t = cpu_readOpcodeWord(cpu, game);
                let mut new_k: uint8_t = cpu_readOpcode(cpu, game);
                if new_k as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int
                    && value_1 as ::core::ffi::c_int == 0x8573 as ::core::ffi::c_int
                {
                    printf(
                        b"Current PC = 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
                        ((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                            | (*cpu).pc as ::core::ffi::c_int,
                    );
                    DumpCpuHistory(game);
                    Die(b"The game has crashed!\n\0" as *const u8 as *const ::core::ffi::c_char);
                }
                (*cpu).k = new_k;
                (*cpu).pc = value_1;
                break;
            }
            93 => {
                let mut low_61: uint32_t = 0 as uint32_t;
                let mut high_61: uint32_t = cpu_adrAbx(cpu, &raw mut low_61, false_0 != 0, game);
                cpu_eor(cpu, low_61, high_61, game);
                break;
            }
            94 => {
                let mut low_62: uint32_t = 0 as uint32_t;
                let mut high_62: uint32_t = cpu_adrAbx(cpu, &raw mut low_62, true_0 != 0, game);
                cpu_lsr(cpu, low_62, high_62, game);
                break;
            }
            95 => {
                let mut low_63: uint32_t = 0 as uint32_t;
                let mut high_63: uint32_t = cpu_adrAlx(cpu, &raw mut low_63, game);
                cpu_eor(cpu, low_63, high_63, game);
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
                    if HookedFunctionRts(
                        (delta == 2 as ::core::ffi::c_int) as ::core::ffi::c_int,
                        game,
                    ) {
                        return;
                    }
                }
                (*cpu).pc = (cpu_pullWord(cpu, game) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16_t;
                break;
            }
            97 => {
                let mut low_64: uint32_t = 0 as uint32_t;
                let mut high_64: uint32_t = cpu_adrIdx(cpu, &raw mut low_64, game);
                cpu_adc(cpu, low_64, high_64, game);
                break;
            }
            98 => {
                let mut value_2: uint16_t = cpu_readOpcodeWord(cpu, game);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int + value_2 as int16_t as ::core::ffi::c_int)
                        as uint16_t,
                    game,
                );
                break;
            }
            99 => {
                let mut low_65: uint32_t = 0 as uint32_t;
                let mut high_65: uint32_t = cpu_adrSr(cpu, &raw mut low_65, game);
                cpu_adc(cpu, low_65, high_65, game);
                break;
            }
            100 => {
                let mut low_66: uint32_t = 0 as uint32_t;
                let mut high_66: uint32_t = cpu_adrDp(cpu, &raw mut low_66, game);
                cpu_stz(cpu, low_66, high_66, game);
                break;
            }
            101 => {
                let mut low_67: uint32_t = 0 as uint32_t;
                let mut high_67: uint32_t = cpu_adrDp(cpu, &raw mut low_67, game);
                cpu_adc(cpu, low_67, high_67, game);
                break;
            }
            102 => {
                let mut low_68: uint32_t = 0 as uint32_t;
                let mut high_68: uint32_t = cpu_adrDp(cpu, &raw mut low_68, game);
                cpu_ror(cpu, low_68, high_68, game);
                break;
            }
            103 => {
                let mut low_69: uint32_t = 0 as uint32_t;
                let mut high_69: uint32_t = cpu_adrIdl(cpu, &raw mut low_69, game);
                cpu_adc(cpu, low_69, high_69, game);
                break;
            }
            104 => {
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | u16::from(cpu_pullByte(cpu, game));
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).a = cpu_pullWord(cpu, game);
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            105 => {
                let mut low_70: uint32_t = 0 as uint32_t;
                let mut high_70: uint32_t = cpu_adrImm(cpu, &raw mut low_70, false_0 != 0);
                cpu_adc(cpu, low_70, high_70, game);
                break;
            }
            106 => {
                let carry = (*cpu).a & 1 != 0;
                if (*cpu).mf {
                    (*cpu).a =
                        (*cpu).a & 0xff00 | ((*cpu).a >> 1 & 0x7f) | (u16::from((*cpu).c) << 7);
                } else {
                    (*cpu).a = (*cpu).a >> 1 | (u16::from((*cpu).c) << 15);
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
                    if HookedFunctionRts(0 as ::core::ffi::c_int, game) {
                        return;
                    }
                }
                (*cpu).pc = (cpu_pullWord(cpu, game) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16_t;
                (*cpu).k = cpu_pullByte(cpu, game);
                break;
            }
            108 => {
                let mut adr: uint16_t = cpu_readOpcodeWord(cpu, game);
                (*cpu).pc = cpu_readWord(
                    cpu,
                    adr as uint32_t,
                    (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                    game,
                );
                break;
            }
            109 => {
                let mut low_71: uint32_t = 0 as uint32_t;
                let mut high_71: uint32_t = cpu_adrAbs(cpu, &raw mut low_71, game);
                cpu_adc(cpu, low_71, high_71, game);
                break;
            }
            110 => {
                let mut low_72: uint32_t = 0 as uint32_t;
                let mut high_72: uint32_t = cpu_adrAbs(cpu, &raw mut low_72, game);
                cpu_ror(cpu, low_72, high_72, game);
                break;
            }
            111 => {
                let mut low_73: uint32_t = 0 as uint32_t;
                let mut high_73: uint32_t = cpu_adrAbl(cpu, &raw mut low_73, game);
                cpu_adc(cpu, low_73, high_73, game);
                break;
            }
            112 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).v);
                break;
            }
            113 => {
                let mut low_74: uint32_t = 0 as uint32_t;
                let mut high_74: uint32_t = cpu_adrIdy(cpu, &raw mut low_74, false_0 != 0, game);
                cpu_adc(cpu, low_74, high_74, game);
                break;
            }
            114 => {
                let mut low_75: uint32_t = 0 as uint32_t;
                let mut high_75: uint32_t = cpu_adrIdp(cpu, &raw mut low_75, game);
                cpu_adc(cpu, low_75, high_75, game);
                break;
            }
            115 => {
                let mut low_76: uint32_t = 0 as uint32_t;
                let mut high_76: uint32_t = cpu_adrIsy(cpu, &raw mut low_76, game);
                cpu_adc(cpu, low_76, high_76, game);
                break;
            }
            116 => {
                let mut low_77: uint32_t = 0 as uint32_t;
                let mut high_77: uint32_t = cpu_adrDpx(cpu, &raw mut low_77, game);
                cpu_stz(cpu, low_77, high_77, game);
                break;
            }
            117 => {
                let mut low_78: uint32_t = 0 as uint32_t;
                let mut high_78: uint32_t = cpu_adrDpx(cpu, &raw mut low_78, game);
                cpu_adc(cpu, low_78, high_78, game);
                break;
            }
            118 => {
                let mut low_79: uint32_t = 0 as uint32_t;
                let mut high_79: uint32_t = cpu_adrDpx(cpu, &raw mut low_79, game);
                cpu_ror(cpu, low_79, high_79, game);
                break;
            }
            119 => {
                let mut low_80: uint32_t = 0 as uint32_t;
                let mut high_80: uint32_t = cpu_adrIly(cpu, &raw mut low_80, game);
                cpu_adc(cpu, low_80, high_80, game);
                break;
            }
            120 => {
                (*cpu).i = true;
                break;
            }
            121 => {
                let mut low_81: uint32_t = 0 as uint32_t;
                let mut high_81: uint32_t = cpu_adrAby(cpu, &raw mut low_81, false_0 != 0, game);
                cpu_adc(cpu, low_81, high_81, game);
                break;
            }
            122 => {
                if (*cpu).xf {
                    (*cpu).y = u16::from(cpu_pullByte(cpu, game));
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).y = cpu_pullWord(cpu, game);
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
                (*cpu).pc = cpu_adrIax(cpu, game);
                break;
            }
            125 => {
                let mut low_82: uint32_t = 0 as uint32_t;
                let mut high_82: uint32_t = cpu_adrAbx(cpu, &raw mut low_82, false_0 != 0, game);
                cpu_adc(cpu, low_82, high_82, game);
                break;
            }
            126 => {
                let mut low_83: uint32_t = 0 as uint32_t;
                let mut high_83: uint32_t = cpu_adrAbx(cpu, &raw mut low_83, true_0 != 0, game);
                cpu_ror(cpu, low_83, high_83, game);
                break;
            }
            127 => {
                let mut low_84: uint32_t = 0 as uint32_t;
                let mut high_84: uint32_t = cpu_adrAlx(cpu, &raw mut low_84, game);
                cpu_adc(cpu, low_84, high_84, game);
                break;
            }
            128 => {
                // Relative branches start after the displacement operand.
                let displacement = cpu_readOpcode(cpu, game) as int8_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement as i16);
                break;
            }
            129 => {
                let mut low_85: uint32_t = 0 as uint32_t;
                let mut high_85: uint32_t = cpu_adrIdx(cpu, &raw mut low_85, game);
                cpu_sta(cpu, low_85, high_85, game);
                break;
            }
            130 => {
                let displacement = cpu_readOpcodeWord(cpu, game) as int16_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
                break;
            }
            131 => {
                let mut low_86: uint32_t = 0 as uint32_t;
                let mut high_86: uint32_t = cpu_adrSr(cpu, &raw mut low_86, game);
                cpu_sta(cpu, low_86, high_86, game);
                break;
            }
            132 => {
                let mut low_87: uint32_t = 0 as uint32_t;
                let mut high_87: uint32_t = cpu_adrDp(cpu, &raw mut low_87, game);
                cpu_sty(cpu, low_87, high_87, game);
                break;
            }
            133 => {
                let mut low_88: uint32_t = 0 as uint32_t;
                let mut high_88: uint32_t = cpu_adrDp(cpu, &raw mut low_88, game);
                cpu_sta(cpu, low_88, high_88, game);
                break;
            }
            134 => {
                let mut low_89: uint32_t = 0 as uint32_t;
                let mut high_89: uint32_t = cpu_adrDp(cpu, &raw mut low_89, game);
                cpu_stx(cpu, low_89, high_89, game);
                break;
            }
            135 => {
                let mut low_90: uint32_t = 0 as uint32_t;
                let mut high_90: uint32_t = cpu_adrIdl(cpu, &raw mut low_90, game);
                cpu_sta(cpu, low_90, high_90, game);
                break;
            }
            136 => {
                if (*cpu).xf {
                    (*cpu).y = (*cpu).y.wrapping_sub(1) & 0xff;
                } else {
                    (*cpu).y = (*cpu).y.wrapping_sub(1);
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            137 => {
                if (*cpu).mf {
                    let result = (*cpu).a as u8 & cpu_readOpcode(cpu, game);
                    (*cpu).z = result == 0;
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    let result = (*cpu).a & cpu_readOpcodeWord(cpu, game);
                    (*cpu).z = result == 0;
                }
                break;
            }
            138 => {
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).x & 0xff);
                } else {
                    (*cpu).a = (*cpu).x;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            139 => {
                cpu_pushByte(cpu, (*cpu).db, game);
                break;
            }
            140 => {
                let mut low_91: uint32_t = 0 as uint32_t;
                let mut high_91: uint32_t = cpu_adrAbs(cpu, &raw mut low_91, game);
                cpu_sty(cpu, low_91, high_91, game);
                break;
            }
            141 => {
                let mut low_92: uint32_t = 0 as uint32_t;
                let mut high_92: uint32_t = cpu_adrAbs(cpu, &raw mut low_92, game);
                cpu_sta(cpu, low_92, high_92, game);
                break;
            }
            142 => {
                let mut low_93: uint32_t = 0 as uint32_t;
                let mut high_93: uint32_t = cpu_adrAbs(cpu, &raw mut low_93, game);
                cpu_stx(cpu, low_93, high_93, game);
                break;
            }
            143 => {
                let mut low_94: uint32_t = 0 as uint32_t;
                let mut high_94: uint32_t = cpu_adrAbl(cpu, &raw mut low_94, game);
                cpu_sta(cpu, low_94, high_94, game);
                break;
            }
            144 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).c);
                break;
            }
            145 => {
                let mut low_95: uint32_t = 0 as uint32_t;
                let mut high_95: uint32_t = cpu_adrIdy(cpu, &raw mut low_95, true_0 != 0, game);
                cpu_sta(cpu, low_95, high_95, game);
                break;
            }
            146 => {
                let mut low_96: uint32_t = 0 as uint32_t;
                let mut high_96: uint32_t = cpu_adrIdp(cpu, &raw mut low_96, game);
                cpu_sta(cpu, low_96, high_96, game);
                break;
            }
            147 => {
                let mut low_97: uint32_t = 0 as uint32_t;
                let mut high_97: uint32_t = cpu_adrIsy(cpu, &raw mut low_97, game);
                cpu_sta(cpu, low_97, high_97, game);
                break;
            }
            148 => {
                let mut low_98: uint32_t = 0 as uint32_t;
                let mut high_98: uint32_t = cpu_adrDpx(cpu, &raw mut low_98, game);
                cpu_sty(cpu, low_98, high_98, game);
                break;
            }
            149 => {
                let mut low_99: uint32_t = 0 as uint32_t;
                let mut high_99: uint32_t = cpu_adrDpx(cpu, &raw mut low_99, game);
                cpu_sta(cpu, low_99, high_99, game);
                break;
            }
            150 => {
                let mut low_100: uint32_t = 0 as uint32_t;
                let mut high_100: uint32_t = cpu_adrDpy(cpu, &raw mut low_100, game);
                cpu_stx(cpu, low_100, high_100, game);
                break;
            }
            151 => {
                let mut low_101: uint32_t = 0 as uint32_t;
                let mut high_101: uint32_t = cpu_adrIly(cpu, &raw mut low_101, game);
                cpu_sta(cpu, low_101, high_101, game);
                break;
            }
            152 => {
                if (*cpu).mf {
                    (*cpu).a = (*cpu).a & 0xff00 | ((*cpu).y & 0xff);
                } else {
                    (*cpu).a = (*cpu).y;
                }
                cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
                break;
            }
            153 => {
                let mut low_102: uint32_t = 0 as uint32_t;
                let mut high_102: uint32_t = cpu_adrAby(cpu, &raw mut low_102, true_0 != 0, game);
                cpu_sta(cpu, low_102, high_102, game);
                break;
            }
            154 => {
                (*cpu).sp = (*cpu).x;
                break;
            }
            155 => {
                if (*cpu).xf {
                    (*cpu).y = (*cpu).x & 0xff;
                } else {
                    (*cpu).y = (*cpu).x;
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            156 => {
                let mut low_103: uint32_t = 0 as uint32_t;
                let mut high_103: uint32_t = cpu_adrAbs(cpu, &raw mut low_103, game);
                cpu_stz(cpu, low_103, high_103, game);
                break;
            }
            157 => {
                let mut low_104: uint32_t = 0 as uint32_t;
                let mut high_104: uint32_t = cpu_adrAbx(cpu, &raw mut low_104, true_0 != 0, game);
                cpu_sta(cpu, low_104, high_104, game);
                break;
            }
            158 => {
                let mut low_105: uint32_t = 0 as uint32_t;
                let mut high_105: uint32_t = cpu_adrAbx(cpu, &raw mut low_105, true_0 != 0, game);
                cpu_stz(cpu, low_105, high_105, game);
                break;
            }
            159 => {
                let mut low_106: uint32_t = 0 as uint32_t;
                let mut high_106: uint32_t = cpu_adrAlx(cpu, &raw mut low_106, game);
                cpu_sta(cpu, low_106, high_106, game);
                break;
            }
            160 => {
                let mut low_107: uint32_t = 0 as uint32_t;
                let mut high_107: uint32_t = cpu_adrImm(cpu, &raw mut low_107, true_0 != 0);
                cpu_ldy(cpu, low_107, high_107, game);
                break;
            }
            161 => {
                let mut low_108: uint32_t = 0 as uint32_t;
                let mut high_108: uint32_t = cpu_adrIdx(cpu, &raw mut low_108, game);
                cpu_lda(cpu, low_108, high_108, game);
                break;
            }
            162 => {
                let mut low_109: uint32_t = 0 as uint32_t;
                let mut high_109: uint32_t = cpu_adrImm(cpu, &raw mut low_109, true_0 != 0);
                cpu_ldx(cpu, low_109, high_109, game);
                break;
            }
            163 => {
                let mut low_110: uint32_t = 0 as uint32_t;
                let mut high_110: uint32_t = cpu_adrSr(cpu, &raw mut low_110, game);
                cpu_lda(cpu, low_110, high_110, game);
                break;
            }
            164 => {
                let mut low_111: uint32_t = 0 as uint32_t;
                let mut high_111: uint32_t = cpu_adrDp(cpu, &raw mut low_111, game);
                cpu_ldy(cpu, low_111, high_111, game);
                break;
            }
            165 => {
                let mut low_112: uint32_t = 0 as uint32_t;
                let mut high_112: uint32_t = cpu_adrDp(cpu, &raw mut low_112, game);
                cpu_lda(cpu, low_112, high_112, game);
                break;
            }
            166 => {
                let mut low_113: uint32_t = 0 as uint32_t;
                let mut high_113: uint32_t = cpu_adrDp(cpu, &raw mut low_113, game);
                cpu_ldx(cpu, low_113, high_113, game);
                break;
            }
            167 => {
                let mut low_114: uint32_t = 0 as uint32_t;
                let mut high_114: uint32_t = cpu_adrIdl(cpu, &raw mut low_114, game);
                cpu_lda(cpu, low_114, high_114, game);
                break;
            }
            168 => {
                if (*cpu).xf {
                    (*cpu).y = (*cpu).a & 0xff;
                } else {
                    (*cpu).y = (*cpu).a;
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            169 => {
                let mut low_115: uint32_t = 0 as uint32_t;
                let mut high_115: uint32_t = cpu_adrImm(cpu, &raw mut low_115, false_0 != 0);
                cpu_lda(cpu, low_115, high_115, game);
                break;
            }
            170 => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).a & 0xff;
                } else {
                    (*cpu).x = (*cpu).a;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            171 => {
                (*cpu).db = cpu_pullByte(cpu, game);
                cpu_setZN(cpu, u16::from((*cpu).db), true);
                break;
            }
            172 => {
                let mut low_116: uint32_t = 0 as uint32_t;
                let mut high_116: uint32_t = cpu_adrAbs(cpu, &raw mut low_116, game);
                cpu_ldy(cpu, low_116, high_116, game);
                break;
            }
            173 => {
                let mut low_117: uint32_t = 0 as uint32_t;
                let mut high_117: uint32_t = cpu_adrAbs(cpu, &raw mut low_117, game);
                cpu_lda(cpu, low_117, high_117, game);
                break;
            }
            174 => {
                let mut low_118: uint32_t = 0 as uint32_t;
                let mut high_118: uint32_t = cpu_adrAbs(cpu, &raw mut low_118, game);
                cpu_ldx(cpu, low_118, high_118, game);
                break;
            }
            175 => {
                let mut low_119: uint32_t = 0 as uint32_t;
                let mut high_119: uint32_t = cpu_adrAbl(cpu, &raw mut low_119, game);
                cpu_lda(cpu, low_119, high_119, game);
                break;
            }
            176 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).c);
                break;
            }
            177 => {
                let mut low_120: uint32_t = 0 as uint32_t;
                let mut high_120: uint32_t = cpu_adrIdy(cpu, &raw mut low_120, false_0 != 0, game);
                cpu_lda(cpu, low_120, high_120, game);
                break;
            }
            178 => {
                let mut low_121: uint32_t = 0 as uint32_t;
                let mut high_121: uint32_t = cpu_adrIdp(cpu, &raw mut low_121, game);
                cpu_lda(cpu, low_121, high_121, game);
                break;
            }
            179 => {
                let mut low_122: uint32_t = 0 as uint32_t;
                let mut high_122: uint32_t = cpu_adrIsy(cpu, &raw mut low_122, game);
                cpu_lda(cpu, low_122, high_122, game);
                break;
            }
            180 => {
                let mut low_123: uint32_t = 0 as uint32_t;
                let mut high_123: uint32_t = cpu_adrDpx(cpu, &raw mut low_123, game);
                cpu_ldy(cpu, low_123, high_123, game);
                break;
            }
            181 => {
                let mut low_124: uint32_t = 0 as uint32_t;
                let mut high_124: uint32_t = cpu_adrDpx(cpu, &raw mut low_124, game);
                cpu_lda(cpu, low_124, high_124, game);
                break;
            }
            182 => {
                let mut low_125: uint32_t = 0 as uint32_t;
                let mut high_125: uint32_t = cpu_adrDpy(cpu, &raw mut low_125, game);
                cpu_ldx(cpu, low_125, high_125, game);
                break;
            }
            183 => {
                let mut low_126: uint32_t = 0 as uint32_t;
                let mut high_126: uint32_t = cpu_adrIly(cpu, &raw mut low_126, game);
                cpu_lda(cpu, low_126, high_126, game);
                break;
            }
            184 => {
                (*cpu).v = false;
                break;
            }
            185 => {
                let mut low_127: uint32_t = 0 as uint32_t;
                let mut high_127: uint32_t = cpu_adrAby(cpu, &raw mut low_127, false_0 != 0, game);
                cpu_lda(cpu, low_127, high_127, game);
                break;
            }
            186 => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).sp & 0xff;
                } else {
                    (*cpu).x = (*cpu).sp;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            187 => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).y & 0xff;
                } else {
                    (*cpu).x = (*cpu).y;
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            188 => {
                let mut low_128: uint32_t = 0 as uint32_t;
                let mut high_128: uint32_t = cpu_adrAbx(cpu, &raw mut low_128, false_0 != 0, game);
                cpu_ldy(cpu, low_128, high_128, game);
                break;
            }
            189 => {
                let mut low_129: uint32_t = 0 as uint32_t;
                let mut high_129: uint32_t = cpu_adrAbx(cpu, &raw mut low_129, false_0 != 0, game);
                cpu_lda(cpu, low_129, high_129, game);
                break;
            }
            190 => {
                let mut low_130: uint32_t = 0 as uint32_t;
                let mut high_130: uint32_t = cpu_adrAby(cpu, &raw mut low_130, false_0 != 0, game);
                cpu_ldx(cpu, low_130, high_130, game);
                break;
            }
            191 => {
                let mut low_131: uint32_t = 0 as uint32_t;
                let mut high_131: uint32_t = cpu_adrAlx(cpu, &raw mut low_131, game);
                cpu_lda(cpu, low_131, high_131, game);
                break;
            }
            192 => {
                let mut low_132: uint32_t = 0 as uint32_t;
                let mut high_132: uint32_t = cpu_adrImm(cpu, &raw mut low_132, true_0 != 0);
                cpu_cpy(cpu, low_132, high_132, game);
                break;
            }
            193 => {
                let mut low_133: uint32_t = 0 as uint32_t;
                let mut high_133: uint32_t = cpu_adrIdx(cpu, &raw mut low_133, game);
                cpu_cmp(cpu, low_133, high_133, game);
                break;
            }
            194 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) & !cpu_readOpcode(cpu, game));
                break;
            }
            195 => {
                let mut low_134: uint32_t = 0 as uint32_t;
                let mut high_134: uint32_t = cpu_adrSr(cpu, &raw mut low_134, game);
                cpu_cmp(cpu, low_134, high_134, game);
                break;
            }
            196 => {
                let mut low_135: uint32_t = 0 as uint32_t;
                let mut high_135: uint32_t = cpu_adrDp(cpu, &raw mut low_135, game);
                cpu_cpy(cpu, low_135, high_135, game);
                break;
            }
            197 => {
                let mut low_136: uint32_t = 0 as uint32_t;
                let mut high_136: uint32_t = cpu_adrDp(cpu, &raw mut low_136, game);
                cpu_cmp(cpu, low_136, high_136, game);
                break;
            }
            198 => {
                let mut low_137: uint32_t = 0 as uint32_t;
                let mut high_137: uint32_t = cpu_adrDp(cpu, &raw mut low_137, game);
                cpu_dec(cpu, low_137, high_137, game);
                break;
            }
            199 => {
                let mut low_138: uint32_t = 0 as uint32_t;
                let mut high_138: uint32_t = cpu_adrIdl(cpu, &raw mut low_138, game);
                cpu_cmp(cpu, low_138, high_138, game);
                break;
            }
            200 => {
                if (*cpu).xf {
                    (*cpu).y = (*cpu).y.wrapping_add(1) & 0xff;
                } else {
                    (*cpu).y = (*cpu).y.wrapping_add(1);
                }
                cpu_setZN(cpu, (*cpu).y, (*cpu).xf);
                break;
            }
            201 => {
                let mut low_139: uint32_t = 0 as uint32_t;
                let mut high_139: uint32_t = cpu_adrImm(cpu, &raw mut low_139, false_0 != 0);
                cpu_cmp(cpu, low_139, high_139, game);
                break;
            }
            202 => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).x.wrapping_sub(1) & 0xff;
                } else {
                    (*cpu).x = (*cpu).x.wrapping_sub(1);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            203 => {
                (*cpu).waiting = true;
                break;
            }
            204 => {
                let mut low_140: uint32_t = 0 as uint32_t;
                let mut high_140: uint32_t = cpu_adrAbs(cpu, &raw mut low_140, game);
                cpu_cpy(cpu, low_140, high_140, game);
                break;
            }
            205 => {
                let mut low_141: uint32_t = 0 as uint32_t;
                let mut high_141: uint32_t = cpu_adrAbs(cpu, &raw mut low_141, game);
                cpu_cmp(cpu, low_141, high_141, game);
                break;
            }
            206 => {
                let mut low_142: uint32_t = 0 as uint32_t;
                let mut high_142: uint32_t = cpu_adrAbs(cpu, &raw mut low_142, game);
                cpu_dec(cpu, low_142, high_142, game);
                break;
            }
            207 => {
                let mut low_143: uint32_t = 0 as uint32_t;
                let mut high_143: uint32_t = cpu_adrAbl(cpu, &raw mut low_143, game);
                cpu_cmp(cpu, low_143, high_143, game);
                break;
            }
            208 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).z);
                break;
            }
            209 => {
                let mut low_144: uint32_t = 0 as uint32_t;
                let mut high_144: uint32_t = cpu_adrIdy(cpu, &raw mut low_144, false_0 != 0, game);
                cpu_cmp(cpu, low_144, high_144, game);
                break;
            }
            210 => {
                let mut low_145: uint32_t = 0 as uint32_t;
                let mut high_145: uint32_t = cpu_adrIdp(cpu, &raw mut low_145, game);
                cpu_cmp(cpu, low_145, high_145, game);
                break;
            }
            211 => {
                let mut low_146: uint32_t = 0 as uint32_t;
                let mut high_146: uint32_t = cpu_adrIsy(cpu, &raw mut low_146, game);
                cpu_cmp(cpu, low_146, high_146, game);
                break;
            }
            212 => {
                let mut low_147: uint32_t = 0 as uint32_t;
                let mut high_147: uint32_t = cpu_adrDp(cpu, &raw mut low_147, game);
                cpu_pushWord(cpu, cpu_readWord(cpu, low_147, high_147, game), game);
                break;
            }
            213 => {
                let mut low_148: uint32_t = 0 as uint32_t;
                let mut high_148: uint32_t = cpu_adrDpx(cpu, &raw mut low_148, game);
                cpu_cmp(cpu, low_148, high_148, game);
                break;
            }
            214 => {
                let mut low_149: uint32_t = 0 as uint32_t;
                let mut high_149: uint32_t = cpu_adrDpx(cpu, &raw mut low_149, game);
                cpu_dec(cpu, low_149, high_149, game);
                break;
            }
            215 => {
                let mut low_150: uint32_t = 0 as uint32_t;
                let mut high_150: uint32_t = cpu_adrIly(cpu, &raw mut low_150, game);
                cpu_cmp(cpu, low_150, high_150, game);
                break;
            }
            216 => {
                (*cpu).d = false;
                break;
            }
            217 => {
                let mut low_151: uint32_t = 0 as uint32_t;
                let mut high_151: uint32_t = cpu_adrAby(cpu, &raw mut low_151, false_0 != 0, game);
                cpu_cmp(cpu, low_151, high_151, game);
                break;
            }
            218 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).x as uint8_t, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).x, game);
                }
                break;
            }
            219 => {
                (*cpu).stopped = true_0 != 0;
                break;
            }
            220 => {
                let mut adr_0: uint16_t = cpu_readOpcodeWord(cpu, game);
                (*cpu).pc = cpu_readWord(
                    cpu,
                    adr_0 as uint32_t,
                    (adr_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                    game,
                );
                (*cpu).k = cpu_read(
                    cpu,
                    (adr_0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as uint32_t,
                    game,
                );
                break;
            }
            221 => {
                let mut low_152: uint32_t = 0 as uint32_t;
                let mut high_152: uint32_t = cpu_adrAbx(cpu, &raw mut low_152, false_0 != 0, game);
                cpu_cmp(cpu, low_152, high_152, game);
                break;
            }
            222 => {
                let mut low_153: uint32_t = 0 as uint32_t;
                let mut high_153: uint32_t = cpu_adrAbx(cpu, &raw mut low_153, true_0 != 0, game);
                cpu_dec(cpu, low_153, high_153, game);
                break;
            }
            223 => {
                let mut low_154: uint32_t = 0 as uint32_t;
                let mut high_154: uint32_t = cpu_adrAlx(cpu, &raw mut low_154, game);
                cpu_cmp(cpu, low_154, high_154, game);
                break;
            }
            224 => {
                let mut low_155: uint32_t = 0 as uint32_t;
                let mut high_155: uint32_t = cpu_adrImm(cpu, &raw mut low_155, true_0 != 0);
                cpu_cpx(cpu, low_155, high_155, game);
                break;
            }
            225 => {
                let mut low_156: uint32_t = 0 as uint32_t;
                let mut high_156: uint32_t = cpu_adrIdx(cpu, &raw mut low_156, game);
                cpu_sbc(cpu, low_156, high_156, game);
                break;
            }
            226 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) | cpu_readOpcode(cpu, game));
                break;
            }
            227 => {
                let mut low_157: uint32_t = 0 as uint32_t;
                let mut high_157: uint32_t = cpu_adrSr(cpu, &raw mut low_157, game);
                cpu_sbc(cpu, low_157, high_157, game);
                break;
            }
            228 => {
                let mut low_158: uint32_t = 0 as uint32_t;
                let mut high_158: uint32_t = cpu_adrDp(cpu, &raw mut low_158, game);
                cpu_cpx(cpu, low_158, high_158, game);
                break;
            }
            229 => {
                let mut low_159: uint32_t = 0 as uint32_t;
                let mut high_159: uint32_t = cpu_adrDp(cpu, &raw mut low_159, game);
                cpu_sbc(cpu, low_159, high_159, game);
                break;
            }
            230 => {
                let mut low_160: uint32_t = 0 as uint32_t;
                let mut high_160: uint32_t = cpu_adrDp(cpu, &raw mut low_160, game);
                cpu_inc(cpu, low_160, high_160, game);
                break;
            }
            231 => {
                let mut low_161: uint32_t = 0 as uint32_t;
                let mut high_161: uint32_t = cpu_adrIdl(cpu, &raw mut low_161, game);
                cpu_sbc(cpu, low_161, high_161, game);
                break;
            }
            232 => {
                if (*cpu).xf {
                    (*cpu).x = (*cpu).x.wrapping_add(1) & 0xff;
                } else {
                    (*cpu).x = (*cpu).x.wrapping_add(1);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            233 => {
                let mut low_162: uint32_t = 0 as uint32_t;
                let mut high_162: uint32_t = cpu_adrImm(cpu, &raw mut low_162, false_0 != 0);
                cpu_sbc(cpu, low_162, high_162, game);
                break;
            }
            235 => {
                (*cpu).a = (*cpu).a.swap_bytes();
                cpu_setZN(cpu, (*cpu).a, true);
                break;
            }
            236 => {
                let mut low_164: uint32_t = 0 as uint32_t;
                let mut high_164: uint32_t = cpu_adrAbs(cpu, &raw mut low_164, game);
                cpu_cpx(cpu, low_164, high_164, game);
                break;
            }
            237 => {
                let mut low_165: uint32_t = 0 as uint32_t;
                let mut high_165: uint32_t = cpu_adrAbs(cpu, &raw mut low_165, game);
                cpu_sbc(cpu, low_165, high_165, game);
                break;
            }
            238 => {
                let mut low_166: uint32_t = 0 as uint32_t;
                let mut high_166: uint32_t = cpu_adrAbs(cpu, &raw mut low_166, game);
                cpu_inc(cpu, low_166, high_166, game);
                break;
            }
            239 => {
                let mut low_167: uint32_t = 0 as uint32_t;
                let mut high_167: uint32_t = cpu_adrAbl(cpu, &raw mut low_167, game);
                cpu_sbc(cpu, low_167, high_167, game);
                break;
            }
            240 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).z);
                break;
            }
            241 => {
                let mut low_168: uint32_t = 0 as uint32_t;
                let mut high_168: uint32_t = cpu_adrIdy(cpu, &raw mut low_168, false_0 != 0, game);
                cpu_sbc(cpu, low_168, high_168, game);
                break;
            }
            242 => {
                let mut low_169: uint32_t = 0 as uint32_t;
                let mut high_169: uint32_t = cpu_adrIdp(cpu, &raw mut low_169, game);
                cpu_sbc(cpu, low_169, high_169, game);
                break;
            }
            243 => {
                let mut low_170: uint32_t = 0 as uint32_t;
                let mut high_170: uint32_t = cpu_adrIsy(cpu, &raw mut low_170, game);
                cpu_sbc(cpu, low_170, high_170, game);
                break;
            }
            244 => {
                let mut w: uint16_t = cpu_readOpcodeWord(cpu, game);
                if w as ::core::ffi::c_int == 0xc2ae as ::core::ffi::c_int {
                    (*cpu).y = 0 as uint16_t;
                }
                cpu_pushWord(cpu, w, game);
                break;
            }
            245 => {
                let mut low_171: uint32_t = 0 as uint32_t;
                let mut high_171: uint32_t = cpu_adrDpx(cpu, &raw mut low_171, game);
                cpu_sbc(cpu, low_171, high_171, game);
                break;
            }
            246 => {
                let mut low_172: uint32_t = 0 as uint32_t;
                let mut high_172: uint32_t = cpu_adrDpx(cpu, &raw mut low_172, game);
                cpu_inc(cpu, low_172, high_172, game);
                break;
            }
            247 => {
                let mut low_173: uint32_t = 0 as uint32_t;
                let mut high_173: uint32_t = cpu_adrIly(cpu, &raw mut low_173, game);
                cpu_sbc(cpu, low_173, high_173, game);
                break;
            }
            248 => {
                (*cpu).d = true;
                break;
            }
            249 => {
                let mut low_174: uint32_t = 0 as uint32_t;
                let mut high_174: uint32_t = cpu_adrAby(cpu, &raw mut low_174, false_0 != 0, game);
                cpu_sbc(cpu, low_174, high_174, game);
                break;
            }
            250 => {
                if (*cpu).xf {
                    (*cpu).x = u16::from(cpu_pullByte(cpu, game));
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    (*cpu).x = cpu_pullWord(cpu, game);
                }
                cpu_setZN(cpu, (*cpu).x, (*cpu).xf);
                break;
            }
            251 => {
                let temp = (*cpu).c;
                (*cpu).c = (*cpu).e;
                (*cpu).e = temp;
                cpu_setFlags(cpu, cpu_getFlags(cpu));
                break;
            }
            252 => {
                let mut value_3: uint16_t = cpu_adrIax(cpu, game);
                cpu_pushWord(
                    cpu,
                    ((*cpu).pc as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16_t,
                    game,
                );
                (*cpu).pc = value_3;
                break;
            }
            253 => {
                let mut low_175: uint32_t = 0 as uint32_t;
                let mut high_175: uint32_t = cpu_adrAbx(cpu, &raw mut low_175, false_0 != 0, game);
                cpu_sbc(cpu, low_175, high_175, game);
                break;
            }
            254 => {
                let mut low_176: uint32_t = 0 as uint32_t;
                let mut high_176: uint32_t = cpu_adrAbx(cpu, &raw mut low_176, true_0 != 0, game);
                cpu_inc(cpu, low_176, high_176, game);
                break;
            }
            255 => {
                let mut low_177: uint32_t = 0 as uint32_t;
                let mut high_177: uint32_t = cpu_adrAlx(cpu, &raw mut low_177, game);
                cpu_sbc(cpu, low_177, high_177, game);
                break;
            }
            234 | _ => {
                break;
            }
        }
    }
}
