//! Movies cannot author malformed Display checkpoint tags, counts or map order.

use super::{CheckpointReader, DisplayStartMetadata, Field, d, r, t};
use std::{collections::BTreeMap, sync::Arc};

// Movies cannot author invalid boolean and Option tags before their payloads.
fn primitive_tags() {
    let mut read = CheckpointReader(&[0, 1, 0xa7]);
    assert_eq!(bool::get(&mut read), Ok(false));
    assert_eq!(bool::get(&mut read), Ok(true));
    assert_eq!(read.0, &[0xa7]);
    let mut read = CheckpointReader(&[0, 1, 0x6a, 0xad]);
    assert_eq!(Option::<u8>::get(&mut read), Ok(None));
    assert_eq!(Option::<u8>::get(&mut read), Ok(Some(0x6a)));
    assert_eq!(read.0, &[0xad]);
    let mut read = CheckpointReader(&[2, 0x73]);
    assert_eq!(
        bool::get(&mut read).err().as_deref(),
        Some("invalid display checkpoint boolean")
    );
    assert_eq!(read.0, &[0x73]);
    let mut read = CheckpointReader(&[0xff, 0x91]);
    assert_eq!(
        Option::<u8>::get(&mut read).err().as_deref(),
        Some("invalid display checkpoint boolean")
    );
    assert_eq!(read.0, &[0x91]);
}

// Movies cannot supply unknown encoded Display phase discriminants.
fn enum_tags() {
    let mut read = CheckpointReader(&[0, 4, 5, 0xe3]);
    assert_eq!(
        d::DisplayPhase::get(&mut read),
        Ok(d::DisplayPhase::Committed)
    );
    assert_eq!(d::DisplayPhase::get(&mut read), Ok(d::DisplayPhase::Failed));
    assert_eq!(
        d::DisplayPhase::get(&mut read).err().as_deref(),
        Some("unknown display checkpoint variant")
    );
    assert_eq!(read.0, &[0xe3]);
}

// Movies cannot forge a sequence count exceeding all remaining encoded bytes.
fn sequence_counts() {
    let bytes = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x82];
    assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), u64::MAX);
    let mut read = CheckpointReader(&bytes);
    assert_eq!(
        Vec::<u8>::get(&mut read).err().as_deref(),
        Some("truncated display checkpoint sequence")
    );
    // count() rejects before collection growth; no element byte was consumed.
    assert_eq!(read.0, &[0x82]);
    let mut read = CheckpointReader(&bytes);
    assert_eq!(
        Arc::<[u8]>::get(&mut read).err().as_deref(),
        Some("truncated display checkpoint sequence")
    );
    assert_eq!(read.0, &[0x82]);
    let mut read = CheckpointReader(&bytes);
    assert_eq!(
        <BTreeMap<u8, bool> as Field>::get(&mut read)
            .err()
            .as_deref(),
        Some("truncated display checkpoint sequence")
    );
    assert_eq!(read.0, &[0x82]);
}

// Movies cannot duplicate or reverse encoded map keys before the next value.
fn map_keys() {
    let mut read = CheckpointReader(&[2, 0, 0, 0, 0, 0, 0, 0, 5, 0, 7, 1, 0xa7]);
    let decoded = <BTreeMap<u8, bool> as Field>::get(&mut read).unwrap();
    assert_eq!(decoded, BTreeMap::from([(5, false), (7, true)]));
    assert_eq!(read.0, &[0xa7]);
    for bytes in [
        [2, 0, 0, 0, 0, 0, 0, 0, 7, 1, 7, 2, 0x4d],
        [2, 0, 0, 0, 0, 0, 0, 0, 7, 1, 6, 2, 0x4d],
    ] {
        let mut read = CheckpointReader(&bytes);
        assert_eq!(
            <BTreeMap<u8, bool> as Field>::get(&mut read)
                .err()
                .as_deref(),
            Some("unordered display checkpoint map")
        );
        // The invalid second bool stays unread: key rejection precedes its value.
        assert_eq!(read.0, &[2, 0x4d]);
    }
}

// Movies cannot corrupt the saved Display body after a valid metadata binding.
fn display_body(
    bytes: &[u8],
    media: Arc<r::SceneMedia>,
    metadata: &DisplayStartMetadata,
    required: &t::DisplayStartRequirements,
) {
    use super::super::component::DisplayCheckpoint;
    assert_eq!(&bytes[..8], b"SMDISPL\x03");
    // Schema 2: 8-byte magic, u64 binding length, binding, 32-byte Revision, phase.
    let binding = usize::try_from(u64::from_le_bytes(bytes[8..16].try_into().unwrap())).unwrap();
    let body = 16 + binding;
    let phase = body + 32;
    assert_eq!(&bytes[body..phase], media.revision().0.as_slice());
    assert_eq!(bytes[phase], 0);
    let mut malformed = bytes.to_vec();
    malformed[phase] = 5;
    assert_eq!(
        DisplayCheckpoint::decode_checkpoint(&malformed, media.clone(), metadata, required)
            .err()
            .as_deref(),
        Some("unknown display checkpoint variant")
    );
    let mut malformed = bytes.to_vec();
    malformed[8..16].copy_from_slice(&[0xff; 8]);
    assert_eq!(
        DisplayCheckpoint::decode_checkpoint(&malformed, media, metadata, required)
            .err()
            .as_deref(),
        Some("truncated actor checkpoint")
    );
}

// Movies cannot exercise independent malformed binary decoder inputs.
pub fn assert_display_checkpoint_rejections(
    bytes: &[u8],
    media: Arc<r::SceneMedia>,
    metadata: &DisplayStartMetadata,
    required: &t::DisplayStartRequirements,
) {
    primitive_tags();
    enum_tags();
    sequence_counts();
    map_keys();
    display_body(bytes, media, metadata, required);
}
