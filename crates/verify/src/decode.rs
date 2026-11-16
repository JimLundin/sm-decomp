//! Independent observation decoding; production state is admitted by each replay provider.

use crate::source;
use std::fmt;

/// Why an independent observation cannot be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The capture ends before a required source; values are never included in diagnostics.
    WramTooShort {
        /// Stable source identity.
        source: &'static str,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WramTooShort { source } => write!(formatter, "work RAM ends before {source}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Read each signed directional travel, preserving its complete fixed-point fraction.
///
/// # Errors
/// A required directional travel word is absent from the independent capture.
pub fn directional_travel(wram: &[u8]) -> Result<types::DirectionalTravels, DecodeError> {
    let mut travels = types::DirectionalTravels::default();
    for (direction, [whole, fraction]) in source::SAMUS_DIRECTIONAL_TRAVEL {
        travels[direction] = types::TravelDistance(types::fixed(
            core::num::Wrapping(whole.read(wram)?),
            fraction.read(wram)?,
        ));
    }
    Ok(travels)
}
