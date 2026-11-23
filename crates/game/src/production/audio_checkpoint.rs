//! The standalone Audio checkpoint wire format.

use crate::state::owned::AudioPin;
use canonical::model::{Model, Pinned};
use canonical::{Canonical, DecodeError, Difference, Reader};

pub(super) struct PortableAudio(pub Model<AudioPin>);

impl PortableAudio {
    // The Audio tag is part of the portable envelope, independent of the live scope enum.
    const TAG: u8 = 2;
}

impl Canonical for PortableAudio {
    // This schema identifies the portable envelope and its precise Audio payload. A payload
    // change needs its own format decision, independently of the live checkpoint codecs.
    const SCHEMA: u64 = {
        assert!(AudioPin::SHAPE == 0xF5B0_435C_9274_A3C7);
        0x029A_5174_21B2_EEB2
    };

    fn encode(&self, out: &mut Vec<u8>) {
        Self::TAG.encode(out);
        self.0.encode(out);
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let tag = u8::decode(input)?;
        if tag != Self::TAG {
            return Err(DecodeError::UnknownVariant {
                type_name: "PortableAudio",
                tag,
            });
        }
        Model::decode(input).map(Self)
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        self.0.diff(&other.0)
    }
}
