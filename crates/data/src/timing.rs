//! Finite bus exposures selected by completed semantic work, separate from live progress.

mod template;
pub use template::{Template, TemplateEffect, TemplateFault, TemplateRun};

use crate::dma::BusAddress;
use canonical::Canonical;

/// Exact elapsed platform time.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MasterTicks(pub u64);

/// Qualified bus cycle class.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cycle {
    /// Six master ticks.
    Six,
    /// Eight master ticks.
    Eight,
    /// Twelve master ticks.
    Twelve,
}
impl Cycle {
    /// Intrinsic cycle duration.
    #[must_use]
    pub const fn ticks(self) -> u8 {
        match self {
            Self::Six => 6,
            Self::Eight => 8,
            Self::Twelve => 12,
        }
    }
}

/// Physical side of one arithmetic edge.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExposureKind {
    /// Sample four ticks before the read ends, then arithmetic.
    Read,
    /// Arithmetic before the full charged write and publication.
    Write,
    /// Six-tick internal opportunity, arithmetic after charging.
    Internal,
    /// Internal opportunity that becomes read-like when admission is pending.
    PendingInternal,
}

/// Hardware interrupt mask policy.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IrqPolicy {
    /// IRQ admission permitted.
    #[default]
    Eligible,
    /// IRQ admission inhibited.
    Masked,
}

/// Typed hardware interrupt source.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interrupt {
    /// Nonmaskable vertical blank interrupt.
    Nmi,
    /// Configured beam interrupt.
    Irq,
}

/// One of the eight hardware transfer slots.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelSlot {
    /// First slot.
    One,
    /// Second slot.
    Two,
    /// Third slot.
    Three,
    /// Fourth slot.
    Four,
    /// Fifth slot.
    Five,
    /// Sixth slot.
    Six,
    /// Seventh slot.
    Seven,
    /// Eighth slot.
    Eight,
}
impl ChannelSlot {
    /// Fixed array ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
            Self::Six => 5,
            Self::Seven => 6,
            Self::Eight => 7,
        }
    }
}

/// Payload destination in the imported hardware device namespace.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransferPort(pub u8);

/// Finite transfer destination pattern.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransferPattern {
    /// One destination.
    #[default]
    Single,
    /// Alternating pair.
    Pair,
    /// Repeated single destination.
    Double,
    /// Two repeated destinations.
    DoublePair,
    /// Four consecutive destinations.
    Quad,
    /// Two alternating pairs.
    PairPair,
    /// Repeated single destination, two bytes.
    DoubleAgain,
    /// Two repeated destinations, four bytes.
    DoublePairAgain,
}
impl TransferPattern {
    /// One payload's destination offsets.
    #[must_use]
    pub const fn offsets(self) -> &'static [u8] {
        match self {
            Self::Single => &[0],
            Self::Pair => &[0, 1],
            Self::Double | Self::DoubleAgain => &[0, 0],
            Self::DoublePair | Self::DoublePairAgain => &[0, 0, 1, 1],
            Self::Quad => &[0, 1, 2, 3],
            Self::PairPair => &[0, 1, 0, 1],
        }
    }
}

/// Hardware configuration of one transfer channel.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelConfig {
    /// Direct source and descriptor table bank.
    pub source_bank: u8,
    /// Indirect data bank.
    pub indirect_bank: u8,
    /// Read source without changing its offset.
    pub fixed: bool,
    /// Decrement the source after each byte.
    pub reverse: bool,
    /// Reload indirect table pointers.
    pub indirect: bool,
    /// Payload destination.
    pub target: TransferPort,
    /// Destination repetition.
    pub pattern: TransferPattern,
}

/// Enable conditions written together at one publication.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InterruptEnable {
    /// NMI transition permission.
    pub nmi: bool,
    /// Beam IRQ comparison mode.
    pub irq: IrqMode,
    /// Automatic controller polling.
    pub controller: bool,
}

/// Enabled beam comparisons for IRQ generation.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IrqMode {
    /// No beam comparison.
    #[default]
    Disabled,
    /// Horizontal comparison on every scanline.
    Horizontal,
    /// Vertical comparison throughout the target line.
    Vertical,
    /// Both target coordinates must match.
    Both,
}
impl IrqMode {
    /// Horizontal comparison is enabled.
    #[must_use]
    pub const fn horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }
    /// Vertical comparison is enabled.
    #[must_use]
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/// Imported publication identity for a downstream device without timing feedback.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationPort(pub u16);

/// First multiplication operand's original device identity.
pub const MULTIPLICAND_PORT: PublicationPort = PublicationPort(0x4202);
/// Multiplication initiation's original device identity.
pub const MULTIPLIER_PORT: PublicationPort = PublicationPort(0x4203);
/// Low dividend operand's device identity.
pub const DIVIDEND_LOW_PORT: PublicationPort = PublicationPort(0x4204);
/// Work RAM data port shared by CPU writes and transfer destinations.
pub const WRAM_DATA_PORT: PublicationPort = PublicationPort(0x2180);
/// First transfer channel's destination configuration identity.
pub const FIRST_CHANNEL_TARGET_PORT: PublicationPort = PublicationPort(0x4301);

/// A single qualified channel field publication, preserving all other channel progress.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelUpdate {
    /// Destination repetition and source progression controls.
    Control {
        /// Indirect table pointer reload.
        indirect: bool,
        /// Fixed bulk source offset.
        fixed: bool,
        /// Decrementing bulk source offset.
        reverse: bool,
        /// Destination sequence.
        pattern: TransferPattern,
    },
    /// Target port.
    Target(TransferPort),
    /// Source offset low byte.
    SourceLow(u8),
    /// Source offset high byte.
    SourceHigh(u8),
    /// Source and descriptor bank.
    SourceBank(u8),
    /// Shared bulk remaining/indirect pointer low byte.
    SizeLow(u8),
    /// Shared bulk remaining/indirect pointer high byte.
    SizeHigh(u8),
}

/// Exact byte publication identity, independent of its owned timing mutation.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct DeviceWrite {
    /// Imported device identity.
    pub port: PublicationPort,
    /// Actual producer byte.
    pub value: u8,
    /// Typed hardware mutation occurring at this same publication.
    pub effect: Publication,
}

/// Actual typed effect accompanying a semantic write.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Publication {
    /// Store multiplication operand.
    Multiplicand(u8),
    /// Begin multiplication when the unit is inactive.
    Multiplier(u8),
    /// Store the inactive division unit's dividend low byte.
    DividendLow(u8),
    /// Enable controller and interrupt sources.
    InterruptEnable(InterruptEnable),
    /// Change only the horizontal target low byte.
    HorizontalLow(u8),
    /// Change only the horizontal target high bit.
    HorizontalHigh(bool),
    /// Change only the vertical target low byte.
    VerticalLow(u8),
    /// Change only the vertical target high bit.
    VerticalHigh(bool),
    /// Change exactly one channel field at its physical byte publication.
    ChannelUpdate(ChannelSlot, ChannelUpdate),
    /// Enable bulk channels, in slot order.
    BulkEnable([bool; 8]),
    /// Enable line transfer channels, in slot order.
    LineEnable([bool; 8]),
    /// Downstream publication with no timing feedback.
    Downstream,
}

/// Owned physical sample class.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sample {
    /// Completed product low byte.
    ProductLow,
    /// Completed product high byte.
    ProductHigh,
    /// First controller result low byte.
    ControllerLow,
    /// First controller result high byte.
    ControllerHigh,
    /// Acknowledge the held IRQ line.
    AcknowledgeIrq,
    /// Acknowledge the held NMI line.
    AcknowledgeNmi,
    /// Vertical/horizontal blanking and controller active status.
    Blanking,
}

/// Optional effect at one physical exposure.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum DeviceEffect {
    /// No device sample or publication.
    None,
    /// Sample after the first read chunk.
    Sample(Sample),
    /// Publish after the full write.
    Publish(DeviceWrite),
}

/// One qualified exposure; no source instruction or expected instant is represented.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Exposure {
    /// Bus duration class.
    pub cycle: Cycle,
    /// Arithmetic and charging order.
    pub kind: ExposureKind,
    /// Sample interrupt transitions before this exposure.
    pub poll_before: bool,
    /// Actual device effect.
    pub effect: DeviceEffect,
}

/// Equivalent exposures compressed by a source-qualified semantic quantity.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// Nonzero number of actual equivalent opportunities.
    pub count: u32,
    /// Physical projection of each opportunity.
    pub exposure: Exposure,
}

/// Immutable hardware projection of one actual semantic work receipt.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    /// Nonempty finite ordered exposures.
    pub runs: Vec<Run>,
}

/// Complete immutable byte region admitted at the data edge.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ContentRegion {
    /// First CPU bus address.
    pub start: BusAddress,
    /// Bytes contained in this region; it may not wrap its bank.
    pub bytes: Vec<u8>,
}

impl PublicationPort {
    /// Recorded output class for an already qualified CPU device.
    ///
    /// # Errors
    /// No qualified output namespace owns this port.
    pub const fn output_class(self) -> Result<u8, PortDecodeError> {
        match self.0 {
            0x2100..=0x213f => Ok(1),
            0x2140..=0x2143 => Ok(2),
            0x2180..=0x2183 => Ok(3),
            0x4200..=0x421f => Ok(4),
            0x4300..=0x437f => Ok(5),
            _ => Err(PortDecodeError::Unsupported),
        }
    }
}
impl TransferPort {
    /// Output identity of the transfer destination at the device-format edge.
    #[must_use]
    pub fn publication_port(self) -> PublicationPort {
        PublicationPort(0x2100_u16.wrapping_add(u16::from(self.0)))
    }
}

/// Unsupported original hardware port or configuration at the data edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDecodeError {
    /// This device class has no admitted mechanism.
    Unsupported,
    /// This configuration violates the admitted hardware premise.
    Configuration,
}

impl DeviceWrite {
    /// Check that the exact byte identity describes the claimed hardware mutation.
    ///
    /// # Errors
    /// Unqualified device/configuration or a byte/mutation mismatch.
    pub fn validate(&self) -> Result<(), PortDecodeError> {
        if Self::from_cpu_port(self.port.0, self.value)?.effect != self.effect {
            return Err(PortDecodeError::Configuration);
        }
        Ok(())
    }
    /// Decode one CPU device write into an exact identity and owned mutation.
    /// Recorder namespaces must already have been qualified as CPU device space.
    ///
    /// # Errors
    /// Unqualified device, reverse transfer direction, overscan or interlace.
    pub fn from_cpu_port(port: u16, value: u8) -> Result<Self, PortDecodeError> {
        let effect = match port {
            0x4200 => Publication::InterruptEnable(InterruptEnable {
                nmi: value & 0x80 != 0,
                irq: match value & 0x30 {
                    0 => IrqMode::Disabled,
                    16 => IrqMode::Horizontal,
                    32 => IrqMode::Vertical,
                    48 => IrqMode::Both,
                    _ => return Err(PortDecodeError::Configuration),
                },
                controller: value & 0x1 != 0,
            }),
            0x4202 => Publication::Multiplicand(value),
            0x4203 => Publication::Multiplier(value),
            0x4204 => Publication::DividendLow(value),
            0x4207 => Publication::HorizontalLow(value),
            0x4208 => Publication::HorizontalHigh(value & 0x1 != 0),
            0x4209 => Publication::VerticalLow(value),
            0x420a => Publication::VerticalHigh(value & 0x1 != 0),
            0x420b => {
                Publication::BulkEnable([1, 2, 4, 8, 16, 32, 64, 128].map(|mask| value & mask != 0))
            }
            0x420c => {
                Publication::LineEnable([1, 2, 4, 8, 16, 32, 64, 128].map(|mask| value & mask != 0))
            }
            0x2133 if value & 0x5 != 0 => return Err(PortDecodeError::Configuration),
            0x2100..=0x2133 | 0x2140..=0x2143 => Publication::Downstream,
            0x4300..=0x4376 => {
                let slot = match (port.wrapping_sub(0x4300)) / 16 {
                    0 => ChannelSlot::One,
                    1 => ChannelSlot::Two,
                    2 => ChannelSlot::Three,
                    3 => ChannelSlot::Four,
                    4 => ChannelSlot::Five,
                    5 => ChannelSlot::Six,
                    6 => ChannelSlot::Seven,
                    7 => ChannelSlot::Eight,
                    _ => return Err(PortDecodeError::Unsupported),
                };
                let update = match port & 0xf {
                    0 => {
                        if value & 0x80 != 0 {
                            return Err(PortDecodeError::Configuration);
                        }
                        let pattern = match value & 0x7 {
                            0 => TransferPattern::Single,
                            1 => TransferPattern::Pair,
                            2 => TransferPattern::Double,
                            3 => TransferPattern::DoublePair,
                            4 => TransferPattern::Quad,
                            5 => TransferPattern::PairPair,
                            6 => TransferPattern::DoubleAgain,
                            7 => TransferPattern::DoublePairAgain,
                            _ => return Err(PortDecodeError::Configuration),
                        };
                        ChannelUpdate::Control {
                            indirect: value & 0x40 != 0,
                            fixed: value & 0x8 != 0,
                            reverse: value & 0x10 != 0,
                            pattern,
                        }
                    }
                    1 => ChannelUpdate::Target(TransferPort(value)),
                    2 => ChannelUpdate::SourceLow(value),
                    3 => ChannelUpdate::SourceHigh(value),
                    4 => ChannelUpdate::SourceBank(value),
                    5 => ChannelUpdate::SizeLow(value),
                    6 => ChannelUpdate::SizeHigh(value),
                    _ => return Err(PortDecodeError::Unsupported),
                };
                Publication::ChannelUpdate(slot, update)
            }
            _ => return Err(PortDecodeError::Unsupported),
        };
        Ok(Self {
            port: PublicationPort(port),
            value,
            effect,
        })
    }
}

impl Sample {
    /// Decode a CPU-owned physical sample class at the data edge.
    ///
    /// # Errors
    /// The device has no admitted physical sample mechanism.
    pub const fn from_cpu_port(port: u16) -> Result<Self, PortDecodeError> {
        match port {
            0x4210 => Ok(Self::AcknowledgeNmi),
            0x4211 => Ok(Self::AcknowledgeIrq),
            0x4212 => Ok(Self::Blanking),
            0x4216 => Ok(Self::ProductLow),
            0x4217 => Ok(Self::ProductHigh),
            0x4218 => Ok(Self::ControllerLow),
            0x4219 => Ok(Self::ControllerHigh),
            _ => Err(PortDecodeError::Unsupported),
        }
    }
}
