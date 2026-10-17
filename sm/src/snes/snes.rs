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
    fn cpu_init(mem: *mut ::core::ffi::c_void, memType: ::core::ffi::c_int) -> *mut Cpu;
    fn cpu_free(cpu: *mut Cpu);
    fn cpu_reset(cpu: *mut Cpu);
    fn cpu_saveload(cpu: *mut Cpu, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn apu_init() -> *mut Apu;
    fn apu_free(apu: *mut Apu);
    fn apu_reset(apu: *mut Apu);
    fn apu_cycle(apu: *mut Apu);
    fn apu_saveload(apu: *mut Apu, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn dma_init(snes: *mut Snes) -> *mut Dma;
    fn dma_free(dma: *mut Dma);
    fn dma_reset(dma: *mut Dma);
    fn dma_read(dma: *mut Dma, adr: uint16_t) -> uint8_t;
    fn dma_write(dma: *mut Dma, adr: uint16_t, val: uint8_t);
    fn dma_initHdma(dma: *mut Dma);
    fn dma_doHdma(dma: *mut Dma);
    fn dma_cycle(dma: *mut Dma) -> bool;
    fn dma_startDma(dma: *mut Dma, val: uint8_t, hdma: bool);
    fn dma_saveload(dma: *mut Dma, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn ppu_init(snes: *mut Snes) -> *mut Ppu;
    fn ppu_free(ppu: *mut Ppu);
    fn ppu_reset(ppu: *mut Ppu);
    fn ppu_checkOverscan(ppu: *mut Ppu) -> bool;
    fn ppu_handleVblank(ppu: *mut Ppu);
    fn ppu_runLine(ppu: *mut Ppu, line: ::core::ffi::c_int);
    fn ppu_read(ppu: *mut Ppu, adr: uint8_t) -> uint8_t;
    fn ppu_write(ppu: *mut Ppu, adr: uint8_t, val: uint8_t);
    fn ppu_saveload(ppu: *mut Ppu, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn cart_init(snes: *mut Snes) -> *mut Cart;
    fn cart_free(cart: *mut Cart);
    fn cart_reset(cart: *mut Cart);
    fn cart_read(cart: *mut Cart, bank: uint8_t, adr: uint16_t) -> uint8_t;
    fn cart_write(cart: *mut Cart, bank: uint8_t, adr: uint16_t, val: uint8_t);
    fn cart_saveload(cart: *mut Cart, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn input_init(snes: *mut Snes) -> *mut Input;
    fn input_free(input: *mut Input);
    fn input_reset(input: *mut Input);
    fn RtlApuWrite(adr: uint32_t, val: uint8_t);
}
pub type size_t = usize;
pub type __int8_t = i8;
pub type __uint8_t = u8;
pub type __int16_t = i16;
pub type __uint16_t = u16;
pub type __int32_t = i32;
pub type __uint32_t = u32;
pub type int8_t = __int8_t;
pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Snes {
    pub cpu: *mut Cpu,
    pub apu: *mut Apu,
    pub ppu: *mut Ppu,
    pub snes_ppu: *mut Ppu,
    pub my_ppu: *mut Ppu,
    pub dma: *mut Dma,
    pub cart: *mut Cart,
    pub input1: *mut Input,
    pub input2: *mut Input,
    pub debug_cycles: bool,
    pub debug_apu_cycles: bool,
    pub disableRender: bool,
    pub runningWhichVersion: uint8_t,
    pub ramAdr: uint32_t,
    pub ram: *mut uint8_t,
    pub padx: [uint8_t; 4],
    pub hPos: uint16_t,
    pub vPos: uint16_t,
    pub frames: uint32_t,
    pub cpuCyclesLeft: uint8_t,
    pub cpuMemOps: uint8_t,
    pub padpad: [uint8_t; 2],
    pub apuCatchupCycles: ::core::ffi::c_double,
    pub hIrqEnabled: bool,
    pub vIrqEnabled: bool,
    pub nmiEnabled: bool,
    pub hTimer: uint16_t,
    pub vTimer: uint16_t,
    pub inNmi: bool,
    pub inIrq: bool,
    pub inVblank: bool,
    pub portAutoReadX: [uint16_t; 4],
    pub autoJoyRead: bool,
    pub autoJoyTimer: uint16_t,
    pub ppuLatch: bool,
    pub multiplyA: uint8_t,
    pub multiplyResult: uint16_t,
    pub divideA: uint16_t,
    pub divideResult: uint16_t,
    pub fastMem: bool,
    pub openBus: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub currentState: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cart {
    pub snes: *mut Snes,
    pub type_0: uint8_t,
    pub rom: *mut uint8_t,
    pub romSize: uint32_t,
    pub ram: *mut uint8_t,
    pub ramSize: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dma {
    pub snes: *mut Snes,
    pub channel: [DmaChannel; 8],
    pub hdmaTimer: uint16_t,
    pub dmaTimer: uint32_t,
    pub dmaBusy: bool,
    pub pad: [uint8_t; 7],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DmaChannel {
    pub bAdr: uint8_t,
    pub aAdr: uint16_t,
    pub aBank: uint8_t,
    pub size: uint16_t,
    pub indBank: uint8_t,
    pub tableAdr: uint16_t,
    pub repCount: uint8_t,
    pub unusedByte: uint8_t,
    pub dmaActive: bool,
    pub hdmaActive: bool,
    pub mode: uint8_t,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub fromB: bool,
    pub unusedBit: bool,
    pub doTransfer: bool,
    pub terminated: bool,
    pub offIndex: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ppu {
    pub snes: *mut Snes,
    pub vram: [uint16_t; 32768],
    pub vramPointer: uint16_t,
    pub vramIncrementOnHigh: bool,
    pub vramIncrement: uint16_t,
    pub vramRemapMode: uint8_t,
    pub vramReadBuffer: uint16_t,
    pub cgram: [uint16_t; 256],
    pub cgramPointer: uint8_t,
    pub cgramSecondWrite: bool,
    pub cgramBuffer: uint8_t,
    pub oam: [uint16_t; 256],
    pub highOam: [uint8_t; 32],
    pub oamAdr: uint8_t,
    pub oamAdrWritten: uint8_t,
    pub oamInHigh: bool,
    pub oamInHighWritten: bool,
    pub oamSecondWrite: bool,
    pub oamBuffer: uint8_t,
    pub objPriority: bool,
    pub objTileAdr1: uint16_t,
    pub objTileAdr2: uint16_t,
    pub objSize: uint8_t,
    pub objPixelBufferXX: [uint8_t; 256],
    pub objPriorityBufferXX: [uint8_t; 256],
    pub timeOver: bool,
    pub rangeOver: bool,
    pub objInterlace: bool,
    pub bgLayer: [BgLayer; 4],
    pub scrollPrev: uint8_t,
    pub scrollPrev2: uint8_t,
    pub mosaicSize: uint8_t,
    pub mosaicStartLine: uint8_t,
    pub layer: [Layer; 5],
    pub m7matrix: [int16_t; 8],
    pub m7prev: uint8_t,
    pub m7largeField: bool,
    pub m7charFill: bool,
    pub m7xFlip: bool,
    pub m7yFlip: bool,
    pub m7extBg: bool,
    pub m7startX: int32_t,
    pub m7startY: int32_t,
    pub windowLayer: [WindowLayer; 6],
    pub window1left: uint8_t,
    pub window1right: uint8_t,
    pub window2left: uint8_t,
    pub window2right: uint8_t,
    pub clipMode: uint8_t,
    pub preventMathMode: uint8_t,
    pub addSubscreen: bool,
    pub subtractColor: bool,
    pub halfColor: bool,
    pub mathEnabled: [bool; 6],
    pub fixedColorR: uint8_t,
    pub fixedColorG: uint8_t,
    pub fixedColorB: uint8_t,
    pub forcedBlank: bool,
    pub brightness: uint8_t,
    pub mode: uint8_t,
    pub bg3priority: bool,
    pub evenFrame: bool,
    pub pseudoHires: bool,
    pub overscan: bool,
    pub frameOverscan: bool,
    pub interlace: bool,
    pub frameInterlace: bool,
    pub directColor: bool,
    pub hCount: uint16_t,
    pub vCount: uint16_t,
    pub hCountSecond: bool,
    pub vCountSecond: bool,
    pub countersLatched: bool,
    pub ppu1openBus: uint8_t,
    pub ppu2openBus: uint8_t,
    pub pixelbuffer_placeholder: uint8_t,
    pub windowsel: uint32_t,
    pub extraLeftCur: uint8_t,
    pub extraRightCur: uint8_t,
    pub extraLeftRight: uint8_t,
    pub screenEnabled: [uint8_t; 2],
    pub screenWindowed: [uint8_t; 2],
    pub mosaicEnabled: uint8_t,
    pub lastBrightnessMult: uint8_t,
    pub lineHasSprites: bool,
    pub bgBuffers: [PpuPixelPrioBufs; 2],
    pub objBuffer: PpuPixelPrioBufs,
    pub renderPitch: uint32_t,
    pub renderBuffer: *mut uint8_t,
    pub brightnessMult: [uint8_t; 63],
    pub brightnessMultHalf: [uint8_t; 64],
    pub mosaicModulo: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuPixelPrioBufs {
    pub data: [PpuZbufType; 256],
}
pub type PpuZbufType = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WindowLayer {
    pub window1enabled: bool,
    pub window2enabled: bool,
    pub window1inversed: bool,
    pub window2inversed: bool,
    pub maskLogic: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Layer {
    pub mainScreenEnabled: bool,
    pub subScreenEnabled: bool,
    pub mainScreenWindowed: bool,
    pub subScreenWindowed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BgLayer {
    pub hScroll: uint16_t,
    pub vScroll: uint16_t,
    pub tilemapWider: bool,
    pub tilemapHigher: bool,
    pub tilemapAdr: uint16_t,
    pub tileAdr: uint16_t,
    pub bigTiles: bool,
    pub mosaicEnabled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: [uint8_t; 65536],
    pub romReadable: bool,
    pub dspAdr: uint8_t,
    pub cycles: uint32_t,
    pub inPorts: [uint8_t; 6],
    pub outPorts: [uint8_t; 4],
    pub timer: [Timer; 3],
    pub cpuCyclesLeft: uint8_t,
    pub pad: [uint8_t; 6],
    pub c2rust_unnamed: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub hist: DspRegWriteHistory,
    pub padpad: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: uint32_t,
    pub addr: [uint8_t; 256],
    pub val: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub cycles: uint8_t,
    pub divider: uint8_t,
    pub target: uint8_t,
    pub counter: uint8_t,
    pub enabled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dsp {
    pub apu_ram: *mut uint8_t,
    pub ram: [uint8_t; 128],
    pub channel: [DspChannel; 8],
    pub dirPage: uint16_t,
    pub evenCycle: bool,
    pub mute: bool,
    pub reset: bool,
    pub masterVolumeL: int8_t,
    pub masterVolumeR: int8_t,
    pub noiseSample: int16_t,
    pub noiseRate: uint16_t,
    pub noiseCounter: uint16_t,
    pub echoWrites: bool,
    pub echoVolumeL: int8_t,
    pub echoVolumeR: int8_t,
    pub feedbackVolume: int8_t,
    pub echoBufferAdr: uint16_t,
    pub echoDelay: uint16_t,
    pub echoRemain: uint16_t,
    pub echoBufferIndex: uint16_t,
    pub firBufferIndex: uint8_t,
    pub firValues: [int8_t; 8],
    pub firBufferL: [int16_t; 8],
    pub firBufferR: [int16_t; 8],
    pub sampleBuffer: [int16_t; 1068],
    pub sampleOffset: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspChannel {
    pub pitch: uint16_t,
    pub pitchCounter: uint16_t,
    pub pitchModulation: bool,
    pub decodeBuffer: [int16_t; 19],
    pub srcn: uint8_t,
    pub decodeOffset: uint16_t,
    pub previousFlags: uint8_t,
    pub old: int16_t,
    pub older: int16_t,
    pub useNoise: bool,
    pub adsrRates: [uint16_t; 4],
    pub rateCounter: uint16_t,
    pub adsrState: uint8_t,
    pub sustainLevel: uint16_t,
    pub useGain: bool,
    pub gainMode: uint8_t,
    pub directGain: bool,
    pub gainValue: uint16_t,
    pub gain: uint16_t,
    pub keyOn: bool,
    pub keyOff: bool,
    pub sampleOut: int16_t,
    pub volumeL: int8_t,
    pub volumeR: int8_t,
    pub echoEnable: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Spc {
    pub apu: *mut Apu,
    pub a: uint8_t,
    pub x: uint8_t,
    pub y: uint8_t,
    pub sp: uint8_t,
    pub pc: uint16_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cpu {
    pub mem: *mut ::core::ffi::c_void,
    pub memType: uint8_t,
    pub a: uint16_t,
    pub x: uint16_t,
    pub y: uint16_t,
    pub sp: uint16_t,
    pub pc: uint16_t,
    pub dp: uint16_t,
    pub k: uint8_t,
    pub db: uint8_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub d: bool,
    pub xf: bool,
    pub mf: bool,
    pub e: bool,
    pub irqWanted: bool,
    pub nmiWanted: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
    pub spBreakpoint: uint16_t,
    pub in_emu: bool,
}
pub type SaveLoadFunc =
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t) -> ();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut snes_frame_counter: ::core::ffi::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn snes_init(mut ram: *mut uint8_t) -> *mut Snes {
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
#[no_mangle]
pub unsafe extern "C" fn snes_free(mut snes: *mut Snes) {
    cpu_free((*snes).cpu);
    apu_free((*snes).apu);
    dma_free((*snes).dma);
    ppu_free((*snes).ppu);
    cart_free((*snes).cart);
    input_free((*snes).input1);
    input_free((*snes).input2);
    free(snes as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn snes_saveload(
    mut snes: *mut Snes,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    cpu_saveload((*snes).cpu, func, ctx);
    apu_saveload((*snes).apu, func, ctx);
    dma_saveload((*snes).dma, func, ctx);
    ppu_saveload((*snes).ppu, func, ctx);
    cart_saveload((*snes).cart, func, ctx);
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*snes).hPos as *mut ::core::ffi::c_void,
        (145 as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_sub(92 as size_t),
    );
    func.expect("non-null function pointer")(
        ctx,
        (*snes).ram as *mut ::core::ffi::c_void,
        0x20000 as ::core::ffi::c_int as size_t,
    );
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*snes).ramAdr as *mut ::core::ffi::c_void,
        4 as size_t,
    );
    (*snes).runningWhichVersion = 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn snes_reset(mut snes: *mut Snes, mut hard: bool) {
    cart_reset((*snes).cart);
    cpu_reset((*snes).cpu);
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
#[no_mangle]
pub unsafe extern "C" fn snes_handle_pos_stuff(mut snes: *mut Snes) {
    if (*snes).hPos as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut startingVblank: bool = false_0 != 0;
        if (*snes).vPos as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*snes).inVblank = false_0 != 0;
            (*snes).inNmi = false_0 != 0;
            dma_initHdma((*snes).dma);
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
            ppu_runLine((*snes).ppu, (*snes).vPos as ::core::ffi::c_int);
        }
    } else if (*snes).hPos as ::core::ffi::c_int == 1024 as ::core::ffi::c_int {
        if !(*snes).inVblank {
            dma_doHdma((*snes).dma);
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
#[no_mangle]
pub unsafe extern "C" fn snes_catchupApu(mut snes: *mut Snes) {
    if (*snes).apuCatchupCycles > 10000 as ::core::ffi::c_int as ::core::ffi::c_double {
        (*snes).apuCatchupCycles = 10000 as ::core::ffi::c_int as ::core::ffi::c_double;
    }
    let mut catchupCycles: ::core::ffi::c_int = (*snes).apuCatchupCycles as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < catchupCycles {
        apu_cycle((*snes).apu);
        i += 1;
    }
    (*snes).apuCatchupCycles -= catchupCycles as ::core::ffi::c_double;
}
#[no_mangle]
pub unsafe extern "C" fn snes_readBBus(mut snes: *mut Snes, mut adr: uint8_t) -> uint8_t {
    if (adr as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int {
        return ppu_read((*snes).ppu, adr);
    }
    if (adr as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        '_c2rust_label: {
            __assert_fail(
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/snes/snes.c\0" as *const u8 as *const ::core::ffi::c_char,
                186 as ::core::ffi::c_uint,
                b"uint8_t snes_readBBus(Snes *, uint8_t)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
        (*snes).apuCatchupCycles = 32 as ::core::ffi::c_int as ::core::ffi::c_double;
        snes_catchupApu(snes);
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
#[no_mangle]
pub unsafe extern "C" fn snes_writeBBus(mut snes: *mut Snes, mut adr: uint8_t, mut val: uint8_t) {
    if (adr as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int {
        ppu_write((*snes).ppu, adr, val);
        return;
    }
    if (adr as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        RtlApuWrite(
            (0x2100 as ::core::ffi::c_int + adr as ::core::ffi::c_int) as uint32_t,
            val,
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
unsafe extern "C" fn SwapInputBits(mut x: uint16_t) -> uint16_t {
    let mut r: uint16_t = 0 as uint16_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        r = (r as ::core::ffi::c_int * 2 as ::core::ffi::c_int
            + (x as ::core::ffi::c_int & 1 as ::core::ffi::c_int)) as uint16_t;
        i += 1;
        x = (x as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16_t;
    }
    return r;
}
unsafe extern "C" fn snes_readReg(mut snes: *mut Snes, mut adr: uint16_t) -> uint8_t {
    match adr as ::core::ffi::c_int {
        16912 => {
            let mut val: uint8_t = 0x2 as uint8_t;
            val = (val as ::core::ffi::c_int
                | ((*snes).inNmi as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
            return (val as ::core::ffi::c_int
                | (*snes).openBus as ::core::ffi::c_int & 0x70 as ::core::ffi::c_int)
                as uint8_t;
        }
        16913 => {
            let mut val_0: uint8_t =
                (((*snes).inIrq as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8_t;
            (*snes).inIrq = false_0 != 0;
            (*(*snes).cpu).irqWanted = false_0 != 0;
            return (val_0 as ::core::ffi::c_int
                | (*snes).openBus as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
                as uint8_t;
        }
        16914 => {
            let mut val_1: uint8_t = ((*snes).autoJoyTimer as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int as uint8_t;
            val_1 = (val_1 as ::core::ffi::c_int
                | (((*snes).hPos as ::core::ffi::c_int >= 1024 as ::core::ffi::c_int)
                    as ::core::ffi::c_int)
                    << 6 as ::core::ffi::c_int) as uint8_t;
            val_1 = (val_1 as ::core::ffi::c_int
                | ((*snes).inVblank as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
            return (val_1 as ::core::ffi::c_int
                | (*snes).openBus as ::core::ffi::c_int & 0x3e as ::core::ffi::c_int)
                as uint8_t;
        }
        16915 => {
            return (((*snes).ppuLatch as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
        }
        16916 => {
            return ((*snes).divideResult as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                as uint8_t;
        }
        16917 => {
            return ((*snes).divideResult as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                as uint8_t;
        }
        16918 => {
            return ((*snes).multiplyResult as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
                as uint8_t;
        }
        16919 => {
            return ((*snes).multiplyResult as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                as uint8_t;
        }
        16920 => {
            return (SwapInputBits((*(*snes).input1).currentState) as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as uint8_t;
        }
        16921 => {
            return (SwapInputBits((*(*snes).input1).currentState) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        16922 | 16924 | 16926 | 16923 | 16925 | 16927 => return 0 as uint8_t,
        _ => return (*snes).openBus,
    };
}
unsafe extern "C" fn snes_writeReg(mut snes: *mut Snes, mut adr: uint16_t, mut val: uint8_t) {
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
            (*snes).multiplyResult =
                ((*snes).multiplyA as ::core::ffi::c_int * val as ::core::ffi::c_int) as uint16_t;
        }
        16900 => {
            (*snes).divideA = ((*snes).divideA as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                | val as ::core::ffi::c_int) as uint16_t;
        }
        16901 => {
            (*snes).divideA = ((*snes).divideA as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
                | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
        }
        16902 => {
            if val as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*snes).divideResult = 0xffff as uint16_t;
                (*snes).multiplyResult = (*snes).divideA;
            } else {
                (*snes).divideResult =
                    ((*snes).divideA as ::core::ffi::c_int / val as ::core::ffi::c_int) as uint16_t;
                (*snes).multiplyResult =
                    ((*snes).divideA as ::core::ffi::c_int % val as ::core::ffi::c_int) as uint16_t;
            }
        }
        16903 => {
            (*snes).hTimer = ((*snes).hTimer as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int
                | val as ::core::ffi::c_int) as uint16_t;
        }
        16904 => {
            (*snes).hTimer = ((*snes).hTimer as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
                | (val as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
        }
        16905 => {
            (*snes).vTimer = ((*snes).vTimer as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int
                | val as ::core::ffi::c_int) as uint16_t;
        }
        16906 => {
            (*snes).vTimer = ((*snes).vTimer as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
                | (val as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
        }
        16907 => {
            if val as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                let mut t: uint32_t = (((*(*snes).dma).channel[1 as ::core::ffi::c_int as usize]
                    .aBank as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int
                    | (*(*snes).dma).channel[1 as ::core::ffi::c_int as usize].aAdr
                        as ::core::ffi::c_int) as uint32_t;
                let mut data: ::core::ffi::c_int = snes_read(snes, t) as ::core::ffi::c_int
                    | (snes_read(snes, t.wrapping_add(1 as uint32_t)) as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int;
            }
            dma_startDma((*snes).dma, val, false_0 != 0);
            while dma_cycle((*snes).dma) {}
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
#[no_mangle]
pub unsafe extern "C" fn snes_read(mut snes: *mut Snes, mut adr: uint32_t) -> uint8_t {
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
            return snes_readBBus(snes, (adr & 0xff as uint32_t) as uint8_t);
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
    return cart_read((*snes).cart, bank, adr as uint16_t);
}
#[no_mangle]
pub unsafe extern "C" fn LogWrite(mut snes: *mut Snes, mut adr: uint32_t, mut val: uint8_t) {
    printf(
        b"@%d: Write to 0x%x = 0x%.2x: 0x%x: r18=0x%x: r20=0x%x: a = 0x%x, x = 0x%x, y = 0x%x, c = %d\n\0"
            as *const u8 as *const ::core::ffi::c_char,
        snes_frame_counter,
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
#[no_mangle]
pub unsafe extern "C" fn snes_write(mut snes: *mut Snes, mut adr: uint32_t, mut val: uint8_t) {
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
            LogWrite(snes, adr, val);
        }
    }
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
            && (bank as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int
    {
        if adr < 0x2000 as uint32_t {
            *(*snes).ram.offset(adr as isize) = val;
            if adr == 0xfffff as uint32_t {
                LogWrite(snes, adr, val);
            }
        }
        if adr >= 0x2100 as uint32_t && adr < 0x2200 as uint32_t {
            snes_writeBBus(snes, (adr & 0xff as uint32_t) as uint8_t, val);
        }
        if adr >= 0x4200 as uint32_t && adr < 0x4220 as uint32_t {
            snes_writeReg(snes, adr as uint16_t, val);
        }
        if adr >= 0x4300 as uint32_t && adr < 0x4380 as uint32_t {
            dma_write((*snes).dma, adr as uint16_t, val);
        }
    }
    cart_write((*snes).cart, bank, adr as uint16_t, val);
}
#[no_mangle]
pub unsafe extern "C" fn snes_cpuRead(mut snes: *mut Snes, mut adr: uint32_t) -> uint8_t {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft =
        ((*snes).cpuCyclesLeft as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint8_t;
    return snes_read(snes, adr);
}
#[no_mangle]
pub unsafe extern "C" fn snes_cpuWrite(mut snes: *mut Snes, mut adr: uint32_t, mut val: uint8_t) {
    (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
    (*snes).cpuCyclesLeft =
        ((*snes).cpuCyclesLeft as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint8_t;
    snes_write(snes, adr, val);
}
