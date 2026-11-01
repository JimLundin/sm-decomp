use crate::execution::DmaRomSource;
use crate::rtl::words;
use crate::snes::snes::Backend;
use ::core::ffi::c_char;
use ::core::mem::transmute;

use crate::c_library::printf;
pub use crate::types::{Dma, Snes};
pub const __ASSERT_FUNCTION: [c_char; 34] =
    unsafe { transmute::<[u8; 34], [c_char; 34]>(*b"uint8_t dma_read(Dma *, uint16_t)\0") };
static B_BUS_OFFSETS: [[i32; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];
static TRANSFER_LENGTHS: [i32; 8] = [1, 2, 2, 4, 4, 4, 2, 4];

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
            // A source in the low half of a high bank is not ROM. The WRAM mirror below $2000
            // reads like any RAM (the original's scrolling sky reads it near the top of a room);
            // the rest is registers or open bus.
            if !self.backend.dma_state().channel[i].fromB
                && self.backend.dma_state().channel[i].aBank & 0x80 != 0
                && (0x2000..0x8000).contains(&self.backend.dma_state().channel[i].aAdr)
                && !*self.host.g_fail
                && !crate::oracle_mode::models_open_bus()
            {
                printf(c"Warning! DMA from addr 0x%x\n".as_ptr(), {
                    let channel = self.backend.dma_state().channel[i];
                    words!(i32, channel, aBank, aAdr)
                });
                *self.host.g_fail = true;
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
                        B_BUS_OFFSETS[usize::from(channel.mode)][usize::from(fresh0)] as u8,
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
        #[cfg(sm_verify_dll)]
        unsafe {
            table_check::begin_frame(self)
        };
        let mut host = unsafe { NativeHdmaHost::new(self) };
        match crate::game::scene::hdma::initialize(&mut host) {
            Ok(()) => {}
            Err(never) => match never {},
        }
    }

    /// Requires the original live, exclusive retained backend for all bus effects.
    pub(crate) unsafe fn step_hdma(&mut self) {
        #[cfg(sm_verify_dll)]
        table_check::begin_pass();
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
            if crate::oracle_mode::models_open_bus() {
                // As bsnes: the A bus cannot reach the B bus or the CPU I/O
                // registers (reads give 0, writes are dropped), and each byte
                // passes through the CPU data bus.
                let valid = a_bank & 0x40 != 0
                    || !matches!(a_address, 0x2100..=0x21ff | 0x4000..=0x421f | 0x4300..=0x437f);
                let value = if from_b {
                    self.read_bbus(b_address)
                } else if valid {
                    self.read_with_dma_source(address, source)
                } else {
                    0
                };
                (*self.snes()).openBus = value;
                if !from_b {
                    self.write_bbus(b_address, value);
                } else if valid {
                    self.write(address, value);
                }
                return;
            }
            if from_b {
                let value = self.read_bbus(b_address);
                self.write(address, value);
            } else {
                let value = self.read_with_dma_source(address, source);
                self.write_bbus(b_address, value);
            }
        }
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
        let _ = TRANSFER_LENGTHS[usize::from(mode)];
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
        #[cfg(sm_verify_dll)]
        table_check::record(destination, value);
        unsafe { self.0.write_bbus(destination, value) };
        Ok(())
    }
}

/// HDMA as per-line register tables (native renderer step 4). With
/// `SM_HDMA_TABLE` set, each frame's HDMA is planned at its initialization
/// from a scratch copy of the channel state with `display::hdma::frame_table`,
/// and the B-bus writes the beam-driven HDMA then makes are compared pass by
/// pass. `SM_HDMA_TABLE_LOG` receives each frame whose writes differ from its
/// table (frame, first differing pass, planned and actual write counts there).
/// The plan applies the frame's planned scanline interrupt control stores
/// (`scanline_irq::table_check`) from the pass after their line, since a
/// table can read the work RAM they write. Any other difference means the
/// frame's tables or channels changed after they were planned, so a beam-free
/// renderer would need that change as an input.
#[cfg(sm_verify_dll)]
mod table_check {
    use super::{Dma, NativeHdmaHost};
    use crate::game::scene::hdma::{self, ChannelByte, ChannelFlag, ChannelWord, HdmaHost};
    use std::cell::RefCell;
    use std::io::Write as _;
    use std::sync::OnceLock;

    /// Lines with an HDMA pass: the beam steps HDMA on rows 0 to 224.
    const STEPS: usize = 225;

    #[derive(Default)]
    struct Frame {
        number: u32,
        planned: Option<Vec<hdma::PassWrites>>,
        actual: Vec<hdma::PassWrites>,
    }

    thread_local! {
        static FRAME: RefCell<Frame> = RefCell::new(Frame::default());
    }

    fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("SM_HDMA_TABLE").is_some())
    }

    /// A scratch copy of the channels and timer that reads tables from the live
    /// bus, as overlaid by the interrupt stores made so far in the frame.
    struct Scratch<'a, 'execution> {
        dma: Dma,
        bus: NativeHdmaHost<'a, 'execution>,
        /// Work RAM bytes the frame's interrupts have stored, by offset.
        stored: Vec<(usize, u8)>,
    }

    impl Scratch<'_, '_> {
        fn store(&mut self, offset: usize, value: u8) {
            match self.stored.iter_mut().find(|(at, _)| *at == offset) {
                Some(entry) => entry.1 = value,
                None => self.stored.push((offset, value)),
            }
        }
    }

    /// The work RAM offset a bus address reads, if it is work RAM.
    fn work_ram(bank: u8, address: u16) -> Option<usize> {
        match bank {
            0x7e | 0x7f => Some((usize::from(bank & 1) << 16) | usize::from(address)),
            0x00..=0x3f | 0x80..=0xbf if address < 0x2000 => Some(usize::from(address)),
            _ => None,
        }
    }

    impl HdmaHost for Scratch<'_, '_> {
        type Error = std::convert::Infallible;

        fn flag(&self, channel: usize, field: ChannelFlag) -> bool {
            let state = &self.dma.channel[channel];
            match field {
                ChannelFlag::Active => state.hdmaActive,
                ChannelFlag::Terminated => state.terminated,
                ChannelFlag::DoTransfer => state.doTransfer,
                ChannelFlag::Indirect => state.indirect,
                ChannelFlag::FromDestination => state.fromB,
                ChannelFlag::DmaActive => state.dmaActive,
            }
        }
        fn byte(&self, channel: usize, field: ChannelByte) -> u8 {
            let state = &self.dma.channel[channel];
            match field {
                ChannelByte::Mode => state.mode,
                ChannelByte::RepeatCount => state.repCount,
                ChannelByte::SourceBank => state.aBank,
                ChannelByte::IndirectBank => state.indBank,
                ChannelByte::Destination => state.bAdr,
                ChannelByte::TransferCursor => state.offIndex,
            }
        }
        fn word(&self, channel: usize, field: ChannelWord) -> u16 {
            let state = &self.dma.channel[channel];
            match field {
                ChannelWord::SourceAddress => state.aAdr,
                ChannelWord::TableCursor => state.tableAdr,
                ChannelWord::IndirectCursor => state.size,
            }
        }
        fn set_flag(&mut self, channel: usize, field: ChannelFlag, value: bool) {
            let state = &mut self.dma.channel[channel];
            match field {
                ChannelFlag::Active => state.hdmaActive = value,
                ChannelFlag::Terminated => state.terminated = value,
                ChannelFlag::DoTransfer => state.doTransfer = value,
                ChannelFlag::Indirect => state.indirect = value,
                ChannelFlag::FromDestination => state.fromB = value,
                ChannelFlag::DmaActive => state.dmaActive = value,
            }
        }
        fn set_byte(&mut self, channel: usize, field: ChannelByte, value: u8) {
            let state = &mut self.dma.channel[channel];
            match field {
                ChannelByte::Mode => state.mode = value,
                ChannelByte::RepeatCount => state.repCount = value,
                ChannelByte::SourceBank => state.aBank = value,
                ChannelByte::IndirectBank => state.indBank = value,
                ChannelByte::Destination => state.bAdr = value,
                ChannelByte::TransferCursor => state.offIndex = value,
            }
        }
        fn set_word(&mut self, channel: usize, field: ChannelWord, value: u16) {
            let state = &mut self.dma.channel[channel];
            match field {
                ChannelWord::SourceAddress => state.aAdr = value,
                ChannelWord::TableCursor => state.tableAdr = value,
                ChannelWord::IndirectCursor => state.size = value,
            }
        }
        fn timer(&self) -> u16 {
            self.dma.hdmaTimer
        }
        fn set_timer(&mut self, value: u16) {
            self.dma.hdmaTimer = value;
        }
        fn invalid_mode(&mut self, mode: u8) -> Self::Error {
            self.bus.invalid_mode(mode)
        }
        fn read_source(
            &mut self,
            channel: usize,
            source: hdma::HdmaSource,
            bank: u8,
            address: u16,
        ) -> Result<u8, Self::Error> {
            let value = self.bus.read_source(channel, source, bank, address)?;
            let stored = work_ram(bank, address)
                .and_then(|offset| self.stored.iter().find(|(at, _)| *at == offset));
            Ok(stored.map_or(value, |&(_, value)| value))
        }
        fn write_source(
            &mut self,
            _channel: usize,
            _source: hdma::HdmaSource,
            _bank: u8,
            _address: u16,
            _value: u8,
        ) -> Result<(), Self::Error> {
            unreachable!("the frame table records from-destination transfers itself")
        }
        fn read_destination(&mut self, _destination: u8) -> Result<u8, Self::Error> {
            unreachable!("the frame table records from-destination transfers itself")
        }
        fn write_destination(&mut self, _destination: u8, _value: u8) -> Result<(), Self::Error> {
            unreachable!("the frame table records its writes itself")
        }
    }

    /// Compare the finished frame with its plan, then plan the next one.
    pub(super) unsafe fn begin_frame(execution: &mut crate::execution::Execution<'_>) {
        if !enabled() {
            return;
        }
        FRAME.with_borrow_mut(|frame| {
            if let Some(planned) = &frame.planned {
                let passes = planned.len().max(frame.actual.len());
                let empty = Vec::new();
                let differing = (0..passes).find(|&pass| {
                    planned.get(pass).unwrap_or(&empty) != frame.actual.get(pass).unwrap_or(&empty)
                });
                if let (Some(pass), Some(path)) = (differing, std::env::var_os("SM_HDMA_TABLE_LOG"))
                {
                    let count =
                        |passes: &Vec<hdma::PassWrites>| passes.get(pass).map_or(0, Vec::len);
                    let mut log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .unwrap();
                    writeln!(
                        log,
                        "{},{pass},{},{}",
                        frame.number,
                        count(planned),
                        count(&frame.actual)
                    )
                    .unwrap();
                }
            }
            let dma = execution.backend.dma_state();
            let mut scratch = Scratch {
                dma,
                bus: unsafe { NativeHdmaHost::new(execution) },
                stored: Vec::new(),
            };
            let mut interrupts = crate::game::scanline_irq::table_check::planned_stores()
                .into_iter()
                .peekable();
            // An interrupt at line N runs after line pass N + 1.
            let between = |scratch: &mut Scratch<'_, '_>, pass: usize| {
                while let Some((_, stores)) =
                    interrupts.next_if(|(line, _)| usize::from(*line) + 2 <= pass)
                {
                    for (field, value) in stores {
                        let offset = match field {
                            sm_game::scene::irq::Store::Current => 0xab,
                            sm_game::scene::irq::Store::Pending => 0xa7,
                            sm_game::scene::irq::Store::Enable => 0x84,
                        };
                        let [low, high] = value.to_le_bytes();
                        scratch.store(offset, low);
                        scratch.store(offset + 1, high);
                    }
                }
            };
            let planned = match hdma::frame_table(&mut scratch, STEPS, between) {
                Ok(planned) => planned,
                Err(never) => match never {},
            };
            frame.number += 1;
            frame.planned = planned;
            frame.actual = vec![Vec::new()];
        });
    }

    pub(super) fn begin_pass() {
        if enabled() {
            FRAME.with_borrow_mut(|frame| frame.actual.push(Vec::new()));
        }
    }

    pub(super) fn record(destination: u8, value: u8) {
        if enabled() {
            FRAME.with_borrow_mut(|frame| {
                if let Some(pass) = frame.actual.last_mut() {
                    pass.push((destination, value));
                }
            });
        }
    }
}
