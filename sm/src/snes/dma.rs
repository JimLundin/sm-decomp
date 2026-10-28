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

    /// Requires the original live, exclusive retained backend for all bus effects.
    pub(crate) unsafe fn initialize_hdma(&mut self) {
        let mut host = unsafe { NativeHdmaHost::new(self) };
        match crate::game::scene::hdma::initialize(&mut host) {
            Ok(()) => {}
            Err(never) => match never {},
        }
    }

    /// Requires the original live, exclusive retained backend for all bus effects.
    pub(crate) unsafe fn step_hdma(&mut self) {
        let mut host = unsafe { NativeHdmaHost::new(self) };
        match crate::game::scene::hdma::step(&mut host) {
            Ok(()) => {}
            Err(never) => match never {},
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

/// A private capability over the original live, exclusive execution interval.
struct NativeHdmaHost<'a, 'execution>(&'a mut crate::execution::Execution<'execution>);
impl<'a, 'execution> NativeHdmaHost<'a, 'execution> {
    /// The backend must stay live and exclusive, including native bus callbacks.
    unsafe fn new(execution: &'a mut crate::execution::Execution<'execution>) -> Self {
        Self(execution)
    }
}
impl crate::game::scene::hdma::HdmaHost for NativeHdmaHost<'_, '_> {
    type Error = std::convert::Infallible;

    fn flag(&self, channel: usize, field: crate::game::scene::hdma::ChannelFlag) -> bool {
        use crate::game::scene::hdma::ChannelFlag as F;
        let state = self.0.backend.dma_state().channel[channel];
        match field {
            F::Active => state.hdmaActive,
            F::Terminated => state.terminated,
            F::DoTransfer => state.doTransfer,
            F::Indirect => state.indirect,
            F::FromDestination => state.fromB,
            F::DmaActive => state.dmaActive,
        }
    }
    fn byte(&self, channel: usize, field: crate::game::scene::hdma::ChannelByte) -> u8 {
        use crate::game::scene::hdma::ChannelByte as F;
        let state = self.0.backend.dma_state().channel[channel];
        match field {
            F::Mode => state.mode,
            F::RepeatCount => state.repCount,
            F::SourceBank => state.aBank,
            F::IndirectBank => state.indBank,
            F::Destination => state.bAdr,
            F::TransferCursor => state.offIndex,
        }
    }
    fn word(&self, channel: usize, field: crate::game::scene::hdma::ChannelWord) -> u16 {
        use crate::game::scene::hdma::ChannelWord as F;
        let state = self.0.backend.dma_state().channel[channel];
        match field {
            F::SourceAddress => state.aAdr,
            F::TableCursor => state.tableAdr,
            F::IndirectCursor => state.size,
        }
    }
    fn set_flag(
        &mut self,
        channel: usize,
        field: crate::game::scene::hdma::ChannelFlag,
        value: bool,
    ) {
        use crate::game::scene::hdma::ChannelFlag as F;
        let mut state = self.0.backend.dma_state();
        match field {
            F::Active => state.channel[channel].hdmaActive = value,
            F::Terminated => state.channel[channel].terminated = value,
            F::DoTransfer => state.channel[channel].doTransfer = value,
            F::Indirect => state.channel[channel].indirect = value,
            F::FromDestination => state.channel[channel].fromB = value,
            F::DmaActive => state.channel[channel].dmaActive = value,
        }
        self.0.backend.set_dma_state(state);
    }
    fn set_byte(
        &mut self,
        channel: usize,
        field: crate::game::scene::hdma::ChannelByte,
        value: u8,
    ) {
        use crate::game::scene::hdma::ChannelByte as F;
        let mut state = self.0.backend.dma_state();
        match field {
            F::Mode => state.channel[channel].mode = value,
            F::RepeatCount => state.channel[channel].repCount = value,
            F::SourceBank => state.channel[channel].aBank = value,
            F::IndirectBank => state.channel[channel].indBank = value,
            F::Destination => state.channel[channel].bAdr = value,
            F::TransferCursor => state.channel[channel].offIndex = value,
        }
        self.0.backend.set_dma_state(state);
    }
    fn set_word(
        &mut self,
        channel: usize,
        field: crate::game::scene::hdma::ChannelWord,
        value: u16,
    ) {
        use crate::game::scene::hdma::ChannelWord as F;
        let mut state = self.0.backend.dma_state();
        match field {
            F::SourceAddress => state.channel[channel].aAdr = value,
            F::TableCursor => state.channel[channel].tableAdr = value,
            F::IndirectCursor => state.channel[channel].size = value,
        }
        self.0.backend.set_dma_state(state);
    }
    fn timer(&self) -> u16 {
        self.0.backend.dma_state().hdmaTimer
    }
    fn set_timer(&mut self, value: u16) {
        let mut state = self.0.backend.dma_state();
        state.hdmaTimer = value;
        self.0.backend.set_dma_state(state);
    }
    fn invalid_mode(&mut self, mode: u8) -> std::convert::Infallible {
        // Preserve the original native table-index failure at its lookup point.
        let _ = transferLength[usize::from(mode)];
        unreachable!("HDMA invalid-mode hook requires an out-of-range mode");
    }
    fn read_source(
        &mut self,
        _channel: usize,
        _source: crate::game::scene::hdma::HdmaSource,
        bank: u8,
        address: u16,
    ) -> Result<u8, Self::Error> {
        // read is exactly read_with_dma_source(address, None); no pinned shortcut.
        Ok(unsafe { self.0.read((u32::from(bank) << 16) | u32::from(address)) })
    }
    fn write_source(
        &mut self,
        _channel: usize,
        _source: crate::game::scene::hdma::HdmaSource,
        bank: u8,
        address: u16,
        value: u8,
    ) -> Result<(), Self::Error> {
        // Captured bank/address survive every previous bus effect.
        unsafe {
            self.0
                .write((u32::from(bank) << 16) | u32::from(address), value)
        };
        Ok(())
    }
    fn read_destination(&mut self, destination: u8) -> Result<u8, Self::Error> {
        Ok(unsafe { self.0.read_bbus(destination) })
    }
    fn write_destination(&mut self, destination: u8, value: u8) -> Result<(), Self::Error> {
        unsafe { self.0.write_bbus(destination, value) };
        Ok(())
    }
}
