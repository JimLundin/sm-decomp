//! Physical backdrop and prepared colour encodings.
use super::borrowed::ColourComponent;
use canonical::Canonical;

/// Actual intensity in a completed backdrop band.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BandIntensity(pub u8);

/// One actual completed band, retained in the producer's storage order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackdropBand {
    /// Single colour component selected by the actual producer.
    pub component: ColourComponent,
    /// Actual completed intensity, validated at the import edge.
    pub intensity: BandIntensity,
}

/// Actual prepared fixed-colour disposition from completed Haze preparation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparedFixedColour {
    /// Logical black, encoded with each physical colour component selected.
    Black,
}

/// Actual completed Haze layer-blend choice.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerBlendDisposition {
    /// Background three is excluded by the completed Haze choice.
    NoBackgroundThree,
}

/// Actual completed second-window disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowTwoDisposition {
    /// The completed choice requests no additional window transformation.
    Unchanged,
}

/// Invalid completed-band physical encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandEncodingFault {
    /// Intensity outside the qualified completed producer range.
    Intensity,
}

/// Encode completed actual bands into physical colour writes without generating bands.
/// # Errors
/// Rejects intensity outside the qualified completed producer range.
pub fn encode_bands(bands: &[BackdropBand; 16]) -> Result<[u8; 16], BandEncodingFault> {
    if bands.iter().any(|band| band.intensity.0 > 15) {
        return Err(BandEncodingFault::Intensity);
    }
    Ok(bands.map(|band| {
        let component = match band.component {
            ColourComponent::Red => 0x20,
            ColourComponent::Green => 0x40,
            ColourComponent::Blue => 0x80,
        };
        component | band.intensity.0
    }))
}

/// Encode the completed logical fixed colour with all three components selected.
#[must_use]
pub const fn encode_fixed_colour(colour: PreparedFixedColour) -> [u8; 3] {
    match colour {
        PreparedFixedColour::Black => [0x20, 0x40, 0x80],
    }
}
