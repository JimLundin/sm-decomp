use crate::game_state::Game;
use crate::rtl::RtlApuWrite;
use crate::snes::apu::{apu_cycle, apu_free, apu_init, apu_reset, apu_saveload};
use crate::snes::cart::{cart_free, cart_init, cart_read, cart_reset, cart_saveload, cart_write};
use crate::snes::cpu::{cpu_free, cpu_init, cpu_reset, cpu_saveload};
use crate::snes::dma::{
    dma_cycle, dma_doHdma, dma_free, dma_init, dma_initHdma, dma_read, dma_reset, dma_saveload,
    dma_startDma, dma_write,
};
use crate::snes::input::{input_free, input_init, input_reset};
use crate::snes::ppu::{
    ppu_checkOverscan, ppu_free, ppu_handleVblank, ppu_init, ppu_read, ppu_reset, ppu_runLine,
    ppu_saveload, ppu_write,
};
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
    size_t, uint16_t, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, SaveLoadFunc, Snes, Spc, Timer, WindowLayer,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub unsafe fn snes_init(mut ram: *mut uint8_t) -> *mut Snes {
    let mut snes: *mut Snes = malloc(::core::mem::size_of::<Snes>() as size_t) as *mut Snes;
    (*snes).ram = ram;
    (*snes).debug_cycles = false_0 != 0;
    (*snes).debug_apu_cycles = false_0 != 0;
    (*snes).runningWhichVersion = 0 as uint8_t;
    (*snes).disableRender = false;
    (*snes).cpuMemOps = 0;
    (*snes).cpuCyclesLeft = 0;
    (*snes).cpu = cpu_init(snes as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int);
    (*snes).apu = apu_init();
    (*snes).dma = dma_init(snes);
    (*snes).my_ppu = ppu_init(snes);
    (*snes).snes_ppu = ppu_init(snes);
    (*snes).ppu = (*snes).snes_ppu;
    (*snes).cart = cart_init(snes);
    (*snes).input1 = input_init(snes);
    (*snes).input2 = input_init(snes);
    return snes;
}
pub unsafe fn snes_free(mut snes: *mut Snes) {
    cpu_free((*snes).cpu);
    apu_free((*snes).apu);
    dma_free((*snes).dma);
    ppu_free((*snes).ppu);
    cart_free((*snes).cart);
    input_free((*snes).input1);
    input_free((*snes).input2);
    free(snes as *mut ::core::ffi::c_void);
}
pub unsafe fn snes_saveload(
    snes: *mut Snes,
    func: Option<SaveLoadFunc>,
    ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    cpu_saveload((*snes).cpu, func, ctx, game);
    apu_saveload((*snes).apu, func, ctx, game);
    dma_saveload((*snes).dma, func, ctx, game);
    ppu_saveload((*snes).ppu, func, ctx, game);
    cart_saveload((*snes).cart, func, ctx, game);
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*snes).hPos as *mut ::core::ffi::c_void,
        54,
        game,
    );
    func.expect("non-null function pointer")(
        ctx,
        (*snes).ram as *mut ::core::ffi::c_void,
        0x20000,
        game,
    );
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*snes).ramAdr as *mut ::core::ffi::c_void,
        4,
        game,
    );
    (*snes).runningWhichVersion = 0;
}
pub unsafe fn snes_reset(snes: *mut Snes, hard: bool, game: &mut Game) {
    cart_reset((*snes).cart);
    cpu_reset((*snes).cpu, game);
    apu_reset((*snes).apu);
    dma_reset((*snes).dma);
    ppu_reset((*snes).my_ppu);
    ppu_reset((*snes).snes_ppu);
    input_reset((*snes).input1);
    input_reset((*snes).input2);
    if hard {
        memset(
            (*snes).ram as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            0x20000 as ::core::ffi::c_int as size_t,
        );
    }
    (*snes).ramAdr = 0;
    (*snes).hPos = 0;
    (*snes).vPos = 0;
    (*snes).frames = 0;
    (*snes).cpuCyclesLeft = 52;
    (*snes).cpuMemOps = 0;
    (*snes).apuCatchupCycles = 0.0f64;
    (*snes).hIrqEnabled = false;
    (*snes).vIrqEnabled = false;
    (*snes).nmiEnabled = false;
    (*snes).hTimer = 0x1ff;
    (*snes).vTimer = 0x1ff;
    (*snes).inNmi = false;
    (*snes).inIrq = false;
    (*snes).inVblank = false;
    (*snes).portAutoReadX = [0; 4];
    (*snes).autoJoyRead = false;
    (*snes).autoJoyTimer = 0;
    (*snes).ppuLatch = false;
    (*snes).multiplyA = 0xff;
    (*snes).multiplyResult = 0xfe01;
    (*snes).divideA = 0xffff;
    (*snes).divideResult = 0x101;
    (*snes).fastMem = false;
    (*snes).openBus = 0;
}
pub unsafe fn snes_handle_pos_stuff(snes: *mut Snes, game: &mut Game) {
    if (*snes).hPos == 0 {
        let mut startingVblank = false;
        if (*snes).vPos == 0 {
            (*snes).inVblank = false;
            (*snes).inNmi = false;
            dma_initHdma((*snes).dma, game);
        } else if (*snes).vPos == 225 {
            startingVblank = !ppu_checkOverscan((*snes).ppu);
        } else if (*snes).vPos == 240 {
            if !(*snes).inVblank {
                startingVblank = true;
            }
        }
        if startingVblank {
            ppu_handleVblank((*snes).ppu);
            (*snes).inVblank = true;
            (*snes).inNmi = true;
            (*(*snes).cpu).nmiWanted = true;
            if (*snes).autoJoyRead {
                (*snes).autoJoyTimer = 0;
            }
        }
    } else if (*snes).hPos == 512 {
        if !(*snes).inVblank && !(*snes).disableRender {
            ppu_runLine((*snes).ppu, i32::from((*snes).vPos), game);
        }
    } else if (*snes).hPos == 1024 {
        if !(*snes).inVblank {
            dma_doHdma((*snes).dma, game);
        }
    }
    (*snes).hPos = (*snes).hPos.wrapping_add(2);
    if (*snes).hPos == 1364 {
        (*snes).hPos = 0;
        (*snes).vPos = (*snes).vPos.wrapping_add(1);
        if (*snes).vPos == 262 {
            (*snes).vPos = 0;
            (*snes).frames = (*snes).frames.wrapping_add(1);
        }
    }
}
pub unsafe fn snes_catchupApu(snes: *mut Snes, game: &mut Game) {
    if (*snes).apuCatchupCycles > 10000.0 {
        (*snes).apuCatchupCycles = 10000.0;
    }
    let cycles = (*snes).apuCatchupCycles as i32;
    for _ in 0..cycles {
        apu_cycle((*snes).apu, game);
    }
    (*snes).apuCatchupCycles -= f64::from(cycles);
}
pub unsafe fn snes_readBBus(snes: *mut Snes, address: u8, game: &mut Game) -> u8 {
    match address {
        0x00..=0x3f => ppu_read((*snes).ppu, address),
        0x40..=0x7f => {
            (*snes).apuCatchupCycles = 32.0;
            snes_catchupApu(snes, game);
            (*(*snes).apu).outPorts[usize::from(address & 3)]
        }
        0x80 => {
            let address = (*snes).ramAdr;
            (*snes).ramAdr = (*snes).ramAdr.wrapping_add(1);
            let value = *(*snes).ram.offset(address as isize);
            (*snes).ramAdr &= 0x1ffff;
            value
        }
        _ => {
            __assert_fail(
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/snes/snes.c\0" as *const u8 as *const ::core::ffi::c_char,
                197 as ::core::ffi::c_uint,
                b"uint8_t snes_readBBus(Snes *, uint8_t)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }
}
pub unsafe fn snes_writeBBus(snes: *mut Snes, address: u8, value: u8, game: &mut Game) {
    match address {
        0x00..=0x3f => ppu_write((*snes).ppu, address, value),
        0x40..=0x7f => RtlApuWrite(0x2100 + u32::from(address), value, game),
        0x80 => {
            let address = (*snes).ramAdr;
            (*snes).ramAdr = (*snes).ramAdr.wrapping_add(1);
            *(*snes).ram.offset(address as isize) = value;
            (*snes).ramAdr &= 0x1ffff;
        }
        0x81 => (*snes).ramAdr = ((*snes).ramAdr & 0x1ff00) | u32::from(value),
        0x82 => (*snes).ramAdr = ((*snes).ramAdr & 0x100ff) | (u32::from(value) << 8),
        0x83 => (*snes).ramAdr = ((*snes).ramAdr & 0xffff) | (u32::from(value & 1) << 16),
        _ => {}
    }
}
fn SwapInputBits(input: u16) -> u16 {
    input.reverse_bits()
}
unsafe fn snes_readReg(snes: *mut Snes, address: u16) -> u8 {
    match address {
        0x4210 => 2 | (u8::from((*snes).inNmi) << 7) | ((*snes).openBus & 0x70),
        0x4211 => {
            let irq = u8::from((*snes).inIrq) << 7;
            (*snes).inIrq = false;
            (*(*snes).cpu).irqWanted = false;
            irq | ((*snes).openBus & 0x7f)
        }
        0x4212 => {
            u8::from((*snes).autoJoyTimer > 0)
                | (u8::from((*snes).hPos >= 1024) << 6)
                | (u8::from((*snes).inVblank) << 7)
                | ((*snes).openBus & 0x3e)
        }
        0x4213 => u8::from((*snes).ppuLatch) << 7,
        0x4214 => (*snes).divideResult as u8,
        0x4215 => ((*snes).divideResult >> 8) as u8,
        0x4216 => (*snes).multiplyResult as u8,
        0x4217 => ((*snes).multiplyResult >> 8) as u8,
        0x4218 => SwapInputBits((*(*snes).input1).currentState) as u8,
        0x4219 => (SwapInputBits((*(*snes).input1).currentState) >> 8) as u8,
        0x421a..=0x421f => 0,
        _ => (*snes).openBus,
    }
}
unsafe fn snes_writeReg(snes: *mut Snes, adr: u16, val: u8, game: &mut Game) {
    match adr {
        0x4200 => {
            (*snes).autoJoyRead = val & 0x1 != 0;
            if !(*snes).autoJoyRead {
                (*snes).autoJoyTimer = 0;
            }
            (*snes).hIrqEnabled = val & 0x10 != 0;
            (*snes).vIrqEnabled = val & 0x20 != 0;
            (*snes).nmiEnabled = val & 0x80 != 0;
            if !(*snes).hIrqEnabled && !(*snes).vIrqEnabled {
                (*snes).inIrq = false;
                (*(*snes).cpu).irqWanted = false;
            }
        }
        0x4201 => {
            if val & 0x80 == 0 && (*snes).ppuLatch {
                ppu_read((*snes).ppu, 0x37);
            }
            (*snes).ppuLatch = val & 0x80 != 0;
        }
        0x4202 => {
            (*snes).multiplyA = val;
        }
        0x4203 => {
            (*snes).multiplyResult = u16::from((*snes).multiplyA) * u16::from(val);
        }
        0x4204 => {
            (*snes).divideA = ((*snes).divideA & 0xff00) | u16::from(val);
        }
        0x4205 => {
            (*snes).divideA = ((*snes).divideA & 0xff) | (u16::from(val) << 8);
        }
        0x4206 => {
            if val == 0 {
                (*snes).divideResult = 0xffff;
                (*snes).multiplyResult = (*snes).divideA;
            } else {
                (*snes).divideResult = (*snes).divideA / u16::from(val);
                (*snes).multiplyResult = (*snes).divideA % u16::from(val);
            }
        }
        0x4207 => {
            (*snes).hTimer = ((*snes).hTimer & 0x100) | u16::from(val);
        }
        0x4208 => {
            (*snes).hTimer = ((*snes).hTimer & 0xff) | (u16::from(val & 1) << 8);
        }
        0x4209 => {
            (*snes).vTimer = ((*snes).vTimer & 0x100) | u16::from(val);
        }
        0x420a => {
            (*snes).vTimer = ((*snes).vTimer & 0xff) | (u16::from(val & 1) << 8);
        }
        0x420b => {
            if val as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                let mut t: uint32_t = (((*(*snes).dma).channel[1 as ::core::ffi::c_int as usize]
                    .aBank as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int
                    | (*(*snes).dma).channel[1 as ::core::ffi::c_int as usize].aAdr
                        as ::core::ffi::c_int) as uint32_t;
                let mut data: ::core::ffi::c_int = snes_read(snes, t, game) as ::core::ffi::c_int
                    | (snes_read(snes, t.wrapping_add(1 as uint32_t), game) as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int;
            }
            dma_startDma((*snes).dma, val, false_0 != 0);
            while dma_cycle((*snes).dma, game) {}
        }
        0x420c => {
            dma_startDma((*snes).dma, val, true_0 != 0);
        }
        0x420d => {
            (*snes).fastMem = val & 0x1 != 0;
        }
        _ => {}
    };
}
pub unsafe fn snes_read(snes: *mut Snes, address: u32, game: &mut Game) -> u8 {
    let bank = (address >> 16) as u8;
    let offset = address as u16;
    if matches!(bank, 0x7e | 0x7f) {
        let index = (usize::from(bank & 1) << 16) | usize::from(offset);
        return *(*snes).ram.add(index);
    }
    if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
        match offset {
            0x0000..=0x1fff => return *(*snes).ram.add(usize::from(offset)),
            0x2100..=0x21ff => return snes_readBBus(snes, offset as u8, game),
            0x4016 | 0x4017 => {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/snes.c\0" as *const u8 as *const ::core::ffi::c_char,
                    396 as ::core::ffi::c_uint,
                    b"uint8_t snes_read(Snes *, uint32_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            0x4200..=0x421f => return snes_readReg(snes, offset),
            0x4300..=0x437f => return dma_read((*snes).dma, offset),
            _ => {}
        }
    }
    cart_read((*snes).cart, bank, offset, game)
}
pub unsafe fn LogWrite(mut snes: *mut Snes, mut adr: uint32_t, mut val: uint8_t, game: &mut Game) {
    printf(
        b"@%d: Write to 0x%x = 0x%.2x: 0x%x: r18=0x%x: r20=0x%x: a = 0x%x, x = 0x%x, y = 0x%x, c = %d\n\0"
            as *const u8 as *const ::core::ffi::c_char,
        game.snes_frame_counter,
        adr,
        val as ::core::ffi::c_int,
        ((*(*snes).cpu).k as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*(*snes).cpu).pc as ::core::ffi::c_int,
        *(*snes).ram.offset(0x12 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            | (*(*snes).ram.offset(0x13 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) << 8 as ::core::ffi::c_int,
        *(*snes).ram.offset(0x14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            | (*(*snes).ram.offset(0x15 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) << 8 as ::core::ffi::c_int,
        (*(*snes).cpu).a as ::core::ffi::c_int,
        (*(*snes).cpu).x as ::core::ffi::c_int,
        (*(*snes).cpu).y as ::core::ffi::c_int,
        (*(*snes).cpu).c as ::core::ffi::c_int,
    );
}
pub unsafe fn snes_write(snes: *mut Snes, address: u32, value: u8, game: &mut Game) {
    let bank = (address >> 16) as u8;
    let offset = address as u16;
    if matches!(bank, 0x7e | 0x7f) {
        let index = (usize::from(bank & 1) << 16) | usize::from(offset);
        *(*snes).ram.add(index) = value;
    }
    if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
        match offset {
            0x0000..=0x1fff => *(*snes).ram.add(usize::from(offset)) = value,
            0x2100..=0x21ff => snes_writeBBus(snes, offset as u8, value, game),
            0x4200..=0x421f => snes_writeReg(snes, offset, value, game),
            0x4300..=0x437f => dma_write((*snes).dma, offset, value),
            _ => {}
        }
    }
    cart_write((*snes).cart, bank, offset, value);
}
pub unsafe fn snes_cpuRead(snes: *mut Snes, address: u32, game: &mut Game) -> u8 {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
    snes_read(snes, address, game)
}
pub unsafe fn snes_cpuWrite(snes: *mut Snes, address: u32, value: u8, game: &mut Game) {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
    snes_write(snes, address, value, game);
}
