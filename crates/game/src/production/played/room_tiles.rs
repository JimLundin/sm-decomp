//! Completed background writes during the shaft room fetch.

use crate::state::video::Video;

/// Progress of the shaft background as reported by the platform and retained by the room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShaftBackground {
    /// The fetch has no completed background write yet.
    #[default]
    Pending,
    /// The effects clear has filled the shared background staging buffer.
    EffectsPrepared,
    /// The selected shaft effects tiles have reached physical video.
    EffectsTransferred,
    /// The shaft's blank background transfer has finished.
    BackgroundTransferred,
}

impl ShaftBackground {
    /// Retain the latest completed work across pending cuts.
    pub fn advance(&mut self, work: Self) {
        if work != Self::Pending {
            *self = work;
        }
    }

    /// The complete shared background staging page once prepared.
    #[must_use]
    pub const fn shadow_word(self) -> Option<u16> {
        match self {
            Self::Pending => None,
            Self::EffectsPrepared | Self::EffectsTransferred => Some(0x184e),
            Self::BackgroundTransferred => Some(0x0338),
        }
    }

    /// Install the completed shaft background write into the carried physical video.
    #[must_use]
    pub fn install_background(self, video: &mut Video, effects: &[u16]) -> Option<()> {
        let (span, fill) = match self {
            Self::Pending => return Some(()),
            Self::EffectsPrepared => (0x5880..0x6000, Some(0x184e)),
            Self::EffectsTransferred if effects.len() == 0x420 => (0x5be0..0x6000, None),
            Self::EffectsTransferred => return None,
            Self::BackgroundTransferred => (0x4800..0x5000, Some(0x0338)),
        };
        let words = video.memory.vram.get_mut(span)?;
        if let Some(fill) = fill {
            words.fill(fill);
        } else {
            words.copy_from_slice(effects);
        }
        Some(())
    }
}
