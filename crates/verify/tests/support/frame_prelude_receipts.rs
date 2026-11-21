//! Independently qualified external prelude completions and exact output encoding.
use canonical::Canonical;
use data::timing::ChannelSlot;
use game::production::frame::borrowed::{ColourComponent, DefaultLayerBlend, HazeOrigin, HazeSlot};
use game::production::frame::borrowed_prelude_receipts::{
    BackdropBand, BandIntensity, LayerBlendDisposition, PreparedFixedColour,
};
use serde::Deserialize;

/// Narrow backdrop buffer completion; it does not complete the remaining Haze work.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HazeBandsReceipt {
    /// Actual current Haze inputs, revalidated before applying this completion.
    pub dependency: HazeOrigin,
    /// Complete actual generated buffer in physical storage order.
    pub bands: [BackdropBand; 16],
    /// Actual producer maximum after its completed band operation.
    pub maximum: u16,
}

/// Actual owner stage accompanying narrow band Ready, with the broader tail pending.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HazeReadyOwnerReceipt {
    /// Actual original Haze dependency.
    pub dependency: HazeOrigin,
    /// Complete owner fields at the qualified narrow Ready gate.
    pub ready: HazeOrigin,
    /// Actual completed prepared fixed colour.
    pub fixed_colour: PreparedFixedColour,
}

/// Complete Haze owner result, separate from the earlier narrow band publication.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HazeTerminalReceipt {
    /// Actual original owner inputs retained for dependency revalidation.
    pub dependency: HazeOrigin,
    /// Complete terminal owner fields supplied by the external actual completion.
    pub terminal: HazeOrigin,
    /// Actual enabled physical channels in owner order.
    pub enabled_channels: Vec<ChannelSlot>,
    /// Actual completed layer-blend choice consumed by later blending.
    pub blend: LayerBlendDisposition,
}

/// Invalid external prelude receipt or its complete actual dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreludeReceiptFault {
    /// Malformed external typed projection.
    Format,
    /// Missing or unsupported current dependency.
    Dependency,
    /// Unsupported terminal output or consumption contract.
    Unsupported,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);
#[derive(Deserialize)]
struct Contract {
    format: String,
    requires_actual_current_dependency_equality: ContractFlag,
    consume_once: ContractFlag,
    expected_callbacks_used_as_inputs: ContractFlag,
    band_bytes_used_as_input_dependency: ContractFlag,
}
impl Contract {
    fn validate(&self) -> Result<(), PreludeReceiptFault> {
        if self.format != "borrowed-haze-owner-receipt-v1"
            || !self.requires_actual_current_dependency_equality.0
            || !self.consume_once.0
            || self.expected_callbacks_used_as_inputs.0
            || self.band_bytes_used_as_input_dependency.0
        {
            return Err(PreludeReceiptFault::Unsupported);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
struct Slot {
    kind: String,
    channel: Option<String>,
    countdown: Option<u16>,
    maximum: Option<u16>,
    component: Option<String>,
}
fn channel(name: &str) -> Result<ChannelSlot, PreludeReceiptFault> {
    match name {
        "Three" => Ok(ChannelSlot::Three),
        "Eight" => Ok(ChannelSlot::Eight),
        _ => Err(PreludeReceiptFault::Unsupported),
    }
}
fn component(name: &str) -> Result<ColourComponent, PreludeReceiptFault> {
    match name {
        "Red" => Ok(ColourComponent::Red),
        "Green" => Ok(ColourComponent::Green),
        "Blue" => Ok(ColourComponent::Blue),
        _ => Err(PreludeReceiptFault::Unsupported),
    }
}
impl Slot {
    fn decode(self) -> Result<HazeSlot, PreludeReceiptFault> {
        let unsupported = PreludeReceiptFault::Unsupported;
        match self.kind.as_str() {
            "Empty"
                if self.channel.is_none()
                    && self.countdown.is_none()
                    && self.maximum.is_none()
                    && self.component.is_none() =>
            {
                Ok(HazeSlot::Empty)
            }
            "BackdropRamp" => Ok(HazeSlot::BackdropRamp {
                channel: channel(self.channel.as_deref().ok_or(unsupported)?)?,
                countdown: self.countdown.ok_or(unsupported)?,
                maximum: self.maximum.ok_or(unsupported)?,
                component: component(self.component.as_deref().ok_or(unsupported)?)?,
            }),
            "KeepTransfer" if self.maximum.is_none() && self.component.is_none() => {
                Ok(HazeSlot::KeepTransfer {
                    channel: channel(self.channel.as_deref().ok_or(unsupported)?)?,
                    countdown: self.countdown.ok_or(unsupported)?,
                })
            }
            _ => Err(unsupported),
        }
    }
}
#[derive(Deserialize)]
struct Owner {
    running: bool,
    power_bomb_inactive: bool,
    enabled: bool,
    default_blend: u16,
    slots: Vec<Slot>,
}
impl Owner {
    fn decode(self) -> Result<HazeOrigin, PreludeReceiptFault> {
        let slots: Vec<_> = self
            .slots
            .into_iter()
            .map(Slot::decode)
            .collect::<Result<_, _>>()?;
        Ok(HazeOrigin {
            running: self.running,
            power_bomb_inactive: self.power_bomb_inactive,
            enabled: self.enabled,
            default_blend: DefaultLayerBlend(self.default_blend),
            slots: slots
                .try_into()
                .map_err(|_| PreludeReceiptFault::Unsupported)?,
        })
    }
}
#[derive(Deserialize)]
struct Band {
    component: String,
    intensity: u8,
}
#[derive(Deserialize)]
struct BufferProjection {
    #[serde(flatten)]
    contract: Contract,
    receipt: String,
    dependency: Owner,
    bands: Vec<Band>,
    ready_maximum: u16,
    full_haze_terminal_publication_at_ready: ContractFlag,
    remaining_source_tail_retained: ContractFlag,
    remaining_source_tail_fully_billed: ContractFlag,
}

/// Import independently sourced actual bands; the caller separately pins source provenance.
/// Revalidate the retained original dependency, consume once at the qualified bands Ready
/// gate, and preserve every remaining physical and semantic Haze operation.
/// # Errors
/// Rejects malformed projections, incomplete dependencies and unsupported Ready contracts.
pub fn haze_bands(
    current: &HazeOrigin,
    projection: &str,
) -> Result<HazeBandsReceipt, PreludeReceiptFault> {
    let receipt: BufferProjection =
        toml::from_str(projection).map_err(|_| PreludeReceiptFault::Format)?;
    receipt.contract.validate()?;
    if receipt.receipt != "HazeBandsReady"
        || receipt.full_haze_terminal_publication_at_ready.0
        || !receipt.remaining_source_tail_retained.0
        || !receipt.remaining_source_tail_fully_billed.0
        || receipt.ready_maximum != 16
    {
        return Err(PreludeReceiptFault::Unsupported);
    }
    let dependency = receipt.dependency.decode()?;
    if dependency != *current {
        return Err(PreludeReceiptFault::Dependency);
    }
    let bands: Vec<_> = receipt
        .bands
        .into_iter()
        .map(|band| {
            Ok(BackdropBand {
                component: component(&band.component)?,
                intensity: BandIntensity(band.intensity),
            })
        })
        .collect::<Result<_, PreludeReceiptFault>>()?;
    let bands = bands
        .try_into()
        .map_err(|_| PreludeReceiptFault::Unsupported)?;
    game::production::frame::borrowed_prelude_receipts::encode_bands(&bands)
        .map_err(|_| PreludeReceiptFault::Unsupported)?;
    Ok(HazeBandsReceipt {
        dependency,
        bands,
        maximum: receipt.ready_maximum,
    })
}
#[derive(Deserialize)]
struct TailProjection {
    #[serde(flatten)]
    contract: Contract,
    receipt: String,
    dependency: Owner,
    terminal: Owner,
    enabled_channels: Vec<String>,
    blend: String,
    full_haze_terminal_only_after_physical_portion: ContractFlag,
}

#[derive(Deserialize)]
struct ReadyProjection {
    format: String,
    ready_owner: Owner,
    publish_full_terminal_at_ready: ContractFlag,
    publish_channel_enable_completion_at_ready: ContractFlag,
    prepared_fixed_colour: String,
    prepared_fixed_colour_source_writers_complete_before_ready: ContractFlag,
    later_owner_tail_remains_actual_work: ContractFlag,
    first_countdown_writer_complete_before_ready: ContractFlag,
    second_countdown_writer_complete_before_ready: ContractFlag,
}

/// Import the actual intermediate owner stage paired with completed band publication.
/// This stage does not complete channel enabling or either countdown update.
/// Source admission independently pins this addendum to the actual bands receipt.
/// # Errors
/// Rejects malformed or unsupported Ready stage contracts and mismatched owner dependencies.
pub fn haze_ready_owner(
    bands: &HazeBandsReceipt,
    projection: &str,
) -> Result<HazeReadyOwnerReceipt, PreludeReceiptFault> {
    let receipt: ReadyProjection =
        toml::from_str(projection).map_err(|_| PreludeReceiptFault::Format)?;
    if receipt.format != "haze-narrow-ready-owner-stage-v1"
        || receipt.publish_full_terminal_at_ready.0
        || receipt.publish_channel_enable_completion_at_ready.0
        || receipt.prepared_fixed_colour != "Black"
        || !receipt
            .prepared_fixed_colour_source_writers_complete_before_ready
            .0
        || !receipt.later_owner_tail_remains_actual_work.0
        || receipt.first_countdown_writer_complete_before_ready.0
        || receipt.second_countdown_writer_complete_before_ready.0
    {
        return Err(PreludeReceiptFault::Unsupported);
    }
    let ready = receipt.ready_owner.decode()?;
    let first = ready.slots.first().ok_or(PreludeReceiptFault::Dependency)?;
    if !matches!(first, HazeSlot::BackdropRamp {maximum, ..} if *maximum == bands.maximum) {
        return Err(PreludeReceiptFault::Dependency);
    }
    let dependency = &bands.dependency;
    if ready.running != dependency.running || ready.power_bomb_inactive != dependency.power_bomb_inactive
        || ready.enabled != dependency.enabled || ready.default_blend != dependency.default_blend
        || !ready.slots.iter().zip(&dependency.slots).enumerate().all(|(slot,(actual, prior))| {
            if slot == 0 {
                matches!((actual,prior), (HazeSlot::BackdropRamp {channel:actual_channel,countdown:actual_countdown,component:actual_component,..}, HazeSlot::BackdropRamp {channel:prior_channel,countdown:prior_countdown,component:prior_component,..}) if actual_channel==prior_channel && actual_countdown==prior_countdown && actual_component==prior_component)
            } else {actual == prior}
        }) {
        return Err(PreludeReceiptFault::Dependency);
    }
    Ok(HazeReadyOwnerReceipt {
        dependency: bands.dependency.clone(),
        ready,
        fixed_colour: PreparedFixedColour::Black,
    })
}

/// Import the separately sourced complete Haze owner result, without executing Haze rules.
/// Apply only after the corresponding full qualified physical portion; retain and
/// revalidate the original dependency through earlier narrow bands publication.
/// # Errors
/// Rejects malformed projections, changed dependencies and unsupported terminal contracts.
pub fn haze_terminal(
    current: &HazeOrigin,
    projection: &str,
) -> Result<HazeTerminalReceipt, PreludeReceiptFault> {
    let receipt: TailProjection =
        toml::from_str(projection).map_err(|_| PreludeReceiptFault::Format)?;
    receipt.contract.validate()?;
    if receipt.receipt != "HazeTerminal"
        || !receipt.full_haze_terminal_only_after_physical_portion.0
        || receipt.blend != "NoBackgroundThree"
    {
        return Err(PreludeReceiptFault::Unsupported);
    }
    let dependency = receipt.dependency.decode()?;
    if dependency != *current {
        return Err(PreludeReceiptFault::Dependency);
    }
    let enabled_channels: Vec<_> = receipt
        .enabled_channels
        .iter()
        .map(|name| channel(name))
        .collect::<Result<_, _>>()?;
    if enabled_channels.as_slice() != [ChannelSlot::Three, ChannelSlot::Eight] {
        return Err(PreludeReceiptFault::Unsupported);
    }
    Ok(HazeTerminalReceipt {
        dependency,
        terminal: receipt.terminal.decode()?,
        enabled_channels,
        blend: LayerBlendDisposition::NoBackgroundThree,
    })
}
