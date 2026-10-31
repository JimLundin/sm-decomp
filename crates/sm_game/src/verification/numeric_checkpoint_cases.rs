//! Independently authored numeric actors exercise fields absent from movie inputs.
use super::{Actors, ActorsCheckpoint, DisplayStartMetadata};

pub fn assert_numeric_checkpoint(metadata: &DisplayStartMetadata) {
    let mut expected = Actors::default();
    expected.slots[0].definition = 0x1234;
    expected.slots[0].x.whole = 0xabcd;
    expected.slots[0].x.fraction = 0x5678;
    expected.slots[0].bank = 0xa3;
    expected.slots[0].hurt_time = 0x81;
    expected.slots[31].parameters = [0x8001, 0xffff];
    expected.saved[0].contact_mode = 0x8001;
    expected.saved[31].name[17] = 0xf3;
    expected.controls.selected = 0xffff;
    expected.combat.last_power = 0x9192;
    expected.staging.descriptor_prefix[31] = 0x80;
    expected.staging.gap = [0x10, 0x20, 0x30, 0x40];
    expected.graphics.palette_routes[3] = 0xfedc;
    expected.drawing.hook_address = [0x11, 0x22, 0x33, 0x44];
    expected.schedule.active[31] = 0xfffd;
    expected.schedule.interactive[0] = 0x1357;
    expected.schedule.colliding[3] = 0x8000;
    expected.compatibility.pre_hook[7] = 0x7f;
    expected.compatibility.scratch[2] = 0x024a;
    expected.crocomire_melt.column_heights[3][31] = 0x30;
    expected.diagnostics.spawn.y = 0x2468;
    expected.diagnostics.gap = [0x55, 0xaa];
    expected.diagnostics.sprite_contacts_disabled = 0xff01;
    let bytes = expected.encode_numeric_checkpoint(metadata).unwrap();
    // The literal format places the first actor after the declared binding.
    let binding_length = u64::from_le_bytes(bytes[9..17].try_into().unwrap()) as usize;
    let first = 17 + binding_length;
    assert_eq!(
        &bytes[first..first + 6],
        &[0x34, 0x12, 0xcd, 0xab, 0x78, 0x56]
    );
    assert_eq!(&bytes[first + 46..first + 48], &[0xa3, 0x81]);
    assert_eq!(&bytes[bytes.len() - 2..], &[0x01, 0xff]);
    let actual = Actors::decode_numeric_checkpoint(&bytes, metadata).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.encode_numeric_checkpoint(metadata).unwrap(), bytes);
    assert_eq!(
        Actors::decode_numeric_checkpoint(&bytes[..bytes.len() - 1], metadata)
            .err()
            .as_deref(),
        Some("truncated actor checkpoint")
    );
    let mut extra = bytes.clone();
    extra.push(0);
    assert_eq!(
        Actors::decode_numeric_checkpoint(&extra, metadata)
            .err()
            .as_deref(),
        Some("trailing actor numeric checkpoint fields")
    );
    let mut changed = metadata.clone();
    changed.identity.capture[0] ^= 1;
    assert_eq!(
        Actors::decode_numeric_checkpoint(&bytes, &changed)
            .err()
            .as_deref(),
        Some("actor numeric checkpoint source identity or Reference changed")
    );
}
