//! Ordered HDMA execution over the actual eight-channel and timer authority.

const DESTINATION_OFFSETS: [[u8; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];
const TRANSFER_LENGTHS: [usize; 8] = [1, 2, 2, 4, 4, 4, 2, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdmaSource {
    Table,
    Indirect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelFlag {
    Active,
    Terminated,
    DoTransfer,
    Indirect,
    FromDestination,
    DmaActive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelByte {
    Mode,
    RepeatCount,
    SourceBank,
    IndirectBank,
    Destination,
    TransferCursor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelWord {
    SourceAddress,
    TableCursor,
    IndirectCursor,
}

/// Field queries read the current authority without side effects. Publications
/// immediately update exactly the selected field. Implementations must not keep
/// a mutable mirror or hold a native DMA-state borrow across a bus operation.
/// All channel indices supplied here are in 0..8; HDMA timer arithmetic is u16.
///
/// Byte operations use the same authority and may update aliased fields. Resolve
/// the fresh source binding for each operation, with the supplied captured full
/// bank/address and provenance. Missing bindings must fail, never supply zeros.
/// Earlier publications and bus effects survive errors or unwinding. Native
/// compatibility implementations retain their actual unsafe entry requirements.
pub trait HdmaHost {
    type Error;

    fn flag(&self, channel: usize, field: ChannelFlag) -> bool;
    fn byte(&self, channel: usize, field: ChannelByte) -> u8;
    fn word(&self, channel: usize, field: ChannelWord) -> u16;
    fn set_flag(&mut self, channel: usize, field: ChannelFlag, value: bool);
    fn set_byte(&mut self, channel: usize, field: ChannelByte, value: u8);
    fn set_word(&mut self, channel: usize, field: ChannelWord, value: u16);
    fn timer(&self) -> u16;
    fn set_timer(&mut self, value: u16);
    /// Called only at the original fresh mode lookup if mode is outside0..8.
    /// The native adapter may preserve its indexing panic; owned uses its error.
    fn invalid_mode(&mut self, mode: u8) -> Self::Error;

    fn read_source(
        &mut self,
        channel: usize,
        source: HdmaSource,
        bank: u8,
        address: u16,
    ) -> Result<u8, Self::Error>;
    fn write_source(
        &mut self,
        channel: usize,
        source: HdmaSource,
        bank: u8,
        address: u16,
        value: u8,
    ) -> Result<(), Self::Error>;
    fn read_destination(&mut self, destination: u8) -> Result<u8, Self::Error>;
    fn write_destination(&mut self, destination: u8, value: u8) -> Result<(), Self::Error>;
}

/// Run once at the original row-start initialization point. Inactive channels
/// remain present; no gameplay object records are needed by this execution core.
pub fn initialize<H: HdmaHost>(host: &mut H) -> Result<(), H::Error> {
    host.set_timer(0);
    let mut happened = false;
    for channel in 0..8 {
        let keep_active = if host.flag(channel, ChannelFlag::Active) {
            happened = true;
            host.set_flag(channel, ChannelFlag::DmaActive, false);
            host.set_byte(channel, ChannelByte::TransferCursor, 0);
            let address = host.word(channel, ChannelWord::SourceAddress);
            host.set_word(channel, ChannelWord::TableCursor, address);
            let repeat = table_byte(host, channel)?;
            host.set_byte(channel, ChannelByte::RepeatCount, repeat);
            if host.byte(channel, ChannelByte::RepeatCount) == 0 {
                host.set_flag(channel, ChannelFlag::Terminated, true);
                false
            } else {
                add_timer(host, 8);
                if host.flag(channel, ChannelFlag::Indirect) {
                    acquire_indirect(host, channel)?;
                }
                host.set_flag(channel, ChannelFlag::DoTransfer, true);
                true
            }
        } else {
            host.set_flag(channel, ChannelFlag::DoTransfer, false);
            true
        };
        if keep_active {
            host.set_flag(channel, ChannelFlag::Terminated, false);
        }
    }
    if happened {
        add_timer(host, 16);
    }
    Ok(())
}

/// Run at the original HDMA beam point. Failure retains every prior cursor,
/// timer, flag and committed byte effect; there is no whole-pass rollback.
pub fn step<H: HdmaHost>(host: &mut H) -> Result<(), H::Error> {
    host.set_timer(0);
    let mut happened = false;
    for channel in 0..8 {
        if host.flag(channel, ChannelFlag::Active) && !host.flag(channel, ChannelFlag::Terminated) {
            happened = true;
            host.set_flag(channel, ChannelFlag::DmaActive, false);
            host.set_byte(channel, ChannelByte::TransferCursor, 0);
            add_timer(host, 8);
            if host.flag(channel, ChannelFlag::DoTransfer) {
                let mut byte = 0;
                loop {
                    // Query mode even for the final failed loop condition.
                    let mode = host.byte(channel, ChannelByte::Mode);
                    let length = TRANSFER_LENGTHS
                        .get(usize::from(mode))
                        .copied()
                        .ok_or_else(|| host.invalid_mode(mode))?;
                    if byte >= length {
                        break;
                    }
                    add_timer(host, 8);
                    let (address, source, bank_field) = if host.flag(channel, ChannelFlag::Indirect)
                    {
                        let address = host.word(channel, ChannelWord::IndirectCursor);
                        host.set_word(
                            channel,
                            ChannelWord::IndirectCursor,
                            address.wrapping_add(1),
                        );
                        (address, HdmaSource::Indirect, ChannelByte::IndirectBank)
                    } else {
                        let address = host.word(channel, ChannelWord::TableCursor);
                        host.set_word(channel, ChannelWord::TableCursor, address.wrapping_add(1));
                        (address, HdmaSource::Table, ChannelByte::SourceBank)
                    };
                    // Capture only after cursor publication, and retain across
                    // both bus operations even when the read aliases fields.
                    let bank = host.byte(channel, bank_field);
                    let destination = host.byte(channel, ChannelByte::Destination);
                    let mode = host.byte(channel, ChannelByte::Mode);
                    let offsets = DESTINATION_OFFSETS
                        .get(usize::from(mode))
                        .ok_or_else(|| host.invalid_mode(mode))?;
                    let destination = destination.wrapping_add(offsets[byte]);
                    let from_destination = host.flag(channel, ChannelFlag::FromDestination);
                    transfer_byte(
                        host,
                        channel,
                        source,
                        bank,
                        address,
                        destination,
                        from_destination,
                    )?;
                    byte += 1;
                }
            }
            let repeat = host.byte(channel, ChannelByte::RepeatCount).wrapping_sub(1);
            host.set_byte(channel, ChannelByte::RepeatCount, repeat);
            let repeat = host.byte(channel, ChannelByte::RepeatCount);
            host.set_flag(channel, ChannelFlag::DoTransfer, repeat & 0x80 != 0);
            if host.byte(channel, ChannelByte::RepeatCount) & 0x7f == 0 {
                let repeat = table_byte(host, channel)?;
                host.set_byte(channel, ChannelByte::RepeatCount, repeat);
                if host.flag(channel, ChannelFlag::Indirect) {
                    acquire_indirect(host, channel)?;
                }
                if host.byte(channel, ChannelByte::RepeatCount) == 0 {
                    host.set_flag(channel, ChannelFlag::Terminated, true);
                }
                host.set_flag(channel, ChannelFlag::DoTransfer, true);
            }
        }
    }
    if happened {
        add_timer(host, 16);
    }
    Ok(())
}

fn add_timer(host: &mut impl HdmaHost, amount: u16) {
    let timer = host.timer().wrapping_add(amount);
    host.set_timer(timer);
}

fn table_byte<H: HdmaHost>(host: &mut H, channel: usize) -> Result<u8, H::Error> {
    let address = host.word(channel, ChannelWord::TableCursor);
    host.set_word(channel, ChannelWord::TableCursor, address.wrapping_add(1));
    let bank = host.byte(channel, ChannelByte::SourceBank);
    host.read_source(channel, HdmaSource::Table, bank, address)
}

fn acquire_indirect<H: HdmaHost>(host: &mut H, channel: usize) -> Result<(), H::Error> {
    let low = u16::from(table_byte(host, channel)?);
    host.set_word(channel, ChannelWord::IndirectCursor, low);
    let captured_low = host.word(channel, ChannelWord::IndirectCursor);
    let high = u16::from(table_byte(host, channel)?);
    host.set_word(
        channel,
        ChannelWord::IndirectCursor,
        captured_low | (high << 8),
    );
    add_timer(host, 16);
    Ok(())
}

fn transfer_byte<H: HdmaHost>(
    host: &mut H,
    channel: usize,
    source: HdmaSource,
    bank: u8,
    address: u16,
    destination: u8,
    from_destination: bool,
) -> Result<(), H::Error> {
    if from_destination {
        let value = host.read_destination(destination)?;
        host.write_source(channel, source, bank, address, value)
    } else {
        let value = host.read_source(channel, source, bank, address)?;
        host.write_destination(destination, value)
    }
}

/// The B-bus writes of one HDMA pass, in order: (destination, value).
pub type PassWrites = Vec<(u8, u8)>;

/// A frame's HDMA as per-pass register tables: the initialization pass, then
/// `steps` line passes. `host` must be a scratch copy of the channel and timer
/// state; its source reads must see the tables as they stand. Returns `None`
/// if a channel transfers from the B bus to memory, which needs the PPU's
/// values at each line and so has no table.
pub fn frame_table<H: HdmaHost>(
    host: &mut H,
    steps: usize,
) -> Result<Option<Vec<PassWrites>>, H::Error> {
    let mut recorder = Recorder {
        host,
        writes: Vec::new(),
        from_destination: false,
    };
    initialize(&mut recorder)?;
    let mut passes = vec![std::mem::take(&mut recorder.writes)];
    for _ in 0..steps {
        step(&mut recorder)?;
        passes.push(std::mem::take(&mut recorder.writes));
    }
    Ok((!recorder.from_destination).then_some(passes))
}

/// Records B-bus writes instead of performing them, over a scratch host.
struct Recorder<'a, H> {
    host: &'a mut H,
    writes: PassWrites,
    from_destination: bool,
}

impl<H: HdmaHost> HdmaHost for Recorder<'_, H> {
    type Error = H::Error;

    fn flag(&self, channel: usize, field: ChannelFlag) -> bool {
        self.host.flag(channel, field)
    }
    fn byte(&self, channel: usize, field: ChannelByte) -> u8 {
        self.host.byte(channel, field)
    }
    fn word(&self, channel: usize, field: ChannelWord) -> u16 {
        self.host.word(channel, field)
    }
    fn set_flag(&mut self, channel: usize, field: ChannelFlag, value: bool) {
        self.host.set_flag(channel, field, value);
    }
    fn set_byte(&mut self, channel: usize, field: ChannelByte, value: u8) {
        self.host.set_byte(channel, field, value);
    }
    fn set_word(&mut self, channel: usize, field: ChannelWord, value: u16) {
        self.host.set_word(channel, field, value);
    }
    fn timer(&self) -> u16 {
        self.host.timer()
    }
    fn set_timer(&mut self, value: u16) {
        self.host.set_timer(value);
    }
    fn invalid_mode(&mut self, mode: u8) -> Self::Error {
        self.host.invalid_mode(mode)
    }
    fn read_source(
        &mut self,
        channel: usize,
        source: HdmaSource,
        bank: u8,
        address: u16,
    ) -> Result<u8, Self::Error> {
        self.host.read_source(channel, source, bank, address)
    }
    fn write_source(
        &mut self,
        _channel: usize,
        _source: HdmaSource,
        _bank: u8,
        _address: u16,
        _value: u8,
    ) -> Result<(), Self::Error> {
        self.from_destination = true;
        Ok(())
    }
    fn read_destination(&mut self, _destination: u8) -> Result<u8, Self::Error> {
        self.from_destination = true;
        Ok(0)
    }
    fn write_destination(&mut self, destination: u8, value: u8) -> Result<(), Self::Error> {
        self.writes.push((destination, value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One channel over a flat 64 KiB table bank.
    struct Scratch {
        active: bool,
        terminated: bool,
        do_transfer: bool,
        mode: u8,
        repeat: u8,
        destination: u8,
        source: u16,
        cursor: u16,
        timer: u16,
        memory: Vec<u8>,
    }

    impl HdmaHost for Scratch {
        type Error = ();
        fn flag(&self, channel: usize, field: ChannelFlag) -> bool {
            channel == 0
                && match field {
                    ChannelFlag::Active => self.active,
                    ChannelFlag::Terminated => self.terminated,
                    ChannelFlag::DoTransfer => self.do_transfer,
                    _ => false,
                }
        }
        fn byte(&self, _: usize, field: ChannelByte) -> u8 {
            match field {
                ChannelByte::Mode => self.mode,
                ChannelByte::RepeatCount => self.repeat,
                ChannelByte::Destination => self.destination,
                _ => 0,
            }
        }
        fn word(&self, _: usize, field: ChannelWord) -> u16 {
            match field {
                ChannelWord::SourceAddress => self.source,
                ChannelWord::TableCursor => self.cursor,
                ChannelWord::IndirectCursor => 0,
            }
        }
        fn set_flag(&mut self, channel: usize, field: ChannelFlag, value: bool) {
            if channel == 0 {
                match field {
                    ChannelFlag::Terminated => self.terminated = value,
                    ChannelFlag::DoTransfer => self.do_transfer = value,
                    _ => {}
                }
            }
        }
        fn set_byte(&mut self, channel: usize, field: ChannelByte, value: u8) {
            if channel == 0 && field == ChannelByte::RepeatCount {
                self.repeat = value;
            }
        }
        fn set_word(&mut self, channel: usize, field: ChannelWord, value: u16) {
            if channel == 0 && field == ChannelWord::TableCursor {
                self.cursor = value;
            }
        }
        fn timer(&self) -> u16 {
            self.timer
        }
        fn set_timer(&mut self, value: u16) {
            self.timer = value;
        }
        fn invalid_mode(&mut self, _: u8) {}
        fn read_source(&mut self, _: usize, _: HdmaSource, _: u8, address: u16) -> Result<u8, ()> {
            Ok(self.memory[usize::from(address)])
        }
        fn write_source(
            &mut self,
            _: usize,
            _: HdmaSource,
            _: u8,
            _: u16,
            _: u8,
        ) -> Result<(), ()> {
            Err(())
        }
        fn read_destination(&mut self, _: u8) -> Result<u8, ()> {
            Err(())
        }
        fn write_destination(&mut self, _: u8, _: u8) -> Result<(), ()> {
            Err(())
        }
    }

    #[test]
    fn a_direct_table_becomes_one_write_per_line() {
        let mut memory = vec![0; 0x10000];
        // 2 lines that write 3 once, 1 line of 9, a 2-line repeat entry writing 5 then 6, the end.
        memory[0x100..0x108].copy_from_slice(&[2, 3, 1, 9, 0x82, 5, 6, 0]);
        let mut host = Scratch {
            active: true,
            terminated: false,
            do_transfer: false,
            mode: 0,
            repeat: 0,
            destination: 0x00,
            source: 0x100,
            cursor: 0,
            timer: 0,
            memory,
        };
        let table = frame_table(&mut host, 6).unwrap().unwrap();
        let lines: Vec<&[(u8, u8)]> = table.iter().map(Vec::as_slice).collect();
        assert_eq!(
            lines,
            [
                &[][..],
                &[(0, 3)],
                &[],
                &[(0, 9)],
                &[(0, 5)],
                &[(0, 6)],
                &[]
            ]
        );
    }
}
