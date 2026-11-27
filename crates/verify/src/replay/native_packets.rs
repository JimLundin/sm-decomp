//! Reviewed source declarations shared by cases that consume the same native capture.

use super::audio::AudioPacket;
use super::index::{Case, InputProvider, Start};
use super::result::Outcome;
use super::sender_offer::SenderPacket;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
/// A provider's typed recipe for authenticating its capture.
pub enum NativePacket {
    /// Sender ingress and its independent FIFO observations.
    Sender(SenderPacket),
    /// Opening-room transport capture and its provenance.
    Audio(AudioPacket),
}

impl NativePacket {
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::Sender(packet) => packet.valid(),
            Self::Audio(packet) => packet.valid(),
        }
    }

    pub(super) fn applies_to(&self, case: &Case) -> bool {
        match self {
            Self::Audio(_) => case.inputs == InputProvider::NativeFrameInputs,
            Self::Sender(_) => {
                matches!(case.start, Start::NativeSenderOffer { .. })
                    || case.inputs == InputProvider::NativeTouchdownInputs
            }
        }
    }
}

impl Case {
    fn native_packet(&self) -> Result<&NativePacket, Outcome> {
        self.native_packet
            .as_ref()
            .or_else(|| super::legacy_packets::native(self))
            .filter(|packet| packet.applies_to(self) && packet.valid())
            .ok_or_else(|| super::data_failure(None, "native packet declaration"))
    }

    pub(super) fn sender_packet(&self) -> Result<&SenderPacket, Outcome> {
        match self.native_packet()? {
            NativePacket::Sender(packet) => Ok(packet),
            _ => Err(super::data_failure(None, "native sender packet")),
        }
    }

    pub(super) fn audio_packet(&self) -> Result<&AudioPacket, Outcome> {
        match self.native_packet()? {
            NativePacket::Audio(packet) => Ok(packet),
            _ => Err(super::data_failure(None, "native audio packet")),
        }
    }
}
