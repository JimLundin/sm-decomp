use super::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[cfg_attr(test, test)]
pub(super) fn observed_record_preserves_literal_wire_and_full_digests() {
    let temp = Temp::new();
    let identity = Identity {
        reference: "a".into(),
        rom: "b".into(),
        host: "c".into(),
        input: "d".into(),
        observation: "e".into(),
        scenario: "f".into(),
    };
    // Construct the wire independently of the trace encoder. SHA256 vectors:
    // zero-filled 128 KiB (System.Security.Cryptography), NIST abc and empty.
    let mut literal = b"SMTRACE2\x1e\x00\x00\x00\x01\x00\x00\x00a\x01\x00\x00\x00b\x01\x00\x00\x00c\x01\x00\x00\x00d\x01\x00\x00\x00e\x01\x00\x00\x00f\x01\x00\x00\x00\x00\x00\x00\x00".to_vec();
    for hex in [
        "fa43239bcee7b97ca62f007cc68487560a39e19f74f3dde7486db3f98df8e471",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ] {
        for at in (0..64).step_by(2) {
            literal.push(u8::from_str_radix(&hex[at..at + 2], 16).unwrap());
        }
    }
    literal.extend_from_slice(b"SMEND001\x01\x00\x00\x00\x00\x00\x00\x00");
    let digest: [u8; 32] = Sha256::digest(&literal).into();
    literal.extend_from_slice(&digest);
    let reference = Reference::decode(&literal, &identity).unwrap();
    let mut comparison = reference.comparison();
    let mut ram = vec![0; WRAM_BYTES];
    let mut external = b"abc".to_vec();
    let observed = comparison.check_observed(1, &ram, &external, b"").unwrap();
    // The observation retains validated digests even if the original buffers
    // are changed before append; no borrowed bytes or mutable cache survive.
    ram[0] = 1;
    external.reverse();
    let mut writer = Writer::create(temp.trace(), &identity).unwrap();
    writer.record_observed(observed).unwrap();
    assert_eq!(comparison.finish().unwrap(), reference.summary());
    assert_eq!(writer.finish().unwrap(), reference.summary());
    assert_eq!(fs::read(temp.trace()).unwrap(), literal);
}

#[cfg_attr(test, test)]
pub(super) fn observed_sequence_shape_and_component_failures_stick() {
    let (temp, _) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    for (ordinal, ram, external, effects, component) in [
        (2, vec![], vec![], vec![], Some(Component::Ordinal)),
        (1, vec![], vec![], vec![], None),
        (1, wram(99), vec![99], b"second;first".to_vec(), Some(Component::Wram)),
        (1, wram(1), vec![99], b"second;first".to_vec(), Some(Component::External)),
        (1, wram(1), vec![1], b"second;first".to_vec(), Some(Component::Effects)),
    ] {
        let mut comparison = reference.comparison();
        let error = comparison.check_observed(ordinal, &ram, &external, &effects).err().unwrap();
        assert_eq!(error.difference().map(|d| d.component), component);
        if component.is_none() {
            assert!(matches!(error, Error::InvalidObservation(_)));
        }
        assert_eq!(comparison.check_observed(1, &wram(1), &[1], b"first;second").err(), Some(error.clone()));
        assert_eq!(comparison.check(1, &wram(1), &[1], b"first;second"), Err(error.clone()));
        assert_eq!(comparison.finish(), Err(error));
    }
    let mut extra = reference.comparison();
    for ordinal in 1..=3 {
        extra.check_observed(ordinal, &wram(ordinal as u8), &[ordinal as u8], b"first;second").unwrap();
    }
    let error = extra.check_observed(4, &[], &[], &[]).err().unwrap();
    assert_eq!(error.difference().unwrap().component, Component::ExtraRecord);
    assert_eq!(extra.finish(), Err(error));
}

#[cfg_attr(test, test)]
pub(super) fn observed_writer_retains_sequence_extent_and_io_poisoning() {
    let (temp, _) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    for mode in 0..3 {
        let path = temp.0.join(format!("observed-failure-{mode}"));
        let mut writer = Writer::create(&path, &identity()).unwrap();
        let mut comparison = reference.comparison();
        let first = comparison.check_observed(1, &wram(1), &[1], b"first;second").unwrap();
        let observed =
            if mode == 0 { comparison.check_observed(2, &wram(2), &[2], b"first;second").unwrap() } else { first };
        if mode == 1 {
            writer.bytes = MAX_TRACE_BYTES - RECORD_BYTES - FOOTER_BYTES + 1;
        } else if mode == 2 {
            // A read-only handle with a one-byte buffer forces record I/O to
            // fail now, rather than accidentally deferring failure to finish.
            writer.file.flush().unwrap();
            writer.file = BufWriter::with_capacity(1, File::open(&path).unwrap());
        }
        let error = writer.record_observed(observed).unwrap_err();
        match mode {
            0 => assert!(matches!(error, Error::InvalidObservation(_))),
            1 => assert!(matches!(error, Error::InvalidReference(_))),
            _ => assert!(matches!(error, Error::Io(_))),
        }
        assert_eq!(writer.record(1, &wram(1), &[1], b"first;second"), Err(error.clone()));
        let mut retry = reference.comparison();
        let observed = retry.check_observed(1, &wram(1), &[1], b"first;second").unwrap();
        assert_eq!(writer.record_observed(observed), Err(error.clone()));
        assert_eq!(writer.finish(), Err(error));
        assert!(Reference::load(&path, &identity()).is_err());
    }
}

#[cfg_attr(test, test)]
pub(super) fn binary_identity_has_independent_ordered_length_prefixed_layout() {
    let value = Identity {
        reference: "a".into(),
        rom: "b".into(),
        host: "c".into(),
        input: "d".into(),
        observation: "e".into(),
        scenario: "\u{e5}".into(),
    };
    let bytes = b"\x01\x00\x00\x00a\x01\x00\x00\x00b\x01\x00\x00\x00c\x01\x00\x00\x00d\x01\x00\x00\x00e\x02\x00\x00\x00\xc3\xa5";
    assert_eq!(value.encode().unwrap(), bytes);
    assert_eq!(Identity::decode(bytes).unwrap(), value);
    for end in 0..bytes.len() {
        assert!(Identity::decode(&bytes[..end]).is_err());
    }
    let mut extra = bytes.to_vec();
    extra.push(0);
    assert!(Identity::decode(&extra).is_err());
    let mut invalid = bytes.to_vec();
    *invalid.last_mut().unwrap() = 255;
    assert!(Identity::decode(&invalid).is_err());
    invalid = bytes.to_vec();
    invalid[..4].copy_from_slice(&[0, 0, 0, 0]);
    assert!(Identity::decode(&invalid).is_err());
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("out/tests");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn trace(&self) -> PathBuf {
        self.0.join("reference.trace")
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn identity() -> Identity {
    Identity {
        reference: "fixed-c-source-and-library".into(),
        rom: "pristine-and-patched-rom".into(),
        host: "fixed-host-schedule".into(),
        input: "input-bytes-sha256".into(),
        observation: "canonical-wram+external-v1+ordered-effects-v1".into(),
        scenario: "three-steps".into(),
    }
}
fn wram(step: u8) -> Vec<u8> {
    let mut ram = vec![0; WRAM_BYTES];
    ram[WRAM_BYTES - 1] = step;
    ram
}
fn fixture() -> (Temp, Summary) {
    let temp = Temp::new();
    let mut writer = Writer::create(temp.trace(), &identity()).unwrap();
    for step in 1..=3 {
        writer.record(step, &wram(step as u8), &[step as u8], b"first;second").unwrap();
    }
    let summary = writer.finish().unwrap();
    (temp, summary)
}
fn check_all(reference: &Reference) -> Summary {
    let mut comparison = reference.comparison();
    for step in 1..=3 {
        comparison.check(step, &wram(step as u8), &[step as u8], b"first;second").unwrap();
    }
    comparison.finish().unwrap()
}
fn checksum(bytes: &mut [u8]) {
    let at = bytes.len() - 32;
    let hash: [u8; 32] = Sha256::digest(&bytes[..at]).into();
    bytes[at..].copy_from_slice(&hash);
}

#[cfg_attr(test, test)]
pub(super) fn compact_roundtrip_preserves_expected_hashes_and_identity() {
    let (temp, sealed) = fixture();
    let bytes = fs::read(temp.trace()).unwrap();
    assert!(bytes.len() < 2048, "three observations never contain full RAM");
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    assert_eq!(reference.identity(), &identity());
    assert_eq!(reference.summary(), sealed);
    assert_eq!(sealed.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
    let first = reference.expected(1).unwrap();
    assert_eq!(first.wram, <[u8; 32]>::from(Sha256::digest(wram(1))));
    assert_eq!(first.external, <[u8; 32]>::from(Sha256::digest([1])));
    assert_eq!(first.effects, <[u8; 32]>::from(Sha256::digest(b"first;second")));
    assert!(reference.expected(0).is_none());
    assert!(reference.expected(4).is_none());
    assert_eq!(check_all(&reference), sealed);
    assert_eq!(check_all(&reference), sealed, "candidate comparisons reuse one immutable reference");
}

#[cfg_attr(test, test)]
pub(super) fn every_identity_field_is_required_and_bound() {
    let (temp, _) = fixture();
    for field in 0..6 {
        let mut expected = identity();
        let values = [
            &mut expected.reference,
            &mut expected.rom,
            &mut expected.host,
            &mut expected.input,
            &mut expected.observation,
            &mut expected.scenario,
        ];
        values.into_iter().nth(field).unwrap().push('!');
        assert!(matches!(Reference::load(temp.trace(), &expected), Err(Error::IdentityMismatch { .. })));
    }
    let mut empty = identity();
    empty.input.clear();
    assert!(matches!(Writer::create(temp.0.join("empty-identity"), &empty), Err(Error::InvalidReference(_))));
    let mut huge = identity();
    huge.scenario = "x".repeat(MAX_IDENTITY_BYTES);
    assert!(matches!(Writer::create(temp.0.join("huge-identity"), &huge), Err(Error::InvalidReference(_))));
}

#[cfg_attr(test, test)]
pub(super) fn all_byte_corruption_truncation_and_appends_are_rejected() {
    let (temp, _) = fixture();
    let bytes = fs::read(temp.trace()).unwrap();
    for offset in 0..bytes.len() {
        let mut changed = bytes.clone();
        changed[offset] ^= 1;
        assert!(Reference::decode(&changed, &identity()).is_err(), "corruption at {offset}");
        assert!(Reference::decode(&bytes[..offset], &identity()).is_err(), "truncation at {offset}");
    }
    let mut appended = bytes.clone();
    appended.push(0);
    assert!(Reference::decode(&appended, &identity()).is_err());
    let mut duplicate_file = bytes.clone();
    duplicate_file.extend(bytes);
    assert!(Reference::decode(&duplicate_file, &identity()).is_err());
}

#[cfg_attr(test, test)]
pub(super) fn malformed_records_are_rejected_even_with_a_recomputed_checksum() {
    let (temp, _) = fixture();
    let original = fs::read(temp.trace()).unwrap();
    let start = 12 + u32::from_le_bytes(original[8..12].try_into().unwrap()) as usize;
    let mut changed = original.clone();
    changed[start + RECORD_BYTES..start + RECORD_BYTES + 8].copy_from_slice(&1u64.to_le_bytes());
    checksum(&mut changed);
    assert!(matches!(Reference::decode(&changed, &identity()), Err(Error::InvalidReference(_))));

    let mut changed = original.clone();
    let first = changed[start..start + RECORD_BYTES].to_vec();
    changed.copy_within(start + RECORD_BYTES..start + 2 * RECORD_BYTES, start);
    changed[start + RECORD_BYTES..start + 2 * RECORD_BYTES].copy_from_slice(&first);
    checksum(&mut changed);
    assert!(Reference::decode(&changed, &identity()).is_err(), "reordered records");

    let mut changed = original.clone();
    changed.drain(start..start + RECORD_BYTES);
    checksum(&mut changed);
    assert!(Reference::decode(&changed, &identity()).is_err(), "missing record");

    let mut changed = original.clone();
    let footer = changed.len() - FOOTER_BYTES;
    changed.splice(footer..footer, original[start..start + RECORD_BYTES].iter().copied());
    checksum(&mut changed);
    assert!(Reference::decode(&changed, &identity()).is_err(), "extra record");

    let mut changed = original.clone();
    let count = changed.len() - FOOTER_BYTES + 8;
    changed[count..count + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    checksum(&mut changed);
    assert!(Reference::decode(&changed, &identity()).is_err(), "wrong count");

    let mut changed = original;
    changed[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    checksum(&mut changed);
    assert!(Reference::decode(&changed, &identity()).is_err(), "unbounded identity");
}

#[cfg_attr(test, test)]
pub(super) fn candidate_state_and_effect_changes_report_first_component_and_stick() {
    let (temp, _) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    for component in [Component::Wram, Component::External, Component::Effects] {
        let mut comparison = reference.comparison();
        comparison.check(1, &wram(1), &[1], b"first;second").unwrap();
        let ram = if component == Component::Wram { wram(99) } else { wram(2) };
        let external: &[u8] = if component == Component::External { &[99] } else { &[2] };
        let effects: &[u8] = if component == Component::Effects { b"second;first" } else { b"first;second" };
        let error = comparison.check(2, &ram, external, effects).unwrap_err();
        let difference = error.difference().unwrap();
        assert_eq!(difference.ordinal, 2);
        assert_eq!(difference.component, component);
        assert_ne!(difference.expected, difference.actual);
        assert_eq!(comparison.check(2, &wram(2), &[2], b"first;second"), Err(error.clone()));
        assert_eq!(comparison.check(3, &wram(3), &[3], b"first;second"), Err(error.clone()));
        assert_eq!(comparison.finish(), Err(error));
    }
}

#[cfg_attr(test, test)]
pub(super) fn candidate_order_missing_and_extra_records_fail_without_searching_ahead() {
    let (temp, _) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    let mut order = reference.comparison();
    let error = order.check(2, &wram(2), &[2], b"first;second").unwrap_err();
    let difference = error.difference().unwrap();
    assert_eq!((difference.ordinal, difference.component), (1, Component::Ordinal));
    assert_eq!((difference.expected_ordinal, difference.actual_ordinal), (Some(1), Some(2)));
    assert_eq!(order.finish(), Err(error));

    let mut missing = reference.comparison();
    missing.check(1, &wram(1), &[1], b"first;second").unwrap();
    let error = missing.finish().unwrap_err();
    let difference = error.difference().unwrap();
    assert_eq!((difference.ordinal, difference.component), (2, Component::MissingRecord));
    let mut extra = reference.comparison();
    for step in 1..=3 {
        extra.check(step, &wram(step as u8), &[step as u8], b"first;second").unwrap();
    }
    let error = extra.check(4, &wram(4), &[4], b"first;second").unwrap_err();
    let difference = error.difference().unwrap();
    assert_eq!((difference.ordinal, difference.component), (4, Component::ExtraRecord));
    assert_eq!(extra.finish(), Err(error));
}

#[cfg_attr(test, test)]
pub(super) fn empty_partial_and_invalid_producers_never_seal_and_existing_files_are_preserved() {
    let temp = Temp::new();
    let writer = Writer::create(temp.trace(), &identity()).unwrap();
    assert!(writer.finish().is_err());
    assert!(Reference::load(temp.trace(), &identity()).is_err());
    let before = fs::read(temp.trace()).unwrap();
    assert!(Writer::create(temp.trace(), &identity()).is_err());
    assert_eq!(fs::read(temp.trace()).unwrap(), before);

    let partial = temp.0.join("partial.trace");
    let mut writer = Writer::create(&partial, &identity()).unwrap();
    writer.record(1, &wram(1), &[1], b"effect").unwrap();
    drop(writer);
    assert!(Reference::load(&partial, &identity()).is_err());

    for (index, invalid_ram) in [false, true].into_iter().enumerate() {
        let path = temp.0.join(format!("invalid-{index}.trace"));
        let mut writer = Writer::create(&path, &identity()).unwrap();
        let error = if invalid_ram {
            writer.record(1, &[0], &[], &[]).unwrap_err()
        } else {
            writer.record(2, &wram(1), &[], &[]).unwrap_err()
        };
        assert_eq!(writer.record(1, &wram(1), &[], &[]), Err(error.clone()));
        assert_eq!(writer.finish(), Err(error));
        assert!(Reference::load(&path, &identity()).is_err());
    }
}

#[cfg_attr(test, test)]
pub(super) fn invalid_candidate_shape_poisons_comparison() {
    let (temp, _) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    let mut comparison = reference.comparison();
    let error = comparison.check(1, &[0], &[], &[]).unwrap_err();
    assert!(matches!(error, Error::InvalidObservation(_)));
    assert_eq!(comparison.finish(), Err(error));
}

#[cfg_attr(test, test)]
pub(super) fn authenticated_hashes_are_immutable_after_loading() {
    let (temp, sealed) = fixture();
    let reference = Reference::load(temp.trace(), &identity()).unwrap();
    fs::write(temp.trace(), b"changed after reference load").unwrap();
    assert!(Reference::load(temp.trace(), &identity()).is_err());
    assert_eq!(check_all(&reference), sealed);
}

#[cfg_attr(test, test)]
pub(super) fn oversized_reference_is_rejected_before_reading_its_payload() {
    let temp = Temp::new();
    File::create(temp.trace()).unwrap().set_len(MAX_TRACE_BYTES as u64 + 1).unwrap();
    assert!(matches!(Reference::load(temp.trace(), &identity()), Err(Error::InvalidReference(_))));
}

#[cfg_attr(test, test)]
pub(super) fn header_length_endian_and_total_extent_are_independent() {
    let value = Identity {
        reference: "x".repeat(258),
        rom: "b".into(),
        host: "c".into(),
        input: "d".into(),
        observation: "e".into(),
        scenario: "f".into(),
    };
    let mut literal = vec![2, 1, 0, 0];
    literal.extend_from_slice(&[b'x'; 258]);
    literal.extend_from_slice(b"\x01\x00\x00\x00b\x01\x00\x00\x00c\x01\x00\x00\x00d\x01\x00\x00\x00e\x01\x00\x00\x00f");
    assert_eq!(value.encode().unwrap(), literal);
    assert_eq!(Identity::decode(&literal).unwrap(), value);

    // Six lengths use 24 bytes; five one-byte fields use 5 bytes.
    let mut at_limit = value;
    at_limit.reference = "x".repeat(16_384 - 29);
    let encoded = at_limit.encode().unwrap();
    assert_eq!(encoded.len(), 16_384);
    assert_eq!(Identity::decode(&encoded).unwrap(), at_limit);
    at_limit.reference.push('x');
    assert!(at_limit.encode().is_err());
}

#[cfg_attr(test, test)]
pub(super) fn each_empty_header_field_refuses_without_other_corruption() {
    let literal =
        b"\x01\x00\x00\x00a\x01\x00\x00\x00b\x01\x00\x00\x00c\x01\x00\x00\x00d\x01\x00\x00\x00e\x01\x00\x00\x00f";
    assert!(Identity::decode(literal).is_ok());
    for field in 0..6 {
        let mut invalid = literal.to_vec();
        let offset = field * 5;
        // Remove only that payload, and declare only that field empty.
        invalid[offset..offset + 4].copy_from_slice(&[0; 4]);
        invalid.remove(offset + 4);
        assert!(Identity::decode(&invalid).is_err(), "field {field}");
    }
}

#[cfg_attr(test, test)]
pub(super) fn full_trace_header_refusals_are_not_checksum_failures() {
    let (temp, _) = fixture();
    let original = fs::read(temp.trace()).unwrap();
    let identity_len = u32::from_le_bytes(original[8..12].try_into().unwrap()) as usize;
    let header_end = 12 + identity_len;
    assert!(Reference::decode(&original, &identity()).is_ok());
    let mut offsets = Vec::new();
    let mut cursor = 12;
    for _ in 0..6 {
        offsets.push(cursor);
        let count = u32::from_le_bytes(original[cursor..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4 + count;
    }
    assert_eq!(cursor, header_end);
    for offset in offsets {
        for extent in [[0, 0, 0, 0], [255, 255, 255, 255]] {
            let mut invalid = original.clone();
            invalid[offset..offset + 4].copy_from_slice(&extent);
            checksum(&mut invalid);
            assert!(Reference::decode(&invalid, &identity()).is_err());
        }
        let mut invalid_utf8 = original.clone();
        invalid_utf8[offset + 4] = 255;
        checksum(&mut invalid_utf8);
        assert!(Reference::decode(&invalid_utf8, &identity()).is_err());
    }
    let mut trailing_identity = original.clone();
    trailing_identity.insert(header_end, 0);
    trailing_identity[8..12].copy_from_slice(&((identity_len + 1) as u32).to_le_bytes());
    checksum(&mut trailing_identity);
    assert!(Reference::decode(&trailing_identity, &identity()).is_err());

    let mut legacy_version = original;
    legacy_version[7] = b'1';
    checksum(&mut legacy_version);
    assert!(Reference::decode(&legacy_version, &identity()).is_err());
}
