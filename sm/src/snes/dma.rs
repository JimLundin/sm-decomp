use crate::game_state::Game;

use crate::snes::snes::{snes_read, snes_readBBus, snes_write, snes_writeBBus};
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

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int32_t, int8_t,
    size_t, uint16_t, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, SaveLoadFunc, Snes, Spc, Timer, WindowLayer,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"uint8_t dma_read(Dma *, uint16_t)\0",
    )
};
static bAdrOffsets: [[::core::ffi::c_int; 4]; 8] = [
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
static transferLength: [::core::ffi::c_int; 8] = [
    1 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
];
pub unsafe fn dma_init(mut snes: *mut Snes) -> *mut Dma {
    let mut dma: *mut Dma = malloc(::core::mem::size_of::<Dma>() as size_t) as *mut Dma;
    (*dma).snes = snes;
    return dma;
}
pub unsafe fn dma_free(mut dma: *mut Dma) {
    free(dma as *mut ::core::ffi::c_void);
}
pub unsafe fn dma_reset(mut dma: *mut Dma) {
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
pub unsafe fn dma_saveload(
    mut dma: *mut Dma,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*dma).channel as *mut ::core::ffi::c_void,
        (225 as size_t)
            .wrapping_add(7 as size_t)
            .wrapping_sub(8 as size_t),
        game,
    );
}
pub unsafe fn dma_read(mut dma: *mut Dma, mut adr: uint16_t) -> uint8_t {
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
pub unsafe fn dma_write(mut dma: *mut Dma, mut adr: uint16_t, mut val: uint8_t) {
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
pub unsafe fn dma_doDma(mut dma: *mut Dma, game: &mut Game) {
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
        && !game.g_fail
    {
        printf(
            b"Warning! DMA from addr 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            ((*dma).channel[i as usize].aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*dma).channel[i as usize].aAdr as ::core::ffi::c_int,
        );
        game.g_fail = true_0 != 0;
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
        game,
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
pub unsafe fn dma_initHdma(mut dma: *mut Dma, game: &mut Game) {
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
                game,
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
                        game,
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
                                game,
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
pub unsafe fn dma_doHdma(mut dma: *mut Dma, game: &mut Game) {
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
                            game,
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
                            game,
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
                    game,
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
                        game,
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
                                game,
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
unsafe fn dma_transferByte(
    mut dma: *mut Dma,
    mut aAdr: uint16_t,
    mut aBank: uint8_t,
    mut bAdr: uint8_t,
    mut fromB: bool,
    game: &mut Game,
) {
    if fromB {
        snes_write(
            (*dma).snes,
            ((aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | aAdr as ::core::ffi::c_int)
                as uint32_t,
            snes_readBBus((*dma).snes, bAdr, game),
            game,
        );
    } else {
        snes_writeBBus(
            (*dma).snes,
            bAdr,
            snes_read(
                (*dma).snes,
                ((aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                    | aAdr as ::core::ffi::c_int) as uint32_t,
                game,
            ),
            game,
        );
    };
}
pub unsafe fn dma_cycle(mut dma: *mut Dma, game: &mut Game) -> bool {
    if (*dma).hdmaTimer as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        (*dma).hdmaTimer =
            ((*dma).hdmaTimer as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16_t;
        return true_0 != 0;
    } else if (*dma).dmaBusy {
        dma_doDma(dma, game);
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe fn dma_startDma(mut dma: *mut Dma, mut val: uint8_t, mut hdma: bool) {
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
