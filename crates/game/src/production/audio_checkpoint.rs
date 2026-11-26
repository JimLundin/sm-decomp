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
    // The portable cue roster extends by appending tags. Existing payloads keep their
    // exact envelope and bytes. The live shape pin guards the full payload definition.
    const SCHEMA: u64 = {
        assert!(AudioPin::SHAPE == 0x8388_ABDA_7254_3060);
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
