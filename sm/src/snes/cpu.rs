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
unsafe fn cpu_adrImm(cpu: *mut Cpu, xFlag: bool) -> (u32, u32) {
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
unsafe fn cpu_adrDp(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    if (*cpu).dp & 0xff != 0 {
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    let low = u32::from((*cpu).dp.wrapping_add(adr));
    (low, u32::from((*cpu).dp.wrapping_add(adr).wrapping_add(1)))
}
unsafe fn cpu_adrDpx(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrDpy(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrIdp(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrIdx(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrIdy(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrIdl(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrIly(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrSr(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u16::from(cpu_readOpcode(cpu, game));
    let low = u32::from((*cpu).sp.wrapping_add(adr));
    (low, u32::from((*cpu).sp.wrapping_add(adr).wrapping_add(1)))
}
unsafe fn cpu_adrIsy(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrAbs(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let adr = u32::from(cpu_readOpcodeWord(cpu, game));
    let low = (u32::from((*cpu).db) << 16) + adr;
    (low, ((u32::from((*cpu).db) << 16) + adr + 1) & 0xffffff)
}
unsafe fn cpu_adrAbx(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrAby(cpu: *mut Cpu, write: bool, game: &mut Game) -> (u32, u32) {
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
unsafe fn cpu_adrAbl(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    let low = adr;
    (low, adr.wrapping_add(1) & 0xffffff)
}
unsafe fn cpu_adrAlx(cpu: *mut Cpu, game: &mut Game) -> (u32, u32) {
    let mut adr = u32::from(cpu_readOpcodeWord(cpu, game));
    adr |= u32::from(cpu_readOpcode(cpu, game)) << 16;
    let low = adr.wrapping_add(u32::from((*cpu).x)) & 0xffffff;
    (
        low,
        adr.wrapping_add(u32::from((*cpu).x)).wrapping_add(1) & 0xffffff,
    )
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
                let (low, high) = cpu_adrIdx(cpu, game);
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
                let (low_0, high_0) = cpu_adrSr(cpu, game);
                cpu_ora(cpu, low_0, high_0, game);
                break;
            }
            4 => {
                let (low_1, high_1) = cpu_adrDp(cpu, game);
                cpu_tsb(cpu, low_1, high_1, game);
                break;
            }
            5 => {
                let (low_2, high_2) = cpu_adrDp(cpu, game);
                cpu_ora(cpu, low_2, high_2, game);
                break;
            }
            6 => {
                let (low_3, high_3) = cpu_adrDp(cpu, game);
                cpu_asl(cpu, low_3, high_3, game);
                break;
            }
            7 => {
                let (low_4, high_4) = cpu_adrIdl(cpu, game);
                cpu_ora(cpu, low_4, high_4, game);
                break;
            }
            8 => {
                cpu_pushByte(cpu, cpu_getFlags(cpu), game);
                break;
            }
            9 => {
                let (low_5, high_5) = cpu_adrImm(cpu, false);
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
                let (low_6, high_6) = cpu_adrAbs(cpu, game);
                cpu_tsb(cpu, low_6, high_6, game);
                break;
            }
            13 => {
                let (low_7, high_7) = cpu_adrAbs(cpu, game);
                cpu_ora(cpu, low_7, high_7, game);
                break;
            }
            14 => {
                let (low_8, high_8) = cpu_adrAbs(cpu, game);
                cpu_asl(cpu, low_8, high_8, game);
                break;
            }
            15 => {
                let (low_9, high_9) = cpu_adrAbl(cpu, game);
                cpu_ora(cpu, low_9, high_9, game);
                break;
            }
            16 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).n);
                break;
            }
            17 => {
                let (low_10, high_10) = cpu_adrIdy(cpu, false, game);
                cpu_ora(cpu, low_10, high_10, game);
                break;
            }
            18 => {
                let (low_11, high_11) = cpu_adrIdp(cpu, game);
                cpu_ora(cpu, low_11, high_11, game);
                break;
            }
            19 => {
                let (low_12, high_12) = cpu_adrIsy(cpu, game);
                cpu_ora(cpu, low_12, high_12, game);
                break;
            }
            20 => {
                let (low_13, high_13) = cpu_adrDp(cpu, game);
                cpu_trb(cpu, low_13, high_13, game);
                break;
            }
            21 => {
                let (low_14, high_14) = cpu_adrDpx(cpu, game);
                cpu_ora(cpu, low_14, high_14, game);
                break;
            }
            22 => {
                let (low_15, high_15) = cpu_adrDpx(cpu, game);
                cpu_asl(cpu, low_15, high_15, game);
                break;
            }
            23 => {
                let (low_16, high_16) = cpu_adrIly(cpu, game);
                cpu_ora(cpu, low_16, high_16, game);
                break;
            }
            24 => {
                (*cpu).c = false;
                break;
            }
            25 => {
                let (low_17, high_17) = cpu_adrAby(cpu, false, game);
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
                let (low_18, high_18) = cpu_adrAbs(cpu, game);
                cpu_trb(cpu, low_18, high_18, game);
                break;
            }
            29 => {
                let (low_19, high_19) = cpu_adrAbx(cpu, false, game);
                cpu_ora(cpu, low_19, high_19, game);
                break;
            }
            30 => {
                let (low_20, high_20) = cpu_adrAbx(cpu, true, game);
                cpu_asl(cpu, low_20, high_20, game);
                break;
            }
            31 => {
                let (low_21, high_21) = cpu_adrAlx(cpu, game);
                cpu_ora(cpu, low_21, high_21, game);
                break;
            }
            32 => {
                let value = cpu_readOpcodeWord(cpu, game);
                cpu_pushWord(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value;
                break;
            }
            33 => {
                let (low_22, high_22) = cpu_adrIdx(cpu, game);
                cpu_and(cpu, low_22, high_22, game);
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
                let (low_23, high_23) = cpu_adrSr(cpu, game);
                cpu_and(cpu, low_23, high_23, game);
                break;
            }
            36 => {
                let (low_24, high_24) = cpu_adrDp(cpu, game);
                cpu_bit(cpu, low_24, high_24, game);
                break;
            }
            37 => {
                let (low_25, high_25) = cpu_adrDp(cpu, game);
                cpu_and(cpu, low_25, high_25, game);
                break;
            }
            38 => {
                let (low_26, high_26) = cpu_adrDp(cpu, game);
                cpu_rol(cpu, low_26, high_26, game);
                break;
            }
            39 => {
                let (low_27, high_27) = cpu_adrIdl(cpu, game);
                cpu_and(cpu, low_27, high_27, game);
                break;
            }
            40 => {
                cpu_setFlags(cpu, cpu_pullByte(cpu, game));
                break;
            }
            41 => {
                let (low_28, high_28) = cpu_adrImm(cpu, false);
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
                let (low_29, high_29) = cpu_adrAbs(cpu, game);
                cpu_bit(cpu, low_29, high_29, game);
                break;
            }
            45 => {
                let (low_30, high_30) = cpu_adrAbs(cpu, game);
                cpu_and(cpu, low_30, high_30, game);
                break;
            }
            46 => {
                let (low_31, high_31) = cpu_adrAbs(cpu, game);
                cpu_rol(cpu, low_31, high_31, game);
                break;
            }
            47 => {
                let (low_32, high_32) = cpu_adrAbl(cpu, game);
                cpu_and(cpu, low_32, high_32, game);
                break;
            }
            48 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).n);
                break;
            }
            49 => {
                let (low_33, high_33) = cpu_adrIdy(cpu, false, game);
                cpu_and(cpu, low_33, high_33, game);
                break;
            }
            50 => {
                let (low_34, high_34) = cpu_adrIdp(cpu, game);
                cpu_and(cpu, low_34, high_34, game);
                break;
            }
            51 => {
                let (low_35, high_35) = cpu_adrIsy(cpu, game);
                cpu_and(cpu, low_35, high_35, game);
                break;
            }
            52 => {
                let (low_36, high_36) = cpu_adrDpx(cpu, game);
                cpu_bit(cpu, low_36, high_36, game);
                break;
            }
            53 => {
                let (low_37, high_37) = cpu_adrDpx(cpu, game);
                cpu_and(cpu, low_37, high_37, game);
                break;
            }
            54 => {
                let (low_38, high_38) = cpu_adrDpx(cpu, game);
                cpu_rol(cpu, low_38, high_38, game);
                break;
            }
            55 => {
                let (low_39, high_39) = cpu_adrIly(cpu, game);
                cpu_and(cpu, low_39, high_39, game);
                break;
            }
            56 => {
                (*cpu).c = true;
                break;
            }
            57 => {
                let (low_40, high_40) = cpu_adrAby(cpu, false, game);
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
                cpu_setZN(cpu, (*cpu).a, false);
                break;
            }
            60 => {
                let (low_41, high_41) = cpu_adrAbx(cpu, false, game);
                cpu_bit(cpu, low_41, high_41, game);
                break;
            }
            61 => {
                let (low_42, high_42) = cpu_adrAbx(cpu, false, game);
                cpu_and(cpu, low_42, high_42, game);
                break;
            }
            62 => {
                let (low_43, high_43) = cpu_adrAbx(cpu, true, game);
                cpu_rol(cpu, low_43, high_43, game);
                break;
            }
            63 => {
                let (low_44, high_44) = cpu_adrAlx(cpu, game);
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
                let (low_45, high_45) = cpu_adrIdx(cpu, game);
                cpu_eor(cpu, low_45, high_45, game);
                break;
            }
            66 => {
                cpu_readOpcode(cpu, game);
                break;
            }
            67 => {
                let (low_46, high_46) = cpu_adrSr(cpu, game);
                cpu_eor(cpu, low_46, high_46, game);
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
                let (low_47, high_47) = cpu_adrDp(cpu, game);
                cpu_eor(cpu, low_47, high_47, game);
                break;
            }
            70 => {
                let (low_48, high_48) = cpu_adrDp(cpu, game);
                cpu_lsr(cpu, low_48, high_48, game);
                break;
            }
            71 => {
                let (low_49, high_49) = cpu_adrIdl(cpu, game);
                cpu_eor(cpu, low_49, high_49, game);
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
                let (low_50, high_50) = cpu_adrImm(cpu, false);
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
                let (low_51, high_51) = cpu_adrAbs(cpu, game);
                cpu_eor(cpu, low_51, high_51, game);
                break;
            }
            78 => {
                let (low_52, high_52) = cpu_adrAbs(cpu, game);
                cpu_lsr(cpu, low_52, high_52, game);
                break;
            }
            79 => {
                let (low_53, high_53) = cpu_adrAbl(cpu, game);
                cpu_eor(cpu, low_53, high_53, game);
                break;
            }
            80 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).v);
                break;
            }
            81 => {
                let (low_54, high_54) = cpu_adrIdy(cpu, false, game);
                cpu_eor(cpu, low_54, high_54, game);
                break;
            }
            82 => {
                let (low_55, high_55) = cpu_adrIdp(cpu, game);
                cpu_eor(cpu, low_55, high_55, game);
                break;
            }
            83 => {
                let (low_56, high_56) = cpu_adrIsy(cpu, game);
                cpu_eor(cpu, low_56, high_56, game);
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
                let (low_57, high_57) = cpu_adrDpx(cpu, game);
                cpu_eor(cpu, low_57, high_57, game);
                break;
            }
            86 => {
                let (low_58, high_58) = cpu_adrDpx(cpu, game);
                cpu_lsr(cpu, low_58, high_58, game);
                break;
            }
            87 => {
                let (low_59, high_59) = cpu_adrIly(cpu, game);
                cpu_eor(cpu, low_59, high_59, game);
                break;
            }
            88 => {
                (*cpu).i = false;
                break;
            }
            89 => {
                let (low_60, high_60) = cpu_adrAby(cpu, false, game);
                cpu_eor(cpu, low_60, high_60, game);
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
                let (low_61, high_61) = cpu_adrAbx(cpu, false, game);
                cpu_eor(cpu, low_61, high_61, game);
                break;
            }
            94 => {
                let (low_62, high_62) = cpu_adrAbx(cpu, true, game);
                cpu_lsr(cpu, low_62, high_62, game);
                break;
            }
            95 => {
                let (low_63, high_63) = cpu_adrAlx(cpu, game);
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
                (*cpu).pc = cpu_pullWord(cpu, game).wrapping_add(1);
                break;
            }
            97 => {
                let (low_64, high_64) = cpu_adrIdx(cpu, game);
                cpu_adc(cpu, low_64, high_64, game);
                break;
            }
            98 => {
                let displacement = cpu_readOpcodeWord(cpu, game) as i16;
                cpu_pushWord(cpu, (*cpu).pc.wrapping_add_signed(displacement), game);
                break;
            }
            99 => {
                let (low_65, high_65) = cpu_adrSr(cpu, game);
                cpu_adc(cpu, low_65, high_65, game);
                break;
            }
            100 => {
                let (low_66, high_66) = cpu_adrDp(cpu, game);
                cpu_stz(cpu, low_66, high_66, game);
                break;
            }
            101 => {
                let (low_67, high_67) = cpu_adrDp(cpu, game);
                cpu_adc(cpu, low_67, high_67, game);
                break;
            }
            102 => {
                let (low_68, high_68) = cpu_adrDp(cpu, game);
                cpu_ror(cpu, low_68, high_68, game);
                break;
            }
            103 => {
                let (low_69, high_69) = cpu_adrIdl(cpu, game);
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
                let (low_70, high_70) = cpu_adrImm(cpu, false);
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
                let (low_71, high_71) = cpu_adrAbs(cpu, game);
                cpu_adc(cpu, low_71, high_71, game);
                break;
            }
            110 => {
                let (low_72, high_72) = cpu_adrAbs(cpu, game);
                cpu_ror(cpu, low_72, high_72, game);
                break;
            }
            111 => {
                let (low_73, high_73) = cpu_adrAbl(cpu, game);
                cpu_adc(cpu, low_73, high_73, game);
                break;
            }
            112 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).v);
                break;
            }
            113 => {
                let (low_74, high_74) = cpu_adrIdy(cpu, false, game);
                cpu_adc(cpu, low_74, high_74, game);
                break;
            }
            114 => {
                let (low_75, high_75) = cpu_adrIdp(cpu, game);
                cpu_adc(cpu, low_75, high_75, game);
                break;
            }
            115 => {
                let (low_76, high_76) = cpu_adrIsy(cpu, game);
                cpu_adc(cpu, low_76, high_76, game);
                break;
            }
            116 => {
                let (low_77, high_77) = cpu_adrDpx(cpu, game);
                cpu_stz(cpu, low_77, high_77, game);
                break;
            }
            117 => {
                let (low_78, high_78) = cpu_adrDpx(cpu, game);
                cpu_adc(cpu, low_78, high_78, game);
                break;
            }
            118 => {
                let (low_79, high_79) = cpu_adrDpx(cpu, game);
                cpu_ror(cpu, low_79, high_79, game);
                break;
            }
            119 => {
                let (low_80, high_80) = cpu_adrIly(cpu, game);
                cpu_adc(cpu, low_80, high_80, game);
                break;
            }
            120 => {
                (*cpu).i = true;
                break;
            }
            121 => {
                let (low_81, high_81) = cpu_adrAby(cpu, false, game);
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
                cpu_setZN(cpu, (*cpu).a, false);
                break;
            }
            124 => {
                (*cpu).pc = cpu_adrIax(cpu, game);
                break;
            }
            125 => {
                let (low_82, high_82) = cpu_adrAbx(cpu, false, game);
                cpu_adc(cpu, low_82, high_82, game);
                break;
            }
            126 => {
                let (low_83, high_83) = cpu_adrAbx(cpu, true, game);
                cpu_ror(cpu, low_83, high_83, game);
                break;
            }
            127 => {
                let (low_84, high_84) = cpu_adrAlx(cpu, game);
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
                let (low_85, high_85) = cpu_adrIdx(cpu, game);
                cpu_sta(cpu, low_85, high_85, game);
                break;
            }
            130 => {
                let displacement = cpu_readOpcodeWord(cpu, game) as int16_t;
                (*cpu).pc = (*cpu).pc.wrapping_add_signed(displacement);
                break;
            }
            131 => {
                let (low_86, high_86) = cpu_adrSr(cpu, game);
                cpu_sta(cpu, low_86, high_86, game);
                break;
            }
            132 => {
                let (low_87, high_87) = cpu_adrDp(cpu, game);
                cpu_sty(cpu, low_87, high_87, game);
                break;
            }
            133 => {
                let (low_88, high_88) = cpu_adrDp(cpu, game);
                cpu_sta(cpu, low_88, high_88, game);
                break;
            }
            134 => {
                let (low_89, high_89) = cpu_adrDp(cpu, game);
                cpu_stx(cpu, low_89, high_89, game);
                break;
            }
            135 => {
                let (low_90, high_90) = cpu_adrIdl(cpu, game);
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
                let (low_91, high_91) = cpu_adrAbs(cpu, game);
                cpu_sty(cpu, low_91, high_91, game);
                break;
            }
            141 => {
                let (low_92, high_92) = cpu_adrAbs(cpu, game);
                cpu_sta(cpu, low_92, high_92, game);
                break;
            }
            142 => {
                let (low_93, high_93) = cpu_adrAbs(cpu, game);
                cpu_stx(cpu, low_93, high_93, game);
                break;
            }
            143 => {
                let (low_94, high_94) = cpu_adrAbl(cpu, game);
                cpu_sta(cpu, low_94, high_94, game);
                break;
            }
            144 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).c);
                break;
            }
            145 => {
                let (low_95, high_95) = cpu_adrIdy(cpu, true, game);
                cpu_sta(cpu, low_95, high_95, game);
                break;
            }
            146 => {
                let (low_96, high_96) = cpu_adrIdp(cpu, game);
                cpu_sta(cpu, low_96, high_96, game);
                break;
            }
            147 => {
                let (low_97, high_97) = cpu_adrIsy(cpu, game);
                cpu_sta(cpu, low_97, high_97, game);
                break;
            }
            148 => {
                let (low_98, high_98) = cpu_adrDpx(cpu, game);
                cpu_sty(cpu, low_98, high_98, game);
                break;
            }
            149 => {
                let (low_99, high_99) = cpu_adrDpx(cpu, game);
                cpu_sta(cpu, low_99, high_99, game);
                break;
            }
            150 => {
                let (low_100, high_100) = cpu_adrDpy(cpu, game);
                cpu_stx(cpu, low_100, high_100, game);
                break;
            }
            151 => {
                let (low_101, high_101) = cpu_adrIly(cpu, game);
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
                let (low_102, high_102) = cpu_adrAby(cpu, true, game);
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
                let (low_103, high_103) = cpu_adrAbs(cpu, game);
                cpu_stz(cpu, low_103, high_103, game);
                break;
            }
            157 => {
                let (low_104, high_104) = cpu_adrAbx(cpu, true, game);
                cpu_sta(cpu, low_104, high_104, game);
                break;
            }
            158 => {
                let (low_105, high_105) = cpu_adrAbx(cpu, true, game);
                cpu_stz(cpu, low_105, high_105, game);
                break;
            }
            159 => {
                let (low_106, high_106) = cpu_adrAlx(cpu, game);
                cpu_sta(cpu, low_106, high_106, game);
                break;
            }
            160 => {
                let (low_107, high_107) = cpu_adrImm(cpu, true);
                cpu_ldy(cpu, low_107, high_107, game);
                break;
            }
            161 => {
                let (low_108, high_108) = cpu_adrIdx(cpu, game);
                cpu_lda(cpu, low_108, high_108, game);
                break;
            }
            162 => {
                let (low_109, high_109) = cpu_adrImm(cpu, true);
                cpu_ldx(cpu, low_109, high_109, game);
                break;
            }
            163 => {
                let (low_110, high_110) = cpu_adrSr(cpu, game);
                cpu_lda(cpu, low_110, high_110, game);
                break;
            }
            164 => {
                let (low_111, high_111) = cpu_adrDp(cpu, game);
                cpu_ldy(cpu, low_111, high_111, game);
                break;
            }
            165 => {
                let (low_112, high_112) = cpu_adrDp(cpu, game);
                cpu_lda(cpu, low_112, high_112, game);
                break;
            }
            166 => {
                let (low_113, high_113) = cpu_adrDp(cpu, game);
                cpu_ldx(cpu, low_113, high_113, game);
                break;
            }
            167 => {
                let (low_114, high_114) = cpu_adrIdl(cpu, game);
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
                let (low_115, high_115) = cpu_adrImm(cpu, false);
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
                let (low_116, high_116) = cpu_adrAbs(cpu, game);
                cpu_ldy(cpu, low_116, high_116, game);
                break;
            }
            173 => {
                let (low_117, high_117) = cpu_adrAbs(cpu, game);
                cpu_lda(cpu, low_117, high_117, game);
                break;
            }
            174 => {
                let (low_118, high_118) = cpu_adrAbs(cpu, game);
                cpu_ldx(cpu, low_118, high_118, game);
                break;
            }
            175 => {
                let (low_119, high_119) = cpu_adrAbl(cpu, game);
                cpu_lda(cpu, low_119, high_119, game);
                break;
            }
            176 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).c);
                break;
            }
            177 => {
                let (low_120, high_120) = cpu_adrIdy(cpu, false, game);
                cpu_lda(cpu, low_120, high_120, game);
                break;
            }
            178 => {
                let (low_121, high_121) = cpu_adrIdp(cpu, game);
                cpu_lda(cpu, low_121, high_121, game);
                break;
            }
            179 => {
                let (low_122, high_122) = cpu_adrIsy(cpu, game);
                cpu_lda(cpu, low_122, high_122, game);
                break;
            }
            180 => {
                let (low_123, high_123) = cpu_adrDpx(cpu, game);
                cpu_ldy(cpu, low_123, high_123, game);
                break;
            }
            181 => {
                let (low_124, high_124) = cpu_adrDpx(cpu, game);
                cpu_lda(cpu, low_124, high_124, game);
                break;
            }
            182 => {
                let (low_125, high_125) = cpu_adrDpy(cpu, game);
                cpu_ldx(cpu, low_125, high_125, game);
                break;
            }
            183 => {
                let (low_126, high_126) = cpu_adrIly(cpu, game);
                cpu_lda(cpu, low_126, high_126, game);
                break;
            }
            184 => {
                (*cpu).v = false;
                break;
            }
            185 => {
                let (low_127, high_127) = cpu_adrAby(cpu, false, game);
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
                let (low_128, high_128) = cpu_adrAbx(cpu, false, game);
                cpu_ldy(cpu, low_128, high_128, game);
                break;
            }
            189 => {
                let (low_129, high_129) = cpu_adrAbx(cpu, false, game);
                cpu_lda(cpu, low_129, high_129, game);
                break;
            }
            190 => {
                let (low_130, high_130) = cpu_adrAby(cpu, false, game);
                cpu_ldx(cpu, low_130, high_130, game);
                break;
            }
            191 => {
                let (low_131, high_131) = cpu_adrAlx(cpu, game);
                cpu_lda(cpu, low_131, high_131, game);
                break;
            }
            192 => {
                let (low_132, high_132) = cpu_adrImm(cpu, true);
                cpu_cpy(cpu, low_132, high_132, game);
                break;
            }
            193 => {
                let (low_133, high_133) = cpu_adrIdx(cpu, game);
                cpu_cmp(cpu, low_133, high_133, game);
                break;
            }
            194 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) & !cpu_readOpcode(cpu, game));
                break;
            }
            195 => {
                let (low_134, high_134) = cpu_adrSr(cpu, game);
                cpu_cmp(cpu, low_134, high_134, game);
                break;
            }
            196 => {
                let (low_135, high_135) = cpu_adrDp(cpu, game);
                cpu_cpy(cpu, low_135, high_135, game);
                break;
            }
            197 => {
                let (low_136, high_136) = cpu_adrDp(cpu, game);
                cpu_cmp(cpu, low_136, high_136, game);
                break;
            }
            198 => {
                let (low_137, high_137) = cpu_adrDp(cpu, game);
                cpu_dec(cpu, low_137, high_137, game);
                break;
            }
            199 => {
                let (low_138, high_138) = cpu_adrIdl(cpu, game);
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
                let (low_139, high_139) = cpu_adrImm(cpu, false);
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
                let (low_140, high_140) = cpu_adrAbs(cpu, game);
                cpu_cpy(cpu, low_140, high_140, game);
                break;
            }
            205 => {
                let (low_141, high_141) = cpu_adrAbs(cpu, game);
                cpu_cmp(cpu, low_141, high_141, game);
                break;
            }
            206 => {
                let (low_142, high_142) = cpu_adrAbs(cpu, game);
                cpu_dec(cpu, low_142, high_142, game);
                break;
            }
            207 => {
                let (low_143, high_143) = cpu_adrAbl(cpu, game);
                cpu_cmp(cpu, low_143, high_143, game);
                break;
            }
            208 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), !(*cpu).z);
                break;
            }
            209 => {
                let (low_144, high_144) = cpu_adrIdy(cpu, false, game);
                cpu_cmp(cpu, low_144, high_144, game);
                break;
            }
            210 => {
                let (low_145, high_145) = cpu_adrIdp(cpu, game);
                cpu_cmp(cpu, low_145, high_145, game);
                break;
            }
            211 => {
                let (low_146, high_146) = cpu_adrIsy(cpu, game);
                cpu_cmp(cpu, low_146, high_146, game);
                break;
            }
            212 => {
                let (low_147, high_147) = cpu_adrDp(cpu, game);
                cpu_pushWord(cpu, cpu_readWord(cpu, low_147, high_147, game), game);
                break;
            }
            213 => {
                let (low_148, high_148) = cpu_adrDpx(cpu, game);
                cpu_cmp(cpu, low_148, high_148, game);
                break;
            }
            214 => {
                let (low_149, high_149) = cpu_adrDpx(cpu, game);
                cpu_dec(cpu, low_149, high_149, game);
                break;
            }
            215 => {
                let (low_150, high_150) = cpu_adrIly(cpu, game);
                cpu_cmp(cpu, low_150, high_150, game);
                break;
            }
            216 => {
                (*cpu).d = false;
                break;
            }
            217 => {
                let (low_151, high_151) = cpu_adrAby(cpu, false, game);
                cpu_cmp(cpu, low_151, high_151, game);
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
                let (low_152, high_152) = cpu_adrAbx(cpu, false, game);
                cpu_cmp(cpu, low_152, high_152, game);
                break;
            }
            222 => {
                let (low_153, high_153) = cpu_adrAbx(cpu, true, game);
                cpu_dec(cpu, low_153, high_153, game);
                break;
            }
            223 => {
                let (low_154, high_154) = cpu_adrAlx(cpu, game);
                cpu_cmp(cpu, low_154, high_154, game);
                break;
            }
            224 => {
                let (low_155, high_155) = cpu_adrImm(cpu, true);
                cpu_cpx(cpu, low_155, high_155, game);
                break;
            }
            225 => {
                let (low_156, high_156) = cpu_adrIdx(cpu, game);
                cpu_sbc(cpu, low_156, high_156, game);
                break;
            }
            226 => {
                cpu_setFlags(cpu, cpu_getFlags(cpu) | cpu_readOpcode(cpu, game));
                break;
            }
            227 => {
                let (low_157, high_157) = cpu_adrSr(cpu, game);
                cpu_sbc(cpu, low_157, high_157, game);
                break;
            }
            228 => {
                let (low_158, high_158) = cpu_adrDp(cpu, game);
                cpu_cpx(cpu, low_158, high_158, game);
                break;
            }
            229 => {
                let (low_159, high_159) = cpu_adrDp(cpu, game);
                cpu_sbc(cpu, low_159, high_159, game);
                break;
            }
            230 => {
                let (low_160, high_160) = cpu_adrDp(cpu, game);
                cpu_inc(cpu, low_160, high_160, game);
                break;
            }
            231 => {
                let (low_161, high_161) = cpu_adrIdl(cpu, game);
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
                let (low_162, high_162) = cpu_adrImm(cpu, false);
                cpu_sbc(cpu, low_162, high_162, game);
                break;
            }
            235 => {
                (*cpu).a = (*cpu).a.swap_bytes();
                cpu_setZN(cpu, (*cpu).a, true);
                break;
            }
            236 => {
                let (low_164, high_164) = cpu_adrAbs(cpu, game);
                cpu_cpx(cpu, low_164, high_164, game);
                break;
            }
            237 => {
                let (low_165, high_165) = cpu_adrAbs(cpu, game);
                cpu_sbc(cpu, low_165, high_165, game);
                break;
            }
            238 => {
                let (low_166, high_166) = cpu_adrAbs(cpu, game);
                cpu_inc(cpu, low_166, high_166, game);
                break;
            }
            239 => {
                let (low_167, high_167) = cpu_adrAbl(cpu, game);
                cpu_sbc(cpu, low_167, high_167, game);
                break;
            }
            240 => {
                cpu_doBranch(cpu, cpu_readOpcode(cpu, game), (*cpu).z);
                break;
            }
            241 => {
                let (low_168, high_168) = cpu_adrIdy(cpu, false, game);
                cpu_sbc(cpu, low_168, high_168, game);
                break;
            }
            242 => {
                let (low_169, high_169) = cpu_adrIdp(cpu, game);
                cpu_sbc(cpu, low_169, high_169, game);
                break;
            }
            243 => {
                let (low_170, high_170) = cpu_adrIsy(cpu, game);
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
                let (low_171, high_171) = cpu_adrDpx(cpu, game);
                cpu_sbc(cpu, low_171, high_171, game);
                break;
            }
            246 => {
                let (low_172, high_172) = cpu_adrDpx(cpu, game);
                cpu_inc(cpu, low_172, high_172, game);
                break;
            }
            247 => {
                let (low_173, high_173) = cpu_adrIly(cpu, game);
                cpu_sbc(cpu, low_173, high_173, game);
                break;
            }
            248 => {
                (*cpu).d = true;
                break;
            }
            249 => {
                let (low_174, high_174) = cpu_adrAby(cpu, false, game);
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
                let value_3 = cpu_adrIax(cpu, game);
                cpu_pushWord(cpu, (*cpu).pc.wrapping_sub(1), game);
                (*cpu).pc = value_3;
                break;
            }
            253 => {
                let (low_175, high_175) = cpu_adrAbx(cpu, false, game);
                cpu_sbc(cpu, low_175, high_175, game);
                break;
            }
            254 => {
                let (low_176, high_176) = cpu_adrAbx(cpu, true, game);
                cpu_inc(cpu, low_176, high_176, game);
                break;
            }
            255 => {
                let (low_177, high_177) = cpu_adrAlx(cpu, game);
                cpu_sbc(cpu, low_177, high_177, game);
                break;
            }
            234 | _ => {
                break;
            }
        }
    }
}
