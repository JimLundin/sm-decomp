use crate::game_state::Game;

extern "C" {
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
}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint64_t, __uint8_t, int16_t,
    int32_t, int8_t, size_t, uint, uint16_t, uint32_t, uint64_t, uint8_t, Apu,
    ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, SaveLoadFunc, Snes, Spc,
    Timer, WindowLayer,
};
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const kPpuExtraLeftRight: C2RustUnnamed_0 = 0;
pub const kPpuXPixels: C2RustUnnamed_0 = 256;
pub use crate::types::{uint32, uint8};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuWindows {
    pub edges: [int16; 6],
    pub nr: uint8,
    pub bits: uint8,
}
pub use crate::types::int16;
pub const kWindow2Inversed: C2RustUnnamed_3 = 4;
pub const kWindow2Enabled: C2RustUnnamed_3 = 8;
pub const kWindow1Inversed: C2RustUnnamed_3 = 1;
pub const kWindow1Enabled: C2RustUnnamed_3 = 2;
pub use crate::types::uint16;
pub const kPaletteShift: C2RustUnnamed_1 = 8;
pub type C2RustUnnamed_1 = ::core::ffi::c_uint;
pub const kPaletteShift_0: C2RustUnnamed_2 = 6;
pub type C2RustUnnamed_2 = ::core::ffi::c_uint;
pub use crate::types::uint64;
pub type C2RustUnnamed_3 = ::core::ffi::c_uint;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"void PpuDrawBackgrounds(Ppu *, int, _Bool)\0",
    )
};
#[inline]
unsafe fn IntMin(mut a: ::core::ffi::c_int, mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if a < b { a } else { b };
}
#[inline]
unsafe fn IntMax(mut a: ::core::ffi::c_int, mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if a > b { a } else { b };
}
static layersPerMode: [[::core::ffi::c_int; 12]; 10] = [
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        2 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
];
static prioritysPerMode: [[::core::ffi::c_int; 12]; 10] = [
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        1 as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        3 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
];
static layerCountPerMode: [::core::ffi::c_int; 10] = [
    12 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
];
static bitDepthsPerMode: [[::core::ffi::c_int; 4]; 10] = [
    [
        2 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        8 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        8 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        8 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        4 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
    [
        8 as ::core::ffi::c_int,
        7 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    ],
];
static spriteSizes: [[::core::ffi::c_int; 2]; 8] = [
    [8 as ::core::ffi::c_int, 16 as ::core::ffi::c_int],
    [8 as ::core::ffi::c_int, 32 as ::core::ffi::c_int],
    [8 as ::core::ffi::c_int, 64 as ::core::ffi::c_int],
    [16 as ::core::ffi::c_int, 32 as ::core::ffi::c_int],
    [16 as ::core::ffi::c_int, 64 as ::core::ffi::c_int],
    [32 as ::core::ffi::c_int, 64 as ::core::ffi::c_int],
    [16 as ::core::ffi::c_int, 32 as ::core::ffi::c_int],
    [16 as ::core::ffi::c_int, 32 as ::core::ffi::c_int],
];
pub unsafe fn ppu_init(mut snes: *mut Snes) -> *mut Ppu {
    let mut ppu: *mut Ppu = malloc(::core::mem::size_of::<Ppu>() as size_t) as *mut Ppu;
    (*ppu).renderBuffer = core::ptr::null_mut();
    (*ppu).renderPitch = 0;
    (*ppu).snes = snes;
    return ppu;
}
pub unsafe fn ppu_free(mut ppu: *mut Ppu) {
    free(ppu as *mut ::core::ffi::c_void);
}
pub unsafe fn ppu_copy(mut ppu: *mut Ppu, mut ppu_src: *mut Ppu) {
    let mut snes: *mut Snes = (*ppu).snes;
    let mut pitch: size_t = (*ppu).renderPitch as size_t;
    let mut renderBuffer: *mut uint8_t = (*ppu).renderBuffer;
    memcpy(
        ppu as *mut ::core::ffi::c_void,
        ppu_src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<Ppu>() as size_t,
    );
    (*ppu).renderBuffer = renderBuffer;
    (*ppu).renderPitch = pitch as uint32_t;
    (*ppu).snes = snes;
}
pub unsafe fn ppu_reset(mut ppu: *mut Ppu) {
    let mut snes: *mut Snes = (*ppu).snes;
    let mut pitch: size_t = (*ppu).renderPitch as size_t;
    let mut renderBuffer: *mut uint8_t = (*ppu).renderBuffer;
    memset(
        ppu as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<Ppu>() as size_t,
    );
    (*ppu).renderBuffer = renderBuffer;
    (*ppu).renderPitch = pitch as uint32_t;
    (*ppu).snes = snes;
    (*ppu).vramPointer = 0 as uint16_t;
    (*ppu).vramIncrementOnHigh = false_0 != 0;
    (*ppu).vramIncrement = 1 as uint16_t;
    (*ppu).vramRemapMode = 0 as uint8_t;
    (*ppu).vramReadBuffer = 0 as uint16_t;
    memset(
        &raw mut (*ppu).cgram as *mut uint16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint16_t; 256]>() as size_t,
    );
    (*ppu).cgramPointer = 0 as uint8_t;
    (*ppu).cgramSecondWrite = false_0 != 0;
    (*ppu).cgramBuffer = 0 as uint8_t;
    memset(
        &raw mut (*ppu).oam as *mut uint16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint16_t; 256]>() as size_t,
    );
    memset(
        &raw mut (*ppu).highOam as *mut uint8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 32]>() as size_t,
    );
    (*ppu).oamAdr = 0 as uint8_t;
    (*ppu).oamAdrWritten = 0 as uint8_t;
    (*ppu).oamInHigh = false_0 != 0;
    (*ppu).oamInHighWritten = false_0 != 0;
    (*ppu).oamSecondWrite = false_0 != 0;
    (*ppu).oamBuffer = 0 as uint8_t;
    (*ppu).objPriority = false_0 != 0;
    (*ppu).objTileAdr1 = 0 as uint16_t;
    (*ppu).objTileAdr2 = 0 as uint16_t;
    (*ppu).objSize = 0 as uint8_t;
    (*ppu).timeOver = false_0 != 0;
    (*ppu).rangeOver = false_0 != 0;
    (*ppu).objInterlace = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*ppu).bgLayer[i as usize].hScroll = 0 as uint16_t;
        (*ppu).bgLayer[i as usize].vScroll = 0 as uint16_t;
        (*ppu).bgLayer[i as usize].tilemapWider = false_0 != 0;
        (*ppu).bgLayer[i as usize].tilemapHigher = false_0 != 0;
        (*ppu).bgLayer[i as usize].tilemapAdr = 0 as uint16_t;
        (*ppu).bgLayer[i as usize].tileAdr = 0 as uint16_t;
        (*ppu).bgLayer[i as usize].bigTiles = false_0 != 0;
        (*ppu).bgLayer[i as usize].mosaicEnabled = false_0 != 0;
        i += 1;
    }
    (*ppu).scrollPrev = 0 as uint8_t;
    (*ppu).scrollPrev2 = 0 as uint8_t;
    (*ppu).mosaicSize = 1 as uint8_t;
    (*ppu).mosaicStartLine = 1 as uint8_t;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 5 as ::core::ffi::c_int {
        (*ppu).layer[i_0 as usize].mainScreenEnabled = false_0 != 0;
        (*ppu).layer[i_0 as usize].subScreenEnabled = false_0 != 0;
        (*ppu).layer[i_0 as usize].mainScreenWindowed = false_0 != 0;
        (*ppu).layer[i_0 as usize].subScreenWindowed = false_0 != 0;
        i_0 += 1;
    }
    memset(
        &raw mut (*ppu).m7matrix as *mut int16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[int16_t; 8]>() as size_t,
    );
    (*ppu).m7prev = 0 as uint8_t;
    (*ppu).m7largeField = false_0 != 0;
    (*ppu).m7charFill = false_0 != 0;
    (*ppu).m7xFlip = false_0 != 0;
    (*ppu).m7yFlip = false_0 != 0;
    (*ppu).m7extBg = false_0 != 0;
    (*ppu).m7startX = 0 as ::core::ffi::c_int as int32_t;
    (*ppu).m7startY = 0 as ::core::ffi::c_int as int32_t;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < 6 as ::core::ffi::c_int {
        (*ppu).windowLayer[i_1 as usize].window1enabled = false_0 != 0;
        (*ppu).windowLayer[i_1 as usize].window2enabled = false_0 != 0;
        (*ppu).windowLayer[i_1 as usize].window1inversed = false_0 != 0;
        (*ppu).windowLayer[i_1 as usize].window2inversed = false_0 != 0;
        (*ppu).windowLayer[i_1 as usize].maskLogic = 0 as uint8_t;
        i_1 += 1;
    }
    (*ppu).window1left = 0 as uint8_t;
    (*ppu).window1right = 0 as uint8_t;
    (*ppu).window2left = 0 as uint8_t;
    (*ppu).window2right = 0 as uint8_t;
    (*ppu).clipMode = 0 as uint8_t;
    (*ppu).preventMathMode = 0 as uint8_t;
    (*ppu).addSubscreen = false_0 != 0;
    (*ppu).subtractColor = false_0 != 0;
    (*ppu).halfColor = false_0 != 0;
    memset(
        &raw mut (*ppu).mathEnabled as *mut bool as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 6]>() as size_t,
    );
    (*ppu).fixedColorR = 0 as uint8_t;
    (*ppu).fixedColorG = 0 as uint8_t;
    (*ppu).fixedColorB = 0 as uint8_t;
    (*ppu).forcedBlank = true_0 != 0;
    (*ppu).brightness = 0 as uint8_t;
    (*ppu).mode = 0 as uint8_t;
    (*ppu).bg3priority = false_0 != 0;
    (*ppu).evenFrame = false_0 != 0;
    (*ppu).pseudoHires = false_0 != 0;
    (*ppu).overscan = false_0 != 0;
    (*ppu).frameOverscan = false_0 != 0;
    (*ppu).interlace = false_0 != 0;
    (*ppu).frameInterlace = false_0 != 0;
    (*ppu).directColor = false_0 != 0;
    (*ppu).hCount = 0 as uint16_t;
    (*ppu).vCount = 0 as uint16_t;
    (*ppu).hCountSecond = false_0 != 0;
    (*ppu).vCountSecond = false_0 != 0;
    (*ppu).countersLatched = false_0 != 0;
    (*ppu).ppu1openBus = 0 as uint8_t;
    (*ppu).ppu2openBus = 0 as uint8_t;
}
pub unsafe fn ppu_saveload(
    mut ppu: *mut Ppu,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*ppu).vram as *mut ::core::ffi::c_void,
        (67313 as size_t).wrapping_sub(8 as size_t),
        game,
    );
}
// These duplicate renderer inputs are outside the legacy serialized PPU range.
pub fn PpuRestoreRenderRouting(ppu: &mut Ppu) {
    ppu.screenEnabled = [0; 2];
    ppu.screenWindowed = [0; 2];
    for (index, layer) in ppu.layer.iter().enumerate() {
        ppu.screenEnabled[0] |= u8::from(layer.mainScreenEnabled) << index;
        ppu.screenEnabled[1] |= u8::from(layer.subScreenEnabled) << index;
        ppu.screenWindowed[0] |= u8::from(layer.mainScreenWindowed) << index;
        ppu.screenWindowed[1] |= u8::from(layer.subScreenWindowed) << index;
    }
    ppu.windowsel = 0;
    for (index, window) in ppu.windowLayer.iter().enumerate() {
        let flags = u32::from(window.window1inversed)
            | (u32::from(window.window1enabled) << 1)
            | (u32::from(window.window2inversed) << 2)
            | (u32::from(window.window2enabled) << 3);
        ppu.windowsel |= flags << (index * 4);
    }
}
pub unsafe fn PpuBeginDrawing(
    mut ppu: *mut Ppu,
    mut pixels: *mut uint8_t,
    mut pitch: size_t,
    mut render_flags: uint32_t,
) {
    (*ppu).renderPitch = pitch as uint as uint32_t;
    (*ppu).renderBuffer = pixels;
}
pub unsafe fn ppu_checkOverscan(mut ppu: *mut Ppu) -> bool {
    (*ppu).frameOverscan = (*ppu).overscan;
    return (*ppu).frameOverscan;
}
pub unsafe fn ppu_handleVblank(mut ppu: *mut Ppu) {
    if !(*ppu).forcedBlank {
        (*ppu).oamAdr = (*ppu).oamAdrWritten;
        (*ppu).oamInHigh = (*ppu).oamInHighWritten;
        (*ppu).oamSecondWrite = false_0 != 0;
    }
    (*ppu).frameInterlace = (*ppu).interlace;
}
#[inline]
unsafe fn ClearBackdrop(mut buf: *mut PpuPixelPrioBufs) {
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[PpuZbufType; 256]>() as usize)
            .wrapping_div(::core::mem::size_of::<PpuZbufType>() as usize)
    {
        *((&raw mut (*buf).data as *mut PpuZbufType).offset(i as isize) as *mut PpuZbufType
            as *mut uint64) = 0x500050005000500 as uint64;
        i = i.wrapping_add(4 as size_t);
    }
}
pub unsafe fn ppu_runLine(mut ppu: *mut Ppu, mut line: ::core::ffi::c_int, game: &mut Game) {
    if line == 0 as ::core::ffi::c_int {
        (*ppu).mosaicStartLine = 1 as uint8_t;
        (*ppu).rangeOver = false_0 != 0;
        (*ppu).timeOver = false_0 != 0;
        (*ppu).evenFrame = !(*ppu).evenFrame;
    } else {
        if (*ppu).brightness as ::core::ffi::c_int
            != (*ppu).lastBrightnessMult as ::core::ffi::c_int
        {
            let mut ppu_brightness: uint8_t = (*ppu).brightness;
            (*ppu).lastBrightnessMult = ppu_brightness;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 32 as ::core::ffi::c_int {
                (*ppu).brightnessMult[i as usize] =
                    ((i << 3 as ::core::ffi::c_int | i >> 2 as ::core::ffi::c_int)
                        * ppu_brightness as ::core::ffi::c_int
                        / 15 as ::core::ffi::c_int) as uint8_t;
                (*ppu).brightnessMultHalf
                    [(i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
                    (*ppu).brightnessMult[i as usize];
                (*ppu).brightnessMultHalf[(i * 2 as ::core::ffi::c_int) as usize] = (*ppu)
                    .brightnessMultHalf
                    [(i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize];
                i += 1;
            }
            memset(
                (&raw mut (*ppu).brightnessMult as *mut uint8_t)
                    .offset(32 as ::core::ffi::c_int as isize) as *mut uint8_t
                    as *mut ::core::ffi::c_void,
                (*ppu).brightnessMult[31 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
                31 as size_t,
            );
        }
        ClearBackdrop(&raw mut (*ppu).objBuffer);
        (*ppu).lineHasSprites = !(*ppu).forcedBlank
            && ppu_evaluateSprites(ppu, line - 1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0;
        // EXTBG composition and enabled mosaic use the per-pixel path.
        let mosaic = (*ppu).mosaicSize > 1 && (*ppu).bgLayer.iter().any(|bg| bg.mosaicEnabled);
        if game.g_new_ppu && !((*ppu).mode == 7 && (*ppu).m7extBg) && !mosaic {
            PpuDrawWholeLine(ppu, line as uint);
        } else {
            if (*ppu).mode as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                ppu_calculateMode7Starts(ppu, line);
            }
            let mut x: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while x < 256 as ::core::ffi::c_int {
                ppu_handlePixel(ppu, x, line);
                x += 1;
            }
        }
    };
}
unsafe fn PpuWindows_Clear(mut win: *mut PpuWindows, mut ppu: *mut Ppu, mut layer: uint) {
    (*win).edges[0 as ::core::ffi::c_int as usize] = -if layer != 2 as uint {
        (*ppu).extraLeftCur as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    } as int16;
    (*win).edges[1 as ::core::ffi::c_int as usize] = (256 as ::core::ffi::c_int
        + (if layer != 2 as uint {
            (*ppu).extraRightCur as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as int16;
    (*win).nr = 1 as uint8;
    (*win).bits = 0 as uint8;
}
unsafe fn PpuWindows_Calc(mut win: *mut PpuWindows, mut ppu: *mut Ppu, mut layer: uint) {
    let mut winflags: uint32 = (*ppu).windowsel as uint32 >> layer.wrapping_mul(4 as uint);
    let mut nr: uint = 1 as uint;
    let mut window_right: ::core::ffi::c_int = 256 as ::core::ffi::c_int
        + (if layer != 2 as uint {
            (*ppu).extraRightCur as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    (*win).edges[0 as ::core::ffi::c_int as usize] = -if layer != 2 as uint {
        (*ppu).extraLeftCur as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    } as int16;
    (*win).edges[1 as ::core::ffi::c_int as usize] = window_right as int16;
    let mut i: uint = 0;
    let mut j: uint = 0;
    let mut t: ::core::ffi::c_int = 0;
    let mut w1_ena: bool = winflags & kWindow1Enabled as ::core::ffi::c_int as uint32 != 0
        && (*ppu).window1left as ::core::ffi::c_int <= (*ppu).window1right as ::core::ffi::c_int;
    if w1_ena {
        if (*ppu).window1left as ::core::ffi::c_int
            > (*win).edges[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        {
            (*win).edges[nr as usize] = (*ppu).window1left as int16;
            nr = nr.wrapping_add(1);
            (*win).edges[nr as usize] = window_right as int16;
        }
        if ((*ppu).window1right as ::core::ffi::c_int + 1 as ::core::ffi::c_int) < window_right {
            (*win).edges[nr as usize] =
                ((*ppu).window1right as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
            nr = nr.wrapping_add(1);
            (*win).edges[nr as usize] = window_right as int16;
        }
    }
    let mut w2_ena: bool = winflags & kWindow2Enabled as ::core::ffi::c_int as uint32 != 0
        && (*ppu).window2left as ::core::ffi::c_int <= (*ppu).window2right as ::core::ffi::c_int;
    if w2_ena {
        i = 0 as uint;
        while i <= nr && {
            t = (*ppu).window2left as ::core::ffi::c_int;
            t != (*win).edges[i as usize] as ::core::ffi::c_int
        } {
            if t < (*win).edges[i as usize] as ::core::ffi::c_int {
                let fresh0 = nr;
                nr = nr.wrapping_add(1);
                j = fresh0;
                while j >= i {
                    (*win).edges[j.wrapping_add(1 as uint) as usize] = (*win).edges[j as usize];
                    j = j.wrapping_sub(1);
                }
                (*win).edges[i as usize] = t as int16;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        while i <= nr && {
            t = (*ppu).window2right as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            t != (*win).edges[i as usize] as ::core::ffi::c_int
        } {
            if t < (*win).edges[i as usize] as ::core::ffi::c_int {
                let fresh1 = nr;
                nr = nr.wrapping_add(1);
                j = fresh1;
                while j >= i {
                    (*win).edges[j.wrapping_add(1 as uint) as usize] = (*win).edges[j as usize];
                    j = j.wrapping_sub(1);
                }
                (*win).edges[i as usize] = t as int16;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    (*win).nr = nr as uint8;
    let mut w1_bits: uint8 = 0 as uint8;
    let mut w2_bits: uint8 = 0 as uint8;
    if w1_ena {
        i = 0 as uint;
        while (*win).edges[i as usize] as ::core::ffi::c_int
            != (*ppu).window1left as ::core::ffi::c_int
        {
            i = i.wrapping_add(1);
        }
        j = i;
        while (*win).edges[j as usize] as ::core::ffi::c_int
            != (*ppu).window1right as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        {
            j = j.wrapping_add(1);
        }
        w1_bits = ((((1 as ::core::ffi::c_int) << j.wrapping_sub(i)) - 1 as ::core::ffi::c_int)
            << i) as uint8;
    }
    if winflags
        & (kWindow1Enabled as ::core::ffi::c_int | kWindow1Inversed as ::core::ffi::c_int) as uint32
        == (kWindow1Enabled as ::core::ffi::c_int | kWindow1Inversed as ::core::ffi::c_int)
            as uint32
    {
        w1_bits = !(w1_bits as ::core::ffi::c_int) as uint8;
    }
    if w2_ena {
        i = 0 as uint;
        while (*win).edges[i as usize] as ::core::ffi::c_int
            != (*ppu).window2left as ::core::ffi::c_int
        {
            i = i.wrapping_add(1);
        }
        j = i;
        while (*win).edges[j as usize] as ::core::ffi::c_int
            != (*ppu).window2right as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        {
            j = j.wrapping_add(1);
        }
        w2_bits = ((((1 as ::core::ffi::c_int) << j.wrapping_sub(i)) - 1 as ::core::ffi::c_int)
            << i) as uint8;
    }
    if winflags
        & (kWindow2Enabled as ::core::ffi::c_int | kWindow2Inversed as ::core::ffi::c_int) as uint32
        == (kWindow2Enabled as ::core::ffi::c_int | kWindow2Inversed as ::core::ffi::c_int)
            as uint32
    {
        w2_bits = !(w2_bits as ::core::ffi::c_int) as uint8;
    }
    (*win).bits = (w1_bits as ::core::ffi::c_int | w2_bits as ::core::ffi::c_int) as uint8;
}
unsafe fn PpuDrawBackground_4bpp(
    mut ppu: *mut Ppu,
    mut y: uint,
    mut sub: bool,
    mut layer: uint,
    mut zhi: PpuZbufType,
    mut zlo: PpuZbufType,
) {
    if (*ppu).screenEnabled[sub as usize] as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << layer
        == 0
    {
        return;
    }
    let mut win: PpuWindows = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
    if (*ppu).screenWindowed[sub as usize] as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << layer
        != 0
    {
        PpuWindows_Calc(&raw mut win, ppu, layer);
    } else {
        PpuWindows_Clear(&raw mut win, ppu, layer);
    };
    let mut bglayer: *mut BgLayer =
        (&raw mut (*ppu).bgLayer as *mut BgLayer).offset(layer as isize) as *mut BgLayer;
    y = y.wrapping_add((*bglayer).vScroll as uint);
    let mut sc_offs: ::core::ffi::c_int = ((*bglayer).tilemapAdr as uint)
        .wrapping_add((y >> 3 as ::core::ffi::c_int & 0x1f as uint) << 5 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if y & 0x100 as uint != 0 && (*bglayer).tilemapHigher as ::core::ffi::c_int != 0 {
        sc_offs += if (*bglayer).tilemapWider as ::core::ffi::c_int != 0 {
            0x800 as ::core::ffi::c_int
        } else {
            0x400 as ::core::ffi::c_int
        };
    }
    let mut tps: [*const uint16; 2] = [
        (&raw mut (*ppu).vram as *mut uint16_t)
            .offset((sc_offs & 0x7fff as ::core::ffi::c_int) as isize) as *mut uint16_t
            as *const uint16,
        (&raw mut (*ppu).vram as *mut uint16_t).offset(
            (sc_offs
                + (if (*bglayer).tilemapWider as ::core::ffi::c_int != 0 {
                    0x400 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                & 0x7fff as ::core::ffi::c_int) as isize,
        ) as *mut uint16_t as *const uint16,
    ];
    let mut tileadr: ::core::ffi::c_int =
        (*ppu).bgLayer[layer as usize].tileAdr as ::core::ffi::c_int;
    let mut pixel: ::core::ffi::c_int = 0;
    let mut tileadr1: ::core::ffi::c_int = ((tileadr + 7 as ::core::ffi::c_int) as uint)
        .wrapping_sub(y & 0x7 as uint)
        as ::core::ffi::c_int;
    let mut tileadr0: ::core::ffi::c_int =
        (tileadr as uint).wrapping_add(y & 0x7 as uint) as ::core::ffi::c_int;
    let mut addr: *const uint16 = ::core::ptr::null::<uint16>();
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if !(win.bits as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << windex != 0) {
            let mut x: uint = (win.edges[windex as usize] as ::core::ffi::c_int
                + (*bglayer).hScroll as ::core::ffi::c_int) as uint;
            let mut w: uint =
                (win.edges[windex.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int
                    - win.edges[windex as usize] as ::core::ffi::c_int) as uint;
            let mut dstz: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(win.edges[windex as usize] as ::core::ffi::c_int as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            let mut tp: *const uint16 = tps[(x >> 8 as ::core::ffi::c_int & 1 as uint) as usize]
                .offset((x >> 3 as ::core::ffi::c_int & 0x1f as uint) as isize);
            let mut tp_last: *const uint16 = tps
                [(x >> 8 as ::core::ffi::c_int & 1 as uint) as usize]
                .offset(31 as ::core::ffi::c_int as isize);
            let mut tp_next: *const uint16 =
                tps[(x >> 8 as ::core::ffi::c_int & 1 as uint ^ 1 as uint) as usize];
            if x & 7 as uint != 0 {
                let mut curw: ::core::ffi::c_int = IntMin(
                    (8 as uint).wrapping_sub(x & 7 as uint) as ::core::ffi::c_int,
                    w as ::core::ffi::c_int,
                );
                w = w.wrapping_sub(curw as uint);
                let mut tile: uint32 = *tp as uint32;
                if tp != tp_last {
                    tp = tp.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31 as ::core::ffi::c_int as isize));
                    tp_last = tp.offset(31 as ::core::ffi::c_int as isize);
                }
                let mut ta: ::core::ffi::c_int = if tile & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z: PpuZbufType = (if tile & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta as uint32)
                        .wrapping_add((tile & 0x3ff as uint32).wrapping_mul(16 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits: uint32 =
                    (*addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        | (*addr.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int) as uint32;
                if bits != 0 {
                    z = (z as uint32).wrapping_add(
                        (tile & 0x1c00 as uint32) >> kPaletteShift_0 as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile & 0x4000 as uint32 != 0 {
                        bits >>= x & 7 as uint;
                        x = x.wrapping_add(curw as uint);
                        loop {
                            pixel = (bits >> 0 as ::core::ffi::c_int & 1 as uint32
                                | bits >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 2 as uint32
                                | bits >> 14 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 4 as uint32
                                | bits >> 21 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 8 as uint32)
                                as ::core::ffi::c_int;
                            if bits
                                & ((0x1010101 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int)
                                    as uint32
                                != 0
                                && z as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits >>= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            curw -= 1;
                            if !(curw != 0) {
                                break;
                            }
                        }
                    } else {
                        bits <<= x & 7 as uint;
                        x = x.wrapping_add(curw as uint);
                        loop {
                            pixel = (bits >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 1 as uint32
                                | bits >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 2 as uint32
                                | bits >> 21 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 4 as uint32
                                | bits >> 28 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 8 as uint32)
                                as ::core::ffi::c_int;
                            if bits & 0x80808080 as uint32 >> 0 as ::core::ffi::c_int != 0
                                && z as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits <<= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            curw -= 1;
                            if !(curw != 0) {
                                break;
                            }
                        }
                    }
                } else {
                    dstz = dstz.offset(curw as isize);
                }
            }
            while w >= 8 as uint {
                let mut tile_0: uint32 = *tp as uint32;
                if tp != tp_last {
                    tp = tp.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31 as ::core::ffi::c_int as isize));
                    tp_last = tp.offset(31 as ::core::ffi::c_int as isize);
                }
                let mut ta_0: ::core::ffi::c_int = if tile_0 & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z_0: PpuZbufType = (if tile_0 & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta_0 as uint32)
                        .wrapping_add((tile_0 & 0x3ff as uint32).wrapping_mul(16 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits_0: uint32 =
                    (*addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        | (*addr.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int) as uint32;
                if bits_0 != 0 {
                    z_0 = (z_0 as uint32).wrapping_add(
                        (tile_0 & 0x1c00 as uint32) >> kPaletteShift_0 as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile_0 & 0x4000 as uint32 != 0 {
                        pixel = (bits_0 >> 0 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 1 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(1 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 2 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(2 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 3 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 3 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 3 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 3 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(3 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 4 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(4 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 5 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 5 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 5 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 5 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(5 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(5 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 6 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 6 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 6 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 6 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(6 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(6 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0
                            & ((0x1010101 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                                as uint32
                            != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(7 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(7 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                    } else {
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 0 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 1 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(1 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 2 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(2 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 3 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(3 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 4 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(4 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 5 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(5 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(5 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 6 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(6 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(6 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                                & 2 as uint32
                            | bits_0 >> 21 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                                & 4 as uint32
                            | bits_0 >> 28 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                                & 8 as uint32)
                            as ::core::ffi::c_int;
                        if bits_0 & 0x80808080 as uint32 >> 7 as ::core::ffi::c_int != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(7 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(7 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                    }
                }
                dstz = dstz.offset(8 as ::core::ffi::c_int as isize);
                w = w.wrapping_sub(8 as uint);
            }
            if w != 0 {
                let mut tile_1: uint32 = *tp as uint32;
                let mut ta_1: ::core::ffi::c_int = if tile_1 & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z_1: PpuZbufType = (if tile_1 & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta_1 as uint32)
                        .wrapping_add((tile_1 & 0x3ff as uint32).wrapping_mul(16 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits_1: uint32 =
                    (*addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        | (*addr.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int) as uint32;
                if bits_1 != 0 {
                    z_1 = (z_1 as uint32).wrapping_add(
                        (tile_1 & 0x1c00 as uint32) >> kPaletteShift_0 as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile_1 & 0x4000 as uint32 != 0 {
                        loop {
                            pixel = (bits_1 >> 0 as ::core::ffi::c_int & 1 as uint32
                                | bits_1 >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 2 as uint32
                                | bits_1 >> 14 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 4 as uint32
                                | bits_1 >> 21 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 8 as uint32)
                                as ::core::ffi::c_int;
                            if bits_1
                                & ((0x1010101 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int)
                                    as uint32
                                != 0
                                && z_1 as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z_1 as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits_1 >>= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            w = w.wrapping_sub(1);
                            if !(w != 0) {
                                break;
                            }
                        }
                    } else {
                        loop {
                            pixel = (bits_1 >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 1 as uint32
                                | bits_1 >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 2 as uint32
                                | bits_1 >> 21 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 4 as uint32
                                | bits_1 >> 28 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 8 as uint32)
                                as ::core::ffi::c_int;
                            if bits_1 & 0x80808080 as uint32 >> 0 as ::core::ffi::c_int != 0
                                && z_1 as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z_1 as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits_1 <<= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            w = w.wrapping_sub(1);
                            if !(w != 0) {
                                break;
                            }
                        }
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawBackground_2bpp(
    mut ppu: *mut Ppu,
    mut y: uint,
    mut sub: bool,
    mut layer: uint,
    mut zhi: PpuZbufType,
    mut zlo: PpuZbufType,
) {
    if (*ppu).screenEnabled[sub as usize] as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << layer
        == 0
    {
        return;
    }
    let mut win: PpuWindows = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
    if (*ppu).screenWindowed[sub as usize] as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << layer
        != 0
    {
        PpuWindows_Calc(&raw mut win, ppu, layer);
    } else {
        PpuWindows_Clear(&raw mut win, ppu, layer);
    };
    let mut bglayer: *mut BgLayer =
        (&raw mut (*ppu).bgLayer as *mut BgLayer).offset(layer as isize) as *mut BgLayer;
    y = y.wrapping_add((*bglayer).vScroll as uint);
    let mut sc_offs: ::core::ffi::c_int = ((*bglayer).tilemapAdr as uint)
        .wrapping_add((y >> 3 as ::core::ffi::c_int & 0x1f as uint) << 5 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if y & 0x100 as uint != 0 && (*bglayer).tilemapHigher as ::core::ffi::c_int != 0 {
        sc_offs += if (*bglayer).tilemapWider as ::core::ffi::c_int != 0 {
            0x800 as ::core::ffi::c_int
        } else {
            0x400 as ::core::ffi::c_int
        };
    }
    let mut tps: [*const uint16; 2] = [
        (&raw mut (*ppu).vram as *mut uint16_t)
            .offset((sc_offs & 0x7fff as ::core::ffi::c_int) as isize) as *mut uint16_t
            as *const uint16,
        (&raw mut (*ppu).vram as *mut uint16_t).offset(
            (sc_offs
                + (if (*bglayer).tilemapWider as ::core::ffi::c_int != 0 {
                    0x400 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                & 0x7fff as ::core::ffi::c_int) as isize,
        ) as *mut uint16_t as *const uint16,
    ];
    let mut tileadr: ::core::ffi::c_int =
        (*ppu).bgLayer[layer as usize].tileAdr as ::core::ffi::c_int;
    let mut pixel: ::core::ffi::c_int = 0;
    let mut tileadr1: ::core::ffi::c_int = ((tileadr + 7 as ::core::ffi::c_int) as uint)
        .wrapping_sub(y & 0x7 as uint)
        as ::core::ffi::c_int;
    let mut tileadr0: ::core::ffi::c_int =
        (tileadr as uint).wrapping_add(y & 0x7 as uint) as ::core::ffi::c_int;
    let mut addr: *const uint16 = ::core::ptr::null::<uint16>();
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if !(win.bits as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << windex != 0) {
            let mut x: uint = (win.edges[windex as usize] as ::core::ffi::c_int
                + (*bglayer).hScroll as ::core::ffi::c_int) as uint;
            let mut w: uint =
                (win.edges[windex.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int
                    - win.edges[windex as usize] as ::core::ffi::c_int) as uint;
            let mut dstz: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(win.edges[windex as usize] as ::core::ffi::c_int as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            let mut tp: *const uint16 = tps[(x >> 8 as ::core::ffi::c_int & 1 as uint) as usize]
                .offset((x >> 3 as ::core::ffi::c_int & 0x1f as uint) as isize);
            let mut tp_last: *const uint16 = tps
                [(x >> 8 as ::core::ffi::c_int & 1 as uint) as usize]
                .offset(31 as ::core::ffi::c_int as isize);
            let mut tp_next: *const uint16 =
                tps[(x >> 8 as ::core::ffi::c_int & 1 as uint ^ 1 as uint) as usize];
            if x & 7 as uint != 0 {
                let mut curw: ::core::ffi::c_int = IntMin(
                    (8 as uint).wrapping_sub(x & 7 as uint) as ::core::ffi::c_int,
                    w as ::core::ffi::c_int,
                );
                w = w.wrapping_sub(curw as uint);
                let mut tile: uint32 = *tp as uint32;
                if tp != tp_last {
                    tp = tp.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31 as ::core::ffi::c_int as isize));
                    tp_last = tp.offset(31 as ::core::ffi::c_int as isize);
                }
                let mut ta: ::core::ffi::c_int = if tile & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z: PpuZbufType = (if tile & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta as uint32)
                        .wrapping_add((tile & 0x3ff as uint32).wrapping_mul(8 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits: uint32 = *addr.offset(0 as ::core::ffi::c_int as isize) as uint32;
                if bits != 0 {
                    z = (z as uint32).wrapping_add(
                        (tile & 0x1c00 as uint32) >> kPaletteShift as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile & 0x4000 as uint32 != 0 {
                        bits >>= x & 7 as uint;
                        x = x.wrapping_add(curw as uint);
                        loop {
                            pixel = (bits >> 0 as ::core::ffi::c_int & 1 as uint32
                                | bits >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 2 as uint32)
                                as ::core::ffi::c_int;
                            if pixel != 0
                                && z as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits >>= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            curw -= 1;
                            if !(curw != 0) {
                                break;
                            }
                        }
                    } else {
                        bits <<= x & 7 as uint;
                        x = x.wrapping_add(curw as uint);
                        loop {
                            pixel = (bits >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 1 as uint32
                                | bits >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 2 as uint32)
                                as ::core::ffi::c_int;
                            if pixel != 0
                                && z as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits <<= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            curw -= 1;
                            if !(curw != 0) {
                                break;
                            }
                        }
                    }
                } else {
                    dstz = dstz.offset(curw as isize);
                }
            }
            while w >= 8 as uint {
                let mut tile_0: uint32 = *tp as uint32;
                if tp != tp_last {
                    tp = tp.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31 as ::core::ffi::c_int as isize));
                    tp_last = tp.offset(31 as ::core::ffi::c_int as isize);
                }
                let mut ta_0: ::core::ffi::c_int = if tile_0 & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z_0: PpuZbufType = (if tile_0 & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta_0 as uint32)
                        .wrapping_add((tile_0 & 0x3ff as uint32).wrapping_mul(8 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits_0: uint32 = *addr.offset(0 as ::core::ffi::c_int as isize) as uint32;
                if bits_0 != 0 {
                    z_0 = (z_0 as uint32).wrapping_add(
                        (tile_0 & 0x1c00 as uint32) >> kPaletteShift as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile_0 & 0x4000 as uint32 != 0 {
                        pixel = (bits_0 >> 0 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 1 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(1 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 2 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(2 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 3 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 3 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(3 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 4 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(4 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 5 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 5 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(5 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(5 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 6 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 6 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(6 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(6 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int & 1 as uint32
                            | bits_0 >> 7 as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(7 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(7 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                    } else {
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(1 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(2 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(3 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(4 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(5 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(5 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(6 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(6 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                        pixel = (bits_0 >> 7 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                            & 1 as uint32
                            | bits_0 >> 14 as ::core::ffi::c_int - 7 as ::core::ffi::c_int
                                & 2 as uint32)
                            as ::core::ffi::c_int;
                        if pixel != 0
                            && z_0 as ::core::ffi::c_int
                                > *dstz.offset(7 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                        {
                            *dstz.offset(7 as ::core::ffi::c_int as isize) =
                                (z_0 as ::core::ffi::c_int + pixel) as PpuZbufType;
                        }
                    }
                }
                dstz = dstz.offset(8 as ::core::ffi::c_int as isize);
                w = w.wrapping_sub(8 as uint);
            }
            if w != 0 {
                let mut tile_1: uint32 = *tp as uint32;
                let mut ta_1: ::core::ffi::c_int = if tile_1 & 0x8000 as uint32 != 0 {
                    tileadr1
                } else {
                    tileadr0
                };
                let mut z_1: PpuZbufType = (if tile_1 & 0x2000 as uint32 != 0 {
                    zhi as ::core::ffi::c_int
                } else {
                    zlo as ::core::ffi::c_int
                }) as PpuZbufType;
                addr = (&raw mut (*ppu).vram as *mut uint16_t).offset(
                    ((ta_1 as uint32)
                        .wrapping_add((tile_1 & 0x3ff as uint32).wrapping_mul(8 as uint32))
                        & 0x7fff as uint32) as isize,
                ) as *mut uint16_t;
                let mut bits_1: uint32 = *addr.offset(0 as ::core::ffi::c_int as isize) as uint32;
                if bits_1 != 0 {
                    z_1 = (z_1 as uint32).wrapping_add(
                        (tile_1 & 0x1c00 as uint32) >> kPaletteShift as ::core::ffi::c_int,
                    ) as PpuZbufType as PpuZbufType;
                    if tile_1 & 0x4000 as uint32 != 0 {
                        loop {
                            pixel = (bits_1 >> 0 as ::core::ffi::c_int & 1 as uint32
                                | bits_1 >> 7 as ::core::ffi::c_int + 0 as ::core::ffi::c_int
                                    & 2 as uint32)
                                as ::core::ffi::c_int;
                            if pixel != 0
                                && z_1 as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z_1 as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits_1 >>= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            w = w.wrapping_sub(1);
                            if !(w != 0) {
                                break;
                            }
                        }
                    } else {
                        loop {
                            pixel = (bits_1 >> 7 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                & 1 as uint32
                                | bits_1 >> 14 as ::core::ffi::c_int - 0 as ::core::ffi::c_int
                                    & 2 as uint32)
                                as ::core::ffi::c_int;
                            if pixel != 0
                                && z_1 as ::core::ffi::c_int
                                    > *dstz.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                            {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (z_1 as ::core::ffi::c_int + pixel) as PpuZbufType;
                            }
                            bits_1 <<= 1 as ::core::ffi::c_int;
                            dstz = dstz.offset(1);
                            w = w.wrapping_sub(1);
                            if !(w != 0) {
                                break;
                            }
                        }
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawBackground_mode7(
    mut ppu: *mut Ppu,
    mut y: uint,
    mut sub: bool,
    mut z: PpuZbufType,
) {
    let mut layer: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*ppu).screenEnabled[sub as usize] as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << layer
        == 0
    {
        return;
    }
    let mut win: PpuWindows = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
    if (*ppu).screenWindowed[sub as usize] as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << layer
        != 0
    {
        PpuWindows_Calc(&raw mut win, ppu, layer as uint);
    } else {
        PpuWindows_Clear(&raw mut win, ppu, layer as uint);
    };
    let mut hScroll: ::core::ffi::c_int =
        (((*ppu).m7matrix[6 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut vScroll: ::core::ffi::c_int =
        (((*ppu).m7matrix[7 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut xCenter: ::core::ffi::c_int =
        (((*ppu).m7matrix[4 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut yCenter: ::core::ffi::c_int =
        (((*ppu).m7matrix[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut clippedH: ::core::ffi::c_int = hScroll - xCenter;
    let mut clippedV: ::core::ffi::c_int = vScroll - yCenter;
    clippedH = if clippedH & 0x2000 as ::core::ffi::c_int != 0 {
        clippedH | !(1023 as ::core::ffi::c_int)
    } else {
        clippedH & 1023 as ::core::ffi::c_int
    };
    clippedV = if clippedV & 0x2000 as ::core::ffi::c_int != 0 {
        clippedV | !(1023 as ::core::ffi::c_int)
    } else {
        clippedV & 1023 as ::core::ffi::c_int
    };
    let mut mosaic_enabled: bool = (*ppu).mosaicEnabled as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
        != 0;
    if mosaic_enabled {
        y = (*ppu).mosaicModulo[y as usize] as uint;
    }
    let mut ry: uint32 = if (*ppu).m7yFlip as ::core::ffi::c_int != 0 {
        (255 as uint32).wrapping_sub(y as uint32)
    } else {
        y as uint32
    };
    let mut m7startX: uint32 =
        (((*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedH
            & !(63 as ::core::ffi::c_int)) as uint32)
            .wrapping_add(
                ((*ppu).m7matrix[1 as ::core::ffi::c_int as usize] as uint32).wrapping_mul(ry)
                    & !(63 as ::core::ffi::c_int) as uint32,
            )
            .wrapping_add(
                ((*ppu).m7matrix[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedV
                    & !(63 as ::core::ffi::c_int)) as uint32,
            )
            .wrapping_add((xCenter << 8 as ::core::ffi::c_int) as uint32);
    let mut m7startY: uint32 =
        (((*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedH
            & !(63 as ::core::ffi::c_int)) as uint32)
            .wrapping_add(
                ((*ppu).m7matrix[3 as ::core::ffi::c_int as usize] as uint32).wrapping_mul(ry)
                    & !(63 as ::core::ffi::c_int) as uint32,
            )
            .wrapping_add(
                ((*ppu).m7matrix[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedV
                    & !(63 as ::core::ffi::c_int)) as uint32,
            )
            .wrapping_add((yCenter << 8 as ::core::ffi::c_int) as uint32);
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if !(win.bits as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << windex != 0) {
            let mut x: ::core::ffi::c_int = win.edges[windex as usize] as ::core::ffi::c_int;
            let mut x2: ::core::ffi::c_int =
                win.edges[windex.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int;
            let mut tile: ::core::ffi::c_int = 0;
            let mut dstz: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(x as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            let mut dstz_end: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(x2 as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            let mut rx: uint32 = (if (*ppu).m7xFlip as ::core::ffi::c_int != 0 {
                255 as ::core::ffi::c_int - x
            } else {
                x
            }) as uint32;
            let mut xpos: uint32 = m7startX.wrapping_add(
                ((*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as uint32).wrapping_mul(rx),
            );
            let mut ypos: uint32 = m7startY.wrapping_add(
                ((*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as uint32).wrapping_mul(rx),
            );
            let mut dx: uint32 = (if (*ppu).m7xFlip as ::core::ffi::c_int != 0 {
                -((*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            } else {
                (*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            }) as uint32;
            let mut dy: uint32 = (if (*ppu).m7xFlip as ::core::ffi::c_int != 0 {
                -((*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            } else {
                (*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            }) as uint32;
            let mut outside_value: uint32 = if (*ppu).m7largeField as ::core::ffi::c_int != 0 {
                0x3ffff as uint32
            } else {
                0xffffffff as uint32
            };
            let mut char_fill: bool = (*ppu).m7charFill;
            if mosaic_enabled {
                let mut w: ::core::ffi::c_int = (*ppu).mosaicSize as ::core::ffi::c_int
                    - (x - (*ppu).mosaicModulo[x as usize] as ::core::ffi::c_int);
                let mut current_block_14: u64;
                loop {
                    w = IntMin(
                        w,
                        dstz_end.offset_from(dstz) as ::core::ffi::c_long as ::core::ffi::c_int,
                    );
                    if xpos | ypos > outside_value {
                        if !char_fill {
                            current_block_14 = 17833034027772472439;
                        } else {
                            tile = 0 as ::core::ffi::c_int;
                            current_block_14 = 5634871135123216486;
                        }
                    } else {
                        tile = (*ppu).vram[(ypos >> 11 as ::core::ffi::c_int & 0x7f as uint32)
                            .wrapping_mul(128 as uint32)
                            .wrapping_add(xpos >> 11 as ::core::ffi::c_int & 0x7f as uint32)
                            as usize] as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int;
                        current_block_14 = 5634871135123216486;
                    }
                    match current_block_14 {
                        5634871135123216486 => {
                            let mut pixel: uint8 = ((*ppu).vram[((tile * 64 as ::core::ffi::c_int)
                                as uint32)
                                .wrapping_add(
                                    (ypos >> 8 as ::core::ffi::c_int & 7 as uint32)
                                        .wrapping_mul(8 as uint32),
                                )
                                .wrapping_add(xpos >> 8 as ::core::ffi::c_int & 7 as uint32)
                                as usize]
                                as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int)
                                as uint8;
                            if pixel != 0 {
                                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                loop {
                                    *dstz.offset(i as isize) = (pixel as ::core::ffi::c_int
                                        + z as ::core::ffi::c_int)
                                        as PpuZbufType;
                                    i += 1;
                                    if !(i != w) {
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    xpos = xpos.wrapping_add(dx.wrapping_mul(w as uint32));
                    ypos = ypos.wrapping_add(dy.wrapping_mul(w as uint32));
                    dstz = dstz.offset(w as isize);
                    w = (*ppu).mosaicSize as ::core::ffi::c_int;
                    if !(dstz_end.offset_from(dstz) as ::core::ffi::c_long
                        != 0 as ::core::ffi::c_long)
                    {
                        break;
                    }
                }
            } else {
                let mut current_block_21: u64;
                loop {
                    if xpos | ypos > outside_value {
                        if !char_fill {
                            current_block_21 = 5689316957504528238;
                        } else {
                            tile = 0 as ::core::ffi::c_int;
                            current_block_21 = 9520865839495247062;
                        }
                    } else {
                        tile = (*ppu).vram[(ypos >> 11 as ::core::ffi::c_int & 0x7f as uint32)
                            .wrapping_mul(128 as uint32)
                            .wrapping_add(xpos >> 11 as ::core::ffi::c_int & 0x7f as uint32)
                            as usize] as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int;
                        current_block_21 = 9520865839495247062;
                    }
                    match current_block_21 {
                        9520865839495247062 => {
                            let mut pixel_0: uint8 = ((*ppu).vram[((tile * 64 as ::core::ffi::c_int)
                                as uint32)
                                .wrapping_add(
                                    (ypos >> 8 as ::core::ffi::c_int & 7 as uint32)
                                        .wrapping_mul(8 as uint32),
                                )
                                .wrapping_add(xpos >> 8 as ::core::ffi::c_int & 7 as uint32)
                                as usize]
                                as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int)
                                as uint8;
                            if pixel_0 != 0 {
                                *dstz.offset(0 as ::core::ffi::c_int as isize) =
                                    (pixel_0 as ::core::ffi::c_int + z as ::core::ffi::c_int)
                                        as PpuZbufType;
                            }
                        }
                        _ => {}
                    }
                    xpos = xpos.wrapping_add(dx);
                    ypos = ypos.wrapping_add(dy);
                    dstz = dstz.offset(1);
                    if !(dstz != dstz_end) {
                        break;
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawSprites(mut ppu: *mut Ppu, mut y: uint, mut sub: uint, mut clear_backdrop: bool) {
    let mut layer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
    if (*ppu).screenEnabled[sub as usize] as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << layer
        == 0
    {
        return;
    }
    let mut win: PpuWindows = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
    if (*ppu).screenWindowed[sub as usize] as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << layer
        != 0
    {
        PpuWindows_Calc(&raw mut win, ppu, layer as uint);
    } else {
        PpuWindows_Clear(&raw mut win, ppu, layer as uint);
    };
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if !(win.bits as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << windex != 0) {
            let mut left: ::core::ffi::c_int = win.edges[windex as usize] as ::core::ffi::c_int;
            let mut width: ::core::ffi::c_int =
                win.edges[windex.wrapping_add(1 as size_t) as usize] as ::core::ffi::c_int - left;
            let mut src: *mut PpuZbufType = (&raw mut (*ppu).objBuffer.data as *mut PpuZbufType)
                .offset(left as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            let mut dst: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(left as isize)
                .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
            if clear_backdrop {
                memcpy(
                    dst as *mut ::core::ffi::c_void,
                    src as *const ::core::ffi::c_void,
                    (width as size_t).wrapping_mul(::core::mem::size_of::<uint16>() as size_t),
                );
            } else {
                loop {
                    if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > *dst.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    {
                        *dst.offset(0 as ::core::ffi::c_int as isize) =
                            *src.offset(0 as ::core::ffi::c_int as isize);
                    }
                    src = src.offset(1);
                    dst = dst.offset(1);
                    width -= 1;
                    if !(width != 0) {
                        break;
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawBackgrounds(mut ppu: *mut Ppu, mut y: ::core::ffi::c_int, mut sub: bool) {
    if (*ppu).mode as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        if (*ppu).lineHasSprites {
            PpuDrawSprites(ppu, y as uint, sub as uint, true_0 != 0);
        }
        if (*ppu).mosaicEnabled as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            != 0
        {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const ::core::ffi::c_char,
                    679 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_4bpp(
                ppu,
                y as uint,
                sub,
                0 as uint,
                0xc000 as PpuZbufType,
                0x8000 as PpuZbufType,
            );
        }
        if (*ppu).mosaicEnabled as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            != 0
        {
            '_c2rust_label_0: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const ::core::ffi::c_char,
                    684 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_4bpp(
                ppu,
                y as uint,
                sub,
                1 as uint,
                0xb100 as PpuZbufType,
                0x7100 as PpuZbufType,
            );
        }
        if (*ppu).mosaicEnabled as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            != 0
        {
            '_c2rust_label_1: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const ::core::ffi::c_char,
                    689 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_2bpp(
                ppu,
                y as uint,
                sub,
                2 as uint,
                0xf200 as PpuZbufType,
                0x1200 as PpuZbufType,
            );
        }
    } else {
        PpuDrawBackground_mode7(ppu, y as uint, sub, 0x5000 as PpuZbufType);
        if (*ppu).lineHasSprites {
            PpuDrawSprites(ppu, y as uint, sub as uint, false_0 != 0);
        }
    };
}
unsafe fn PpuDrawWholeLine(mut ppu: *mut Ppu, mut y: uint) {
    if (*ppu).forcedBlank {
        let mut dst: *mut uint8 = (*ppu).renderBuffer.offset(
            (y as uint32_t)
                .wrapping_sub(1 as uint32_t)
                .wrapping_mul((*ppu).renderPitch) as isize,
        ) as *mut uint8;
        let mut n: size_t = (::core::mem::size_of::<uint32>() as size_t).wrapping_mul(
            (256 as ::core::ffi::c_int
                + (*ppu).extraLeftRight as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                as size_t,
        );
        memset(dst as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, n);
        return;
    }
    ClearBackdrop(
        (&raw mut (*ppu).bgBuffers as *mut PpuPixelPrioBufs)
            .offset(0 as ::core::ffi::c_int as isize) as *mut PpuPixelPrioBufs,
    );
    PpuDrawBackgrounds(ppu, y as ::core::ffi::c_int, false_0 != 0);
    let mut math_enabled: uint32 = 0 as uint32;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6 as ::core::ffi::c_int {
        math_enabled |= (((*ppu).mathEnabled[i as usize] as ::core::ffi::c_int) << i) as uint32;
        i += 1;
    }
    let mut rendered_subscreen: bool = false_0 != 0;
    if (*ppu).preventMathMode as ::core::ffi::c_int != 3 as ::core::ffi::c_int
        && (*ppu).addSubscreen as ::core::ffi::c_int != 0
        && math_enabled != 0
    {
        ClearBackdrop(
            (&raw mut (*ppu).bgBuffers as *mut PpuPixelPrioBufs)
                .offset(1 as ::core::ffi::c_int as isize) as *mut PpuPixelPrioBufs,
        );
        if (*ppu).screenEnabled[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            PpuDrawBackgrounds(ppu, y as ::core::ffi::c_int, true_0 != 0);
            rendered_subscreen = true_0 != 0;
        }
    }
    let mut cwin: PpuWindows = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
    PpuWindows_Calc(&raw mut cwin, ppu, 5 as uint);
    static kCwBitsMod: [uint8; 8] = [
        0 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0xff as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    let mut cw_clip_math: uint32 = (cwin.bits as ::core::ffi::c_int
        & kCwBitsMod[(*ppu).clipMode as usize] as ::core::ffi::c_int
        ^ kCwBitsMod[((*ppu).clipMode as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
        | (cwin.bits as ::core::ffi::c_int
            & kCwBitsMod[(*ppu).preventMathMode as usize] as ::core::ffi::c_int
            ^ kCwBitsMod
                [((*ppu).preventMathMode as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32;
    let mut dst_0: *mut uint32 = (*ppu).renderBuffer.offset(
        (y as uint32_t)
            .wrapping_sub(1 as uint32_t)
            .wrapping_mul((*ppu).renderPitch) as isize,
    ) as *mut uint8_t as *mut uint32;
    let mut dst_org: *mut uint32 = dst_0;
    dst_0 = dst_0.offset(
        ((*ppu).extraLeftRight as ::core::ffi::c_int - (*ppu).extraLeftCur as ::core::ffi::c_int)
            as isize,
    );
    let mut windex: uint32 = 0 as uint32;
    loop {
        let mut left: uint32 = (cwin.edges[windex as usize] as ::core::ffi::c_int
            + kPpuExtraLeftRight as ::core::ffi::c_int) as uint32;
        let mut right: uint32 = (cwin.edges[windex.wrapping_add(1 as uint32) as usize]
            as ::core::ffi::c_int
            + kPpuExtraLeftRight as ::core::ffi::c_int) as uint32;
        let mut clip_color_mask: uint32 = (if cw_clip_math & 1 as uint32 != 0 {
            0x1f as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint32;
        let mut math_enabled_cur: uint32 = if cw_clip_math & 0x100 as uint32 != 0 {
            math_enabled
        } else {
            0 as uint32
        };
        let mut fixed_color: uint32 = ((*ppu).fixedColorR as ::core::ffi::c_int
            | ((*ppu).fixedColorG as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | ((*ppu).fixedColorB as ::core::ffi::c_int) << 10 as ::core::ffi::c_int)
            as uint32;
        if math_enabled_cur == 0 as uint32
            || fixed_color == 0 as uint32 && !(*ppu).halfColor && !rendered_subscreen
        {
            // The original do-loop emits one pixel even for an empty span.
            let count = if right > left {
                (right - left) as usize
            } else {
                1
            };
            draw_palette_span(
                core::slice::from_raw_parts_mut(dst_0, count),
                core::slice::from_raw_parts(
                    (*ppu).bgBuffers[0].data.as_ptr().add(left as usize),
                    count,
                ),
                &(*ppu).cgram,
                &(*ppu).brightnessMult,
                clip_color_mask as u16,
            );
            dst_0 = dst_0.add(count);
        } else {
            let mut half_color_map: *mut uint8 = if (*ppu).halfColor as ::core::ffi::c_int != 0 {
                &raw mut (*ppu).brightnessMultHalf as *mut uint8
            } else {
                &raw mut (*ppu).brightnessMult as *mut uint8
            };
            math_enabled_cur |= (((*ppu).addSubscreen as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
                | ((*ppu).subtractColor as ::core::ffi::c_int) << 9 as ::core::ffi::c_int)
                as uint32;
            let mut i_1: uint32 = left;
            loop {
                let mut color_0: uint32 =
                    (*ppu).cgram[((*ppu).bgBuffers[0 as ::core::ffi::c_int as usize].data
                        [i_1 as usize] as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as usize] as uint32;
                let mut color2: uint32 = 0;
                let mut main_layer: uint8 =
                    ((*ppu).bgBuffers[0 as ::core::ffi::c_int as usize].data[i_1 as usize]
                        as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int
                        & 0xf as ::core::ffi::c_int) as uint8;
                let mut r: uint32 = color_0 & clip_color_mask;
                let mut g: uint32 = color_0 >> 5 as ::core::ffi::c_int & clip_color_mask;
                let mut b: uint32 = color_0 >> 10 as ::core::ffi::c_int & clip_color_mask;
                let mut color_map: *mut uint8 = &raw mut (*ppu).brightnessMult as *mut uint8;
                if math_enabled_cur
                    & ((1 as ::core::ffi::c_int) << main_layer as ::core::ffi::c_int) as uint32
                    != 0
                {
                    if math_enabled_cur & 0x100 as uint32 != 0 {
                        if (*ppu).bgBuffers[1 as ::core::ffi::c_int as usize].data[i_1 as usize]
                            as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            color2 = (*ppu).cgram[((*ppu).bgBuffers
                                [1 as ::core::ffi::c_int as usize]
                                .data[i_1 as usize]
                                as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_int)
                                as usize] as uint32;
                            color_map = half_color_map;
                        } else {
                            color2 = fixed_color;
                        }
                    } else {
                        color2 = fixed_color;
                        color_map = half_color_map;
                    }
                    let mut r2: uint32 = color2 & 0x1f as uint32;
                    let mut g2: uint32 = color2 >> 5 as ::core::ffi::c_int & 0x1f as uint32;
                    let mut b2: uint32 = color2 >> 10 as ::core::ffi::c_int & 0x1f as uint32;
                    if math_enabled_cur & 0x200 as uint32 != 0 {
                        r = if r >= r2 {
                            r.wrapping_sub(r2)
                        } else {
                            0 as uint32
                        };
                        g = if g >= g2 {
                            g.wrapping_sub(g2)
                        } else {
                            0 as uint32
                        };
                        b = if b >= b2 {
                            b.wrapping_sub(b2)
                        } else {
                            0 as uint32
                        };
                    } else {
                        r = r.wrapping_add(r2);
                        g = g.wrapping_add(g2);
                        b = b.wrapping_add(b2);
                    }
                }
                *dst_0.offset(0 as ::core::ffi::c_int as isize) =
                    (*color_map.offset(b as isize) as ::core::ffi::c_int
                        | (*color_map.offset(g as isize) as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int
                        | (*color_map.offset(r as isize) as ::core::ffi::c_int)
                            << 16 as ::core::ffi::c_int) as uint32;
                dst_0 = dst_0.offset(1);
                i_1 = i_1.wrapping_add(1);
                if !(i_1 < right) {
                    break;
                }
            }
        }
        cw_clip_math >>= 1 as ::core::ffi::c_int;
        windex = windex.wrapping_add(1);
        if !(windex < cwin.nr as uint32) {
            break;
        }
    }
}
unsafe fn ppu_handlePixel(mut ppu: *mut Ppu, mut x: ::core::ffi::c_int, mut y: ::core::ffi::c_int) {
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*ppu).forcedBlank {
        let mut mainLayer: ::core::ffi::c_int =
            ppu_getPixel(ppu, x, y, false_0 != 0, &raw mut r, &raw mut g, &raw mut b);
        let mut colorWindowState: bool = ppu_getWindowState(ppu, 5 as ::core::ffi::c_int, x);
        if (*ppu).clipMode as ::core::ffi::c_int == 3 as ::core::ffi::c_int
            || (*ppu).clipMode as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                && colorWindowState as ::core::ffi::c_int != 0
            || (*ppu).clipMode as ::core::ffi::c_int == 1 as ::core::ffi::c_int && !colorWindowState
        {
            r = 0 as ::core::ffi::c_int;
            g = 0 as ::core::ffi::c_int;
            b = 0 as ::core::ffi::c_int;
        }
        let mut secondLayer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
        let mut mathEnabled: bool = mainLayer < 6 as ::core::ffi::c_int
            && (*ppu).mathEnabled[mainLayer as usize] as ::core::ffi::c_int != 0
            && !((*ppu).preventMathMode as ::core::ffi::c_int == 3 as ::core::ffi::c_int
                || (*ppu).preventMathMode as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                    && colorWindowState as ::core::ffi::c_int != 0
                || (*ppu).preventMathMode as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && !colorWindowState);
        if mathEnabled as ::core::ffi::c_int != 0 && (*ppu).addSubscreen as ::core::ffi::c_int != 0
            || (*ppu).pseudoHires as ::core::ffi::c_int != 0
            || (*ppu).mode as ::core::ffi::c_int == 5 as ::core::ffi::c_int
            || (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int
        {
            secondLayer = ppu_getPixel(
                ppu,
                x,
                y,
                true_0 != 0,
                &raw mut r2,
                &raw mut g2,
                &raw mut b2,
            );
        }
        if mathEnabled {
            if (*ppu).subtractColor {
                r -= if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    r2
                } else {
                    (*ppu).fixedColorR as ::core::ffi::c_int
                };
                g -= if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    g2
                } else {
                    (*ppu).fixedColorG as ::core::ffi::c_int
                };
                b -= if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    b2
                } else {
                    (*ppu).fixedColorB as ::core::ffi::c_int
                };
            } else {
                r += if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    r2
                } else {
                    (*ppu).fixedColorR as ::core::ffi::c_int
                };
                g += if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    g2
                } else {
                    (*ppu).fixedColorG as ::core::ffi::c_int
                };
                b += if (*ppu).addSubscreen as ::core::ffi::c_int != 0
                    && secondLayer != 5 as ::core::ffi::c_int
                {
                    b2
                } else {
                    (*ppu).fixedColorB as ::core::ffi::c_int
                };
            }
            if (*ppu).halfColor as ::core::ffi::c_int != 0
                && (secondLayer != 5 as ::core::ffi::c_int || !(*ppu).addSubscreen)
            {
                r >>= 1 as ::core::ffi::c_int;
                g >>= 1 as ::core::ffi::c_int;
                b >>= 1 as ::core::ffi::c_int;
            }
            if r > 31 as ::core::ffi::c_int {
                r = 31 as ::core::ffi::c_int;
            }
            if g > 31 as ::core::ffi::c_int {
                g = 31 as ::core::ffi::c_int;
            }
            if b > 31 as ::core::ffi::c_int {
                b = 31 as ::core::ffi::c_int;
            }
            if r < 0 as ::core::ffi::c_int {
                r = 0 as ::core::ffi::c_int;
            }
            if g < 0 as ::core::ffi::c_int {
                g = 0 as ::core::ffi::c_int;
            }
            if b < 0 as ::core::ffi::c_int {
                b = 0 as ::core::ffi::c_int;
            }
        }
        if !((*ppu).pseudoHires as ::core::ffi::c_int != 0
            || (*ppu).mode as ::core::ffi::c_int == 5 as ::core::ffi::c_int
            || (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int)
        {
            r2 = r;
            g2 = g;
            b2 = b;
        }
    }
    let mut row: ::core::ffi::c_int = y - 1 as ::core::ffi::c_int;
    let mut pixelBuffer: *mut uint8 = (*ppu).renderBuffer.offset(
        (row as uint32_t)
            .wrapping_mul((*ppu).renderPitch)
            .wrapping_add(
                ((x + (*ppu).extraLeftRight as ::core::ffi::c_int) * 4 as ::core::ffi::c_int)
                    as uint32_t,
            ) as isize,
    ) as *mut uint8_t as *mut uint8;
    *pixelBuffer.offset(0 as ::core::ffi::c_int as isize) =
        ((b << 3 as ::core::ffi::c_int | b >> 2 as ::core::ffi::c_int)
            * (*ppu).brightness as ::core::ffi::c_int
            / 15 as ::core::ffi::c_int) as uint8;
    *pixelBuffer.offset(1 as ::core::ffi::c_int as isize) =
        ((g << 3 as ::core::ffi::c_int | g >> 2 as ::core::ffi::c_int)
            * (*ppu).brightness as ::core::ffi::c_int
            / 15 as ::core::ffi::c_int) as uint8;
    *pixelBuffer.offset(2 as ::core::ffi::c_int as isize) =
        ((r << 3 as ::core::ffi::c_int | r >> 2 as ::core::ffi::c_int)
            * (*ppu).brightness as ::core::ffi::c_int
            / 15 as ::core::ffi::c_int) as uint8;
    *pixelBuffer.offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
}
unsafe fn ppu_getPixel(
    mut ppu: *mut Ppu,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut sub: bool,
    mut r: *mut ::core::ffi::c_int,
    mut g: *mut ::core::ffi::c_int,
    mut b: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut actMode: ::core::ffi::c_int = if (*ppu).mode as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*ppu).bg3priority as ::core::ffi::c_int != 0
    {
        8 as ::core::ffi::c_int
    } else {
        (*ppu).mode as ::core::ffi::c_int
    };
    actMode = if (*ppu).mode as ::core::ffi::c_int == 7 as ::core::ffi::c_int
        && (*ppu).m7extBg as ::core::ffi::c_int != 0
    {
        9 as ::core::ffi::c_int
    } else {
        actMode
    };
    let mut layer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    let mut pixel: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < layerCountPerMode[actMode as usize] {
        let mut curLayer: ::core::ffi::c_int = layersPerMode[actMode as usize][i as usize];
        let mut curPriority: ::core::ffi::c_int = prioritysPerMode[actMode as usize][i as usize];
        let mut layerActive: bool = false_0 != 0;
        if !sub {
            layerActive = (*ppu).layer[curLayer as usize].mainScreenEnabled as ::core::ffi::c_int
                != 0
                && (!(*ppu).layer[curLayer as usize].mainScreenWindowed
                    || !ppu_getWindowState(ppu, curLayer, x));
        } else {
            layerActive = (*ppu).layer[curLayer as usize].subScreenEnabled as ::core::ffi::c_int
                != 0
                && (!(*ppu).layer[curLayer as usize].subScreenWindowed
                    || !ppu_getWindowState(ppu, curLayer, x));
        }
        if layerActive {
            if curLayer < 4 as ::core::ffi::c_int {
                let mut lx: ::core::ffi::c_int = x;
                let mut ly: ::core::ffi::c_int = y;
                if (*ppu).bgLayer[curLayer as usize].mosaicEnabled as ::core::ffi::c_int != 0
                    && (*ppu).mosaicSize as ::core::ffi::c_int > 1 as ::core::ffi::c_int
                {
                    lx -= lx % (*ppu).mosaicSize as ::core::ffi::c_int;
                    ly -= (ly - (*ppu).mosaicStartLine as ::core::ffi::c_int)
                        % (*ppu).mosaicSize as ::core::ffi::c_int;
                }
                if (*ppu).mode as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                    pixel = ppu_getPixelForMode7(ppu, lx, curLayer, curPriority != 0);
                } else {
                    lx += (*ppu).bgLayer[curLayer as usize].hScroll as ::core::ffi::c_int;
                    if (*ppu).mode as ::core::ffi::c_int == 5 as ::core::ffi::c_int
                        || (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int
                    {
                        lx *= 2 as ::core::ffi::c_int;
                        lx += if sub as ::core::ffi::c_int != 0
                            || (*ppu).bgLayer[curLayer as usize].mosaicEnabled as ::core::ffi::c_int
                                != 0
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        };
                        if (*ppu).interlace {
                            ly *= 2 as ::core::ffi::c_int;
                            ly += if (*ppu).evenFrame as ::core::ffi::c_int != 0
                                || (*ppu).bgLayer[curLayer as usize].mosaicEnabled
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                0 as ::core::ffi::c_int
                            } else {
                                1 as ::core::ffi::c_int
                            };
                        }
                    }
                    ly += (*ppu).bgLayer[curLayer as usize].vScroll as ::core::ffi::c_int;
                    if (*ppu).mode as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                        || (*ppu).mode as ::core::ffi::c_int == 4 as ::core::ffi::c_int
                        || (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int
                    {
                        ppu_handleOPT(ppu, curLayer, &raw mut lx, &raw mut ly);
                    }
                    pixel = ppu_getPixelForBgLayer(
                        ppu,
                        lx & 0x3ff as ::core::ffi::c_int,
                        ly & 0x3ff as ::core::ffi::c_int,
                        curLayer,
                        curPriority != 0,
                    );
                }
            } else {
                pixel = 0 as ::core::ffi::c_int;
                if (*ppu).objBuffer.data[(x + kPpuExtraLeftRight as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    >> 12 as ::core::ffi::c_int
                    == curPriority * 4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                {
                    pixel = (*ppu).objBuffer.data
                        [(x + kPpuExtraLeftRight as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int;
                }
            }
        }
        if pixel > 0 as ::core::ffi::c_int {
            layer = curLayer;
            break;
        } else {
            i += 1;
        }
    }
    if (*ppu).directColor as ::core::ffi::c_int != 0
        && layer < 4 as ::core::ffi::c_int
        && bitDepthsPerMode[actMode as usize][layer as usize] == 8 as ::core::ffi::c_int
    {
        *r = (pixel & 0x7 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (pixel & 0x100 as ::core::ffi::c_int) >> 7 as ::core::ffi::c_int;
        *g = (pixel & 0x38 as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int
            | (pixel & 0x200 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        *b = (pixel & 0xc0 as ::core::ffi::c_int) >> 3 as ::core::ffi::c_int
            | (pixel & 0x400 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    } else {
        let mut color: uint16_t = (*ppu).cgram[(pixel & 0xff as ::core::ffi::c_int) as usize];
        *r = color as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
        *g = color as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
        *b = color as ::core::ffi::c_int >> 10 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
    }
    if layer == 4 as ::core::ffi::c_int && pixel < 0xc0 as ::core::ffi::c_int {
        layer = 6 as ::core::ffi::c_int;
    }
    return layer;
}
unsafe fn ppu_handleOPT(
    mut ppu: *mut Ppu,
    mut layer: ::core::ffi::c_int,
    mut lx: *mut ::core::ffi::c_int,
    mut ly: *mut ::core::ffi::c_int,
) {
    let mut x: ::core::ffi::c_int = *lx;
    let mut y: ::core::ffi::c_int = *ly;
    let mut column: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
        column = x
            - (x & 0xf as ::core::ffi::c_int)
            - ((*ppu).bgLayer[layer as usize].hScroll as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int)
            >> 4 as ::core::ffi::c_int;
    } else {
        column = x
            - (x & 0x7 as ::core::ffi::c_int)
            - ((*ppu).bgLayer[layer as usize].hScroll as ::core::ffi::c_int
                & 0xfff8 as ::core::ffi::c_int)
            >> 3 as ::core::ffi::c_int;
    }
    if column > 0 as ::core::ffi::c_int {
        let mut valid: ::core::ffi::c_int = if layer == 0 as ::core::ffi::c_int {
            0x2000 as ::core::ffi::c_int
        } else {
            0x4000 as ::core::ffi::c_int
        };
        let mut hOffset: uint16_t = ppu_getOffsetValue(
            ppu,
            column - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        let mut vOffset: uint16_t = 0 as uint16_t;
        if (*ppu).mode as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            if hOffset as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                vOffset = hOffset;
                hOffset = 0 as uint16_t;
            }
        } else {
            vOffset = ppu_getOffsetValue(
                ppu,
                column - 1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        }
        if (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
            if hOffset as ::core::ffi::c_int & valid != 0 {
                *lx = ((hOffset as ::core::ffi::c_int & 0x3f8 as ::core::ffi::c_int)
                    + column * 8 as ::core::ffi::c_int)
                    * 2 as ::core::ffi::c_int
                    | x & 0xf as ::core::ffi::c_int;
            }
        } else if hOffset as ::core::ffi::c_int & valid != 0 {
            *lx = (hOffset as ::core::ffi::c_int & 0x3f8 as ::core::ffi::c_int)
                + column * 8 as ::core::ffi::c_int
                | x & 0x7 as ::core::ffi::c_int;
        }
        if vOffset as ::core::ffi::c_int & valid != 0 {
            *ly = (vOffset as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int)
                + (y - (*ppu).bgLayer[layer as usize].vScroll as ::core::ffi::c_int);
        }
    }
}
unsafe fn ppu_getOffsetValue(
    mut ppu: *mut Ppu,
    mut col: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
) -> uint16_t {
    let mut x: ::core::ffi::c_int = col * 8 as ::core::ffi::c_int
        + (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].hScroll as ::core::ffi::c_int;
    let mut y: ::core::ffi::c_int = row * 8 as ::core::ffi::c_int
        + (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].vScroll as ::core::ffi::c_int;
    let mut tileBits: ::core::ffi::c_int =
        if (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].bigTiles as ::core::ffi::c_int != 0 {
            4 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int
        };
    let mut tileHighBit: ::core::ffi::c_int =
        if (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].bigTiles as ::core::ffi::c_int != 0 {
            0x200 as ::core::ffi::c_int
        } else {
            0x100 as ::core::ffi::c_int
        };
    let mut tilemapAdr: uint16_t =
        ((*ppu).bgLayer[2 as ::core::ffi::c_int as usize].tilemapAdr as ::core::ffi::c_int
            + ((y >> tileBits & 0x1f as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
                | x >> tileBits & 0x1f as ::core::ffi::c_int)) as uint16_t;
    if x & tileHighBit != 0
        && (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].tilemapWider as ::core::ffi::c_int != 0
    {
        tilemapAdr = (tilemapAdr as ::core::ffi::c_int + 0x400 as ::core::ffi::c_int) as uint16_t;
    }
    if y & tileHighBit != 0
        && (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].tilemapHigher as ::core::ffi::c_int != 0
    {
        tilemapAdr = (tilemapAdr as ::core::ffi::c_int
            + if (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].tilemapWider as ::core::ffi::c_int
                != 0
            {
                0x800 as ::core::ffi::c_int
            } else {
                0x400 as ::core::ffi::c_int
            }) as uint16_t;
    }
    return (*ppu).vram[(tilemapAdr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize];
}
unsafe fn ppu_getPixelForBgLayer(
    mut ppu: *mut Ppu,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut layer: ::core::ffi::c_int,
    mut priority: bool,
) -> ::core::ffi::c_int {
    let mut wideTiles: bool = (*ppu).bgLayer[layer as usize].bigTiles as ::core::ffi::c_int != 0
        || (*ppu).mode as ::core::ffi::c_int == 5 as ::core::ffi::c_int
        || (*ppu).mode as ::core::ffi::c_int == 6 as ::core::ffi::c_int;
    let mut tileBitsX: ::core::ffi::c_int = if wideTiles as ::core::ffi::c_int != 0 {
        4 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int
    };
    let mut tileHighBitX: ::core::ffi::c_int = if wideTiles as ::core::ffi::c_int != 0 {
        0x200 as ::core::ffi::c_int
    } else {
        0x100 as ::core::ffi::c_int
    };
    let mut tileBitsY: ::core::ffi::c_int =
        if (*ppu).bgLayer[layer as usize].bigTiles as ::core::ffi::c_int != 0 {
            4 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int
        };
    let mut tileHighBitY: ::core::ffi::c_int =
        if (*ppu).bgLayer[layer as usize].bigTiles as ::core::ffi::c_int != 0 {
            0x200 as ::core::ffi::c_int
        } else {
            0x100 as ::core::ffi::c_int
        };
    let mut tilemapAdr: uint16_t = ((*ppu).bgLayer[layer as usize].tilemapAdr as ::core::ffi::c_int
        + ((y >> tileBitsY & 0x1f as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | x >> tileBitsX & 0x1f as ::core::ffi::c_int))
        as uint16_t;
    if x & tileHighBitX != 0
        && (*ppu).bgLayer[layer as usize].tilemapWider as ::core::ffi::c_int != 0
    {
        tilemapAdr = (tilemapAdr as ::core::ffi::c_int + 0x400 as ::core::ffi::c_int) as uint16_t;
    }
    if y & tileHighBitY != 0
        && (*ppu).bgLayer[layer as usize].tilemapHigher as ::core::ffi::c_int != 0
    {
        tilemapAdr = (tilemapAdr as ::core::ffi::c_int
            + if (*ppu).bgLayer[layer as usize].tilemapWider as ::core::ffi::c_int != 0 {
                0x800 as ::core::ffi::c_int
            } else {
                0x400 as ::core::ffi::c_int
            }) as uint16_t;
    }
    let mut tile: uint16_t =
        (*ppu).vram[(tilemapAdr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize];
    if (tile as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
        != priority as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut paletteNum: ::core::ffi::c_int =
        (tile as ::core::ffi::c_int & 0x1c00 as ::core::ffi::c_int) >> 10 as ::core::ffi::c_int;
    let mut row: ::core::ffi::c_int =
        if tile as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            7 as ::core::ffi::c_int - (y & 0x7 as ::core::ffi::c_int)
        } else {
            y & 0x7 as ::core::ffi::c_int
        };
    let mut col: ::core::ffi::c_int =
        if tile as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0 {
            x & 0x7 as ::core::ffi::c_int
        } else {
            7 as ::core::ffi::c_int - (x & 0x7 as ::core::ffi::c_int)
        };
    let mut tileNum: ::core::ffi::c_int = tile as ::core::ffi::c_int & 0x3ff as ::core::ffi::c_int;
    if wideTiles {
        if (x & 8 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
            ^ (tile as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
            != 0
        {
            tileNum += 1 as ::core::ffi::c_int;
        }
    }
    if (*ppu).bgLayer[layer as usize].bigTiles {
        if (y & 8 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
            ^ (tile as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
            != 0
        {
            tileNum += 0x10 as ::core::ffi::c_int;
        }
    }
    let mut bitDepth: ::core::ffi::c_int = bitDepthsPerMode[(*ppu).mode as usize][layer as usize];
    if (*ppu).mode as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        paletteNum += 8 as ::core::ffi::c_int * layer;
    }
    let mut paletteSize: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
    let mut plane1: uint16_t = (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr
        as ::core::ffi::c_int
        + (tileNum & 0x3ff as ::core::ffi::c_int) * 4 as ::core::ffi::c_int * bitDepth
        + row
        & 0x7fff as ::core::ffi::c_int) as usize];
    let mut pixel: ::core::ffi::c_int =
        plane1 as ::core::ffi::c_int >> col & 1 as ::core::ffi::c_int;
    pixel |= (plane1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int + col
        & 1 as ::core::ffi::c_int)
        << 1 as ::core::ffi::c_int;
    if bitDepth > 2 as ::core::ffi::c_int {
        paletteSize = 16 as ::core::ffi::c_int;
        let mut plane2: uint16_t =
            (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as ::core::ffi::c_int
                + (tileNum & 0x3ff as ::core::ffi::c_int) * 4 as ::core::ffi::c_int * bitDepth
                + 8 as ::core::ffi::c_int
                + row
                & 0x7fff as ::core::ffi::c_int) as usize];
        pixel |= (plane2 as ::core::ffi::c_int >> col & 1 as ::core::ffi::c_int)
            << 2 as ::core::ffi::c_int;
        pixel |= (plane2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int + col
            & 1 as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int;
    }
    if bitDepth > 4 as ::core::ffi::c_int {
        paletteSize = 256 as ::core::ffi::c_int;
        let mut plane3: uint16_t =
            (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as ::core::ffi::c_int
                + (tileNum & 0x3ff as ::core::ffi::c_int) * 4 as ::core::ffi::c_int * bitDepth
                + 16 as ::core::ffi::c_int
                + row
                & 0x7fff as ::core::ffi::c_int) as usize];
        pixel |= (plane3 as ::core::ffi::c_int >> col & 1 as ::core::ffi::c_int)
            << 4 as ::core::ffi::c_int;
        pixel |= (plane3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int + col
            & 1 as ::core::ffi::c_int)
            << 5 as ::core::ffi::c_int;
        let mut plane4: uint16_t =
            (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as ::core::ffi::c_int
                + (tileNum & 0x3ff as ::core::ffi::c_int) * 4 as ::core::ffi::c_int * bitDepth
                + 24 as ::core::ffi::c_int
                + row
                & 0x7fff as ::core::ffi::c_int) as usize];
        pixel |= (plane4 as ::core::ffi::c_int >> col & 1 as ::core::ffi::c_int)
            << 6 as ::core::ffi::c_int;
        pixel |= (plane4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int + col
            & 1 as ::core::ffi::c_int)
            << 7 as ::core::ffi::c_int;
    }
    return if pixel == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        paletteSize * paletteNum + pixel
    };
}
unsafe fn ppu_calculateMode7Starts(mut ppu: *mut Ppu, mut y: ::core::ffi::c_int) {
    let mut hScroll: ::core::ffi::c_int =
        (((*ppu).m7matrix[6 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut vScroll: ::core::ffi::c_int =
        (((*ppu).m7matrix[7 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut xCenter: ::core::ffi::c_int =
        (((*ppu).m7matrix[4 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut yCenter: ::core::ffi::c_int =
        (((*ppu).m7matrix[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 3 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int;
    let mut clippedH: ::core::ffi::c_int = hScroll - xCenter;
    let mut clippedV: ::core::ffi::c_int = vScroll - yCenter;
    clippedH = if clippedH & 0x2000 as ::core::ffi::c_int != 0 {
        clippedH | !(1023 as ::core::ffi::c_int)
    } else {
        clippedH & 1023 as ::core::ffi::c_int
    };
    clippedV = if clippedV & 0x2000 as ::core::ffi::c_int != 0 {
        clippedV | !(1023 as ::core::ffi::c_int)
    } else {
        clippedV & 1023 as ::core::ffi::c_int
    };
    if (*ppu).bgLayer[0 as ::core::ffi::c_int as usize].mosaicEnabled as ::core::ffi::c_int != 0
        && (*ppu).mosaicSize as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        y -= (y - (*ppu).mosaicStartLine as ::core::ffi::c_int)
            % (*ppu).mosaicSize as ::core::ffi::c_int;
    }
    let mut ry: uint8_t = (if (*ppu).m7yFlip as ::core::ffi::c_int != 0 {
        255 as ::core::ffi::c_int - y
    } else {
        y
    }) as uint8_t;
    (*ppu).m7startX = (((*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        * clippedH
        & !(63 as ::core::ffi::c_int))
        + ((*ppu).m7matrix[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            * ry as ::core::ffi::c_int
            & !(63 as ::core::ffi::c_int))
        + ((*ppu).m7matrix[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedV
            & !(63 as ::core::ffi::c_int))
        + (xCenter << 8 as ::core::ffi::c_int)) as int32_t;
    (*ppu).m7startY = (((*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        * clippedH
        & !(63 as ::core::ffi::c_int))
        + ((*ppu).m7matrix[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            * ry as ::core::ffi::c_int
            & !(63 as ::core::ffi::c_int))
        + ((*ppu).m7matrix[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int * clippedV
            & !(63 as ::core::ffi::c_int))
        + (yCenter << 8 as ::core::ffi::c_int)) as int32_t;
}
unsafe fn ppu_getPixelForMode7(
    mut ppu: *mut Ppu,
    mut x: ::core::ffi::c_int,
    mut layer: ::core::ffi::c_int,
    mut priority: bool,
) -> ::core::ffi::c_int {
    let mut rx: uint8_t = (if (*ppu).m7xFlip as ::core::ffi::c_int != 0 {
        255 as ::core::ffi::c_int - x
    } else {
        x
    }) as uint8_t;
    let mut xPos: ::core::ffi::c_int = (*ppu).m7startX as ::core::ffi::c_int
        + (*ppu).m7matrix[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            * rx as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    let mut yPos: ::core::ffi::c_int = (*ppu).m7startY as ::core::ffi::c_int
        + (*ppu).m7matrix[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            * rx as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    let mut outsideMap: bool = xPos < 0 as ::core::ffi::c_int
        || xPos >= 1024 as ::core::ffi::c_int
        || yPos < 0 as ::core::ffi::c_int
        || yPos >= 1024 as ::core::ffi::c_int;
    xPos &= 0x3ff as ::core::ffi::c_int;
    yPos &= 0x3ff as ::core::ffi::c_int;
    if !(*ppu).m7largeField {
        outsideMap = false_0 != 0;
    }
    let mut tile: uint8_t = (if outsideMap as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        (*ppu).vram[((yPos >> 3 as ::core::ffi::c_int) * 128 as ::core::ffi::c_int
            + (xPos >> 3 as ::core::ffi::c_int)) as usize] as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
    }) as uint8_t;
    let mut pixel: uint8_t = (if outsideMap as ::core::ffi::c_int != 0 && !(*ppu).m7charFill {
        0 as ::core::ffi::c_int
    } else {
        (*ppu).vram[(tile as ::core::ffi::c_int * 64 as ::core::ffi::c_int
            + (yPos & 7 as ::core::ffi::c_int) * 8 as ::core::ffi::c_int
            + (xPos & 7 as ::core::ffi::c_int)) as usize] as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int
    }) as uint8_t;
    if layer == 1 as ::core::ffi::c_int {
        if (pixel as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) as ::core::ffi::c_int
            != priority as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        return pixel as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int;
    }
    return pixel as ::core::ffi::c_int;
}
unsafe fn ppu_getWindowState(
    mut ppu: *mut Ppu,
    mut layer: ::core::ffi::c_int,
    mut x: ::core::ffi::c_int,
) -> bool {
    if !(*ppu).windowLayer[layer as usize].window1enabled
        && !(*ppu).windowLayer[layer as usize].window2enabled
    {
        return false_0 != 0;
    }
    if (*ppu).windowLayer[layer as usize].window1enabled as ::core::ffi::c_int != 0
        && !(*ppu).windowLayer[layer as usize].window2enabled
    {
        let mut test: bool = x >= (*ppu).window1left as ::core::ffi::c_int
            && x <= (*ppu).window1right as ::core::ffi::c_int;
        return if (*ppu).windowLayer[layer as usize].window1inversed as ::core::ffi::c_int != 0 {
            !test as ::core::ffi::c_int
        } else {
            test as ::core::ffi::c_int
        } != 0;
    }
    if !(*ppu).windowLayer[layer as usize].window1enabled
        && (*ppu).windowLayer[layer as usize].window2enabled as ::core::ffi::c_int != 0
    {
        let mut test_0: bool = x >= (*ppu).window2left as ::core::ffi::c_int
            && x <= (*ppu).window2right as ::core::ffi::c_int;
        return if (*ppu).windowLayer[layer as usize].window2inversed as ::core::ffi::c_int != 0 {
            !test_0 as ::core::ffi::c_int
        } else {
            test_0 as ::core::ffi::c_int
        } != 0;
    }
    let mut test1: bool = x >= (*ppu).window1left as ::core::ffi::c_int
        && x <= (*ppu).window1right as ::core::ffi::c_int;
    let mut test2: bool = x >= (*ppu).window2left as ::core::ffi::c_int
        && x <= (*ppu).window2right as ::core::ffi::c_int;
    if (*ppu).windowLayer[layer as usize].window1inversed {
        test1 = !test1;
    }
    if (*ppu).windowLayer[layer as usize].window2inversed {
        test2 = !test2;
    }
    match (*ppu).windowLayer[layer as usize].maskLogic as ::core::ffi::c_int {
        0 => return test1 as ::core::ffi::c_int != 0 || test2 as ::core::ffi::c_int != 0,
        1 => return test1 as ::core::ffi::c_int != 0 && test2 as ::core::ffi::c_int != 0,
        2 => return test1 as ::core::ffi::c_int != test2 as ::core::ffi::c_int,
        3 => return test1 as ::core::ffi::c_int == test2 as ::core::ffi::c_int,
        _ => {}
    }
    return false_0 != 0;
}
unsafe fn ppu_evaluateSprites(mut ppu: *mut Ppu, mut line: ::core::ffi::c_int) -> bool {
    let mut index: uint8_t = (if (*ppu).objPriority as ::core::ffi::c_int != 0 {
        (*ppu).oamAdr as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint8_t;
    let mut spritesFound: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tilesFound: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 128 as ::core::ffi::c_int {
        let mut y: uint8_t = ((*ppu).oam[index as usize] as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8_t;
        let mut row: uint8_t = (line - y as ::core::ffi::c_int) as uint8_t;
        let mut spriteSize: ::core::ffi::c_int = spriteSizes[(*ppu).objSize as usize][((*ppu)
            .highOam[(index as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            >> (index as ::core::ffi::c_int & 7 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            as usize];
        let mut spriteHeight: ::core::ffi::c_int = if (*ppu).objInterlace as ::core::ffi::c_int != 0
        {
            spriteSize / 2 as ::core::ffi::c_int
        } else {
            spriteSize
        };
        if (row as ::core::ffi::c_int) < spriteHeight {
            let mut x: ::core::ffi::c_int =
                (*ppu).oam[index as usize] as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
            x |= ((*ppu).highOam[(index as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                >> (index as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int;
            if x > 255 as ::core::ffi::c_int {
                x -= 512 as ::core::ffi::c_int;
            }
            if x > -spriteSize {
                spritesFound += 1;
                if spritesFound > 32 as ::core::ffi::c_int {
                    (*ppu).rangeOver = true_0 != 0;
                    break;
                } else {
                    if (*ppu).objInterlace {
                        row = (row as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                            + (if (*ppu).evenFrame as ::core::ffi::c_int != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                1 as ::core::ffi::c_int
                            })) as uint8_t;
                    }
                    let mut oam1: ::core::ffi::c_int = (*ppu).oam
                        [(index as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int;
                    let mut tile: ::core::ffi::c_int = oam1 & 0xff as ::core::ffi::c_int;
                    let mut objAdr: ::core::ffi::c_int = if oam1 & 0x100 as ::core::ffi::c_int != 0
                    {
                        (*ppu).objTileAdr2 as ::core::ffi::c_int
                    } else {
                        (*ppu).objTileAdr1 as ::core::ffi::c_int
                    };
                    let mut palette: ::core::ffi::c_int =
                        (oam1 & 0xe00 as ::core::ffi::c_int) >> 9 as ::core::ffi::c_int;
                    let mut hFlipped: bool = oam1 & 0x4000 as ::core::ffi::c_int != 0;
                    if oam1 & 0x8000 as ::core::ffi::c_int != 0 {
                        row = (spriteSize - 1 as ::core::ffi::c_int - row as ::core::ffi::c_int)
                            as uint8_t;
                    }
                    let mut paletteBase: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int
                        + 16 as ::core::ffi::c_int
                            * ((oam1 & 0xe00 as ::core::ffi::c_int) >> 9 as ::core::ffi::c_int);
                    let mut prio: ::core::ffi::c_int = (((oam1 & 0x3000 as ::core::ffi::c_int)
                        >> 12 as ::core::ffi::c_int)
                        * 4 as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int)
                        * 16 as ::core::ffi::c_int
                        + 4 as ::core::ffi::c_int
                        + (if oam1 & 0x800 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            2 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        });
                    let mut z: PpuZbufType =
                        (paletteBase + (prio << 8 as ::core::ffi::c_int)) as PpuZbufType;
                    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while col < spriteSize {
                        if col + x > -(8 as ::core::ffi::c_int)
                            && col + x < 256 as ::core::ffi::c_int
                        {
                            tilesFound += 1;
                            if tilesFound > 34 as ::core::ffi::c_int {
                                (*ppu).timeOver = true_0 != 0;
                                break;
                            } else {
                                let mut usedCol: ::core::ffi::c_int =
                                    if oam1 & 0x4000 as ::core::ffi::c_int != 0 {
                                        spriteSize - 1 as ::core::ffi::c_int - col
                                    } else {
                                        col
                                    };
                                let mut usedTile: ::core::ffi::c_int = ((oam1
                                    & 0xff as ::core::ffi::c_int)
                                    >> 4 as ::core::ffi::c_int)
                                    + (row as ::core::ffi::c_int >> 3 as ::core::ffi::c_int)
                                    << 4 as ::core::ffi::c_int
                                    | (oam1 & 0xf as ::core::ffi::c_int)
                                        + (usedCol >> 3 as ::core::ffi::c_int)
                                        & 0xf as ::core::ffi::c_int;
                                let mut addr: *mut uint16 =
                                    (&raw mut (*ppu).vram as *mut uint16_t).offset(
                                        (objAdr
                                            + usedTile * 16 as ::core::ffi::c_int
                                            + (row as ::core::ffi::c_int
                                                & 0x7 as ::core::ffi::c_int)
                                            & 0x7fff as ::core::ffi::c_int)
                                            as isize,
                                    ) as *mut uint16;
                                let mut plane: uint32 = (*addr
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    | (*addr.offset(8 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        << 16 as ::core::ffi::c_int)
                                    as uint32;
                                let mut px_left: ::core::ffi::c_int = IntMax(
                                    -(col + x + kPpuExtraLeftRight as ::core::ffi::c_int),
                                    0 as ::core::ffi::c_int,
                                );
                                let mut px_right: ::core::ffi::c_int = IntMin(
                                    256 as ::core::ffi::c_int
                                        + kPpuExtraLeftRight as ::core::ffi::c_int
                                        - (col + x),
                                    8 as ::core::ffi::c_int,
                                );
                                let mut dst: *mut PpuZbufType = (&raw mut (*ppu).objBuffer.data
                                    as *mut PpuZbufType)
                                    .offset(col as isize)
                                    .offset(x as isize)
                                    .offset(px_left as isize)
                                    .offset(kPpuExtraLeftRight as ::core::ffi::c_int as isize);
                                if px_left < px_right {
                                    draw_sprite_tile_span(
                                        core::slice::from_raw_parts_mut(
                                            dst,
                                            (px_right - px_left) as usize,
                                        ),
                                        plane,
                                        z,
                                        px_left as u32,
                                        hFlipped,
                                    );
                                }
                            }
                        }
                        col += 8 as ::core::ffi::c_int;
                    }
                    if tilesFound > 34 as ::core::ffi::c_int {
                        break;
                    }
                }
            }
        }
        index = (index as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        i += 1;
    }
    return tilesFound != 0 as ::core::ffi::c_int;
}
unsafe fn ppu_getVramRemap(mut ppu: *mut Ppu) -> uint16_t {
    let mut adr: uint16_t = (*ppu).vramPointer;
    match (*ppu).vramRemapMode as ::core::ffi::c_int {
        0 => return adr,
        1 => {
            return (adr as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                    >> 5 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                    << 3 as ::core::ffi::c_int) as uint16_t;
        }
        2 => {
            return (adr as ::core::ffi::c_int & 0xfe00 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0x1c0 as ::core::ffi::c_int)
                    >> 6 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
                    << 3 as ::core::ffi::c_int) as uint16_t;
        }
        3 => {
            return (adr as ::core::ffi::c_int & 0xfc00 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0x380 as ::core::ffi::c_int)
                    >> 7 as ::core::ffi::c_int
                | (adr as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
                    << 3 as ::core::ffi::c_int) as uint16_t;
        }
        _ => {}
    }
    return adr;
}
pub unsafe fn ppu_read(mut ppu: *mut Ppu, mut adr: uint8_t) -> uint8_t {
    match adr as ::core::ffi::c_int {
        4 | 20 | 36 | 5 | 21 | 37 | 6 | 22 | 38 | 8 | 24 | 40 | 9 | 25 | 41 | 10 | 26 | 42 => {
            return (*ppu).ppu1openBus
        }
        52 | 53 | 54 => {
            let mut result: ::core::ffi::c_int = (*ppu).m7matrix[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                * ((*ppu).m7matrix[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int);
            (*ppu).ppu1openBus = (result
                >> 8 as ::core::ffi::c_int
                    * (adr as ::core::ffi::c_int - 0x34 as ::core::ffi::c_int)
                & 0xff as ::core::ffi::c_int) as uint8_t;
            return (*ppu).ppu1openBus;
        }
        55 => {
            (*ppu).hCount =
                ((*(*ppu).snes).hPos as ::core::ffi::c_int / 4 as ::core::ffi::c_int) as uint16_t;
            (*ppu).vCount = (*(*ppu).snes).vPos;
            (*ppu).countersLatched = true_0 != 0;
            return (*(*ppu).snes).openBus;
        }
        56 => {
            let mut ret: uint8_t = 0 as uint8_t;
            if (*ppu).oamInHigh {
                ret = (*ppu).highOam[(((*ppu).oamAdr as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                    << 1 as ::core::ffi::c_int
                    | (*ppu).oamSecondWrite as ::core::ffi::c_int)
                    as usize];
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                ret = ((*ppu).oam[(*ppu).oamAdr as usize] as ::core::ffi::c_int
                    & 0xff as ::core::ffi::c_int) as uint8_t;
            } else {
                let fresh2 = (*ppu).oamAdr;
                (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                ret = ((*ppu).oam[fresh2 as usize] as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                    as uint8_t;
                if (*ppu).oamAdr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    (*ppu).oamInHigh = true_0 != 0;
                }
            }
            (*ppu).oamSecondWrite = !(*ppu).oamSecondWrite;
            (*ppu).ppu1openBus = ret;
            return ret;
        }
        57 => {
            let mut val: uint16_t = (*ppu).vramReadBuffer;
            if !(*ppu).vramIncrementOnHigh {
                (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as ::core::ffi::c_int
                    & 0x7fff as ::core::ffi::c_int)
                    as usize];
                (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                    + (*ppu).vramIncrement as ::core::ffi::c_int)
                    as uint16_t;
            }
            (*ppu).ppu1openBus =
                (val as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            return (val as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
        }
        58 => {
            let mut val_0: uint16_t = (*ppu).vramReadBuffer;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as ::core::ffi::c_int
                    & 0x7fff as ::core::ffi::c_int)
                    as usize];
                (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                    + (*ppu).vramIncrement as ::core::ffi::c_int)
                    as uint16_t;
            }
            (*ppu).ppu1openBus =
                (val_0 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
            return (val_0 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        59 => {
            let mut ret_0: uint8_t = 0 as uint8_t;
            if !(*ppu).cgramSecondWrite {
                ret_0 = ((*ppu).cgram[(*ppu).cgramPointer as usize] as ::core::ffi::c_int
                    & 0xff as ::core::ffi::c_int) as uint8_t;
            } else {
                let fresh3 = (*ppu).cgramPointer;
                (*ppu).cgramPointer = (*ppu).cgramPointer.wrapping_add(1);
                ret_0 = ((*ppu).cgram[fresh3 as usize] as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int
                    & 0x7f as ::core::ffi::c_int
                    | (*ppu).ppu2openBus as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int)
                    as uint8_t;
            }
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            (*ppu).ppu2openBus = ret_0;
            return ret_0;
        }
        60 => {
            let mut val_1: uint8_t = 0 as uint8_t;
            if (*ppu).hCountSecond {
                val_1 = ((*ppu).hCount as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    | (*ppu).ppu2openBus as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int)
                    as uint8_t;
            } else {
                val_1 =
                    ((*ppu).hCount as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            }
            (*ppu).hCountSecond = !(*ppu).hCountSecond;
            (*ppu).ppu2openBus = val_1;
            return val_1;
        }
        61 => {
            let mut val_2: uint8_t = 0 as uint8_t;
            if (*ppu).vCountSecond {
                val_2 = ((*ppu).vCount as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    | (*ppu).ppu2openBus as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int)
                    as uint8_t;
            } else {
                val_2 =
                    ((*ppu).vCount as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            }
            (*ppu).vCountSecond = !(*ppu).vCountSecond;
            (*ppu).ppu2openBus = val_2;
            return val_2;
        }
        62 => {
            let mut val_3: uint8_t = 0x1 as uint8_t;
            val_3 = (val_3 as ::core::ffi::c_int
                | (*ppu).ppu1openBus as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int)
                as uint8_t;
            val_3 = (val_3 as ::core::ffi::c_int
                | ((*ppu).rangeOver as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
                as uint8_t;
            val_3 = (val_3 as ::core::ffi::c_int
                | ((*ppu).timeOver as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
            (*ppu).ppu1openBus = val_3;
            return val_3;
        }
        63 => {
            let mut val_4: uint8_t = 0x3 as uint8_t;
            val_4 = (val_4 as ::core::ffi::c_int
                | (*ppu).ppu2openBus as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int)
                as uint8_t;
            val_4 = (val_4 as ::core::ffi::c_int
                | ((*ppu).countersLatched as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
                as uint8_t;
            val_4 = (val_4 as ::core::ffi::c_int
                | ((*ppu).evenFrame as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
            (*ppu).countersLatched = false_0 != 0;
            (*ppu).hCountSecond = false_0 != 0;
            (*ppu).vCountSecond = false_0 != 0;
            (*ppu).ppu2openBus = val_4;
            return val_4;
        }
        _ => return (*(*ppu).snes).openBus,
    };
}
pub unsafe fn ppu_write(mut ppu: *mut Ppu, mut adr: uint8_t, mut val: uint8_t) {
    let mut current_block_164: u64;
    match adr as ::core::ffi::c_int {
        0 => {
            (*ppu).brightness = (val as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint8_t;
            (*ppu).forcedBlank = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            current_block_164 = 16754622181974910496;
        }
        1 => {
            (*ppu).objSize = (val as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as uint8_t;
            (*ppu).objTileAdr1 = ((val as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
                << 13 as ::core::ffi::c_int) as uint16_t;
            (*ppu).objTileAdr2 = ((*ppu).objTileAdr1 as ::core::ffi::c_int
                + (((val as ::core::ffi::c_int & 0x18 as ::core::ffi::c_int)
                    + 8 as ::core::ffi::c_int)
                    << 9 as ::core::ffi::c_int)) as uint16_t;
            current_block_164 = 16754622181974910496;
        }
        2 => {
            (*ppu).oamAdr = val;
            (*ppu).oamAdrWritten = (*ppu).oamAdr;
            (*ppu).oamInHigh = (*ppu).oamInHighWritten;
            (*ppu).oamSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        3 => {
            (*ppu).objPriority = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            (*ppu).oamInHigh = val as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
            (*ppu).oamInHighWritten = (*ppu).oamInHigh;
            (*ppu).oamAdr = (*ppu).oamAdrWritten;
            (*ppu).oamSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        4 => {
            if (*ppu).oamInHigh {
                (*ppu).highOam[(((*ppu).oamAdr as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                    << 1 as ::core::ffi::c_int
                    | (*ppu).oamSecondWrite as ::core::ffi::c_int)
                    as usize] = val;
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                (*ppu).oamBuffer = val;
            } else {
                let fresh4 = (*ppu).oamAdr;
                (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                (*ppu).oam[fresh4 as usize] =
                    ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                        | (*ppu).oamBuffer as ::core::ffi::c_int) as uint16_t;
                if (*ppu).oamAdr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    (*ppu).oamInHigh = true_0 != 0;
                }
            }
            (*ppu).oamSecondWrite = !(*ppu).oamSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        5 => {
            (*ppu).mode = (val as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int) as uint8_t;
            (*ppu).bg3priority = val as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[0 as ::core::ffi::c_int as usize].bigTiles =
                val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[1 as ::core::ffi::c_int as usize].bigTiles =
                val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].bigTiles =
                val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[3 as ::core::ffi::c_int as usize].bigTiles =
                val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            current_block_164 = 16754622181974910496;
        }
        6 => {
            (*ppu).bgLayer[0 as ::core::ffi::c_int as usize].mosaicEnabled =
                val as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[1 as ::core::ffi::c_int as usize].mosaicEnabled =
                val as ::core::ffi::c_int & 0x2 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[2 as ::core::ffi::c_int as usize].mosaicEnabled =
                val as ::core::ffi::c_int & 0x4 as ::core::ffi::c_int != 0;
            (*ppu).bgLayer[3 as ::core::ffi::c_int as usize].mosaicEnabled =
                val as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0;
            (*ppu).mosaicSize = ((val as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as uint8_t;
            (*ppu).mosaicStartLine = 0 as uint8_t;
            current_block_164 = 16754622181974910496;
        }
        7 | 8 | 9 | 10 => {
            let bg = &mut (*ppu).bgLayer[usize::from(adr - 7)];
            bg.tilemapWider = val & 1 != 0;
            bg.tilemapHigher = val & 2 != 0;
            bg.tilemapAdr = u16::from(val & 0xfc) << 8;
            current_block_164 = 16754622181974910496;
        }
        11 | 12 => {
            let first_bg = usize::from(adr - 11) * 2;
            (*ppu).bgLayer[first_bg].tileAdr = u16::from(val & 0x0f) << 12;
            (*ppu).bgLayer[first_bg + 1].tileAdr = u16::from(val & 0xf0) << 8;
            current_block_164 = 16754622181974910496;
        }
        13 => {
            (*ppu).m7matrix[6] = ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 6174974146017752131;
        }
        15 | 17 | 19 => {
            current_block_164 = 6174974146017752131;
        }
        14 => {
            (*ppu).m7matrix[7] = ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 5892776923941496671;
        }
        16 | 18 | 20 => {
            current_block_164 = 5892776923941496671;
        }
        21 => {
            if val as ::core::ffi::c_int & 3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*ppu).vramIncrement = 1 as uint16_t;
            } else if val as ::core::ffi::c_int & 3 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            {
                (*ppu).vramIncrement = 32 as uint16_t;
            } else {
                (*ppu).vramIncrement = 128 as uint16_t;
            }
            (*ppu).vramRemapMode = ((val as ::core::ffi::c_int & 0xc as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int) as uint8_t;
            (*ppu).vramIncrementOnHigh =
                val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            current_block_164 = 16754622181974910496;
        }
        22 => {
            (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int
                | val as ::core::ffi::c_int) as uint16_t;
            (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as ::core::ffi::c_int
                & 0x7fff as ::core::ffi::c_int)
                as usize];
            current_block_164 = 16754622181974910496;
        }
        23 => {
            (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int
                | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as ::core::ffi::c_int
                & 0x7fff as ::core::ffi::c_int)
                as usize];
            current_block_164 = 16754622181974910496;
        }
        24 => {
            let mut vramAdr: uint16_t = ppu_getVramRemap(ppu);
            (*ppu).vram[(vramAdr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize] =
                ((*ppu).vram
                    [(vramAdr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int
                    | val as ::core::ffi::c_int) as uint16_t;
            if !(*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                    + (*ppu).vramIncrement as ::core::ffi::c_int)
                    as uint16_t;
            }
            current_block_164 = 16754622181974910496;
        }
        25 => {
            let mut vramAdr_0: uint16_t = ppu_getVramRemap(ppu);
            (*ppu).vram
                [(vramAdr_0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize] =
                ((*ppu).vram
                    [(vramAdr_0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & 0xff as ::core::ffi::c_int
                    | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    as uint16_t;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer = ((*ppu).vramPointer as ::core::ffi::c_int
                    + (*ppu).vramIncrement as ::core::ffi::c_int)
                    as uint16_t;
            }
            current_block_164 = 16754622181974910496;
        }
        26 => {
            (*ppu).m7largeField = val & 0x80 != 0;
            (*ppu).m7charFill = val & 0x40 != 0;
            (*ppu).m7yFlip = val & 0x02 != 0;
            (*ppu).m7xFlip = val & 0x01 != 0;
            current_block_164 = 16754622181974910496;
        }
        27 | 28 | 29 | 30 => {
            (*ppu).m7matrix[usize::from(adr - 0x1b)] =
                (u16::from(val) << 8 | u16::from((*ppu).m7prev)) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 16754622181974910496;
        }
        31 | 32 => {
            (*ppu).m7matrix[usize::from(adr - 0x1b)] =
                ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 16754622181974910496;
        }
        33 => {
            (*ppu).cgramPointer = val;
            (*ppu).cgramSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        34 => {
            if !(*ppu).cgramSecondWrite {
                (*ppu).cgramBuffer = val;
            } else {
                let fresh5 = (*ppu).cgramPointer;
                (*ppu).cgramPointer = (*ppu).cgramPointer.wrapping_add(1);
                (*ppu).cgram[fresh5 as usize] =
                    ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                        | (*ppu).cgramBuffer as ::core::ffi::c_int) as uint16_t;
            }
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        35 | 36 | 37 => {
            if adr as ::core::ffi::c_int == 0x23 as ::core::ffi::c_int {
                (*ppu).windowsel =
                    (*ppu).windowsel & !(0xff as ::core::ffi::c_int) as uint32_t | val as uint32_t;
            } else if adr as ::core::ffi::c_int == 0x24 as ::core::ffi::c_int {
                (*ppu).windowsel = (*ppu).windowsel & !(0xff00 as ::core::ffi::c_int) as uint32_t
                    | ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32_t;
            } else if adr as ::core::ffi::c_int == 0x25 as ::core::ffi::c_int {
                (*ppu).windowsel = (*ppu).windowsel & !(0xff0000 as ::core::ffi::c_int) as uint32_t
                    | ((val as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32_t;
            }
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as usize]
                .window1inversed = val as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as usize]
                .window1enabled = val as ::core::ffi::c_int & 0x2 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as usize]
                .window2inversed = val as ::core::ffi::c_int & 0x4 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as usize]
                .window2enabled = val as ::core::ffi::c_int & 0x8 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize]
                .window1inversed = val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize]
                .window1enabled = val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize]
                .window2inversed = val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
            (*ppu).windowLayer[((adr as ::core::ffi::c_int - 0x23 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize]
                .window2enabled = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            current_block_164 = 16754622181974910496;
        }
        38 => {
            (*ppu).window1left = val;
            current_block_164 = 16754622181974910496;
        }
        39 => {
            (*ppu).window1right = val;
            current_block_164 = 16754622181974910496;
        }
        40 => {
            (*ppu).window2left = val;
            current_block_164 = 16754622181974910496;
        }
        41 => {
            (*ppu).window2right = val;
            current_block_164 = 16754622181974910496;
        }
        42 => {
            (*ppu).windowLayer[0 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int) as uint8_t;
            (*ppu).windowLayer[1 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                    as uint8_t;
            (*ppu).windowLayer[2 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int >> 4 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                    as uint8_t;
            (*ppu).windowLayer[3 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int >> 6 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                    as uint8_t;
            current_block_164 = 16754622181974910496;
        }
        43 => {
            (*ppu).windowLayer[4 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int) as uint8_t;
            (*ppu).windowLayer[5 as ::core::ffi::c_int as usize].maskLogic =
                (val as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                    as uint8_t;
            current_block_164 = 16754622181974910496;
        }
        44 => {
            (*ppu).screenEnabled[0] = val;
            (*ppu).layer[0].mainScreenEnabled = val & 0x01 != 0;
            (*ppu).layer[1].mainScreenEnabled = val & 0x02 != 0;
            (*ppu).layer[2].mainScreenEnabled = val & 0x04 != 0;
            (*ppu).layer[3].mainScreenEnabled = val & 0x08 != 0;
            (*ppu).layer[4].mainScreenEnabled = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        45 => {
            (*ppu).screenEnabled[1] = val;
            (*ppu).layer[0].subScreenEnabled = val & 0x01 != 0;
            (*ppu).layer[1].subScreenEnabled = val & 0x02 != 0;
            (*ppu).layer[2].subScreenEnabled = val & 0x04 != 0;
            (*ppu).layer[3].subScreenEnabled = val & 0x08 != 0;
            (*ppu).layer[4].subScreenEnabled = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        46 => {
            (*ppu).screenWindowed[0] = val;
            (*ppu).layer[0].mainScreenWindowed = val & 0x01 != 0;
            (*ppu).layer[1].mainScreenWindowed = val & 0x02 != 0;
            (*ppu).layer[2].mainScreenWindowed = val & 0x04 != 0;
            (*ppu).layer[3].mainScreenWindowed = val & 0x08 != 0;
            (*ppu).layer[4].mainScreenWindowed = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        47 => {
            (*ppu).screenWindowed[1] = val;
            (*ppu).layer[0].subScreenWindowed = val & 0x01 != 0;
            (*ppu).layer[1].subScreenWindowed = val & 0x02 != 0;
            (*ppu).layer[2].subScreenWindowed = val & 0x04 != 0;
            (*ppu).layer[3].subScreenWindowed = val & 0x08 != 0;
            (*ppu).layer[4].subScreenWindowed = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        48 => {
            (*ppu).directColor = val & 0x01 != 0;
            (*ppu).addSubscreen = val & 0x02 != 0;
            (*ppu).preventMathMode = (val & 0x30) >> 4;
            (*ppu).clipMode = (val & 0xc0) >> 6;
            current_block_164 = 16754622181974910496;
        }
        49 => {
            (*ppu).subtractColor = val & 0x80 != 0;
            (*ppu).halfColor = val & 0x40 != 0;
            for layer in 0..6 {
                (*ppu).mathEnabled[layer] = val & (1 << layer) != 0;
            }
            current_block_164 = 16754622181974910496;
        }
        50 => {
            if val & 0x80 != 0 {
                (*ppu).fixedColorB = val & 0x1f;
            }
            if val & 0x40 != 0 {
                (*ppu).fixedColorG = val & 0x1f;
            }
            if val & 0x20 != 0 {
                (*ppu).fixedColorR = val & 0x1f;
            }
            current_block_164 = 16754622181974910496;
        }
        51 => {
            (*ppu).interlace = val & 0x01 != 0;
            (*ppu).objInterlace = val & 0x02 != 0;
            (*ppu).overscan = val & 0x04 != 0;
            (*ppu).pseudoHires = val & 0x08 != 0;
            (*ppu).m7extBg = val & 0x40 != 0;
            current_block_164 = 16754622181974910496;
        }
        _ => {
            current_block_164 = 16754622181974910496;
        }
    }
    match current_block_164 {
        5892776923941496671 => {
            let layer = (usize::from(adr) - 0x0e) / 2;
            (*ppu).bgLayer[layer].vScroll =
                ((u16::from(val) << 8) | u16::from((*ppu).scrollPrev)) & 0x3ff;
            (*ppu).scrollPrev = val;
        }
        6174974146017752131 => {
            let layer = (usize::from(adr) - 0x0d) / 2;
            (*ppu).bgLayer[layer].hScroll = ((u16::from(val) << 8)
                | u16::from((*ppu).scrollPrev & 0xf8)
                | u16::from((*ppu).scrollPrev2 & 0x07))
                & 0x3ff;
            (*ppu).scrollPrev = val;
            (*ppu).scrollPrev2 = val;
        }
        _ => {}
    };
}
pub unsafe fn PpuGetCurrentRenderScale(
    mut ppu: *mut Ppu,
    mut render_flags: uint32_t,
) -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}

fn draw_palette_span(
    dst: &mut [u32],
    pixels: &[u16],
    palette: &[u16; 256],
    brightness: &[u8; 63],
    mask: u16,
) {
    for (dst, &pixel) in dst.iter_mut().zip(pixels) {
        let color = palette[usize::from(pixel & 0xff)];
        *dst = u32::from(brightness[usize::from(color & mask)]) << 16
            | u32::from(brightness[usize::from((color >> 5) & mask)]) << 8
            | u32::from(brightness[usize::from((color >> 10) & mask)]);
    }
}

const SPRITE_BIT_SPREAD: [u32; 256] = {
    let mut table = [0; 256];
    let mut byte = 0;
    while byte < table.len() {
        let mut bit = 0;
        while bit < 8 {
            table[byte] |= (((byte >> bit) & 1) as u32) << (bit * 4);
            bit += 1;
        }
        byte += 1;
    }
    table
};

fn draw_sprite_tile_span(dst: &mut [u16], plane: u32, z: u16, px_left: u32, hflip: bool) {
    let mut pixels = SPRITE_BIT_SPREAD[(plane & 0xff) as usize]
        | (SPRITE_BIT_SPREAD[((plane >> 8) & 0xff) as usize] << 1)
        | (SPRITE_BIT_SPREAD[((plane >> 16) & 0xff) as usize] << 2)
        | (SPRITE_BIT_SPREAD[(plane >> 24) as usize] << 3);
    if hflip {
        pixels >>= px_left * 4;
        for dst in dst {
            let pixel = (pixels & 0xf) as u16;
            if pixel != 0 && *dst & 0xff == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels >>= 4;
        }
    } else {
        pixels <<= px_left * 4;
        for dst in dst {
            let pixel = (pixels >> 28) as u16;
            if pixel != 0 && *dst & 0xff == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels <<= 4;
        }
    }
}
