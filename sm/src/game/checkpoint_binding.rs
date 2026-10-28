//! Pure checkpoint provenance shared by numeric owners and detached consumers.
#![forbid(unsafe_code)]
use crate::game::scene::{component::DisplayStartMetadata, start::ReferenceProfile};

pub(in crate::game) fn audio_reference_bytes(value: &ReferenceProfile) -> Vec<u8> {
    let mut bytes = Vec::new();
    for identity in [
        value.fingerprint,
        value.release,
        value.capture,
        value.arithmetic_policy,
        value.input_policy,
        value.observation_policy,
        value.color_policy,
        value.slowdown_policy,
        value.glitch_policy,
        value.audio_timing_policy,
    ] {
        bytes.extend_from_slice(&identity);
    }
    bytes.extend_from_slice(&value.ticks_numerator.to_le_bytes());
    bytes.extend_from_slice(&value.ticks_denominator.to_le_bytes());
    for word in [value.image.width, value.image.height] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(&value.image.crop_origin.x.to_le_bytes());
    bytes.extend_from_slice(&value.image.crop_origin.y.to_le_bytes());
    bytes.push(match value.image.color {
        crate::game::scene::records::ColorInterpretation::Bgr15Native => 0,
    });
    bytes.push(match value.image.quantization {
        crate::game::scene::records::SceneQuantization::Native => 0,
        crate::game::scene::records::SceneQuantization::Reference => 1,
    });
    bytes
}

pub(in crate::game) fn observation_binding(metadata: &DisplayStartMetadata) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&metadata.identity.schema.to_le_bytes());
    for identity in [
        metadata.identity.gameplay,
        metadata.identity.reference,
        metadata.identity.capture,
        metadata.identity.media.0,
    ] {
        bytes.extend_from_slice(&identity);
    }
    bytes.extend_from_slice(&metadata.identity.committed_tick.to_le_bytes());
    bytes.extend_from_slice(&metadata.identity.next_input_record.to_le_bytes());
    bytes.extend_from_slice(&audio_reference_bytes(&metadata.reference));
    bytes
}
