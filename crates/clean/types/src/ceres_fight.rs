//! Looks used to select ordinary Ceres fight art.

use serde::{Deserialize, Serialize};

const LAST_BEAT: u8 = 9;

/// Spec: `WingsBeat`. The current wing look, numbered 0 through 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct WingBeat(u8);

impl WingBeat {
    /// Restore a value in its allowed range.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value <= LAST_BEAT {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The value as a number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for WingBeat {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("WingBeat is out of range")
    }
}

impl From<WingBeat> for u8 {
    fn from(value: WingBeat) -> Self {
        value.get()
    }
}

/// Spec: `RibsBeat`. Selects the renderer's two imported strips for a ribs reveal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RibLook {
    /// Ribs fully closed.
    Closed,
    /// The same look while opening and closing.
    PartlyOpen,
    /// Ribs fully open.
    Wide,
}
