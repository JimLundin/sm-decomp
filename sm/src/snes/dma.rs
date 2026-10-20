use crate::game_state::Game;
use crate::set_fields;
use ::core::ffi::{c_char, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{__assert_fail, free, malloc, printf};
use crate::snes::snes::{snes_read, snes_readBBus, snes_write, snes_writeBBus};
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, SaveLoadFunc, Snes, Spc,
    Timer, WindowLayer,
};
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const __ASSERT_FUNCTION: [c_char; 34] =
    unsafe { transmute::<[u8; 34], [c_char; 34]>(*b"uint8_t dma_read(Dma *, uint16_t)\0") };
static bAdrOffsets: [[i32; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];
static transferLength: [i32; 8] = [1, 2, 2, 4, 4, 4, 2, 4];
pub unsafe fn dma_init(snes: *mut Snes) -> *mut Dma {
    let dma: *mut Dma = malloc(size_of::<Dma>() as size_t) as *mut Dma;
    (*dma).snes = snes;
    dma
}
pub unsafe fn dma_free(dma: *mut Dma) {
    free(dma as *mut c_void);
}
pub unsafe fn dma_reset(dma: *mut Dma) {
    for i in 0..8 {
        (*dma).channel[i].bAdr = 0xff;
        (*dma).channel[i].aAdr = 0xffff;
        (*dma).channel[i].aBank = 0xff;
        (*dma).channel[i].size = 0xffff;
        (*dma).channel[i].indBank = 0xff;
        (*dma).channel[i].tableAdr = 0xffff;
        set_fields! {[(*dma).channel[i]], 0xff; repCount, unusedByte}
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
    set_fields! {[(*dma)], 0; hdmaTimer, dmaTimer}
    (*dma).dmaBusy = false;
}
pub unsafe fn dma_saveload(
    dma: *mut Dma,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*dma).channel as *mut c_void,
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
            crate::native_assertion! {
                @failure
                b"0\0", b"src/snes/dma.c\0", 118u32, pointer __ASSERT_FUNCTION.as_ptr()
            };
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
pub unsafe fn dma_doDma(dma: *mut Dma, game: &mut Game) {
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
            b"Warning! DMA from addr 0x%x\n\0" as *const u8 as *const c_char,
            ((*dma).channel[i].aBank as i32) << 16i32 | (*dma).channel[i].aAdr as i32,
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
// Advance the table pointer before the bus read, as HDMA does for every byte.
unsafe fn hdma_read_table_byte(dma: *mut Dma, channel: usize, game: &mut Game) -> u8 {
    let address = (*dma).channel[channel].tableAdr;
    (*dma).channel[channel].tableAdr = address.wrapping_add(1);
    snes_read(
        (*dma).snes,
        (u32::from((*dma).channel[channel].aBank) << 16) | u32::from(address),
        game,
    )
}
unsafe fn hdma_load_indirect_address(dma: *mut Dma, channel: usize, game: &mut Game) {
    (*dma).channel[channel].size = u16::from(hdma_read_table_byte(dma, channel, game));
    (*dma).channel[channel].size =
        (*dma).channel[channel].size | (u16::from(hdma_read_table_byte(dma, channel, game)) << 8);
    (*dma).hdmaTimer = (*dma).hdmaTimer.wrapping_add(16);
}
pub unsafe fn dma_initHdma(dma: *mut Dma, game: &mut Game) {
    (*dma).hdmaTimer = 0u16;
    let mut hdmaHappened: bool = false_0 != 0;
    let mut keep_channel_active: bool;
    let mut i: i32 = 0i32;
    while i < 8i32 {
        keep_channel_active = if (*dma).channel[i as usize].hdmaActive {
            hdmaHappened = true_0 != 0;
            (*dma).channel[i as usize].dmaActive = false_0 != 0;
            (*dma).channel[i as usize].offIndex = 0u8;
            (*dma).channel[i as usize].tableAdr = (*dma).channel[i as usize].aAdr;
            (*dma).channel[i as usize].repCount = hdma_read_table_byte(dma, i as usize, game);
            if (*dma).channel[i as usize].repCount as i32 == 0i32 {
                (*dma).channel[i as usize].terminated = true_0 != 0;
                false
            } else {
                (*dma).hdmaTimer = ((*dma).hdmaTimer as i32 + 8i32) as u16;
                if (*dma).channel[i as usize].indirect {
                    hdma_load_indirect_address(dma, i as usize, game);
                }
                (*dma).channel[i as usize].doTransfer = true_0 != 0;
                true
            }
        } else {
            (*dma).channel[i as usize].doTransfer = false_0 != 0;
            true
        };
        if keep_channel_active {
            (*dma).channel[i as usize].terminated = false_0 != 0;
        }
        i += 1;
    }
    if hdmaHappened {
        (*dma).hdmaTimer = ((*dma).hdmaTimer as i32 + 16i32) as u16;
    }
}
pub unsafe fn dma_doHdma(dma: *mut Dma, game: &mut Game) {
    (*dma).hdmaTimer = 0u16;
    let mut hdmaHappened: bool = false_0 != 0;
    let mut i: i32 = 0i32;
    while i < 8i32 {
        if (*dma).channel[i as usize].hdmaActive as i32 != 0
            && !(*dma).channel[i as usize].terminated
        {
            hdmaHappened = true_0 != 0;
            (*dma).channel[i as usize].dmaActive = false_0 != 0;
            (*dma).channel[i as usize].offIndex = 0u8;
            (*dma).hdmaTimer = ((*dma).hdmaTimer as i32 + 8i32) as u16;
            if (*dma).channel[i as usize].doTransfer {
                let mut j: i32 = 0i32;
                while j < transferLength[(*dma).channel[i as usize].mode as usize] {
                    (*dma).hdmaTimer = ((*dma).hdmaTimer as i32 + 8i32) as u16;
                    if (*dma).channel[i as usize].indirect {
                        let fresh4 = (*dma).channel[i as usize].size;
                        (*dma).channel[i as usize].size =
                            (*dma).channel[i as usize].size.wrapping_add(1);
                        dma_transferByte(
                            dma,
                            fresh4,
                            (*dma).channel[i as usize].indBank,
                            ((*dma).channel[i as usize].bAdr as i32
                                + bAdrOffsets[(*dma).channel[i as usize].mode as usize][j as usize])
                                as u8,
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
                            ((*dma).channel[i as usize].bAdr as i32
                                + bAdrOffsets[(*dma).channel[i as usize].mode as usize][j as usize])
                                as u8,
                            (*dma).channel[i as usize].fromB,
                            game,
                        );
                    }
                    j += 1;
                }
            }
            (*dma).channel[i as usize].repCount =
                (*dma).channel[i as usize].repCount.wrapping_sub(1);
            (*dma).channel[i as usize].doTransfer =
                (*dma).channel[i as usize].repCount as i32 & 0x80i32 != 0;
            if (*dma).channel[i as usize].repCount as i32 & 0x7fi32 == 0i32 {
                (*dma).channel[i as usize].repCount = hdma_read_table_byte(dma, i as usize, game);
                if (*dma).channel[i as usize].indirect {
                    hdma_load_indirect_address(dma, i as usize, game);
                }
                if (*dma).channel[i as usize].repCount as i32 == 0i32 {
                    (*dma).channel[i as usize].terminated = true_0 != 0;
                }
                (*dma).channel[i as usize].doTransfer = true_0 != 0;
            }
        }
        i += 1;
    }
    if hdmaHappened {
        (*dma).hdmaTimer = ((*dma).hdmaTimer as i32 + 16i32) as u16;
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
    let channels = &mut (*dma).channel;
    for (i, channel) in channels.iter_mut().enumerate() {
        let active = mask & (1 << i) != 0;
        if hdma {
            channel.hdmaActive = active;
        } else {
            channel.dmaActive = active;
        }
    }
    if !hdma {
        (*dma).dmaBusy = mask != 0;
        (*dma).dmaTimer = (*dma)
            .dmaTimer
            .wrapping_add(if (*dma).dmaBusy { 16 } else { 0 });
    }
}
