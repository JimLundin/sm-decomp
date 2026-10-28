//! Ordered HDMA execution over the actual eight-channel and timer authority.
#![forbid(unsafe_code)]

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
pub(crate) enum HdmaSource {
    Table,
    Indirect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChannelFlag {
    Active,
    Terminated,
    DoTransfer,
    Indirect,
    FromDestination,
    DmaActive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChannelByte {
    Mode,
    RepeatCount,
    SourceBank,
    IndirectBank,
    Destination,
    TransferCursor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChannelWord {
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
pub(crate) trait HdmaHost {
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
pub(crate) fn initialize<H: HdmaHost>(host: &mut H) -> Result<(), H::Error> {
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
pub(crate) fn step<H: HdmaHost>(host: &mut H) -> Result<(), H::Error> {
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
