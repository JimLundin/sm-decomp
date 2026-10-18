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
pub unsafe fn dma_reset(dma: *mut Dma) {
    for i in 0..8 {
        (*dma).channel[i].bAdr = 0xff;
        (*dma).channel[i].aAdr = 0xffff;
        (*dma).channel[i].aBank = 0xff;
        (*dma).channel[i].size = 0xffff;
        (*dma).channel[i].indBank = 0xff;
        (*dma).channel[i].tableAdr = 0xffff;
        (*dma).channel[i].repCount = 0xff;
        (*dma).channel[i].unusedByte = 0xff;
        (*dma).channel[i].dmaActive = false;
        (*dma).channel[i].hdmaActive = false;
        (*dma).channel[i].mode = 7;
        (*dma).channel[i].fixed = true;
        (*dma).channel[i].decrement = true;
        (*dma).channel[i].indirect = true;
        (*dma).channel[i].fromB = true;
        (*dma).channel[i].unusedBit = true;
        (*dma).channel[i].doTransfer = false;
        (*dma).channel[i].terminated = false;
        (*dma).channel[i].offIndex = 0;
    }
    (*dma).hdmaTimer = 0;
    (*dma).dmaTimer = 0;
    (*dma).dmaBusy = false;
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
pub unsafe fn dma_read(dma: *mut Dma, address: u16) -> u8 {
    let channel = &(*dma).channel[usize::from((address >> 4) & 7)];
    match address & 0x0f {
        0 => {
            channel.mode
                | (u8::from(channel.fixed) << 3)
                | (u8::from(channel.decrement) << 4)
                | (u8::from(channel.unusedBit) << 5)
                | (u8::from(channel.indirect) << 6)
                | (u8::from(channel.fromB) << 7)
        }
        1 => channel.bAdr,
        2 => channel.aAdr as u8,
        3 => (channel.aAdr >> 8) as u8,
        4 => channel.aBank,
        5 => channel.size as u8,
        6 => (channel.size >> 8) as u8,
        7 => channel.indBank,
        8 => channel.tableAdr as u8,
        9 => (channel.tableAdr >> 8) as u8,
        10 => channel.repCount,
        11 | 15 => channel.unusedByte,
        _ => {
            __assert_fail(
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/snes/dma.c\0" as *const u8 as *const ::core::ffi::c_char,
                118 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    }
}
pub unsafe fn dma_write(dma: *mut Dma, address: u16, value: u8) {
    let channel = &mut (*dma).channel[usize::from((address >> 4) & 7)];
    match address & 0x0f {
        0 => {
            channel.mode = value & 7;
            channel.fixed = value & 0x08 != 0;
            channel.decrement = value & 0x10 != 0;
            channel.unusedBit = value & 0x20 != 0;
            channel.indirect = value & 0x40 != 0;
            channel.fromB = value & 0x80 != 0;
        }
        1 => channel.bAdr = value,
        2 => channel.aAdr = (channel.aAdr & 0xff00) | u16::from(value),
        3 => channel.aAdr = (channel.aAdr & 0x00ff) | (u16::from(value) << 8),
        4 => channel.aBank = value,
        5 => channel.size = (channel.size & 0xff00) | u16::from(value),
        6 => channel.size = (channel.size & 0x00ff) | (u16::from(value) << 8),
        7 => channel.indBank = value,
        8 => channel.tableAdr = (channel.tableAdr & 0xff00) | u16::from(value),
        9 => channel.tableAdr = (channel.tableAdr & 0x00ff) | (u16::from(value) << 8),
        10 => channel.repCount = value,
        11 | 15 => channel.unusedByte = value,
        _ => {}
    }
}
pub unsafe fn dma_doDma(mut dma: *mut Dma, game: &mut Game) {
    if (*dma).dmaTimer > 0 {
        (*dma).dmaTimer = (*dma).dmaTimer.wrapping_sub(2);
        return;
    }
    let mut i = 0usize;
    while i < 8 {
        if (*dma).channel[i].dmaActive {
            break;
        }
        i += 1;
    }
    if i == 8 {
        (*dma).dmaBusy = false;
        return;
    }
    if !(*dma).channel[i].fromB
        && (*dma).channel[i].aBank & 0x80 != 0
        && (*dma).channel[i].aAdr & 0x8000 == 0
        && !game.g_fail
    {
        printf(
            b"Warning! DMA from addr 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            ((*dma).channel[i].aBank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*dma).channel[i].aAdr as ::core::ffi::c_int,
        );
        game.g_fail = true_0 != 0;
    }
    let fresh0 = (*dma).channel[i].offIndex;
    (*dma).channel[i].offIndex = (*dma).channel[i].offIndex.wrapping_add(1);
    dma_transferByte(
        dma,
        (*dma).channel[i].aAdr,
        (*dma).channel[i].aBank,
        (*dma).channel[i].bAdr.wrapping_add(
            bAdrOffsets[usize::from((*dma).channel[i].mode)][usize::from(fresh0)] as u8,
        ),
        (*dma).channel[i].fromB,
        game,
    );
    (*dma).channel[i].offIndex &= 3;
    (*dma).dmaTimer = (*dma).dmaTimer.wrapping_add(6);
    if !(*dma).channel[i].fixed {
        (*dma).channel[i].aAdr = if (*dma).channel[i].decrement {
            (*dma).channel[i].aAdr.wrapping_sub(1)
        } else {
            (*dma).channel[i].aAdr.wrapping_add(1)
        };
    }
    (*dma).channel[i].size = (*dma).channel[i].size.wrapping_sub(1);
    if (*dma).channel[i].size == 0 {
        (*dma).channel[i].offIndex = 0;
        (*dma).channel[i].dmaActive = false;
        (*dma).dmaTimer = (*dma).dmaTimer.wrapping_add(8);
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
    dma: *mut Dma,
    a_address: u16,
    a_bank: u8,
    b_address: u8,
    from_b: bool,
    game: &mut Game,
) {
    let address = (u32::from(a_bank) << 16) | u32::from(a_address);
    let snes = (*dma).snes;
    if from_b {
        let value = snes_readBBus(snes, b_address, game);
        snes_write(snes, address, value, game);
    } else {
        let value = snes_read(snes, address, game);
        snes_writeBBus(snes, b_address, value, game);
    }
}
pub unsafe fn dma_cycle(dma: *mut Dma, game: &mut Game) -> bool {
    if (*dma).hdmaTimer > 0 {
        (*dma).hdmaTimer = (*dma).hdmaTimer.wrapping_sub(2);
        true
    } else if (*dma).dmaBusy {
        dma_doDma(dma, game);
        true
    } else {
        false
    }
}
pub unsafe fn dma_startDma(dma: *mut Dma, mask: u8, hdma: bool) {
    for i in 0..8 {
        let active = mask & (1 << i) != 0;
        if hdma {
            (*dma).channel[i].hdmaActive = active;
        } else {
            (*dma).channel[i].dmaActive = active;
        }
    }
    if !hdma {
        (*dma).dmaBusy = mask != 0;
        (*dma).dmaTimer = (*dma)
            .dmaTimer
            .wrapping_add(if (*dma).dmaBusy { 16 } else { 0 });
    }
}
