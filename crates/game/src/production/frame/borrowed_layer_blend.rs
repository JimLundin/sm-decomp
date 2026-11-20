//! Actual borrowed layer-blend defaults and prepared completion, without blending rules.
use super::borrowed_prelude_receipts::{LayerBlendDisposition, WindowTwoDisposition};
use crate::state::video::{ColourMathLayers, ColourMathSelect, Layers, WindowSettings};
use canonical::Canonical;

/// Actual immutable default operands offered to the current layer-blend owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerBlendDefaults {
    /// Actual default main-screen layers.
    pub main_screen: Layers,
    /// Actual default sub-screen layers.
    pub sub_screen: Layers,
    /// Actual default colour-math selection.
    pub colour_math_select: ColourMathSelect,
    /// Actual default colour-math layers.
    pub colour_math_layers: ColourMathLayers,
}

/// Complete prepared settings owned by the admitted layer-blend operation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparedLayerSettings {
    /// Actual background-pair and object/colour window selections.
    pub window_select: [WindowSettings; 3],
    /// Actual prepared main-screen layers.
    pub main_screen: Layers,
    /// Actual prepared sub-screen layers.
    pub sub_screen: Layers,
    /// Actual prepared main-screen window mask.
    pub main_window_mask: Layers,
    /// Actual prepared sub-screen window mask.
    pub sub_window_mask: Layers,
    /// Actual next-gameplay colour-math selection.
    pub colour_math_select: ColourMathSelect,
    /// Actual next-gameplay colour-math layers.
    pub colour_math_layers: ColourMathLayers,
}

/// Separately sourced actual layer-blend owner completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct LayerBlendReceipt {
    /// Actual current immutable defaults consumed by this owner.
    pub dependency: LayerBlendDefaults,
    /// Actual completed Haze disposition required by this completion.
    pub haze: LayerBlendDisposition,
    /// Complete actual prepared owner result.
    pub terminal: PreparedLayerSettings,
    /// Actual completed second-window disposition.
    pub window_two: WindowTwoDisposition,
}

/// Invalid restored layer membership at the exact output edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerEncodingFault {
    /// Bits outside the architectural background and object layers.
    Layers,
}
fn encode_layers(layers: Layers) -> Result<u8, LayerEncodingFault> {
    if layers.bits > 31 {
        return Err(LayerEncodingFault::Layers);
    }
    u8::try_from(layers.bits).map_err(|_| LayerEncodingFault::Layers)
}

/// Encode the supplied complete prepared settings without executing blend rules.
/// The caller applies these nine fields atomically and preserves all other owners.
/// # Errors
/// Rejects invalid restored layer membership before producing any output.
pub fn encode(settings: PreparedLayerSettings) -> Result<[u8; 9], LayerEncodingFault> {
    let [window_one_two, window_three_four, window_objects] = settings.window_select;
    Ok([
        window_one_two.0,
        window_three_four.0,
        window_objects.0,
        encode_layers(settings.main_screen)?,
        encode_layers(settings.sub_screen)?,
        encode_layers(settings.main_window_mask)?,
        encode_layers(settings.sub_window_mask)?,
        settings.colour_math_select.0,
        settings.colour_math_layers.0,
    ])
}
