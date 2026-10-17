extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn Die(error: *const ::core::ffi::c_char);
    fn DumpCpuHistory();
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn cart_init(mut snes: *mut Snes) -> *mut Cart {
    let mut cart: *mut Cart = malloc(::core::mem::size_of::<Cart>() as size_t) as *mut Cart;
    (*cart).snes = snes;
    (*cart).type_0 = 0 as uint8_t;
    (*cart).rom = ::core::ptr::null_mut::<uint8_t>();
    (*cart).romSize = 0 as uint32_t;
    (*cart).ram = ::core::ptr::null_mut::<uint8_t>();
    (*cart).ramSize = 0 as uint32_t;
    return cart;
}
#[no_mangle]
pub unsafe extern "C" fn cart_free(mut cart: *mut Cart) {
    free(cart as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cart_reset(mut cart: *mut Cart) {}
#[no_mangle]
pub unsafe extern "C" fn cart_saveload(
    mut cart: *mut Cart,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        (*cart).ram as *mut ::core::ffi::c_void,
        (*cart).ramSize as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cart_load(
    mut cart: *mut Cart,
    mut type_0: ::core::ffi::c_int,
    mut rom: *mut uint8_t,
    mut romSize: ::core::ffi::c_int,
    mut ramSize: ::core::ffi::c_int,
) {
    (*cart).type_0 = type_0 as uint8_t;
    if !(*cart).rom.is_null() {
        free((*cart).rom as *mut ::core::ffi::c_void);
    }
    if !(*cart).ram.is_null() {
        free((*cart).ram as *mut ::core::ffi::c_void);
    }
    (*cart).rom = malloc(romSize as size_t) as *mut uint8_t;
    (*cart).romSize = romSize as uint32_t;
    if ramSize > 0 as ::core::ffi::c_int {
        (*cart).ram = malloc(ramSize as size_t) as *mut uint8_t;
        memset(
            (*cart).ram as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ramSize as size_t,
        );
    } else {
        (*cart).ram = ::core::ptr::null_mut::<uint8_t>();
    }
    (*cart).ramSize = ramSize as uint32_t;
    memcpy(
        (*cart).rom as *mut ::core::ffi::c_void,
        rom as *const ::core::ffi::c_void,
        romSize as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cart_read(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
) -> uint8_t {
    match (*cart).type_0 as ::core::ffi::c_int {
        0 => {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
                    57 as ::core::ffi::c_uint,
                    b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            };
            return (*(*cart).snes).openBus;
        }
        1 => return cart_readLorom(cart, bank, adr),
        2 => return cart_readHirom(cart, bank, adr),
        _ => {}
    }
    '_c2rust_label_0: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            62 as ::core::ffi::c_uint,
            b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    return (*(*cart).snes).openBus;
}
#[no_mangle]
pub unsafe extern "C" fn cart_write(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    match (*cart).type_0 as ::core::ffi::c_int {
        1 => {
            cart_writeLorom(cart, bank, adr, val);
        }
        2 => {
            cart_writeHirom(cart, bank, adr, val);
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn cart_readLorom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
) -> uint8_t {
    let original_bank = bank;
    if (bank as ::core::ffi::c_int >= 0x70 as ::core::ffi::c_int
        && (bank as ::core::ffi::c_int) < 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int)
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        return *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if adr as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
    {
        return *(*cart).rom.offset(
            (((bank as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).romSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    printf(
        b"While trying to read from 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
        (bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | adr as ::core::ffi::c_int,
    );
    let snes = &*(*cart).snes;
    let cpu = &*snes.cpu;
    eprintln!(
        "Invalid LoROM read ${original_bank:02X}:{adr:04X}, frame {}: PC ${:02X}:{:04X} A ${:04X} X ${:04X} Y ${:04X} SP ${:04X} DP ${:04X} DB ${:02X}",
        crate::rtl::snes_frame_counter,
        cpu.k, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp, cpu.db,
    );
    eprintln!(
        "CPU flags: N={} V={} M={} X={} D={} I={} Z={} C={} E={}",
        cpu.n, cpu.v, cpu.mf, cpu.xf, cpu.d, cpu.i, cpu.z, cpu.c, cpu.e,
    );
    let mut direct_page = [0u8; 13];
    for (index, value) in direct_page.iter_mut().enumerate() {
        let address = cpu.dp.wrapping_add(0x44 + index as u16);
        *value = *snes.ram.add(usize::from(address));
    }
    eprintln!("Direct page $44..$50: {direct_page:02X?}");
    DumpCpuHistory();
    Die(b"The game crashed in cart_readLorom\0" as *const u8 as *const ::core::ffi::c_char);
    return (*(*cart).snes).openBus;
}
unsafe extern "C" fn cart_writeLorom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    if (bank as ::core::ffi::c_int >= 0x70 as ::core::ffi::c_int
        && (bank as ::core::ffi::c_int) < 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int > 0xf0 as ::core::ffi::c_int)
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        ) = val;
    }
}
unsafe extern "C" fn cart_readHirom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
) -> uint8_t {
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        && adr as ::core::ffi::c_int >= 0x6000 as ::core::ffi::c_int
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        return *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 13 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    if adr as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
    {
        return *(*cart).rom.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).romSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_uint,
            b"uint8_t cart_readHirom(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    return (*(*cart).snes).openBus;
}
unsafe extern "C" fn cart_writeHirom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        && adr as ::core::ffi::c_int >= 0x6000 as ::core::ffi::c_int
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 13 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        ) = val;
    }
}
