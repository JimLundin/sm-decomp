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
static cyclesPerOpcode: [u8; 256] = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5, 2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5, 2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5, 2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
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
pub unsafe fn cpu_reset(cpu: *mut Cpu, game: &mut Game) {
    (*cpu).a = 0;
    (*cpu).x = 0;
    (*cpu).y = 0;
    (*cpu).sp = 0x100;
    (*cpu).pc = cpu_readWord(cpu, 0xfffc, 0xfffd, game);
    (*cpu).dp = 0;
    (*cpu).k = 0;
    (*cpu).db = 0;
    (*cpu).c = false;
    (*cpu).z = false;
    (*cpu).v = false;
    (*cpu).n = false;
    (*cpu).i = true;
    (*cpu).d = false;
    (*cpu).xf = true;
    (*cpu).mf = true;
    (*cpu).e = true;
    (*cpu).irqWanted = false;
    (*cpu).nmiWanted = false;
    (*cpu).waiting = false;
    (*cpu).stopped = false;
    (*cpu).cyclesUsed = 0;
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
            cpu_doInterrupt(cpu, false, game);
        } else {
            cpu_doInterrupt(cpu, true, game);
        }
    }
    let opcode = cpu_readOpcode(cpu, game);
    (*cpu).cyclesUsed = (*cpu)
        .cyclesUsed
        .wrapping_add(cyclesPerOpcode[usize::from(opcode)]);
    cpu_doOpcode(cpu, opcode, game);
    i32::from((*cpu).cyclesUsed)
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
unsafe fn cpu_doInterrupt(cpu: *mut Cpu, irq: bool, game: &mut Game) {
    if !(*cpu).e {
        cpu_pushByte(cpu, (*cpu).k, game);
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    cpu_pushWord(cpu, (*cpu).pc, game);
    let flags = cpu_getFlags(cpu);
    // Bit 4 is the cleared B flag on an emulation-mode hardware interrupt.
    cpu_pushByte(cpu, if (*cpu).e { flags & !0x10 } else { flags }, game);
    (*cpu).i = true;
    (*cpu).d = false;
    (*cpu).k = 0;
    let vector = match ((*cpu).e, irq) {
        (false, false) => 0xffea,
        (false, true) => 0xffee,
        (true, false) => 0xfffa,
        (true, true) => 0xfffe,
    };
    (*cpu).pc = cpu_readWord(cpu, vector, vector + 1, game);
}
unsafe fn address_immediate(cpu: *mut Cpu, xFlag: bool) -> (u32, u32) {
    let byte = if xFlag { (*cpu).xf } else { (*cpu).mf };
    let pc = (*cpu).pc;
    (*cpu).pc = (*cpu).pc.wrapping_add(1);
    let low = u32::from((*cpu).k) << 16 | u32::from(pc);
    let high = if byte {
        0
    } else {
        let pc = (*cpu).pc;
        (*cpu).pc = (*cpu).pc.wrapping_add(1);
        u32::from((*cpu).k) << 16 | u32::from(pc)
    };
    (low, high)
}
unsafe fn address_direct_page(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = u32::from((*cpu).dp.wrapping_add(adr));
    (low, u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)))
}
unsafe fn address_direct_page_x(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = u32::from((*cpu).dp.wrapping_add(adr).wrapping_add((*cpu).x));
    (
        low,
        u32::from(
            (*cpu)
                .dp
                .wrapping_add(adr)
                .wrapping_add((*cpu).x)
                .wrapping_add(1),
        ),
    )
}
unsafe fn address_direct_page_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = u32::from((*cpu).dp.wrapping_add(adr).wrapping_add((*cpu).y));
    (
        low,
        u32::from(
            (*cpu)
                .dp
                .wrapping_add(adr)
                .wrapping_add((*cpu).y)
                .wrapping_add(1),
        ),
    )
}
unsafe fn address_direct_page_indirect(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
    let low = (u32::from((*cpu).db) << 16) + pointer;
    (low, ((u32::from((*cpu).db) << 16) + pointer + 1) & 0xffffff)
}
unsafe fn address_direct_page_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
    let low = (u32::from((*cpu).db) << 16) + pointer;
    (low, ((u32::from((*cpu).db) << 16) + pointer + 1) & 0xffffff)
}
unsafe fn address_direct_page_indirect_y(
    cpu: *mut Cpu,
    write: bool,
    game: &mut Game,
) -> (u32, u32) {
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
    let low = ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y)) & 0xffffff;
    (
        low,
        ((u32::from((*cpu).db) << 16) + pointer + u32::from((*cpu).y) + 1) & 0xffffff,
    )
}
unsafe fn address_direct_page_indirect_long(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
    let low = pointer;
    (low, pointer.wrapping_add(1) & 0xffffff)
}
unsafe fn address_direct_page_indirect_long_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
    let low = pointer.wrapping_add(u32::from((*cpu).y)) & 0xffffff;
    (
        low,
        pointer.wrapping_add(u32::from((*cpu).y)).wrapping_add(1) & 0xffffff,
    )
}
unsafe fn address_stack_relative(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    let low = u32::from((*cpu).sp.wrapping_add(adr));
    (low, u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1)))
}
unsafe fn address_stack_relative_indirect_y(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    let pointer = u32::from(cpu_readWord(
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
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    let low = (u32::from((*cpu).db) << 16) + adr;
    (low, ((u32::from((*cpu).db) << 16) + adr + 1) & 0xffffff)
}
unsafe fn address_absolute_x(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    if write && (!(*cpu).xf || adr >> 8 != (adr + u32::from((*cpu).x)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).x)) & 0xffffff;
    (
        low,
        ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).x) + 1) & 0xffffff,
    )
}
unsafe fn address_absolute_y(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    if write && (!(*cpu).xf || adr >> 8 != (adr + u32::from((*cpu).y)) >> 8) {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).y)) & 0xffffff;
    (
        low,
        ((u32::from((*cpu).db) << 16) + adr + u32::from((*cpu).y) + 1) & 0xffffff,
    )
}
unsafe fn address_absolute_long(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    let low = adr;
    (low, adr.wrapping_add(1) & 0xffffff)
}
unsafe fn address_absolute_long_x(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    let low = adr.wrapping_add(u32::from((*cpu).x)) & 0xffffff;
    (
        low,
        adr.wrapping_add(u32::from((*cpu).x)).wrapping_add(1) & 0xffffff,
    )
}
unsafe fn address_absolute_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> uint16_t {
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
unsafe fn cpu_adc(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value: uint8_t = cpu_read(cpu, low, game);
        let mut result: i32 = 0;
        if (*cpu).d {
            result = (i32::from((*cpu).a) & 0xf) + (i32::from(value) & 0xf) + i32::from((*cpu).c);
            if result > 0x9 {
                result = (result + 0x6 & 0xf) + 0x10;
            }
            result = (i32::from((*cpu).a) & 0xf0) + (i32::from(value) & 0xf0) + result;
        } else {
            result = (i32::from((*cpu).a) & 0xff) + i32::from(value) + i32::from((*cpu).c);
        }
        (*cpu).v = i32::from((*cpu).a) & 0x80 == i32::from(value) & 0x80
            && i32::from(value) & 0x80 != result & 0x80;
        if (*cpu).d && result > 0x9f {
            result += 0x60;
        }
        (*cpu).c = result > 0xff;
        (*cpu).a = (i32::from((*cpu).a) & 0xff00 | result & 0xff) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value_0: uint16_t = cpu_readWord(cpu, low, high, game);
        let mut result_0: i32 = 0;
        if (*cpu).d {
            result_0 =
                (i32::from((*cpu).a) & 0xf) + (i32::from(value_0) & 0xf) + i32::from((*cpu).c);
            if result_0 > 0x9 {
                result_0 = (result_0 + 0x6 & 0xf) + 0x10;
            }
            result_0 = (i32::from((*cpu).a) & 0xf0) + (i32::from(value_0) & 0xf0) + result_0;
            if result_0 > 0x9f {
                result_0 = (result_0 + 0x60 & 0xff) + 0x100;
            }
            result_0 = (i32::from((*cpu).a) & 0xf00) + (i32::from(value_0) & 0xf00) + result_0;
            if result_0 > 0x9ff {
                result_0 = (result_0 + 0x600 & 0xfff) + 0x1000;
            }
            result_0 = (i32::from((*cpu).a) & 0xf000) + (i32::from(value_0) & 0xf000) + result_0;
        } else {
            result_0 = i32::from((*cpu).a) + i32::from(value_0) + i32::from((*cpu).c);
        }
        (*cpu).v = i32::from((*cpu).a) & 0x8000 == i32::from(value_0) & 0x8000
            && i32::from(value_0) & 0x8000 != result_0 & 0x8000;
        if (*cpu).d && result_0 > 0x9fff {
            result_0 += 0x6000;
        }
        (*cpu).c = result_0 > 0xffff;
        (*cpu).a = result_0 as uint16_t;
    }
    cpu_setZN(cpu, (*cpu).a, (*cpu).mf);
}
unsafe fn cpu_sbc(cpu: *mut Cpu, low: uint32_t, high: uint32_t, game: &mut Game) {
    if (*cpu).mf {
        let value = !cpu_read(cpu, low, game);
        let mut result: i32 = 0;
        if (*cpu).d {
            result = (i32::from((*cpu).a) & 0xf) + (i32::from(value) & 0xf) + i32::from((*cpu).c);
            if result < 0x10 {
                result = result - 0x6 & (if (result - 0x6) < 0 { 0xf } else { 0x1f });
            }
            result = (i32::from((*cpu).a) & 0xf0) + (i32::from(value) & 0xf0) + result;
        } else {
            result = (i32::from((*cpu).a) & 0xff) + i32::from(value) + i32::from((*cpu).c);
        }
        (*cpu).v = i32::from((*cpu).a) & 0x80 == i32::from(value) & 0x80
            && i32::from(value) & 0x80 != result & 0x80;
        if (*cpu).d && result < 0x100 {
            result -= 0x60;
        }
        (*cpu).c = result > 0xff;
        (*cpu).a = (i32::from((*cpu).a) & 0xff00 | result & 0xff) as uint16_t;
    } else {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
        let value_0 = !cpu_readWord(cpu, low, high, game);
        let mut result_0: i32 = 0;
        if (*cpu).d {
            result_0 =
                (i32::from((*cpu).a) & 0xf) + (i32::from(value_0) & 0xf) + i32::from((*cpu).c);
            if result_0 < 0x10 {
                result_0 = result_0 - 0x6 & (if (result_0 - 0x6) < 0 { 0xf } else { 0x1f });
            }
            result_0 = (i32::from((*cpu).a) & 0xf0) + (i32::from(value_0) & 0xf0) + result_0;
            if result_0 < 0x100 {
                result_0 = result_0 - 0x60 & (if (result_0 - 0x60) < 0 { 0xff } else { 0x1ff });
            }
            result_0 = (i32::from((*cpu).a) & 0xf00) + (i32::from(value_0) & 0xf00) + result_0;
            if result_0 < 0x1000 {
                result_0 = result_0 - 0x600
                    & (if (result_0 - 0x600) < 0 {
                        0xfff
                    } else {
                        0x1fff
                    });
            }
            result_0 = (i32::from((*cpu).a) & 0xf000) + (i32::from(value_0) & 0xf000) + result_0;
        } else {
            result_0 = i32::from((*cpu).a) + i32::from(value_0) + i32::from((*cpu).c);
        }
        (*cpu).v = i32::from((*cpu).a) & 0x8000 == i32::from(value_0) & 0x8000
            && i32::from(value_0) & 0x8000 != result_0 & 0x8000;
        if (*cpu).d && result_0 < 0x10000 {
            result_0 -= 0x6000;
        }
        (*cpu).c = result_0 > 0xffff;
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
unsafe fn cpu_doOpcode(cpu: *mut Cpu, mut opcode: uint8_t, game: &mut Game) {
    // Preserve the inherited history value of 0xffffffff when PC is zero.
    let cur_pc = (i32::from((*cpu).k) << 16 | (i32::from((*cpu).pc) - 1)) as u32;
    game.pc_hist[game.pc_hist_ctr as usize] = cur_pc;
    game.pc_hist_ctr = game.pc_hist_ctr.wrapping_add(1) & 7;
    loop {
        match opcode {
            0 => {
                let mut addr: uint32_t =
                    (((*cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*cpu).pc as ::core::ffi::c_int
                        | 0x800000 as ::core::ffi::c_int) as uint32_t;
                opcode = CpuOpcodeHook((addr as uint32).wrapping_sub(1 as uint32), game) as uint8_t;
                match opcode {
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
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            2 => {
                cpu_readOpcode(cpu, game);
                cpu_pushByte(cpu, (*cpu).k, game);
                cpu_pushWord(cpu, (*cpu).pc, game);
                cpu_pushByte(cpu, cpu_getFlags(cpu), game);
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).i = true;
                (*cpu).d = false;
                (*cpu).k = 0 as uint8_t;
                (*cpu).pc = cpu_readWord(cpu, 0xffe4 as uint32_t, 0xffe5 as uint32_t, game);
                break;
            }
            3 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            4 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_tsb(cpu, low, high, game);
                break;
            }
            5 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            6 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_asl(cpu, low, high, game);
                break;
            }
            7 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            8 => {
                cpu_pushByte(cpu, cpu_getFlags(cpu), game);
                break;
            }
            9 => {
                let (low, high) = address_immediate(cpu, false);
                cpu_ora(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_tsb(cpu, low, high, game);
                break;
            }
            13 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            14 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_asl(cpu, low, high, game);
                break;
            }
            15 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            16 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).n);
                break;
            }
            17 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            18 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            19 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            20 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_trb(cpu, low, high, game);
                break;
            }
            21 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            22 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_asl(cpu, low, high, game);
                break;
            }
            23 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            24 => {
                (*cpu).c = false;
                break;
            }
            25 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_ora(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_trb(cpu, low, high, game);
                break;
            }
            29 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            30 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_asl(cpu, low, high, game);
                break;
            }
            31 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_ora(cpu, low, high, game);
                break;
            }
            32 => {
                let value = cpu_readOpcodeWord(cpu, game);
                cpu_pushWord(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value;
                break;
            }
            33 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            34 => {
                let value_0 = cpu_readOpcodeWord(cpu, game);
                let newK = cpu_readOpcode(cpu, game);
                cpu_pushByte(cpu, (*cpu).k, game);
                cpu_pushWord(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value_0;
                (*cpu).k = newK;
                break;
            }
            35 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            36 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_bit(cpu, low, high, game);
                break;
            }
            37 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            38 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_rol(cpu, low, high, game);
                break;
            }
            39 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            40 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu, game));
                break;
            }
            41 => {
                let (low, high) = address_immediate(cpu, false);
                cpu_and(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_bit(cpu, low, high, game);
                break;
            }
            45 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            46 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_rol(cpu, low, high, game);
                break;
            }
            47 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            48 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).n);
                break;
            }
            49 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            50 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            51 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            52 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_bit(cpu, low, high, game);
                break;
            }
            53 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            54 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_rol(cpu, low, high, game);
                break;
            }
            55 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            56 => {
                (*cpu).c = true;
                break;
            }
            57 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_and(cpu, low, high, game);
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
                cpu_setZN(cpu, (*cpu).a, false);
                break;
            }
            60 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_bit(cpu, low, high, game);
                break;
            }
            61 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_and(cpu, low, high, game);
                break;
            }
            62 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_rol(cpu, low, high, game);
                break;
            }
            63 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_and(cpu, low, high, game);
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
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            66 => {
                cpu_readOpcode(cpu, game);
                break;
            }
            67 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            68 => {
                let dest = cpu_readOpcode(cpu, game);
                let src = cpu_readOpcode(cpu, game);
                (*cpu).db = dest;
                cpu_write(
                    cpu,
                    u32::from(dest) << 16 | u32::from((*cpu).y),
                    cpu_read(cpu, u32::from(src) << 16 | u32::from((*cpu).x), game),
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
            69 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            70 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_lsr(cpu, low, high, game);
                break;
            }
            71 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            72 => {
                if (*cpu).mf {
                    cpu_pushByte(cpu, (*cpu).a as u8, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).a, game);
                }
                break;
            }
            73 => {
                let (low, high) = address_immediate(cpu, false);
                cpu_eor(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            78 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_lsr(cpu, low, high, game);
                break;
            }
            79 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            80 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).v);
                break;
            }
            81 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            82 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            83 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            84 => {
                let dest_0 = cpu_readOpcode(cpu, game);
                let src_0 = cpu_readOpcode(cpu, game);
                (*cpu).db = dest_0;
                cpu_write(
                    cpu,
                    u32::from(dest_0) << 16 | u32::from((*cpu).y),
                    cpu_read(cpu, u32::from(src_0) << 16 | u32::from((*cpu).x), game),
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
            85 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            86 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_lsr(cpu, low, high, game);
                break;
            }
            87 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            88 => {
                (*cpu).i = false;
                break;
            }
            89 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            90 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).y as u8, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).y, game);
                }
                break;
            }
            91 => {
                (*cpu).dp = (*cpu).a;
                cpu_setZN(cpu, (*cpu).dp, false);
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
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_eor(cpu, low, high, game);
                break;
            }
            94 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_lsr(cpu, low, high, game);
                break;
            }
            95 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_eor(cpu, low, high, game);
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
                (*cpu).pc = cpu_pullWord(cpu, game).wrapping_add(1);
                break;
            }
            97 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            98 => {
                let displacement = cpu_readOpcodeWord(cpu, game) as i16;
                cpu_pushWord(cpu, (*cpu).pc.wrapping_add_signed(displacement), game);
                break;
            }
            99 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            100 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_stz(cpu, low, high, game);
                break;
            }
            101 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            102 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_ror(cpu, low, high, game);
                break;
            }
            103 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_adc(cpu, low, high, game);
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
                let (low, high) = address_immediate(cpu, false);
                cpu_adc(cpu, low, high, game);
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
                (*cpu).pc = cpu_pullWord(cpu, game).wrapping_add(1);
                (*cpu).k = cpu_pullByte(cpu, game);
                break;
            }
            108 => {
                let adr = cpu_readOpcodeWord(cpu, game);
                (*cpu).pc = cpu_readWord(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
                break;
            }
            109 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            110 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_ror(cpu, low, high, game);
                break;
            }
            111 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            112 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).v);
                break;
            }
            113 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            114 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            115 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            116 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_stz(cpu, low, high, game);
                break;
            }
            117 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            118 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_ror(cpu, low, high, game);
                break;
            }
            119 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            120 => {
                (*cpu).i = true;
                break;
            }
            121 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_adc(cpu, low, high, game);
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
                cpu_setZN(cpu, (*cpu).a, false);
                break;
            }
            124 => {
                (*cpu).pc = address_absolute_indexed_indirect(cpu, game);
                break;
            }
            125 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            126 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_ror(cpu, low, high, game);
                break;
            }
            127 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_adc(cpu, low, high, game);
                break;
            }
            128 => {
                // Relative branches start after the displacement operand.
                let displacement = cpu_readOpcode(cpu, game) as int8_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement as i16);
                break;
            }
            129 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            130 => {
                let displacement = cpu_readOpcodeWord(cpu, game) as int16_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
                break;
            }
            131 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            132 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_sty(cpu, low, high, game);
                break;
            }
            133 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            134 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_stx(cpu, low, high, game);
                break;
            }
            135 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_sta(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_sty(cpu, low, high, game);
                break;
            }
            141 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            142 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_stx(cpu, low, high, game);
                break;
            }
            143 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            144 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).c);
                break;
            }
            145 => {
                let (low, high) = address_direct_page_indirect_y(cpu, true, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            146 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            147 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            148 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_sty(cpu, low, high, game);
                break;
            }
            149 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            150 => {
                let (low, high) = address_direct_page_y(cpu, game);
                cpu_stx(cpu, low, high, game);
                break;
            }
            151 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_sta(cpu, low, high, game);
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
                let (low, high) = address_absolute_y(cpu, true, game);
                cpu_sta(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_stz(cpu, low, high, game);
                break;
            }
            157 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            158 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_stz(cpu, low, high, game);
                break;
            }
            159 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_sta(cpu, low, high, game);
                break;
            }
            160 => {
                let (low, high) = address_immediate(cpu, true);
                cpu_ldy(cpu, low, high, game);
                break;
            }
            161 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            162 => {
                let (low, high) = address_immediate(cpu, true);
                cpu_ldx(cpu, low, high, game);
                break;
            }
            163 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            164 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_ldy(cpu, low, high, game);
                break;
            }
            165 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            166 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_ldx(cpu, low, high, game);
                break;
            }
            167 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_lda(cpu, low, high, game);
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
                let (low, high) = address_immediate(cpu, false);
                cpu_lda(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_ldy(cpu, low, high, game);
                break;
            }
            173 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            174 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_ldx(cpu, low, high, game);
                break;
            }
            175 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            176 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).c);
                break;
            }
            177 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            178 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            179 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            180 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_ldy(cpu, low, high, game);
                break;
            }
            181 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            182 => {
                let (low, high) = address_direct_page_y(cpu, game);
                cpu_ldx(cpu, low, high, game);
                break;
            }
            183 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            184 => {
                (*cpu).v = false;
                break;
            }
            185 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_lda(cpu, low, high, game);
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
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_ldy(cpu, low, high, game);
                break;
            }
            189 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            190 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_ldx(cpu, low, high, game);
                break;
            }
            191 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_lda(cpu, low, high, game);
                break;
            }
            192 => {
                let (low, high) = address_immediate(cpu, true);
                cpu_cpy(cpu, low, high, game);
                break;
            }
            193 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            194 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) & !cpu_readOpcode(cpu, game));
                break;
            }
            195 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            196 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_cpy(cpu, low, high, game);
                break;
            }
            197 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            198 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_dec(cpu, low, high, game);
                break;
            }
            199 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_cmp(cpu, low, high, game);
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
                let (low, high) = address_immediate(cpu, false);
                cpu_cmp(cpu, low, high, game);
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
                let (low, high) = address_absolute(cpu, game);
                cpu_cpy(cpu, low, high, game);
                break;
            }
            205 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            206 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_dec(cpu, low, high, game);
                break;
            }
            207 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            208 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).z);
                break;
            }
            209 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            210 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            211 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            212 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_pushWord(cpu, cpu_readWord(cpu, low, high, game), game);
                break;
            }
            213 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            214 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_dec(cpu, low, high, game);
                break;
            }
            215 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            216 => {
                (*cpu).d = false;
                break;
            }
            217 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            218 => {
                if (*cpu).xf {
                    cpu_pushByte(cpu, (*cpu).x as u8, game);
                } else {
                    (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                    cpu_pushWord(cpu, (*cpu).x, game);
                }
                break;
            }
            219 => {
                (*cpu).stopped = true;
                break;
            }
            220 => {
                let adr = cpu_readOpcodeWord(cpu, game);
                (*cpu).pc = cpu_readWord(cpu, u32::from(adr), u32::from(adr.wrapping_add(1)), game);
                (*cpu).k = cpu_read(cpu, u32::from(adr.wrapping_add(2)), game);
                break;
            }
            221 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            222 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_dec(cpu, low, high, game);
                break;
            }
            223 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_cmp(cpu, low, high, game);
                break;
            }
            224 => {
                let (low, high) = address_immediate(cpu, true);
                cpu_cpx(cpu, low, high, game);
                break;
            }
            225 => {
                let (low, high) = address_direct_page_indexed_indirect(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            226 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) | cpu_readOpcode(cpu, game));
                break;
            }
            227 => {
                let (low, high) = address_stack_relative(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            228 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_cpx(cpu, low, high, game);
                break;
            }
            229 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            230 => {
                let (low, high) = address_direct_page(cpu, game);
                cpu_inc(cpu, low, high, game);
                break;
            }
            231 => {
                let (low, high) = address_direct_page_indirect_long(cpu, game);
                cpu_sbc(cpu, low, high, game);
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
                let (low, high) = address_immediate(cpu, false);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            235 => {
                (*cpu).a = (*cpu).a.swap_bytes();
                cpu_setZN(cpu, (*cpu).a, true);
                break;
            }
            236 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_cpx(cpu, low, high, game);
                break;
            }
            237 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            238 => {
                let (low, high) = address_absolute(cpu, game);
                cpu_inc(cpu, low, high, game);
                break;
            }
            239 => {
                let (low, high) = address_absolute_long(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            240 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).z);
                break;
            }
            241 => {
                let (low, high) = address_direct_page_indirect_y(cpu, false, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            242 => {
                let (low, high) = address_direct_page_indirect(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            243 => {
                let (low, high) = address_stack_relative_indirect_y(cpu, game);
                cpu_sbc(cpu, low, high, game);
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
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            246 => {
                let (low, high) = address_direct_page_x(cpu, game);
                cpu_inc(cpu, low, high, game);
                break;
            }
            247 => {
                let (low, high) = address_direct_page_indirect_long_y(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            248 => {
                (*cpu).d = true;
                break;
            }
            249 => {
                let (low, high) = address_absolute_y(cpu, false, game);
                cpu_sbc(cpu, low, high, game);
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
                let value_3 = address_absolute_indexed_indirect(cpu, game);
                cpu_pushWord(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value_3;
                break;
            }
            253 => {
                let (low, high) = address_absolute_x(cpu, false, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            254 => {
                let (low, high) = address_absolute_x(cpu, true, game);
                cpu_inc(cpu, low, high, game);
                break;
            }
            255 => {
                let (low, high) = address_absolute_long_x(cpu, game);
                cpu_sbc(cpu, low, high, game);
                break;
            }
            234 | _ => {
                break;
            }
        }
    }
}
