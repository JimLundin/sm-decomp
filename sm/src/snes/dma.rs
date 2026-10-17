extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snes_readBBus(snes: *mut Snes, adr: uint8_t) -> uint8_t;
    fn snes_writeBBus(snes: *mut Snes, adr: uint8_t, val: uint8_t);
    fn snes_read(snes: *mut Snes, adr: uint32_t) -> uint8_t;
    fn snes_write(snes: *mut Snes, adr: uint32_t, val: uint8_t);
    static mut g_fail: bool;
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"uint8_t dma_read(Dma *, uint16_t)\0",
    )
};
static mut bAdrOffsets: [[::core::ffi::c_int; 4]; 8] = [
    [
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ],
    [
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ],
];
static mut transferLength: [::core::ffi::c_int; 8] = [
    1 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
];
#[no_mangle]
pub unsafe extern "C" fn dma_init(mut snes: *mut Snes) -> *mut Dma {
    let mut dma: *mut Dma = malloc(::core::mem::size_of::<Dma>() as size_t) as *mut Dma;
    (*dma).snes = snes;
    return dma;
}
#[no_mangle]
pub unsafe extern "C" fn dma_free(mut dma: *mut Dma) {
    free(dma as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dma_reset(mut dma: *mut Dma) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        (*dma).channel[i as usize].bAdr = 0xff as uint8_t;
        (*dma).channel[i as usize].aAdr = 0xffff as uint16_t;
        (*dma).channel[i as usize].aBank = 0xff as uint8_t;
        (*dma).channel[i as usize].size = 0xffff as uint16_t;
        (*dma).channel[i as usize].indBank = 0xff as uint8_t;
        (*dma).channel[i as usize].tableAdr = 0xffff as uint16_t;
        (*dma).channel[i as usize].repCount = 0xff as uint8_t;
        (*dma).channel[i as usize].unusedByte = 0xff as uint8_t;
        (*dma).channel[i as usize].dmaActive = false_0 != 0;
        (*dma).channel[i as usize].hdmaActive = false_0 != 0;
        (*dma).channel[i as usize].mode = 7 as uint8_t;
        (*dma).channel[i as usize].fixed = true_0 != 0;
        (*dma).channel[i as usize].decrement = true_0 != 0;
        (*dma).channel[i as usize].indirect = true_0 != 0;
        (*dma).channel[i as usize].fromB = true_0 != 0;
        (*dma).channel[i as usize].unusedBit = true_0 != 0;
        (*dma).channel[i as usize].doTransfer = false_0 != 0;
        (*dma).channel[i as usize].terminated = false_0 != 0;
        (*dma).channel[i as usize].offIndex = 0 as uint8_t;
        i += 1;
    }
    (*dma).hdmaTimer = 0 as uint16_t;
    (*dma).dmaTimer = 0 as uint32_t;
    (*dma).dmaBusy = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn dma_saveload(
    mut dma: *mut Dma,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*dma).channel as *mut ::core::ffi::c_void,
        (225 as size_t)
            .wrapping_add(7 as size_t)
            .wrapping_sub(8 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn dma_read(mut dma: *mut Dma, mut adr: uint16_t) -> uint8_t {
    let mut c: uint8_t = ((adr as ::core::ffi::c_int & 0x70 as ::core::ffi::c_int)
        >> 4 as ::core::ffi::c_int) as uint8_t;
    match adr as ::core::ffi::c_int & 0xf as ::core::ffi::c_int {
        0 => {
            let mut val: uint8_t = (*dma).channel[c as usize].mode;
            val = (val as ::core::ffi::c_int
                | ((*dma).channel[c as usize].fixed as ::core::ffi::c_int)
                    << 3 as ::core::ffi::c_int) as uint8_t;
            val = (val as ::core::ffi::c_int
                | ((*dma).channel[c as usize].decrement as ::core::ffi::c_int)
                    << 4 as ::core::ffi::c_int) as uint8_t;
            val = (val as ::core::ffi::c_int
                | ((*dma).channel[c as usize].unusedBit as ::core::ffi::c_int)
                    << 5 as ::core::ffi::c_int) as uint8_t;
            val = (val as ::core::ffi::c_int
                | ((*dma).channel[c as usize].indirect as ::core::ffi::c_int)
                    << 6 as ::core::ffi::c_int) as uint8_t;
            val = (val as ::core::ffi::c_int
                | ((*dma).channel[c as usize].fromB as ::core::ffi::c_int)
                    << 7 as ::core::ffi::c_int) as uint8_t;
            return val;
        }
        1 => return (*dma).channel[c as usize].bAdr,
        2 => {
            return ((*dma).channel[c as usize].aAdr as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as uint8_t;
        }
        3 => {
            return ((*dma).channel[c as usize].aAdr as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        4 => return (*dma).channel[c as usize].aBank,
        5 => {
            return ((*dma).channel[c as usize].size as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as uint8_t;
        }
        6 => {
            return ((*dma).channel[c as usize].size as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        7 => return (*dma).channel[c as usize].indBank,
        8 => {
            return ((*dma).channel[c as usize].tableAdr as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as uint8_t;
        }
        9 => {
            return ((*dma).channel[c as usize].tableAdr as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        10 => return (*dma).channel[c as usize].repCount,
        11 | 15 => return (*dma).channel[c as usize].unusedByte,
        _ => {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/dma.c\0" as *const u8 as *const ::core::ffi::c_char,
                    118 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
            return (*(*dma).snes).openBus;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn dma_write(mut dma: *mut Dma, mut adr: uint16_t, mut val: uint8_t) {
    let mut c: uint8_t = ((adr as ::core::ffi::c_int & 0x70 as ::core::ffi::c_int)
        >> 4 as ::core::ffi::c_int) as uint8_t;
    match adr as ::core::ffi::c_int & 0xf as ::core::ffi::c_int {
        0 => {
            (*dma).channel[c as usize].mode =
                (val as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int) as uint8_t;
            (*dma).channel[c as usize].fixed =
                val as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0;
            (*dma).channel[c as usize].decrement =
                val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
            (*dma).channel[c as usize].unusedBit =
                val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
            (*dma).channel[c as usize].indirect =
                val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
            (*dma).channel[c as usize].fromB =
                val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
        }
        1 => {
            (*dma).channel[c as usize].bAdr = val;
        }
        2 => {
            (*dma).channel[c as usize].aAdr =
                ((*dma).channel[c as usize].aAdr as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int
                    | val as ::core::ffi::c_int) as uint16_t;
        }
        3 => {
            (*dma).channel[c as usize].aAdr =
                ((*dma).channel[c as usize].aAdr as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
                    | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    as uint16_t;
        }
        4 => {
            (*dma).channel[c as usize].aBank = val;
        }
        5 => {
            (*dma).channel[c as usize].size =
                ((*dma).channel[c as usize].size as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int
                    | val as ::core::ffi::c_int) as uint16_t;
        }
        6 => {
            (*dma).channel[c as usize].size =
                ((*dma).channel[c as usize].size as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
                    | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    as uint16_t;
        }
        7 => {
            (*dma).channel[c as usize].indBank = val;
        }
        8 => {
            (*dma).channel[c as usize].tableAdr =
                ((*dma).channel[c as usize].tableAdr as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int
                    | val as ::core::ffi::c_int) as uint16_t;
        }
        9 => {
            (*dma).channel[c as usize].tableAdr = ((*dma).channel[c as usize].tableAdr
                as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int
                | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
        }
        10 => {
            (*dma).channel[c as usize].repCount = val;
        }
        11 | 15 => {
            (*dma).channel[c as usize].unusedByte = val;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn dma_doDma(mut dma: *mut Dma) {
    if (*dma).dmaTimer > 0 as uint32_t {
        (*dma).dmaTimer = (*dma).dmaTimer.wrapping_sub(2 as uint32_t);
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        if (*dma).channel[i as usize].dmaActive {
            break;
        }
        i += 1;
    }
    if i == 8 as ::core::ffi::c_int {
        (*dma).dmaBusy = false_0 != 0;
        return;
    }
    if !(*dma).channel[i as usize].fromB
        && (*dma).channel[i as usize].aBank as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
        && (*dma).channel[i as usize].aAdr as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        && !g_fail
    {
        printf(
            b"Warning! DMA from addr 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            ((*dma).channel[i as usize].aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*dma).channel[i as usize].aAdr as ::core::ffi::c_int,
        );
        g_fail = true_0 != 0;
    }
    let fresh0 = (*dma).channel[i as usize].offIndex;
    (*dma).channel[i as usize].offIndex = (*dma).channel[i as usize].offIndex.wrapping_add(1);
    dma_transferByte(
        dma,
        (*dma).channel[i as usize].aAdr,
        (*dma).channel[i as usize].aBank,
        ((*dma).channel[i as usize].bAdr as ::core::ffi::c_int
            + bAdrOffsets[(*dma).channel[i as usize].mode as usize][fresh0 as usize])
            as uint8_t,
        (*dma).channel[i as usize].fromB,
    );
    (*dma).channel[i as usize].offIndex = ((*dma).channel[i as usize].offIndex
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as uint8_t;
    (*dma).dmaTimer = (*dma).dmaTimer.wrapping_add(6 as uint32_t);
    if !(*dma).channel[i as usize].fixed {
        (*dma).channel[i as usize].aAdr = ((*dma).channel[i as usize].aAdr as ::core::ffi::c_int
            + if (*dma).channel[i as usize].decrement as ::core::ffi::c_int != 0 {
                -(1 as ::core::ffi::c_int)
            } else {
                1 as ::core::ffi::c_int
            }) as uint16_t;
    }
    (*dma).channel[i as usize].size = (*dma).channel[i as usize].size.wrapping_sub(1);
    if (*dma).channel[i as usize].size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*dma).channel[i as usize].offIndex = 0 as uint8_t;
        (*dma).channel[i as usize].dmaActive = false_0 != 0;
        (*dma).dmaTimer = (*dma).dmaTimer.wrapping_add(8 as uint32_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn dma_initHdma(mut dma: *mut Dma) {
    (*dma).hdmaTimer = 0 as uint16_t;
    let mut hdmaHappened: bool = false_0 != 0;
    let mut current_block_16: u64;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        if (*dma).channel[i as usize].hdmaActive {
            hdmaHappened = true_0 != 0;
            (*dma).channel[i as usize].dmaActive = false_0 != 0;
            (*dma).channel[i as usize].offIndex = 0 as uint8_t;
            (*dma).channel[i as usize].tableAdr = (*dma).channel[i as usize].aAdr;
            let fresh1 = (*dma).channel[i as usize].tableAdr;
            (*dma).channel[i as usize].tableAdr =
                (*dma).channel[i as usize].tableAdr.wrapping_add(1);
            (*dma).channel[i as usize].repCount = snes_read(
                (*dma).snes,
                (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int
                    | fresh1 as ::core::ffi::c_int) as uint32_t,
            );
            if (*dma).channel[i as usize].repCount as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                (*dma).channel[i as usize].terminated = true_0 != 0;
                current_block_16 = 12237857397564741460;
            } else {
                (*dma).hdmaTimer =
                    ((*dma).hdmaTimer as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16_t;
                if (*dma).channel[i as usize].indirect {
                    let fresh2 = (*dma).channel[i as usize].tableAdr;
                    (*dma).channel[i as usize].tableAdr =
                        (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                    (*dma).channel[i as usize].size = snes_read(
                        (*dma).snes,
                        (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int
                            | fresh2 as ::core::ffi::c_int) as uint32_t,
                    ) as uint16_t;
                    let fresh3 = (*dma).channel[i as usize].tableAdr;
                    (*dma).channel[i as usize].tableAdr =
                        (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                    (*dma).channel[i as usize].size =
                        ((*dma).channel[i as usize].size as ::core::ffi::c_int
                            | (snes_read(
                                (*dma).snes,
                                (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                                    << 16 as ::core::ffi::c_int
                                    | fresh3 as ::core::ffi::c_int)
                                    as uint32_t,
                            ) as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int) as uint16_t;
                    (*dma).hdmaTimer = ((*dma).hdmaTimer as ::core::ffi::c_int
                        + 16 as ::core::ffi::c_int)
                        as uint16_t;
                }
                (*dma).channel[i as usize].doTransfer = true_0 != 0;
                current_block_16 = 7976072742316086414;
            }
        } else {
            (*dma).channel[i as usize].doTransfer = false_0 != 0;
            current_block_16 = 7976072742316086414;
        }
        match current_block_16 {
            7976072742316086414 => {
                (*dma).channel[i as usize].terminated = false_0 != 0;
            }
            _ => {}
        }
        i += 1;
    }
    if hdmaHappened {
        (*dma).hdmaTimer =
            ((*dma).hdmaTimer as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn dma_doHdma(mut dma: *mut Dma) {
    (*dma).hdmaTimer = 0 as uint16_t;
    let mut hdmaHappened: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        if (*dma).channel[i as usize].hdmaActive as ::core::ffi::c_int != 0
            && !(*dma).channel[i as usize].terminated
        {
            hdmaHappened = true_0 != 0;
            (*dma).channel[i as usize].dmaActive = false_0 != 0;
            (*dma).channel[i as usize].offIndex = 0 as uint8_t;
            (*dma).hdmaTimer =
                ((*dma).hdmaTimer as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16_t;
            if (*dma).channel[i as usize].doTransfer {
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < transferLength[(*dma).channel[i as usize].mode as usize] {
                    (*dma).hdmaTimer = ((*dma).hdmaTimer as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int)
                        as uint16_t;
                    if (*dma).channel[i as usize].indirect {
                        let fresh4 = (*dma).channel[i as usize].size;
                        (*dma).channel[i as usize].size =
                            (*dma).channel[i as usize].size.wrapping_add(1);
                        dma_transferByte(
                            dma,
                            fresh4,
                            (*dma).channel[i as usize].indBank,
                            ((*dma).channel[i as usize].bAdr as ::core::ffi::c_int
                                + bAdrOffsets[(*dma).channel[i as usize].mode as usize][j as usize])
                                as uint8_t,
                            (*dma).channel[i as usize].fromB,
                        );
                    } else {
                        let fresh5 = (*dma).channel[i as usize].tableAdr;
                        (*dma).channel[i as usize].tableAdr =
                            (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                        dma_transferByte(
                            dma,
                            fresh5,
                            (*dma).channel[i as usize].aBank,
                            ((*dma).channel[i as usize].bAdr as ::core::ffi::c_int
                                + bAdrOffsets[(*dma).channel[i as usize].mode as usize][j as usize])
                                as uint8_t,
                            (*dma).channel[i as usize].fromB,
                        );
                    }
                    j += 1;
                }
            }
            (*dma).channel[i as usize].repCount =
                (*dma).channel[i as usize].repCount.wrapping_sub(1);
            (*dma).channel[i as usize].doTransfer = (*dma).channel[i as usize].repCount
                as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                != 0;
            if (*dma).channel[i as usize].repCount as ::core::ffi::c_int
                & 0x7f as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                let fresh6 = (*dma).channel[i as usize].tableAdr;
                (*dma).channel[i as usize].tableAdr =
                    (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                (*dma).channel[i as usize].repCount = snes_read(
                    (*dma).snes,
                    (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int
                        | fresh6 as ::core::ffi::c_int) as uint32_t,
                );
                if (*dma).channel[i as usize].indirect {
                    let fresh7 = (*dma).channel[i as usize].tableAdr;
                    (*dma).channel[i as usize].tableAdr =
                        (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                    (*dma).channel[i as usize].size = snes_read(
                        (*dma).snes,
                        (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int
                            | fresh7 as ::core::ffi::c_int) as uint32_t,
                    ) as uint16_t;
                    let fresh8 = (*dma).channel[i as usize].tableAdr;
                    (*dma).channel[i as usize].tableAdr =
                        (*dma).channel[i as usize].tableAdr.wrapping_add(1);
                    (*dma).channel[i as usize].size =
                        ((*dma).channel[i as usize].size as ::core::ffi::c_int
                            | (snes_read(
                                (*dma).snes,
                                (((*dma).channel[i as usize].aBank as ::core::ffi::c_int)
                                    << 16 as ::core::ffi::c_int
                                    | fresh8 as ::core::ffi::c_int)
                                    as uint32_t,
                            ) as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int) as uint16_t;
                    (*dma).hdmaTimer = ((*dma).hdmaTimer as ::core::ffi::c_int
                        + 16 as ::core::ffi::c_int)
                        as uint16_t;
                }
                if (*dma).channel[i as usize].repCount as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    (*dma).channel[i as usize].terminated = true_0 != 0;
                }
                (*dma).channel[i as usize].doTransfer = true_0 != 0;
            }
        }
        i += 1;
    }
    if hdmaHappened {
        (*dma).hdmaTimer =
            ((*dma).hdmaTimer as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe extern "C" fn dma_transferByte(
    mut dma: *mut Dma,
    mut aAdr: uint16_t,
    mut aBank: uint8_t,
    mut bAdr: uint8_t,
    mut fromB: bool,
) {
    if fromB {
        snes_write(
            (*dma).snes,
            ((aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | aAdr as ::core::ffi::c_int)
                as uint32_t,
            snes_readBBus((*dma).snes, bAdr),
        );
    } else {
        snes_writeBBus(
            (*dma).snes,
            bAdr,
            snes_read(
                (*dma).snes,
                ((aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                    | aAdr as ::core::ffi::c_int) as uint32_t,
            ),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn dma_cycle(mut dma: *mut Dma) -> bool {
    if (*dma).hdmaTimer as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        (*dma).hdmaTimer =
            ((*dma).hdmaTimer as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16_t;
        return true_0 != 0;
    } else if (*dma).dmaBusy {
        dma_doDma(dma);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn dma_startDma(mut dma: *mut Dma, mut val: uint8_t, mut hdma: bool) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        if hdma {
            (*dma).channel[i as usize].hdmaActive =
                val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i != 0;
        } else {
            (*dma).channel[i as usize].dmaActive =
                val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i != 0;
        }
        i += 1;
    }
    if !hdma {
        (*dma).dmaBusy = val != 0;
        (*dma).dmaTimer = (*dma).dmaTimer.wrapping_add(
            (if (*dma).dmaBusy as ::core::ffi::c_int != 0 {
                16 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as uint32_t,
        );
    }
}
