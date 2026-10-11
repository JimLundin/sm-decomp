use port::{WRAM_SIZE, Wram, enemy_projectiles, n00b_tube, plm_instructions};

#[test]
fn shard_phase_comes_from_argument_but_motion_targets_global_slot() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1991, 34);
    r.set_word(0x1ab7, 0);
    r.set_word(0x1ab7 + 34, 0xffff);
    r.set_word(0x1b23 + 34, 0xffff);
    r.set_word(0x1aff + 34, 0xff00);
    r.set_word(0x1adb + 34, 0xff00);
    r.set_word(0x1a93 + 34, 100);
    n00b_tube::preinstruction(&mut r, &[], 0xd89f, 0).unwrap();
    // Phase zero selects table index 128, whose signed sine is 256.
    assert_eq!(r.word(0x1b23 + 34), 0);
    assert_eq!(r.word(0x1aff + 34), 0x3f00);
    assert_eq!(r.word(0x1ab7 + 34), 3);
    assert_eq!(r.word(0x1a93 + 34), 99);
    assert_eq!(r.word(0x1a4b + 34), 0);
    assert_eq!(r.word(0x1ab7), 0);
}

#[test]
fn velocity_instruction_preserves_byte_after_rng_in_unaligned_read() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x5e5, 0);
    r.set_byte(0x5e7, 0xab);
    assert_eq!(n00b_tube::instruction(&mut r, &[], 0xd5e1, 0xd51d, 34), Ok(0xd51d));
    assert_eq!(r.word(0x5e5), 0x111);
    assert_eq!(r.word(0x1ab7 + 34), 0xab01);
    assert_eq!(r.word(0x1adb + 34), 192);
}

#[test]
fn shard_fractional_carry_deletes_at_signed_camera_boundary() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997, 0xd912);
    r.set_word(0x1b23, 0xffff);
    r.set_word(0x1aff, 0xff00);
    r.set_word(0x1ab7, 1);
    r.set_word(0x1a93, 287);
    r.set_word(0x1a6f, 0xff00);
    r.set_word(0x1adb, 1);
    n00b_tube::preinstruction(&mut r, &[], 0xd7fd, 0).unwrap();
    assert_eq!(r.word(0x1b23), 0);
    assert_eq!(r.word(0x1aff), 0);
    assert_eq!(r.word(0x1a93), 288);
    assert_eq!(r.word(0x1997), 0);
    r.set_word(0x1997, 0xd912);
    r.set_word(0x1a93, 0xffff);
    r.set_word(0x1adb, 0);
    n00b_tube::preinstruction(&mut r, &[], 0xd7fd, 0).unwrap();
    assert_eq!(r.word(0x1997), 0xd912);
}

#[test]
fn sixteen_effect_caller_obeys_partial_pool_order_and_final_parameter() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x40000];
    for (pointer, init) in [(0xd912_u16, 0xd6c9_u16), (0xd920, 0xd774)] {
        let p = 0x30000 + usize::from(pointer & 0x7fff);
        rom[p..p + 2].copy_from_slice(&init.to_le_bytes());
    }
    for k in (0..36).step_by(2) {
        r.set_word(0x1997 + k, 1);
    }
    for k in [0, 2, 16, 34] {
        r.set_word(0x1997 + k, 0);
    }
    r.set_word(0x7a5, 16);
    assert_eq!(plm_instructions::instruction(&mut r, &rom, 0xd543, 0xd509, 0), Ok(Some(0xd509)));
    for (k, x) in [(34, 40), (16, 32), (2, 76), (0, 56)] {
        assert_eq!(r.word(0x1997 + k), 0xd912);
        assert_eq!(r.word(0x1b23 + k), x);
    }
    assert_eq!(r.word(0x1993), 10);
    // Exhausted allocation preserves the C behavior even without a readable ROM.
    enemy_projectiles::spawn_room(&mut r, &[], 0xffff, 123).unwrap();
    assert_eq!(r.word(0x1993), 123);
}
