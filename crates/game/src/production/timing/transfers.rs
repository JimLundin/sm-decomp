use super::hardware::{Hardware, LineTransfer, Payload};
use super::{Event, EventKind, Fault};
use data::dma::BusAddress;
use data::timing::{ChannelSlot, ContentRegion, Cycle, TransferPort};

const SLOTS: [ChannelSlot; 8] = [
    ChannelSlot::One,
    ChannelSlot::Two,
    ChannelSlot::Three,
    ChannelSlot::Four,
    ChannelSlot::Five,
    ChannelSlot::Six,
    ChannelSlot::Seven,
    ChannelSlot::Eight,
];

pub(super) fn read(regions: &[ContentRegion], address: BusAddress) -> Result<u8, Fault> {
    let mut found = None;
    for region in regions {
        if region.start.bank != address.bank {
            continue;
        }
        let Some(offset) = address.offset.checked_sub(region.start.offset) else {
            continue;
        };
        if let Some(value) = region.bytes.get(usize::from(offset))
            && found.replace(*value).is_some()
        {
            return Err(Fault::ContentOverlap);
        }
    }
    found.ok_or(Fault::MissingContent)
}

impl Hardware {
    fn flush(&mut self, next: Option<Payload>, events: &mut Vec<Event>) -> Result<(), Fault> {
        for payload in self.transfers.pipeline.iter().chain(next.iter()) {
            data::timing::DeviceWrite::from_cpu_port(
                payload.target.publication_port().0,
                payload.value,
            )
            .map_err(|_| Fault::UnsupportedDevice)?;
        }
        if let Some(payload) = self.transfers.pipeline.take() {
            events.push(Event {
                at: self.now,
                kind: EventKind::Payload(payload),
            });
        }
        self.transfers.pipeline = next;
        Ok(())
    }
    fn recognize_transfer(&mut self) {
        let beam = &mut self.beam;
        if !beam.thresholds.initialize && beam.position.horizontal >= beam.thresholds.initialize_at
        {
            beam.thresholds.initialize = true;
            for channel in &mut self.transfers.channels {
                channel.line.completed = false;
                channel.line.payload = false;
            }
            if self
                .transfers
                .channels
                .iter()
                .any(|channel| channel.line.enabled)
            {
                self.transfers.pending_line = Some(LineTransfer::Initialize);
            }
        }
        if !beam.thresholds.visible && beam.position.horizontal >= 1104 {
            beam.thresholds.visible = true;
            if self
                .transfers
                .channels
                .iter()
                .any(|channel| channel.line.enabled && !channel.line.completed)
            {
                self.transfers.pending_line = Some(LineTransfer::Visible);
            }
        }
    }
    fn table_read(&mut self, slot: ChannelSlot, regions: &[ContentRegion]) -> Result<u8, Fault> {
        let channel = self
            .transfers
            .channels
            .get_mut(slot.ordinal())
            .ok_or(Fault::InvalidState)?;
        let value = read(regions, channel.table_address())?;
        channel.table = channel.table.wrapping_add(1);
        Ok(value)
    }
    fn update_channel(
        &mut self,
        slot: ChannelSlot,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        self.dma_charge(4, events)?;
        let channel = self
            .transfers
            .channels
            .get(slot.ordinal())
            .ok_or(Fault::InvalidState)?;
        let descriptor = read(regions, channel.table_address())?;
        self.dma_charge(4, events)?;
        self.flush(None, events)?;
        let channel = self
            .transfers
            .channels
            .get_mut(slot.ordinal())
            .ok_or(Fault::InvalidState)?;
        if channel.lines & 0x7f != 0 {
            return Ok(());
        }
        channel.lines = descriptor;
        channel.table = channel.table.wrapping_add(1);
        channel.line.completed = descriptor == 0;
        channel.line.payload = !channel.line.completed;
        if !channel.config.indirect {
            return Ok(());
        }
        self.dma_charge(4, events)?;
        let first = u16::from(self.table_read(slot, regions)?).wrapping_shl(8);
        self.transfers
            .channels
            .get_mut(slot.ordinal())
            .ok_or(Fault::InvalidState)?
            .pointer = first;
        self.dma_charge(4, events)?;
        self.flush(None, events)?;
        let channel = self
            .transfers
            .channels
            .get(slot.ordinal())
            .ok_or(Fault::InvalidState)?;
        let later_active = self
            .transfers
            .channels
            .iter()
            .skip(slot.ordinal().wrapping_add(1))
            .any(|other| other.line.enabled && !other.line.completed);
        if !channel.line.completed || later_active {
            self.dma_charge(4, events)?;
            let second = u16::from(self.table_read(slot, regions)?).wrapping_shl(8);
            self.transfers
                .channels
                .get_mut(slot.ordinal())
                .ok_or(Fault::InvalidState)?
                .pointer = (first >> 8) | second;
            self.dma_charge(4, events)?;
            self.flush(None, events)?;
        }
        Ok(())
    }
    fn initialize_line(
        &mut self,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        self.dma_charge(8, events)?;
        self.flush(None, events)?;
        for slot in SLOTS {
            let channel = self
                .transfers
                .channels
                .get_mut(slot.ordinal())
                .ok_or(Fault::InvalidState)?;
            if !channel.line.enabled {
                continue;
            }
            channel.bulk_enabled = false;
            channel.table = channel.source_offset;
            channel.lines = 0;
            self.update_channel(slot, regions, events)?;
        }
        self.interrupts.locked = true;
        Ok(())
    }
    fn line_run(
        &mut self,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        self.dma_charge(8, events)?;
        self.flush(None, events)?;
        for slot in SLOTS {
            let channel = self
                .transfers
                .channels
                .get_mut(slot.ordinal())
                .ok_or(Fault::InvalidState)?;
            if !channel.line.enabled || channel.line.completed {
                continue;
            }
            channel.bulk_enabled = false;
            if !channel.line.payload {
                continue;
            }
            events.push(Event {
                at: self.now,
                kind: EventKind::LineService(slot),
            });
            let offsets = channel.config.pattern.offsets();
            for offset in offsets {
                self.dma_charge(4, events)?;
                let channel = self
                    .transfers
                    .channels
                    .get_mut(slot.ordinal())
                    .ok_or(Fault::InvalidState)?;
                let value = if channel.config.indirect {
                    let value = read(
                        regions,
                        BusAddress {
                            bank: channel.config.indirect_bank,
                            offset: channel.pointer,
                        },
                    )?;
                    channel.pointer = channel.pointer.wrapping_add(1);
                    value
                } else {
                    let value = read(regions, channel.table_address())?;
                    channel.table = channel.table.wrapping_add(1);
                    value
                };
                let target = TransferPort(channel.config.target.0.wrapping_add(*offset));
                self.dma_charge(4, events)?;
                self.flush(Some(Payload { target, value }), events)?;
            }
        }
        for slot in SLOTS {
            let channel = self
                .transfers
                .channels
                .get_mut(slot.ordinal())
                .ok_or(Fault::InvalidState)?;
            if !channel.line.enabled || channel.line.completed {
                continue;
            }
            channel.lines = channel.lines.wrapping_sub(1);
            channel.line.payload = channel.lines & 0x80 != 0;
            self.update_channel(slot, regions, events)?;
        }
        self.interrupts.locked = true;
        Ok(())
    }
    fn reject_nested(&mut self) -> Result<(), Fault> {
        self.recognize_transfer();
        if self.transfers.pending_line.is_some() {
            return Err(Fault::NestedTransfer);
        }
        Ok(())
    }
    fn bulk_run(
        &mut self,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        self.dma_charge(8, events)?;
        self.flush(None, events)?;
        self.reject_nested()?;
        for slot in SLOTS {
            let channel = self
                .transfers
                .channels
                .get(slot.ordinal())
                .ok_or(Fault::InvalidState)?;
            if !channel.bulk_enabled {
                continue;
            }
            events.push(Event {
                at: self.now,
                kind: EventKind::BulkService(slot),
            });
            let remaining = if channel.pointer == 0 {
                65536
            } else {
                u32::from(channel.pointer)
            };
            let offsets = channel.config.pattern.offsets();
            let mut ordinal = 0_usize;
            for _ in 0..remaining {
                let channel = self
                    .transfers
                    .channels
                    .get_mut(slot.ordinal())
                    .ok_or(Fault::InvalidState)?;
                let address = BusAddress {
                    bank: channel.config.source_bank,
                    offset: channel.source_offset,
                };
                if !channel.config.fixed {
                    channel.source_offset = if channel.config.reverse {
                        channel.source_offset.wrapping_sub(1)
                    } else {
                        channel.source_offset.wrapping_add(1)
                    };
                }
                let target = TransferPort(
                    channel
                        .config
                        .target
                        .0
                        .wrapping_add(*offsets.get(ordinal).ok_or(Fault::InvalidState)?),
                );
                ordinal = ordinal
                    .wrapping_add(1)
                    .checked_rem(offsets.len())
                    .ok_or(Fault::InvalidState)?;
                self.dma_charge(4, events)?;
                let value = read(regions, address)?;
                self.dma_charge(4, events)?;
                self.flush(Some(Payload { target, value }), events)?;
                self.reject_nested()?;
                let channel = self
                    .transfers
                    .channels
                    .get_mut(slot.ordinal())
                    .ok_or(Fault::InvalidState)?;
                channel.pointer = channel.pointer.wrapping_sub(1);
            }
            self.dma_charge(8, events)?;
            self.flush(None, events)?;
            self.reject_nested()?;
            self.transfers
                .channels
                .get_mut(slot.ordinal())
                .ok_or(Fault::InvalidState)?
                .bulk_enabled = false;
        }
        self.interrupts.locked = true;
        Ok(())
    }
    fn finish_transfer(&mut self, cycle: Cycle, events: &mut Vec<Event>) -> Result<(), Fault> {
        let ticks = u64::from(cycle.ticks());
        let correction = ticks.wrapping_sub(
            self.transfers
                .intrinsic
                .checked_rem(ticks)
                .ok_or(Fault::InvalidState)?,
        );
        self.charge(correction, true, events)?;
        self.transfers.active = false;
        Ok(())
    }
    fn align_transfer(&mut self, events: &mut Vec<Event>) -> Result<(), Fault> {
        let residue = self
            .beam
            .position
            .horizontal
            .wrapping_add(u16::from(self.beam.residue))
            & 0x7;
        self.dma_charge(u64::from(8_u16.wrapping_sub(residue)), events)
    }
    pub(super) fn arbitrate(
        &mut self,
        cycle: Cycle,
        regions: &[ContentRegion],
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        if self.transfers.active {
            if let Some(kind) = self.transfers.pending_line.take()
                && self
                    .transfers
                    .channels
                    .iter()
                    .any(|channel| channel.line.enabled)
            {
                if self
                    .transfers
                    .channels
                    .iter()
                    .any(|channel| channel.bulk_enabled)
                {
                    return Err(Fault::NestedTransfer);
                }
                self.align_transfer(events)?;
                match kind {
                    LineTransfer::Initialize => self.initialize_line(regions, events)?,
                    LineTransfer::Visible => self.line_run(regions, events)?,
                }
                self.finish_transfer(cycle, events)?;
            }
            if std::mem::take(&mut self.transfers.pending_bulk)
                && self
                    .transfers
                    .channels
                    .iter()
                    .any(|channel| channel.bulk_enabled)
            {
                self.align_transfer(events)?;
                self.bulk_run(regions, events)?;
                self.finish_transfer(cycle, events)?;
            }
        }
        self.recognize_transfer();
        if !self.transfers.active
            && (self.transfers.pending_bulk || self.transfers.pending_line.is_some())
        {
            self.transfers.intrinsic = 0;
            self.transfers.active = true;
        }
        Ok(())
    }
}
