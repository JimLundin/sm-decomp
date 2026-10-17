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
    fn dsp_getSamples(dsp: *mut Dsp, sampleData: *mut int16_t, samplesPerFrame: ::core::ffi::c_int);
    fn cart_load(
        cart: *mut Cart,
        type_0: ::core::ffi::c_int,
        rom: *mut uint8_t,
        romSize: ::core::ffi::c_int,
        ramSize: ::core::ffi::c_int,
    );
    fn snes_reset(snes: *mut Snes, hard: bool);
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CartHeader {
    pub headerVersion: uint8_t,
    pub name: [::core::ffi::c_char; 22],
    pub speed: uint8_t,
    pub type_0: uint8_t,
    pub coprocessor: uint8_t,
    pub chips: uint8_t,
    pub romSize: uint32_t,
    pub ramSize: uint32_t,
    pub region: uint8_t,
    pub maker: uint8_t,
    pub version: uint8_t,
    pub checksumComplement: uint16_t,
    pub checksum: uint16_t,
    pub makerCode: [::core::ffi::c_char; 3],
    pub gameCode: [::core::ffi::c_char; 5],
    pub flashSize: uint32_t,
    pub exRamSize: uint32_t,
    pub specialVersion: uint8_t,
    pub exCoprocessor: uint8_t,
    pub score: int16_t,
    pub pal: bool,
    pub cartType: uint8_t,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn snes_loadRom(
    mut snes: *mut Snes,
    mut data: *const uint8_t,
    mut length: ::core::ffi::c_int,
) -> bool {
    if length < 0x8000 as ::core::ffi::c_int {
        printf(
            b"Failed to load rom: rom to small (%d bytes)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            length,
        );
        return false_0 != 0;
    }
    let mut headers: [CartHeader; 4] = [CartHeader {
        headerVersion: 0,
        name: [0; 22],
        speed: 0,
        type_0: 0,
        coprocessor: 0,
        chips: 0,
        romSize: 0,
        ramSize: 0,
        region: 0,
        maker: 0,
        version: 0,
        checksumComplement: 0,
        checksum: 0,
        makerCode: [0; 3],
        gameCode: [0; 5],
        flashSize: 0,
        exRamSize: 0,
        specialVersion: 0,
        exCoprocessor: 0,
        score: 0,
        pal: false,
        cartType: 0,
    }; 4];
    memset(
        &raw mut headers as *mut CartHeader as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[CartHeader; 4]>() as size_t,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        headers[i as usize].score = -(50 as ::core::ffi::c_int) as int16_t;
        i += 1;
    }
    if length >= 0x8000 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x7fc0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(0 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x8200 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x81c0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(1 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x10000 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0xffc0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(2 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x10200 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x101c0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(3 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    let mut max: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 4 as ::core::ffi::c_int {
        if headers[i_0 as usize].score as ::core::ffi::c_int > max {
            max = headers[i_0 as usize].score as ::core::ffi::c_int;
            used = i_0;
        }
        i_0 += 1;
    }
    if used & 1 as ::core::ffi::c_int != 0 {
        data = data.offset(0x200 as ::core::ffi::c_int as isize);
        length -= 0x200 as ::core::ffi::c_int;
    }
    if headers[used as usize].cartType as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
        printf(
            b"Failed to load rom: unsupported type (%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            headers[used as usize].cartType as ::core::ffi::c_int,
        );
        return false_0 != 0;
    }
    let mut newLength: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
    while !(length <= newLength) {
        newLength *= 2 as ::core::ffi::c_int;
    }
    let mut newData: *mut uint8_t = malloc(newLength as size_t) as *mut uint8_t;
    memcpy(
        newData as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        length as size_t,
    );
    let mut test: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while length != newLength {
        if length & test != 0 {
            memcpy(
                newData.offset(length as isize) as *mut ::core::ffi::c_void,
                newData.offset(length as isize).offset(-(test as isize))
                    as *const ::core::ffi::c_void,
                test as size_t,
            );
            length += test;
        }
        test *= 2 as ::core::ffi::c_int;
    }
    cart_load(
        (*snes).cart,
        headers[used as usize].cartType as ::core::ffi::c_int,
        newData,
        newLength,
        (if headers[used as usize].chips as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            headers[used as usize].ramSize
        } else {
            0 as uint32_t
        }) as ::core::ffi::c_int,
    );
    snes_reset(snes, true_0 != 0);
    free(newData as *mut ::core::ffi::c_void);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn snes_setSamples(
    mut snes: *mut Snes,
    mut sampleData: *mut int16_t,
    mut samplesPerFrame: ::core::ffi::c_int,
) {
    dsp_getSamples((*(*snes).apu).dsp, sampleData, samplesPerFrame);
}
unsafe extern "C" fn readHeader(
    mut data: *const uint8_t,
    mut length: ::core::ffi::c_int,
    mut location: ::core::ffi::c_int,
    mut header: *mut CartHeader,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 21 as ::core::ffi::c_int {
        let mut ch: uint8_t = *data.offset((location + i) as isize);
        if ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
            && (ch as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
        {
            (*header).name[i as usize] = ch as ::core::ffi::c_char;
        } else {
            (*header).name[i as usize] = '.' as i32 as ::core::ffi::c_char;
        }
        i += 1;
    }
    (*header).name[21 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
    (*header).speed = (*data.offset((location + 0x15 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint8_t;
    (*header).type_0 = (*data.offset((location + 0x15 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as uint8_t;
    (*header).coprocessor = (*data.offset((location + 0x16 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint8_t;
    (*header).chips = (*data.offset((location + 0x16 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as uint8_t;
    (*header).romSize = ((0x400 as ::core::ffi::c_int)
        << *data.offset((location + 0x17 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint32_t;
    (*header).ramSize = ((0x400 as ::core::ffi::c_int)
        << *data.offset((location + 0x18 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint32_t;
    (*header).region = *data.offset((location + 0x19 as ::core::ffi::c_int) as isize);
    (*header).maker = *data.offset((location + 0x1a as ::core::ffi::c_int) as isize);
    (*header).version = *data.offset((location + 0x1b as ::core::ffi::c_int) as isize);
    (*header).checksumComplement = (((*data.offset((location + 0x1d as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + *data.offset((location + 0x1c as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint16_t;
    (*header).checksum = (((*data.offset((location + 0x1f as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + *data.offset((location + 0x1e as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint16_t;
    (*header).headerVersion = 1 as uint8_t;
    if (*header).maker as ::core::ffi::c_int == 0x33 as ::core::ffi::c_int {
        (*header).headerVersion = 3 as uint8_t;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < 2 as ::core::ffi::c_int {
            let mut ch_0: uint8_t =
                *data.offset((location - 0x10 as ::core::ffi::c_int + i_0) as isize);
            if ch_0 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                && (ch_0 as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                (*header).makerCode[i_0 as usize] = ch_0 as ::core::ffi::c_char;
            } else {
                (*header).makerCode[i_0 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_0 += 1;
        }
        (*header).makerCode[2 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < 4 as ::core::ffi::c_int {
            let mut ch_1: uint8_t =
                *data.offset((location - 0xe as ::core::ffi::c_int + i_1) as isize);
            if ch_1 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                && (ch_1 as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                (*header).gameCode[i_1 as usize] = ch_1 as ::core::ffi::c_char;
            } else {
                (*header).gameCode[i_1 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_1 += 1;
        }
        (*header).gameCode[4 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        (*header).flashSize = ((0x400 as ::core::ffi::c_int)
            << *data.offset((location - 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            as uint32_t;
        (*header).exRamSize = ((0x400 as ::core::ffi::c_int)
            << *data.offset((location - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            as uint32_t;
        (*header).specialVersion = *data.offset((location - 2 as ::core::ffi::c_int) as isize);
        (*header).exCoprocessor = *data.offset((location - 1 as ::core::ffi::c_int) as isize);
    } else if *data.offset((location + 0x14 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*header).headerVersion = 2 as uint8_t;
        (*header).exCoprocessor = *data.offset((location - 1 as ::core::ffi::c_int) as isize);
    }
    (*header).pal = (*header).region as ::core::ffi::c_int >= 0x2 as ::core::ffi::c_int
        && (*header).region as ::core::ffi::c_int <= 0xc as ::core::ffi::c_int
        || (*header).region as ::core::ffi::c_int == 0x11 as ::core::ffi::c_int;
    (*header).cartType = (if location < 0x9000 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    }) as uint8_t;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    score += if (*header).speed as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        || (*header).speed as ::core::ffi::c_int == 3 as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(4 as ::core::ffi::c_int)
    };
    score += if (*header).type_0 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int
        || (*header).type_0 as ::core::ffi::c_int == 5 as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).coprocessor as ::core::ffi::c_int <= 5 as ::core::ffi::c_int
        || (*header).coprocessor as ::core::ffi::c_int >= 0xe as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).chips as ::core::ffi::c_int <= 6 as ::core::ffi::c_int
        || (*header).chips as ::core::ffi::c_int == 9 as ::core::ffi::c_int
        || (*header).chips as ::core::ffi::c_int == 0xa as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).region as ::core::ffi::c_int <= 0x14 as ::core::ffi::c_int {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).checksum as ::core::ffi::c_int
        + (*header).checksumComplement as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        8 as ::core::ffi::c_int
    } else {
        -(6 as ::core::ffi::c_int)
    };
    let mut resetVector: uint16_t = (*data.offset((location + 0x3c as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | (*data.offset((location + 0x3d as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16_t;
    score += if resetVector as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int {
        8 as ::core::ffi::c_int
    } else {
        -(20 as ::core::ffi::c_int)
    };
    let mut opcodeLoc: ::core::ffi::c_int = location + 0x40 as ::core::ffi::c_int
        - 0x8000 as ::core::ffi::c_int
        + (resetVector as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int);
    let mut opcode: uint8_t = 0xff as uint8_t;
    if opcodeLoc < length {
        opcode = *data.offset(opcodeLoc as isize);
    } else {
        score -= 14 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0x78 as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int
    {
        score += 6 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0x4c as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x5c as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x9c as ::core::ffi::c_int
    {
        score += 3 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0xff as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0xdb as ::core::ffi::c_int
    {
        score -= 6 as ::core::ffi::c_int;
    }
    (*header).score = score as int16_t;
}
