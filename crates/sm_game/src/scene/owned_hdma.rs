//! HDMA execution over the current display channels and declared source owners.
#![forbid(unsafe_code)]

use super::{
    commit::DisplayImage,
    hdma::{self, ChannelByte, ChannelFlag, ChannelWord, HdmaHost, HdmaSource},
    hdma_sources::{ContentKind, HdmaCatalog},
    state::{DisplayError, DisplayState},
};

/// Actual G destination effects. The same current display and resident image
/// reach both operations; no captured register copy or default byte is allowed.
/// An unsupported destination returns its error at the original byte access.
pub trait Destinations {
    fn read_destination(
        &mut self,
        state: &mut DisplayState,
        image: &mut DisplayImage,
        destination: u8,
    ) -> Result<u8, DisplayError>;
    fn write_destination(
        &mut self,
        state: &mut DisplayState,
        image: &mut DisplayImage,
        destination: u8,
        value: u8,
    ) -> Result<(), DisplayError>;
}

struct OwnedHdma<'a, E> {
    state: &'a mut DisplayState,
    image: &'a mut DisplayImage,
    content: &'a HdmaCatalog,
    effects: &'a mut E,
}

pub fn initialize(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
) -> Result<(), DisplayError> {
    hdma::initialize(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
    })
}

pub fn step(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
) -> Result<(), DisplayError> {
    hdma::step(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
    })
}

impl<E: Destinations> HdmaHost for OwnedHdma<'_, E> {
    type Error = DisplayError;

    fn flag(&self, channel: usize, field: ChannelFlag) -> bool {
        let channel = &self.state.hdma.channels[channel];
        match field {
            ChannelFlag::Active => channel.active,
            ChannelFlag::Terminated => channel.terminated,
            ChannelFlag::DoTransfer => channel.do_transfer,
            ChannelFlag::Indirect => channel.indirect,
            ChannelFlag::FromDestination => channel.from_destination,
            ChannelFlag::DmaActive => channel.dma_active,
        }
    }
    fn byte(&self, channel: usize, field: ChannelByte) -> u8 {
        let channel = &self.state.hdma.channels[channel];
        match field {
            ChannelByte::Mode => channel.mode,
            ChannelByte::RepeatCount => channel.repeat_count,
            ChannelByte::SourceBank => channel.source_bank,
            ChannelByte::IndirectBank => channel.indirect_bank,
            ChannelByte::Destination => channel.destination,
            ChannelByte::TransferCursor => channel.transfer_cursor,
        }
    }
    fn word(&self, channel: usize, field: ChannelWord) -> u16 {
        let channel = &self.state.hdma.channels[channel];
        match field {
            ChannelWord::SourceAddress => channel.source_address,
            ChannelWord::TableCursor => channel.table_cursor,
            ChannelWord::IndirectCursor => channel.indirect_cursor,
        }
    }
    fn set_flag(&mut self, channel: usize, field: ChannelFlag, value: bool) {
        let channel = &mut self.state.hdma.channels[channel];
        match field {
            ChannelFlag::Active => channel.active = value,
            ChannelFlag::Terminated => channel.terminated = value,
            ChannelFlag::DoTransfer => channel.do_transfer = value,
            ChannelFlag::Indirect => channel.indirect = value,
            ChannelFlag::FromDestination => channel.from_destination = value,
            ChannelFlag::DmaActive => channel.dma_active = value,
        }
    }
    fn set_byte(&mut self, channel: usize, field: ChannelByte, value: u8) {
        let channel = &mut self.state.hdma.channels[channel];
        match field {
            ChannelByte::Mode => channel.mode = value,
            ChannelByte::RepeatCount => channel.repeat_count = value,
            ChannelByte::SourceBank => channel.source_bank = value,
            ChannelByte::IndirectBank => channel.indirect_bank = value,
            ChannelByte::Destination => channel.destination = value,
            ChannelByte::TransferCursor => channel.transfer_cursor = value,
        }
    }
    fn set_word(&mut self, channel: usize, field: ChannelWord, value: u16) {
        let channel = &mut self.state.hdma.channels[channel];
        match field {
            ChannelWord::SourceAddress => channel.source_address = value,
            ChannelWord::TableCursor => channel.table_cursor = value,
            ChannelWord::IndirectCursor => channel.indirect_cursor = value,
        }
    }
    fn timer(&self) -> u16 {
        self.state.hdma.hdma_timer
    }
    fn set_timer(&mut self, value: u16) {
        self.state.hdma.hdma_timer = value;
    }
    fn invalid_mode(&mut self, _mode: u8) -> DisplayError {
        DisplayError::Geometry
    }

    fn read_source(
        &mut self,
        channel: usize,
        source: HdmaSource,
        bank: u8,
        address: u16,
    ) -> Result<u8, DisplayError> {
        let (binding, kind) = match source {
            HdmaSource::Table => (
                self.state.hdma.channels[channel].table.clone(),
                ContentKind::Table,
            ),
            HdmaSource::Indirect => (
                self.state.hdma.channels[channel].source.clone(),
                ContentKind::Indirect,
            ),
        };
        binding.read(
            self.content,
            &mut self.state.hdma_live,
            &mut self.state.hdma,
            kind,
            bank,
            address,
        )
    }
    fn write_source(
        &mut self,
        channel: usize,
        source: HdmaSource,
        bank: u8,
        address: u16,
        value: u8,
    ) -> Result<(), DisplayError> {
        // Reverse transfer resolves this binding after its destination read.
        let binding = match source {
            HdmaSource::Table => self.state.hdma.channels[channel].table.clone(),
            HdmaSource::Indirect => self.state.hdma.channels[channel].source.clone(),
        };
        binding.write(
            &mut self.state.hdma_live,
            &mut self.state.hdma,
            bank,
            address,
            value,
        )
    }
    fn read_destination(&mut self, destination: u8) -> Result<u8, DisplayError> {
        self.effects
            .read_destination(self.state, self.image, destination)
    }
    fn write_destination(&mut self, destination: u8, value: u8) -> Result<(), DisplayError> {
        self.effects
            .write_destination(self.state, self.image, destination, value)
    }
}
