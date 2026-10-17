use snes::{Reference, Subject};
struct CacheScope(std::path::PathBuf);
impl CacheScope {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-paired-scope-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for CacheScope {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn identities(rom: &[u8], pads: &[u16]) -> pipeline_model::CheckpointIdentities {
    pipeline_model::CheckpointIdentities {
        rom_sha256: snes::paired::sha256(rom),
        movie_sha256: "b".repeat(64),
        input_sha256: snes::paired::sha256(movie::write_inputs(pads).as_bytes()),
        source_sha256: "d".repeat(64),
        hardware_sha256: "e".repeat(64),
        profile: "lsnes-profile".into(),
        serializer: "paired-v2".into(),
        compiler: "fixture".into(),
        features: vec!["lsnes-profile".into()],
        settings: pipeline_model::CheckpointSettings {
            seed: 0,
            mode: None,
            candidate_sha256: None,
            disabled_keys: vec![],
        },
        dispatch_sha256: None,
    }
}
mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/test_steps.rs"));
}
fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    let code = [
        0x18, 0xfb, 0xc2, 0x30, 0xa9, 0x34, 0x12, 0x8d, 0x10, 0, 0xe2, 0x20, 0xa9, 0x80, 0x8d, 0, 0x42, 0xa2, 3, 0,
        0xca, 0xd0, 0xfd, 0xa9, 0xab, 0x8d, 0x12, 0, 0x80, 0xf3,
    ];
    rom[..code.len()].copy_from_slice(&code);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM CHECKPOINT TEST   ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}
#[test]
fn separate_reference_and_subject_envelopes_restore_complete_pending_state_and_counts() {
    let rom = rom();
    let mut r = Reference::new(&rom, 0).unwrap();
    let mut s = Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    r.set_input(0x1000);
    s.set_input(0x1000);
    for _ in 0..100 {
        r.step_instruction().unwrap();
        s.step_instruction().unwrap();
    }
    let a = r.checkpoint_state();
    let b = s.checkpoint_state();
    let mut restored_r = Reference::new(&rom, 0).unwrap();
    let mut restored_s = Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    restored_r.restore_checkpoint_state(&a).unwrap();
    restored_s.restore_checkpoint_state(&b).unwrap();
    assert_eq!(r.instruction_snapshot(), restored_r.instruction_snapshot());
    assert_eq!(s.instruction_snapshot(), restored_s.instruction_snapshot());
    assert_eq!(s.coverage().generated_dispatches, restored_s.coverage().generated_dispatches);
    assert!(restored_r.restore_checkpoint_state(&b).is_err());
    assert!(restored_s.restore_checkpoint_state(&a).is_err());
    for _ in 0..100 {
        let cold = r.step_instruction_observed().unwrap();
        let warm = restored_r.step_instruction_observed().unwrap();
        assert_eq!(cold.events, warm.events);
        assert_eq!(cold.after, warm.after);
        let cold = s.step_instruction_observed().unwrap();
        let warm = restored_s.step_instruction_observed().unwrap();
        assert_eq!(cold.events, warm.events);
        assert_eq!(cold.after, warm.after);
    }
}

#[test]
fn pair_cache_requires_cold_next_frame_validation_and_rejects_corruption_or_other_identities() {
    let mut rom = rom();
    let program = [
        0x18, 0xfb, 0xa9, 0x40, 0x8d, 1, 0x43, 0xa9, 0x7e, 0x8d, 4, 0x43, 0xa9, 0, 0x8d, 2, 0x43, 0x8d, 3, 0x43, 0x8d,
        5, 0x43, 0xa9, 0x20, 0x8d, 6, 0x43, 0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0x80, 0xe8,
    ];
    rom[..program.len()].copy_from_slice(&program);
    let mut r = Reference::new(&rom, 0).unwrap();
    let mut s = Subject::fallback_only(&rom, 0).unwrap();
    r.set_input(0);
    s.set_input(0);
    for _ in 0..50 {
        r.step_instruction().unwrap();
        s.step_instruction().unwrap();
    }
    let identities = identities(&rom, &[0, 0, 0, 0]);
    let pair = snes::paired::capture(
        &r,
        &s,
        identities.clone(),
        0,
        0,
        snes::paired::sha256(b"cold fixture instruction history"),
    )
    .unwrap();
    let mut warm_r = Reference::new(&rom, 0).unwrap();
    let mut warm_s = Subject::fallback_only(&rom, 0).unwrap();
    snes::paired::restore(&pair, &identities, &mut warm_r, &mut warm_s).unwrap();
    let report = snes::paired::validate_next_frame(&mut r, &mut s, &mut warm_r, &mut warm_s, 0).unwrap();
    assert_eq!(report.status, "pass");
    assert_eq!(report.completed_frames, 1);
    let mut corrupt = pair.clone();
    corrupt.reference[10] ^= 1;
    assert!(snes::paired::restore(&corrupt, &identities, &mut warm_r, &mut warm_s).is_err());
    let bytes = snes::paired::encode_pair(&pair).unwrap();
    assert!(bytes.len() < pair.reference.len() + pair.subject.len() + 4096);
    let decoded = snes::paired::decode_pair(&bytes).unwrap();
    snes::paired::restore(&decoded, &identities, &mut warm_r, &mut warm_s).unwrap();
    let mut altered = bytes.clone();
    altered.push(0);
    assert!(snes::paired::decode_pair(&altered).is_err());
    altered = bytes.clone();
    altered[4] = 1;
    assert!(snes::paired::decode_pair(&altered).is_err());
    altered = bytes.clone();
    altered[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(snes::paired::decode_pair(&altered).is_err());
    assert!(snes::paired::decode_pair(&bytes[..bytes.len() - 1]).is_err());
    let mut other = identities.clone();
    other.source_sha256 = "f".repeat(64);
    assert!(snes::paired::restore(&pair, &other, &mut warm_r, &mut warm_s).is_err());
}

#[test]
fn persisted_cache_rebuilds_corrupt_or_changed_candidates_and_records_cold_validation() {
    let mut rom = rom();
    let program = [
        0x18, 0xfb, 0xa9, 0x40, 0x8d, 1, 0x43, 0xa9, 0x7e, 0x8d, 4, 0x43, 0xa9, 0, 0x8d, 2, 0x43, 0x8d, 3, 0x43, 0x8d,
        5, 0x43, 0xa9, 0x20, 0x8d, 6, 0x43, 0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0x80, 0xe8,
    ];
    rom[..program.len()].copy_from_slice(&program);
    let ids = identities(&rom, &[0, 0, 0, 0]);
    let scope = CacheScope::new("public");
    let path = scope.0.join("cache");
    let _ = std::fs::remove_dir_all(&path);
    let (pair, created) =
        snes::paired::prepare_cache(&rom, 0, &[], &[0, 0, 0, 0], (0, 50), ids.clone(), &path).unwrap();
    assert_eq!(created.action, "created");
    assert_eq!(created.validation.completed_frames, 1);
    assert_eq!(created.validation.observed_rollovers, 2);
    assert!(path.join("pair.bin").is_file());
    assert!(path.join("cache.toml").is_file());
    assert!(path.join("validation.toml").is_file());
    assert!(!path.join("pair.json").exists());
    let (_, reused) = snes::paired::prepare_cache(&rom, 0, &[], &[0, 0, 0, 0], (0, 50), ids.clone(), &path).unwrap();
    assert_eq!(reused.action, "reused");
    std::fs::write(path.join("pair.bin"), b"truncated").unwrap();
    let (_, rebuilt) = snes::paired::prepare_cache(&rom, 0, &[], &[0, 0, 0, 0], (0, 50), ids.clone(), &path).unwrap();
    assert_eq!(rebuilt.action, "rebuilt");
    assert!(rebuilt.rejection.is_some());
    let mut changed = ids;
    changed.source_sha256 = "f".repeat(64);
    let (new_pair, changed) =
        snes::paired::prepare_cache(&rom, 0, &[], &[0, 0, 0, 0], (0, 50), changed, &path).unwrap();
    assert_eq!(changed.action, "rebuilt");
    assert_ne!(pair.pair_sha256, new_pair.pair_sha256);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn faithful_cache_for_incorrect_generated_store_retains_warm_first_mismatch() {
    fn wrong(m: &mut runtime::Machine) {
        m.write8(0x4301, 0x41);
    }
    let mut rom = rom();
    let program = [
        0x18, 0xfb, 0xa9, 0x40, 0x8d, 1, 0x43, 0xa9, 0x7e, 0x8d, 4, 0x43, 0xa9, 0, 0x8d, 2, 0x43, 0x8d, 3, 0x43, 0x8d,
        5, 0x43, 0xa9, 0x20, 0x8d, 6, 0x43, 0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0x80, 0xe8,
    ];
    rom[..program.len()].copy_from_slice(&program);
    let table = &[(0x8004, 3, &[0x8d, 1, 0x43][..], wrong as runtime::StepFn)];
    let ids = identities(&rom, &[0, 0, 0, 0]);
    let scope = CacheScope::new("bad-store");
    let path = scope.0.join("cache");
    let _ = std::fs::remove_dir_all(&path);
    let (pair, cache) = snes::paired::prepare_cache(&rom, 0, table, &[0, 0, 0, 0], (0, 3), ids, &path).unwrap();
    assert_eq!(cache.validation.status, "pass");
    assert_eq!(cache.validation.candidate_reference_comparison, "divergence");
    let mut r = Reference::new(&rom, 0).unwrap();
    let mut s = Subject::with_steps(&rom, 0, table).unwrap();
    snes::paired::restore(&pair, &pair.identities, &mut r, &mut s).unwrap();
    let report =
        snes::focused::compare_restored(&mut r, &mut s, &[0, 0, 0, 0], (0, 0, 3), 2, &path.join("focus"), "warm")
            .unwrap();
    assert_eq!(report.status, "divergence");
    assert_eq!(report.first_mismatch.as_ref().unwrap().subject.generated_key, Some((0x8004, 3)));
    let _ = std::fs::remove_dir_all(path);
}
