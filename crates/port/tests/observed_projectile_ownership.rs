use ownership::ProjectileHost;
use port::{WRAM_SIZE, Wram, observed_projectile_ownership as ownership};

fn rom() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rom/sm.sfc")).unwrap()
}
fn host() -> ProjectileHost {
    ProjectileHost::from_words([11, 22, 33, 44, 55, 66, 77, 88])
}
fn spawn(r: &mut Wram<'_>, rom: &[u8], host: &mut ProjectileHost, id: u16, parameter: u16) {
    assert!(ownership::supports_call(r, rom, "SpawnEprojWithRoomGfx", id.into(), parameter.into()).unwrap());
    ownership::dispatch_host(r, rom, host, "SpawnEprojWithRoomGfx", id.into(), parameter.into()).unwrap();
}

#[test]
fn full_pool_admits_invalid_definition_without_rom_and_only_sets_parameter() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut expected = bytes.clone();
    expected[0x1993..0x1995].copy_from_slice(&0xbeefu16.to_le_bytes());
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut h = host();
    spawn(&mut r, &[], &mut h, 0xffff, 0xbeef);
    assert_eq!(h, host());
    assert_eq!(bytes, expected);
}

#[test]
fn disabled_scheduler_admits_unrecognized_state_without_accessing_rom() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x198d, 0x7fff);
    assert!(ownership::supports_call(&r, &[], "EprojRunAll", 0, 0).unwrap());
    let before = bytes.clone();
    let mut h = host();
    ownership::dispatch_host(&mut Wram::new(&mut bytes).unwrap(), &[], &mut h, "EprojRunAll", 0, 0).unwrap();
    assert_eq!(h, host());
    assert_eq!(bytes, before);
}

#[test]
fn all_observed_definitions_and_all_free_slots_are_admitted() {
    let rom = rom();
    for id in [0x9642, 0x9650, 0x965e, 0x966c, 0x967a, 0x9688, 0x9696, 0xa387, 0xa395] {
        for free in (0..36).step_by(2) {
            let mut bytes = vec![0; WRAM_SIZE];
            let mut r = Wram::new(&mut bytes).unwrap();
            for k in (0..36).step_by(2) {
                r.set_word(0x1997 + k, if k == free { 0 } else { 0xffff });
            }
            r.set_word(0xe54, 128); // Ridley initializer still reads enemy0.
            r.set_word(0xf7a, 123);
            r.set_word(0xf7e, 222);
            r.set_word(0xf7a + 128, 999);
            r.set_word(0xf7e + 128, 888);
            let mut h = host();
            spawn(&mut r, &rom, &mut h, id, 2);
            assert_eq!(r.word(0x1997 + free), id);
            assert_eq!(h, host());
            if id == 0x9642 {
                assert_eq!(r.word(0x1a4b + free), 148);
                assert_eq!(r.word(0x1a93 + free), 179);
            }
        }
    }
}

#[test]
fn refusal_is_immutable_for_definition_script_callback_and_whole_frame() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut h = host();
    spawn(&mut r, &rom, &mut h, 0xa387, 0);
    r.set_word(0x198d, 0x8000);
    assert!(ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x1997, 0xffff); // Refuse entire traversal before valid higher slot.
    let before = bytes.clone();
    assert!(!ownership::supports_call(&Wram::new(&mut bytes).unwrap(), &rom, "EprojRunAll", 0, 0).unwrap());
    assert_eq!(bytes, before);
    assert_eq!(h, host());
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997, 0);
    r.set_word(0x1a03 + 34, 0xa3a3);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x1a03 + 34, 0xa328);
    r.set_word(0x1b47 + 34, 0xa28e); // Operand bytes are not script entry points.
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    let mut altered = rom.clone();
    altered[0x30000 + (0x95a6 & 0x7fff)] ^= 1;
    assert!(!ownership::supports_call(&r, &altered, "SpawnEprojWithRoomGfx", 0x9642, 0).unwrap());
    let mut altered = rom.clone();
    altered[0x30000 + (0x964c & 0x7fff)] ^= 1;
    assert!(!ownership::supports_call(&r, &altered, "SpawnEprojWithRoomGfx", 0x9642, 0).unwrap());
    assert!(!ownership::supports_call(&r, &rom, "SpawnEprojWithRoomGfx", 0x966c, 3).unwrap());
    assert!(!ownership::supports_call(&r, &rom, "SpawnEprojWithRoomGfx", 0x9696, 36).unwrap());
    assert!(!ownership::supports_call(&r, &rom, "SpawnEprojWithRoomGfx", 0x8000, 0).unwrap());
}

fn fireball() -> (Vec<u8>, Vec<u8>, ProjectileHost) {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x198d, 0x8000);
    r.set_word(0x7a5, 32);
    r.set_word(0xf7a, 160);
    r.set_word(0xf7e, 203);
    r.set_word(0x7832, 0x0100);
    r.set_word(0x7834, 0);
    let mut h = host();
    spawn(&mut r, &rom, &mut h, 0x9642, 1);
    (bytes, rom, h)
}

#[test]
fn collision_extents_cycles_sound_and_repeat_identity_are_checked_before_admission() {
    let (mut bytes, rom, _) = fireball();
    let mut r = Wram::new(&mut bytes).unwrap();
    assert!(ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x7a5, 0);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x7a5, 32);
    assert!(!ownership::supports_call(&r, &rom[..0x38000], "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x1a4b + 34, 0);
    assert!(ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x1a4b + 34, 185);
    r.set_byte(0x647, 16);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_byte(0x647, 0);
    // A two-block extension cycle inside the possible collision rectangle.
    let block = 10 * 32 + 11;
    r.set_word(0x10002 + block * 2, 0x5000);
    r.set_byte(0x16402 + block, 1);
    r.set_word(0x10002 + (block + 1) * 2, 0x5000);
    r.set_byte(0x16402 + block + 1, 255);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    r.set_word(0x10002 + block * 2, 0);
    r.set_word(0x10002 + (block + 1) * 2, 0);
    r.set_word(0x1997 + 34, 0x966c);
    r.set_word(0x1a03 + 34, 0x8170);
    r.set_word(0x1b47 + 34, 0x960c);
    r.set_word(0x1b8f + 34, 1);
    r.set_word(0x1b23 + 34, 0xffff);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
}

#[test]
fn native_scripts_impact_split_repeat_and_delete_remain_admitted() {
    for vertical in [false, true] {
        let (mut bytes, rom, mut h) = fireball();
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x7832, 0);
        r.set_word(0x1ab7 + 34, if vertical { 0 } else { 0x0e00 });
        r.set_word(0x1adb + 34, if vertical { 0x0e00 } else { 0 });
        for i in 0..32 {
            let block = if vertical { 11 * 32 + i } else { i * 32 + 12 };
            r.set_word(0x10002 + block * 2, 0x8000);
        }
        let mut saw_explosion = false;
        let mut saw_split = false;
        let mut saw_repeat = false;
        for _ in 0..140 {
            assert!(ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
            ownership::dispatch_host(&mut r, &rom, &mut h, "EprojRunAll", 0, 0).unwrap();
            let ids: Vec<_> = (0..36).step_by(2).map(|k| r.word(0x1997 + k)).collect();
            saw_explosion |= ids.contains(&if vertical { 0x9650 } else { 0x965e });
            let splits = ids.iter().filter(|id| matches!(**id, 0x966c | 0x967a | 0x9688 | 0x9696)).count();
            saw_split |= splits >= 2;
            saw_repeat |= splits >= 4;
        }
        assert!(saw_explosion && saw_split && saw_repeat);
        assert!((0..36).step_by(2).all(|k| r.word(0x1997 + k) == 0));
        assert_eq!(&h.words()[2..], &host().words()[2..]);
        assert_ne!(&h.words()[..2], &host().words()[..2]);
    }
}

#[test]
fn direct_dispatch_reports_an_error_after_mutation_without_retry_or_rollback() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0xa395);
    r.set_word(0x1a03 + 34, 0x8170);
    r.set_word(0x1b8f + 34, 2);
    r.set_word(0x1997, 0xffff);
    r.set_word(0x1a03, 0xffff);
    let mut h = host();
    assert!(ownership::dispatch_host(&mut r, &rom, &mut h, "EprojRunAll", 0, 0).is_err());
    assert_eq!(r.word(0x1b8f + 34), 1);
    assert_eq!(r.word(0x1991), 0);
    assert_eq!(h, host());
}

#[test]
fn coordinate_envelopes_admit_native_wrapping_and_check_contiguous_scan_indices() {
    let rom = rom();
    for (x, y, width, admit) in [
        (41, 121, 16, true),
        (0, 128, 16, true),
        (65535, 128, 16, true),
        (128, 0, 16, true),
        (128, 65535, 16, true),
        (128, 0, 255, false),
    ] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x198d, 0x8000);
        r.set_word(0x7a5, width);
        let mut h = host();
        spawn(&mut r, &rom, &mut h, 0x9642, 0);
        r.set_word(0x1a4b + 34, x);
        r.set_word(0x1a93 + 34, y);
        r.set_word(0x1ab7 + 34, 0xfb3c);
        r.set_word(0x1adb + 34, 370);
        assert_eq!(
            ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap(),
            admit,
            "x={x} y={y} width={width}"
        );
        if admit {
            ownership::dispatch_host(&mut r, &rom, &mut h, "EprojRunAll", 0, 0).unwrap();
        }
    }
    let (mut bytes, _, _) = fireball();
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b + 34, 0);
    r.set_word(0x1a93 + 34, 128);
    assert!(ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
    // A vertical radius scan beginning at wrapped column4095 increments to
    //4096, even when the corresponding pixel coordinate has wrapped to zero.
    // The conservative envelope must inspect that contiguous block as well.
    let block = 8 * 16 + 4096;
    r.set_word(0x10002 + block * 2, 0x5000);
    r.set_byte(0x16402 + block, 1);
    r.set_word(0x10002 + (block + 1) * 2, 0x5000);
    r.set_byte(0x16402 + block + 1, 255);
    assert!(!ownership::supports_call(&r, &rom, "EprojRunAll", 0, 0).unwrap());
}

fn packed_gfx(enemy: u16, definition: u16) -> i32 {
    (u32::from(enemy) | (u32::from(definition) << 16)) as i32
}

#[test]
fn with_gfx_refuses_invalid_source_definition_or_split_parameter_without_mutation() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let before = bytes.clone();
    let r = Wram::new(&mut bytes).unwrap();
    let h = host();
    for (parameter, enemy, id) in
        [(0, 1, 0xa387), (0, 0x800, 0xa395), (0, 64, 0xffff), (3, 64, 0x966c), (36, 64, 0x9688)]
    {
        assert!(!ownership::supports_call(&r, &rom, "SpawnEprojWithGfx", parameter, packed_gfx(enemy, id)).unwrap());
    }
    assert!(!ownership::supports_call(&r, &[], "SpawnEprojWithGfx", 0, packed_gfx(64, 0xa387)).unwrap());
    assert_eq!(h, host());
    assert_eq!(bytes, before);
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut h = h;
    assert!(ownership::dispatch_host(&mut r, &[], &mut h, "SpawnEprojWithGfx", 0, packed_gfx(1, 0xffff)).is_err());
    assert_eq!(h, host());
    assert_eq!(bytes, before);
}

#[test]
fn with_gfx_saturation_validates_source_then_only_writes_parameter_without_rom() {
    for enemy in [0, 64, 0x7c0] {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        let mut expected = bytes.clone();
        expected[0x1993..0x1995].copy_from_slice(&0xbeefu16.to_le_bytes());
        let mut r = Wram::new(&mut bytes).unwrap();
        let mut h = host();
        let packed = packed_gfx(enemy, 0xffff);
        assert!(ownership::supports_call(&r, &[], "SpawnEprojWithGfx", 0xbeef, packed).unwrap());
        assert!(!ownership::supports_call(&r, &[], "SpawnEprojWithGfx", 0xbeef, packed_gfx(1, 0xffff)).unwrap());
        ownership::dispatch_host(&mut r, &[], &mut h, "SpawnEprojWithGfx", 0xbeef, packed).unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(h, host());
    }
}

#[test]
fn with_gfx_ceres_common_overrides_selected_enemy_gfx_and_retains_partial_reset() {
    let rom = rom();
    for id in [0xa387, 0xa395] {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        // Highest free slot is2; dirty destination state distinguishes fields
        // reset by allocation/common setup from the source-index alias.
        r.set_word(0x1997 + 2, 0);
        r.set_word(0xe54, 128);
        r.set_word(0xaf6, 321);
        r.set_word(0xafa, 654);
        r.set_word(0xf96 + 64, 0x1234);
        r.set_word(0xf98 + 64, 0x4000);
        r.set_word(0xf96 + 128, 0xffff);
        let mut h = host();
        let packed = packed_gfx(64, id);
        assert!(ownership::supports_call(&r, &rom, "SpawnEprojWithGfx", 0xbeef, packed).unwrap());
        ownership::dispatch_host(&mut r, &rom, &mut h, "SpawnEprojWithGfx", 0xbeef, packed).unwrap();
        assert_eq!(r.word(0x1997 + 2), id);
        assert_eq!(r.word(0x1993), 0xbeef);
        assert_eq!(r.word(0x19bb + 2), 0);
        assert_eq!(r.word(0x1a4b + 2), 321);
        assert_eq!(r.word(0x1a93 + 2), if id == 0xa387 { 682 } else { 97 });
        for p in [0x19df, 0x1a27, 0x1a6f, 0x1ab7, 0x1adb, 0x1b23, 0x1bfb] {
            assert_eq!(r.word(p + 2), 0);
        }
        assert_eq!(r.word(0x1aff + 2), if id == 0xa387 { 60 } else { 0 });
        assert_eq!(r.word(0x1b8f + 2), 1);
        assert_eq!(r.word(0x1b6b + 2), 0x8000);
        assert_eq!(r.word(0x1bd7 + 2), 0x3000);
        assert_eq!(r.word(0x1bb3 + 2), 0x0101);
        // Allocation/common setup does not touch unrelated per-slot storage.
        assert_eq!(r.word(0xf3c8 + 2), 0xa5a5);
        assert_eq!(r.word(0xf410 + 2), 0xa5a5);
        assert_eq!(r.word(0xe54), 128);
        assert_eq!(h, host());
    }
}

#[test]
fn with_gfx_split_copies_projectile_source_while_ridley_uses_enemy_zero() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xe54, 128);
    r.set_word(0x1a4b, 432);
    r.set_word(0x1a93, 543);
    r.set_word(0x1aff, 3);
    r.set_word(0xf96 + 64, 0x1234);
    r.set_word(0xf98 + 64, 0x4000);
    let mut h = host();
    ownership::dispatch_host(&mut r, &rom, &mut h, "SpawnEprojWithGfx", 0, packed_gfx(64, 0x966c)).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 432);
    assert_eq!(r.word(0x1a93 + 34), 543);
    assert_eq!(r.word(0x1aff + 34), 3);
    assert_eq!(r.word(0x1b23 + 34), 0x966c);
    assert_eq!(r.word(0x19bb + 34), 2560);
    r.set_word(0xf7a, 100);
    r.set_word(0xf7e, 200);
    r.set_word(0xf7a + 64, 1000);
    r.set_word(0xf7e + 64, 2000);
    r.set_word(0x7832, 0xfb00);
    r.set_word(0x7834, 0x0600);
    ownership::dispatch_host(&mut r, &rom, &mut h, "SpawnEprojWithGfx", 7, packed_gfx(64, 0x9642)).unwrap();
    assert_eq!(r.word(0x1a4b + 32), 125);
    assert_eq!(r.word(0x1a93 + 32), 157);
    assert_eq!(r.word(0x1ab7 + 32), 0xfb00);
    assert_eq!(r.word(0x1adb + 32), 0x0600);
    assert_eq!(r.word(0x19bb + 32), 2560);
    assert_eq!(r.word(0xe54), 128);
    assert_eq!(h, host());
}
