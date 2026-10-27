use crate::execution::DmaRomSource;
use crate::rtl::words;
use crate::snes::snes::Backend;
use ::core::ffi::c_char;
use ::core::mem::transmute;

use crate::c_library::printf;
pub use crate::types::{Dma, Snes, size_t};
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

#[forbid(unsafe_code)]
impl Backend {
    pub(crate) fn write_dma_register(&self, address: u16, value: u8) {
        let mut dma = self.dma_state();
        let channel = &mut dma.channel[usize::from((address >> 4) & 7)];
        match address & 0x0f {
            0 => {
                set_fields! {[channel]; mode = value & 7, fixed = value & 0x08 != 0,
                decrement = value & 0x10 != 0, unusedBit = value & 0x20 != 0,
                indirect = value & 0x40 != 0, fromB = value & 0x80 != 0, };
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
        self.set_dma_state(dma);
    }
}

#[forbid(unsafe_code)]
impl crate::execution::Execution<'_> {
    pub(crate) fn reset_dma(&mut self) {
        let mut dma = self.backend.dma_state();
        for i in 0..8 {
            set_fields! {[dma.channel[i]]; bAdr = 0xff, aAdr = 0xffff, aBank = 0xff,
            size = 0xffff, indBank = 0xff, tableAdr = 0xffff, };
            set_fields! {[dma.channel[i]], 0xff; repCount, unusedByte}
            dma.channel[i].dmaActive = false;
            set_fields! {[dma.channel[i]]; hdmaActive = false, mode = 7, fixed = true,
            decrement = true, indirect = true, fromB = true, unusedBit = true,
            doTransfer = false, terminated = false, offIndex = 0, };
        }
        set_fields! {[dma], 0; hdmaTimer, dmaTimer}
        dma.dmaBusy = false;
        self.backend.set_dma_state(dma);
    }

    pub(crate) fn start_dma(&mut self, mask: u8, hdma: bool) {
        let mut dma = self.backend.dma_state();
        for (i, channel) in dma.channel.iter_mut().enumerate() {
            let active = mask & (1 << i) != 0;
            if hdma {
                channel.hdmaActive = active;
            } else {
                channel.dmaActive = active;
            }
        }
        if !hdma {
            dma.dmaBusy = mask != 0;
            dma.dmaTimer = dma.dmaTimer.wrapping_add(if dma.dmaBusy { 16 } else { 0 });
        }
        self.backend.set_dma_state(dma);
    }
}

// Publish pure state changes before every recursive bus or native operation.
impl crate::execution::Execution<'_> {
    pub(crate) unsafe fn read_dma_register(&self, address: u16) -> u8 {
        unsafe {
            let channel = self.backend.dma_state().channel[usize::from((address >> 4) & 7)];
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
                        b"0\0", b"src/snes/dma.c\0", 118, pointer __ASSERT_FUNCTION.as_ptr()
                    };
                }
            }
        }
    }

    pub(crate) unsafe fn do_dma(&mut self) {
        unsafe { self.do_dma_with_source(None) }
    }

    unsafe fn do_dma_with_source(&mut self, source: Option<&DmaRomSource<'_>>) {
        unsafe {
            let i = {
                let mut dma = self.backend.dma_state();
                if dma.dmaTimer > 0 {
                    dma.dmaTimer = dma.dmaTimer.wrapping_sub(2);
                    self.backend.set_dma_state(dma);
                    return;
                }
                let mut i = 0;
                while i < 8 {
                    if dma.channel[i].dmaActive {
                        break;
                    }
                    i += 1;
                }
                if i == 8 {
                    dma.dmaBusy = false;
                    self.backend.set_dma_state(dma);
                    return;
                }
                i
            };
            if !self.backend.dma_state().channel[i].fromB
                && self.backend.dma_state().channel[i].aBank & 0x80 != 0
                && self.backend.dma_state().channel[i].aAdr & 0x8000 == 0
                && !*self.host.g_fail
            {
                printf(c"Warning! DMA from addr 0x%x\n".as_ptr(), {
                    let channel = self.backend.dma_state().channel[i];
                    words!(i32, channel, aBank, aAdr)
                });
                *self.host.g_fail = true_0 != 0;
            }
            let fresh0 = {
                let mut dma = self.backend.dma_state();
                let previous = dma.channel[i].offIndex;
                dma.channel[i].offIndex = dma.channel[i].offIndex.wrapping_add(1);
                self.backend.set_dma_state(dma);
                previous
            };
            let (address, bank, b_address, from_b) = {
                let channel = self.backend.dma_state().channel[i];
                (
                    channel.aAdr,
                    channel.aBank,
                    channel.bAdr.wrapping_add(
                        bAdrOffsets[usize::from(channel.mode)][usize::from(fresh0)] as u8,
                    ),
                    channel.fromB,
                )
            };
            self.transfer_dma_byte(address, bank, b_address, from_b, source);
            {
                let mut dma = self.backend.dma_state();
                dma.channel[i].offIndex &= 3;
                dma.dmaTimer = dma.dmaTimer.wrapping_add(6);
                if !dma.channel[i].fixed {
                    dma.channel[i].aAdr = if dma.channel[i].decrement {
                        dma.channel[i].aAdr.wrapping_sub(1)
                    } else {
                        dma.channel[i].aAdr.wrapping_add(1)
                    };
                }
                dma.channel[i].size = dma.channel[i].size.wrapping_sub(1);
                if dma.channel[i].size == 0 {
                    set_fields! {[dma.channel[i]]; offIndex = 0, dmaActive = false, };
                    dma.dmaTimer = dma.dmaTimer.wrapping_add(8);
                }
                self.backend.set_dma_state(dma);
            }
        }
    }

    // Advance the table pointer before the bus read, as HDMA does for every byte.
    unsafe fn hdma_table_byte(&mut self, channel: usize) -> u8 {
        unsafe {
            let address = {
                let mut dma = self.backend.dma_state();
                let address = dma.channel[channel].tableAdr;
                dma.channel[channel].tableAdr = address.wrapping_add(1);
                self.backend.set_dma_state(dma);
                address
            };
            let bank = self.backend.dma_state().channel[channel].aBank;
            self.read((u32::from(bank) << 16) | u32::from(address))
        }
    }

    unsafe fn hdma_indirect_address(&mut self, channel: usize) {
        unsafe {
            let low = u16::from(self.hdma_table_byte(channel));
            let captured_low = {
                let mut dma = self.backend.dma_state();
                dma.channel[channel].size = low;
                let captured_low = dma.channel[channel].size;
                self.backend.set_dma_state(dma);
                captured_low
            };
            let high = u16::from(self.hdma_table_byte(channel));
            {
                let mut dma = self.backend.dma_state();
                dma.channel[channel].size = captured_low | (high << 8);
                dma.hdmaTimer = dma.hdmaTimer.wrapping_add(16);
                self.backend.set_dma_state(dma);
            }
        }
    }

    pub(crate) unsafe fn initialize_hdma(&mut self) {
        unsafe {
            {
                let mut dma = self.backend.dma_state();
                dma.hdmaTimer = 0;
                self.backend.set_dma_state(dma);
            }
            let mut hdmaHappened: bool = false_0 != 0;
            for i in 0i32..8 {
                let keep_channel_active: bool = if self.backend.dma_state().channel[i as usize]
                    .hdmaActive
                {
                    hdmaHappened = true_0 != 0;
                    {
                        let mut dma = self.backend.dma_state();
                        set_fields! {[dma.channel[i as usize]]; dmaActive = false_0 != 0, offIndex = 0,
                        tableAdr = dma.channel[i as usize].aAdr, };
                        self.backend.set_dma_state(dma);
                    }
                    let rep_count = self.hdma_table_byte(i as usize);
                    {
                        let mut dma = self.backend.dma_state();
                        dma.channel[i as usize].repCount = rep_count;
                        self.backend.set_dma_state(dma);
                    }
                    if self.backend.dma_state().channel[i as usize].repCount as i32 == 0 {
                        let mut dma = self.backend.dma_state();
                        dma.channel[i as usize].terminated = true_0 != 0;
                        self.backend.set_dma_state(dma);
                        false
                    } else {
                        {
                            let mut dma = self.backend.dma_state();
                            dma.hdmaTimer = (dma.hdmaTimer as i32 + 8) as u16;
                            self.backend.set_dma_state(dma);
                        }
                        if self.backend.dma_state().channel[i as usize].indirect {
                            self.hdma_indirect_address(i as usize);
                        }
                        {
                            let mut dma = self.backend.dma_state();
                            dma.channel[i as usize].doTransfer = true_0 != 0;
                            self.backend.set_dma_state(dma);
                        }
                        true
                    }
                } else {
                    let mut dma = self.backend.dma_state();
                    dma.channel[i as usize].doTransfer = false_0 != 0;
                    self.backend.set_dma_state(dma);
                    true
                };
                if keep_channel_active {
                    let mut dma = self.backend.dma_state();
                    dma.channel[i as usize].terminated = false_0 != 0;
                    self.backend.set_dma_state(dma);
                }
            }
            if hdmaHappened {
                let mut dma = self.backend.dma_state();
                dma.hdmaTimer = (dma.hdmaTimer as i32 + 16) as u16;
                self.backend.set_dma_state(dma);
            }
        }
    }

    pub(crate) unsafe fn step_hdma(&mut self) {
        unsafe {
            {
                let mut dma = self.backend.dma_state();
                dma.hdmaTimer = 0;
                self.backend.set_dma_state(dma);
            }
            let mut hdmaHappened: bool = false_0 != 0;
            let mut i: i32 = 0;
            while i < 8 {
                if self.backend.dma_state().channel[i as usize].hdmaActive as i32 != 0
                    && !self.backend.dma_state().channel[i as usize].terminated
                {
                    hdmaHappened = true_0 != 0;
                    {
                        let mut dma = self.backend.dma_state();
                        set_fields! {[dma.channel[i as usize]]; dmaActive = false_0 != 0, offIndex = 0, };
                        dma.hdmaTimer = (dma.hdmaTimer as i32 + 8) as u16;
                        self.backend.set_dma_state(dma);
                    }
                    if self.backend.dma_state().channel[i as usize].doTransfer {
                        let mut j: i32 = 0;
                        while j < transferLength
                            [self.backend.dma_state().channel[i as usize].mode as usize]
                        {
                            {
                                let mut dma = self.backend.dma_state();
                                dma.hdmaTimer = (dma.hdmaTimer as i32 + 8) as u16;
                                self.backend.set_dma_state(dma);
                            }
                            if self.backend.dma_state().channel[i as usize].indirect {
                                let fresh4 = {
                                    let mut dma = self.backend.dma_state();
                                    let previous = dma.channel[i as usize].size;
                                    dma.channel[i as usize].size =
                                        dma.channel[i as usize].size.wrapping_add(1);
                                    self.backend.set_dma_state(dma);
                                    previous
                                };
                                let (bank, b_address, from_b) = {
                                    let channel = self.backend.dma_state().channel[i as usize];
                                    (
                                        channel.indBank,
                                        (channel.bAdr as i32
                                            + bAdrOffsets[channel.mode as usize][j as usize])
                                            as u8,
                                        channel.fromB,
                                    )
                                };
                                self.transfer_dma_byte(fresh4, bank, b_address, from_b, None);
                            } else {
                                let fresh5 = {
                                    let mut dma = self.backend.dma_state();
                                    let previous = dma.channel[i as usize].tableAdr;
                                    dma.channel[i as usize].tableAdr =
                                        dma.channel[i as usize].tableAdr.wrapping_add(1);
                                    self.backend.set_dma_state(dma);
                                    previous
                                };
                                let (bank, b_address, from_b) = {
                                    let channel = self.backend.dma_state().channel[i as usize];
                                    (
                                        channel.aBank,
                                        (channel.bAdr as i32
                                            + bAdrOffsets[channel.mode as usize][j as usize])
                                            as u8,
                                        channel.fromB,
                                    )
                                };
                                self.transfer_dma_byte(fresh5, bank, b_address, from_b, None);
                            }
                            j += 1;
                        }
                    }
                    {
                        let mut dma = self.backend.dma_state();
                        dma.channel[i as usize].repCount =
                            dma.channel[i as usize].repCount.wrapping_sub(1);
                        dma.channel[i as usize].doTransfer =
                            dma.channel[i as usize].repCount & 0x80 != 0;
                        self.backend.set_dma_state(dma);
                    }
                    if self.backend.dma_state().channel[i as usize].repCount & 0x7f == 0 {
                        let rep_count = self.hdma_table_byte(i as usize);
                        {
                            let mut dma = self.backend.dma_state();
                            dma.channel[i as usize].repCount = rep_count;
                            self.backend.set_dma_state(dma);
                        }
                        if self.backend.dma_state().channel[i as usize].indirect {
                            self.hdma_indirect_address(i as usize);
                        }
                        {
                            let mut dma = self.backend.dma_state();
                            if dma.channel[i as usize].repCount as i32 == 0 {
                                dma.channel[i as usize].terminated = true_0 != 0;
                            }
                            dma.channel[i as usize].doTransfer = true_0 != 0;
                            self.backend.set_dma_state(dma);
                        }
                    }
                }
                i += 1;
            }
            if hdmaHappened {
                let mut dma = self.backend.dma_state();
                dma.hdmaTimer = (dma.hdmaTimer as i32 + 16) as u16;
                self.backend.set_dma_state(dma);
            }
        }
    }

    unsafe fn transfer_dma_byte(
        &mut self,
        a_address: u16,
        a_bank: u8,
        b_address: u8,
        from_b: bool,
        source: Option<&DmaRomSource<'_>>,
    ) {
        unsafe {
            let address = (u32::from(a_bank) << 16) | u32::from(a_address);
            if from_b {
                let value = self.read_bbus(b_address);
                self.write(address, value);
            } else {
                let value = self.read_with_dma_source(address, source);
                self.write_bbus(b_address, value);
            }
        }
    }

    pub(crate) unsafe fn step_dma(&mut self) -> bool {
        unsafe { self.step_dma_with_source(None) }
    }

    pub(crate) unsafe fn step_dma_with_source(
        &mut self,
        source: Option<&DmaRomSource<'_>>,
    ) -> bool {
        unsafe {
            let busy = {
                let mut dma = self.backend.dma_state();
                if dma.hdmaTimer > 0 {
                    dma.hdmaTimer = dma.hdmaTimer.wrapping_sub(2);
                    self.backend.set_dma_state(dma);
                    return true;
                }
                dma.dmaBusy
            };
            if busy {
                match source {
                    Some(source) => self.do_dma_with_source(Some(source)),
                    None => self.do_dma(),
                }
                true
            } else {
                false
            }
        }
    }
}
