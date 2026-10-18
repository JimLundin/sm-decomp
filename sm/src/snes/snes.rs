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
    mut snes: *mut Snes,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
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
        (145 as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_sub(92 as size_t),
        game,
    );
    func.expect("non-null function pointer")(
        ctx,
        (*snes).ram as *mut ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
        game,
    );
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*snes).ramAdr as *mut ::core::ffi::c_void,
        4 as size_t,
        game,
    );
    (*snes).runningWhichVersion = 0 as uint8_t;
}
pub unsafe fn snes_reset(mut snes: *mut Snes, mut hard: bool, game: &mut Game) {
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
    (*snes).ramAdr = 0 as uint32_t;
    (*snes).hPos = 0 as uint16_t;
    (*snes).vPos = 0 as uint16_t;
    (*snes).frames = 0 as uint32_t;
    (*snes).cpuCyclesLeft = 52 as uint8_t;
    (*snes).cpuMemOps = 0 as uint8_t;
    (*snes).apuCatchupCycles = 0.0f64;
    (*snes).hIrqEnabled = false_0 != 0;
    (*snes).vIrqEnabled = false_0 != 0;
    (*snes).nmiEnabled = false_0 != 0;
    (*snes).hTimer = 0x1ff as uint16_t;
    (*snes).vTimer = 0x1ff as uint16_t;
    (*snes).inNmi = false_0 != 0;
    (*snes).inIrq = false_0 != 0;
    (*snes).inVblank = false_0 != 0;
    memset(
        &raw mut (*snes).portAutoReadX as *mut uint16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint16_t; 4]>() as size_t,
    );
    (*snes).autoJoyRead = false_0 != 0;
    (*snes).autoJoyTimer = 0 as uint16_t;
    (*snes).ppuLatch = false_0 != 0;
    (*snes).multiplyA = 0xff as uint8_t;
    (*snes).multiplyResult = 0xfe01 as uint16_t;
    (*snes).divideA = 0xffff as uint16_t;
    (*snes).divideResult = 0x101 as uint16_t;
    (*snes).fastMem = false_0 != 0;
    (*snes).openBus = 0 as uint8_t;
}
pub unsafe fn snes_handle_pos_stuff(mut snes: *mut Snes, game: &mut Game) {
    if (*snes).hPos as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut startingVblank: bool = false_0 != 0;
        if (*snes).vPos as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*snes).inVblank = false_0 != 0;
            (*snes).inNmi = false_0 != 0;
            dma_initHdma((*snes).dma, game);
        } else if (*snes).vPos as ::core::ffi::c_int == 225 as ::core::ffi::c_int {
            startingVblank = !ppu_checkOverscan((*snes).ppu);
        } else if (*snes).vPos as ::core::ffi::c_int == 240 as ::core::ffi::c_int {
            if !(*snes).inVblank {
                startingVblank = true_0 != 0;
            }
        }
        if startingVblank {
            ppu_handleVblank((*snes).ppu);
            (*snes).inVblank = true_0 != 0;
            (*snes).inNmi = true_0 != 0;
            (*(*snes).cpu).nmiWanted = true_0 != 0;
            if (*snes).autoJoyRead {
                (*snes).autoJoyTimer = 0 as uint16_t;
            }
        }
    } else if (*snes).hPos as ::core::ffi::c_int == 512 as ::core::ffi::c_int {
        if !(*snes).inVblank && !(*snes).disableRender {
            ppu_runLine((*snes).ppu, (*snes).vPos as ::core::ffi::c_int, game);
        }
    } else if (*snes).hPos as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
        if !(*snes).inVblank {
            dma_doHdma((*snes).dma, game);
        }
    }
    (*snes).hPos = ((*snes).hPos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16_t;
    if (*snes).hPos as ::core::ffi::c_int == 1364 as ::core::ffi::c_int {
        (*snes).hPos = 0 as uint16_t;
        (*snes).vPos = (*snes).vPos.wrapping_add(1);
        if (*snes).vPos as ::core::ffi::c_int == 262 as ::core::ffi::c_int {
            (*snes).vPos = 0 as uint16_t;
            (*snes).frames = (*snes).frames.wrapping_add(1);
        }
    }
}
pub unsafe fn snes_catchupApu(mut snes: *mut Snes, game: &mut Game) {
    if (*snes).apuCatchupCycles > 10000 as ::core::ffi::c_int as ::core::ffi::c_double {
        (*snes).apuCatchupCycles = 10000 as ::core::ffi::c_int as ::core::ffi::c_double;
    }
    let mut catchupCycles: ::core::ffi::c_int = (*snes).apuCatchupCycles as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < catchupCycles {
        apu_cycle((*snes).apu, game);
        i += 1;
    }
    (*snes).apuCatchupCycles -= catchupCycles as ::core::ffi::c_double;
}
pub unsafe fn snes_readBBus(mut snes: *mut Snes, mut adr: uint8_t, game: &mut Game) -> uint8_t {
    if (adr as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int {
        return ppu_read((*snes).ppu, adr);
    }
    if (adr as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        (*snes).apuCatchupCycles = 32 as ::core::ffi::c_int as ::core::ffi::c_double;
        snes_catchupApu(snes, game);
        return (*(*snes).apu).outPorts
            [(adr as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int) as usize];
    }
    if adr as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int {
        let fresh0 = (*snes).ramAdr;
        (*snes).ramAdr = (*snes).ramAdr.wrapping_add(1);
        let mut ret: uint8_t = *(*snes).ram.offset(fresh0 as isize);
        (*snes).ramAdr &= 0x1ffff as uint32_t;
        return ret;
    }
    '_c2rust_label_0: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/snes.c\0" as *const u8 as *const ::core::ffi::c_char,
            197 as ::core::ffi::c_uint,
            b"uint8_t snes_readBBus(Snes *, uint8_t)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*snes).openBus;
}
pub unsafe fn snes_writeBBus(
    mut snes: *mut Snes,
    mut adr: uint8_t,
    mut val: uint8_t,
    game: &mut Game,
) {
    if (adr as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int {
        ppu_write((*snes).ppu, adr, val);
        return;
    }
    if (adr as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        RtlApuWrite(
            (0x2100 as ::core::ffi::c_int + adr as ::core::ffi::c_int) as uint32_t,
            val,
            game,
        );
        return;
    }
    match adr as ::core::ffi::c_int {
        128 => {
            let fresh1 = (*snes).ramAdr;
            (*snes).ramAdr = (*snes).ramAdr.wrapping_add(1);
            *(*snes).ram.offset(fresh1 as isize) = val;
            (*snes).ramAdr &= 0x1ffff as uint32_t;
        }
        129 => {
            (*snes).ramAdr = (*snes).ramAdr & 0x1ff00 as uint32_t | val as uint32_t;
        }
        130 => {
            (*snes).ramAdr = (*snes).ramAdr & 0x100ff as uint32_t
                | ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32_t;
        }
        131 => {
            (*snes).ramAdr = (*snes).ramAdr & 0xffff as uint32_t
                | ((val as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int) as uint32_t;
        }
        _ => {}
    };
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
unsafe fn snes_writeReg(mut snes: *mut Snes, mut adr: uint16_t, mut val: uint8_t, game: &mut Game) {
    match adr as ::core::ffi::c_int {
        16896 => {
            (*snes).autoJoyRead = val as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0;
            if !(*snes).autoJoyRead {
                (*snes).autoJoyTimer = 0 as uint16_t;
            }
            (*snes).hIrqEnabled = val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
            (*snes).vIrqEnabled = val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
            (*snes).nmiEnabled = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            if !(*snes).hIrqEnabled && !(*snes).vIrqEnabled {
                (*snes).inIrq = false_0 != 0;
                (*(*snes).cpu).irqWanted = false_0 != 0;
            }
        }
        16897 => {
            if val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
                && (*snes).ppuLatch as ::core::ffi::c_int != 0
            {
                ppu_read((*snes).ppu, 0x37 as uint8_t);
            }
            (*snes).ppuLatch = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
        }
        16898 => {
            (*snes).multiplyA = val;
        }
        16899 => {
            (*snes).multiplyResult = u16::from((*snes).multiplyA) * u16::from(val);
        }
        16900 => {
            (*snes).divideA = ((*snes).divideA & 0xff00) | u16::from(val);
        }
        16901 => {
            (*snes).divideA = ((*snes).divideA & 0xff) | (u16::from(val) << 8);
        }
        16902 => {
            if val == 0 {
                (*snes).divideResult = 0xffff;
                (*snes).multiplyResult = (*snes).divideA;
            } else {
                (*snes).divideResult = (*snes).divideA / u16::from(val);
                (*snes).multiplyResult = (*snes).divideA % u16::from(val);
            }
        }
        16903 => {
            (*snes).hTimer = ((*snes).hTimer & 0x100) | u16::from(val);
        }
        16904 => {
            (*snes).hTimer = ((*snes).hTimer & 0xff) | (u16::from(val & 1) << 8);
        }
        16905 => {
            (*snes).vTimer = ((*snes).vTimer & 0x100) | u16::from(val);
        }
        16906 => {
            (*snes).vTimer = ((*snes).vTimer & 0xff) | (u16::from(val & 1) << 8);
        }
        16907 => {
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
        16908 => {
            dma_startDma((*snes).dma, val, true_0 != 0);
        }
        16909 => {
            (*snes).fastMem = val as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0;
        }
        _ => {}
    };
}
pub unsafe fn snes_read(mut snes: *mut Snes, mut adr: uint32_t, game: &mut Game) -> uint8_t {
    let mut bank: uint8_t = (adr >> 16 as ::core::ffi::c_int) as uint8_t;
    adr &= 0xffff as uint32_t;
    if bank as ::core::ffi::c_int == 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
    {
        return *(*snes).ram.offset(
            (((bank as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
                as uint32_t
                | adr) as isize,
        );
    }
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
            && (bank as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int
    {
        if adr < 0x2000 as uint32_t {
            return *(*snes).ram.offset(adr as isize);
        }
        if adr >= 0x2100 as uint32_t && adr < 0x2200 as uint32_t {
            return snes_readBBus(snes, (adr & 0xff as uint32_t) as uint8_t, game);
        }
        if adr == 0x4016 as uint32_t || adr == 0x4017 as uint32_t {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/snes.c\0" as *const u8 as *const ::core::ffi::c_char,
                    396 as ::core::ffi::c_uint,
                    b"uint8_t snes_read(Snes *, uint32_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            };
        }
        if adr >= 0x4200 as uint32_t && adr < 0x4220 as uint32_t
            || adr >= 0x4218 as uint32_t && adr < 0x4220 as uint32_t
        {
            return snes_readReg(snes, adr as uint16_t);
        }
        if adr >= 0x4300 as uint32_t && adr < 0x4380 as uint32_t {
            return dma_read((*snes).dma, adr as uint16_t);
        }
    }
    return cart_read((*snes).cart, bank, adr as uint16_t, game);
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
pub unsafe fn snes_write(
    mut snes: *mut Snes,
    mut adr: uint32_t,
    mut val: uint8_t,
    game: &mut Game,
) {
    let mut bank: uint8_t = (adr >> 16 as ::core::ffi::c_int) as uint8_t;
    adr &= 0xffff as uint32_t;
    if bank as ::core::ffi::c_int == 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
    {
        let mut addr: uint32_t = ((bank as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32_t
            | adr;
        *(*snes).ram.offset(addr as isize) = val;
        if addr == 0xfffff as uint32_t {
            LogWrite(snes, adr, val, game);
        }
    }
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
            && (bank as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int
    {
        if adr < 0x2000 as uint32_t {
            *(*snes).ram.offset(adr as isize) = val;
            if adr == 0xfffff as uint32_t {
                LogWrite(snes, adr, val, game);
            }
        }
        if adr >= 0x2100 as uint32_t && adr < 0x2200 as uint32_t {
            snes_writeBBus(snes, (adr & 0xff as uint32_t) as uint8_t, val, game);
        }
        if adr >= 0x4200 as uint32_t && adr < 0x4220 as uint32_t {
            snes_writeReg(snes, adr as uint16_t, val, game);
        }
        if adr >= 0x4300 as uint32_t && adr < 0x4380 as uint32_t {
            dma_write((*snes).dma, adr as uint16_t, val);
        }
    }
    cart_write((*snes).cart, bank, adr as uint16_t, val);
}
pub unsafe fn snes_cpuRead(mut snes: *mut Snes, mut adr: uint32_t, game: &mut Game) -> uint8_t {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft =
        ((*snes).cpuCyclesLeft as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint8_t;
    return snes_read(snes, adr, game);
}
pub unsafe fn snes_cpuWrite(
    mut snes: *mut Snes,
    mut adr: uint32_t,
    mut val: uint8_t,
    game: &mut Game,
) {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft =
        ((*snes).cpuCyclesLeft as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint8_t;
    snes_write(snes, adr, val, game);
}
