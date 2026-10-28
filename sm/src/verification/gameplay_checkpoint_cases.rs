//! Numeric composition is checked separately from full binary Session restore.
use super::{DisplayStartMetadata, Session};

pub(crate) fn assert_gameplay_checkpoint(metadata: &DisplayStartMetadata) {
    let world = crate::room_world_fixture::binding(false);
    let mut expected = Session::with_world(world.clone());
    expected.botwoon_death.pending_hit = 0x8001;
    expected.botwoon_death.active = 0xffff;
    expected.botwoon_death.tail_ready = 0x1234;
    expected.botwoon_death.previous_health = 0xfedc;
    expected.botwoon_death.body_landed = 0x8000;
    expected.player.x = 0xfffd;
    expected.player.x_fraction = 0x1357;
    expected.projectiles.shots[9].y_fraction = 0x9876;
    expected.enemy_projectiles.slots[17].x.fraction = 0xabcd;
    expected.compatibility_enemy_scratch = 0x2468;
    expected.room.entry.room =
        crate::game::room_entry::world::Identity::Known(crate::room_world_fixture::ROOM_A);
    let bytes = expected.encode_gameplay_checkpoint(metadata).unwrap();
    let decoded = Session::decode_gameplay_checkpoint(&bytes, metadata, world.clone()).unwrap();
    // These separate owners are explicitly supplied for this numeric-only case;
    // copying them here does not establish their full binary restore contract.
    let restored = decoded.into_session(
        expected.actor_identity.clone(),
        expected.control.clone(),
        expected.presentation.clone(),
    );
    assert_eq!(restored, expected);
    assert_eq!(
        restored.encode_gameplay_checkpoint(metadata).unwrap(),
        bytes
    );
    assert_eq!(
        Session::decode_gameplay_checkpoint(&bytes[..bytes.len() - 1], metadata, world.clone())
            .err()
            .as_deref(),
        Some("truncated actor checkpoint")
    );
    let mut extra = bytes.clone();
    extra.push(0);
    assert_eq!(
        Session::decode_gameplay_checkpoint(&extra, metadata, world.clone())
            .err()
            .as_deref(),
        Some("trailing gameplay checkpoint components")
    );
    let mut oversized = bytes.clone();
    oversized[11..19].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(
        Session::decode_gameplay_checkpoint(&oversized, metadata, world.clone())
            .err()
            .as_deref(),
        Some("truncated actor checkpoint")
    );
    assert_eq!(
        Session::decode_gameplay_checkpoint(
            &bytes,
            metadata,
            crate::room_world_fixture::binding(true)
        )
        .err()
        .as_deref(),
        Some("room checkpoint authored World definition or keys changed")
    );
    let mut changed = metadata.clone();
    changed.identity.capture[0] ^= 1;
    assert_eq!(
        Session::decode_gameplay_checkpoint(&bytes, &changed, world)
            .err()
            .as_deref(),
        Some("actor numeric checkpoint source identity or Reference changed")
    );
    let auxiliary = expected.encode_auxiliary_checkpoint(metadata).unwrap();
    let binding = u64::from_le_bytes(auxiliary[8..16].try_into().unwrap()) as usize;
    assert_eq!(
        &auxiliary[16 + binding..26 + binding],
        &[1, 0x80, 0xff, 0xff, 0x34, 0x12, 0xdc, 0xfe, 0, 0x80]
    );
    let mut old_schema = auxiliary;
    old_schema[7] = 1;
    assert_eq!(
        Session::decode_auxiliary_checkpoint(&old_schema, metadata)
            .err()
            .as_deref(),
        Some("unsupported auxiliary gameplay checkpoint schema")
    );
}
