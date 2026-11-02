//! HDMA execution over the current display channels and declared source owners.

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
        context: &mut super::commit::SettlementContext<'_, '_>,
        destination: u8,
    ) -> Result<u8, DisplayError>;
    fn write_destination(
        &mut self,
        context: &mut super::commit::SettlementContext<'_, '_>,
        destination: u8,
        value: u8,
    ) -> Result<(), DisplayError>;
}

struct OwnedHdma<'a, 'b, E> {
    state: &'a mut DisplayState,
    image: &'a mut DisplayImage,
    content: &'a HdmaCatalog,
    effects: &'a mut E,
    window: Option<&'a mut crate::menu_window_state::WindowState>,
    controls: Option<&'a mut crate::fx_sources::ExternalControls<'b>>,
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
        window: None,
        controls: None,
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
        window: None,
        controls: None,
    })
}

/// Execute with the actual current menu owner for explicitly excluded source lanes.
/// Older descriptions with no window lanes retain their original source semantics.
pub fn initialize_with_window(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
    window: &mut crate::menu_window_state::WindowState,
) -> Result<(), DisplayError> {
    hdma::initialize(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
        window: Some(window),
        controls: None,
    })
}
pub fn step_with_window(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
    window: &mut crate::menu_window_state::WindowState,
) -> Result<(), DisplayError> {
    hdma::step(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
        window: Some(window),
        controls: None,
    })
}

pub(crate) fn initialize_with_controls(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
    controls: &mut crate::fx_sources::ExternalControls<'_>,
) -> Result<(), DisplayError> {
    hdma::initialize(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
        window: None,
        controls: Some(controls),
    })
}
pub(crate) fn step_with_controls(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    content: &HdmaCatalog,
    effects: &mut impl Destinations,
    controls: &mut crate::fx_sources::ExternalControls<'_>,
) -> Result<(), DisplayError> {
    hdma::step(&mut OwnedHdma {
        state,
        image,
        content,
        effects,
        window: None,
        controls: Some(controls),
    })
}

impl<E: Destinations> HdmaHost for OwnedHdma<'_, '_, E> {
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
        if let Some(controls) = self.controls.as_deref_mut() {
            let mut live = crate::fx_sources::CurrentSources {
                live: &mut self.state.hdma_live,
                lighting: &mut self.state.visual.fireflea,
                workspace: &mut self.state.workspace,
                controls,
            };
            return binding.read(
                self.content,
                &mut live,
                &mut self.state.hdma,
                kind,
                bank,
                address,
            );
        }
        let mut live = crate::fx_sources::DisplaySources {
            live: &mut self.state.hdma_live,
            window: self.window.as_deref_mut(),
            workspace: &mut self.state.workspace,
            lighting: &mut self.state.visual.fireflea,
        };
        binding.read(
            self.content,
            &mut live,
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
        if let Some(controls) = self.controls.as_deref_mut() {
            let mut live = crate::fx_sources::CurrentSources {
                live: &mut self.state.hdma_live,
                lighting: &mut self.state.visual.fireflea,
                workspace: &mut self.state.workspace,
                controls,
            };
            return binding.write(&mut live, &mut self.state.hdma, bank, address, value);
        }
        let mut live = crate::fx_sources::DisplaySources {
            live: &mut self.state.hdma_live,
            window: self.window.as_deref_mut(),
            workspace: &mut self.state.workspace,
            lighting: &mut self.state.visual.fireflea,
        };
        binding.write(&mut live, &mut self.state.hdma, bank, address, value)
    }
    fn read_destination(&mut self, destination: u8) -> Result<u8, DisplayError> {
        let mut context = super::commit::SettlementContext::new(
            self.state,
            self.image,
            None,
            self.window.as_deref_mut(),
            self.controls.as_deref_mut(),
        );
        self.effects.read_destination(&mut context, destination)
    }
    fn write_destination(&mut self, destination: u8, value: u8) -> Result<(), DisplayError> {
        let mut context = super::commit::SettlementContext::new(
            self.state,
            self.image,
            None,
            self.window.as_deref_mut(),
            self.controls.as_deref_mut(),
        );
        self.effects
            .write_destination(&mut context, destination, value)
    }
}
