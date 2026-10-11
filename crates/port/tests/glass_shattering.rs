use port::{WRAM_SIZE, Wram, enemy_projectile_instructions as instructions, enemy_projectiles, glass_shattering};

fn native_definitions() -> Vec<u8> {
    let mut rom = vec![0; 0x38000];
    rom[0x34efc..0x34f06].copy_from_slice(&[0xc5, 0xcd, 0x9b, 0xce, 0x93, 0xcc, 0, 0, 0, 0x30]);
    rom[0x34f0a..0x34f14].copy_from_slice(&[0x6d, 0xce, 0xfb, 0x84, 0xb3, 0xcd, 0, 0, 0, 0x30]);
    for index in 0..16 {
        rom[0x34e41 + index * 2..0x34e43 + index * 2].copy_from_slice(&(0xcc93u16 + index as u16 * 36).to_le_bytes());
    }
    rom
}

#[test]
fn shard_cull_keeps_rng_and_velocity_but_preserves_fractional_motion() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997 + 34, 0xcefc);
    r.set_word(0x1a4b + 34, 0);
    r.set_word(0x1a27 + 34, 0);
    r.set_word(0x1a93 + 34, 255);
    r.set_word(0x1a6f + 34, 0xff00);
    r.set_word(0x1ab7 + 34, 0xffff);
    r.set_word(0x1adb + 34, 1);
    r.set_word(0x5e5, 1234);
    glass_shattering::update(&mut r, &[], 34).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 0xffff);
    assert_eq!(r.word(0x1a27 + 34), 0xff00);
    assert_eq!(r.word(0x1a93 + 34), 256);
    assert_eq!(r.word(0x1a6f + 34), 0);
    assert_eq!(r.word(0x1997 + 34), 0);
    assert_eq!(r.word(0x1adb + 34), 1);
    assert_eq!(r.word(0x5e5), 1234);
}

#[test]
fn aliased_sparkle_uses_two_rng_draws_and_source_parameter_floor() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b + 34, 0xffff);
    r.set_word(0x1a93 + 34, 0);
    r.set_word(0x5e5, 0);
    glass_shattering::initialize(&mut r, &[], 0xce6d, 34, 35).unwrap();
    // NextRandom(0)=273 and NextRandom(273)=1638.
    assert_eq!(r.word(0x1a4b + 34), 0);
    assert_eq!(r.word(0x1a93 + 34), 0xfff6);
    assert_eq!(r.word(0x5e5), 1638);
    assert_eq!(r.word(0x19bb + 34), 1600);
    assert_eq!(r.word(0x1ab7 + 34), 0xa5a5);
}

#[test]
fn four_shards_consume_rng_only_for_successful_allocations() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    for k in (0..36).step_by(2) {
        r.set_word(0x1997 + k, if k < 4 { 0 } else { 0xcefc });
    }
    r.set_word(0x1c27, 78);
    r.set_word(0x1c87 + 78, 33);
    r.set_word(0x7a5, 16);
    let mut rom = native_definitions();
    rom[0x25001..0x25009].copy_from_slice(&[0, 0, 2, 0, 4, 0, 5, 0]);
    assert_eq!(glass_shattering::instruction(&mut r, &rom, 0xd001).unwrap(), 0xd009);
    assert_eq!(r.word(0x1997 + 2), 0xcefc);
    assert_eq!(r.word(0x1997), 0xcefc);
    assert_eq!(r.word(0x1993), 5);
    assert_eq!(r.word(0x1c29), 0);
    assert_eq!(r.word(0x1c2b), 1);
    // Exactly six RNG advances: two allocated shards, three draws each.
    assert_eq!(r.word(0x5e5), 17762);
    assert_eq!(r.byte(0x676), 0x2e);
}

#[test]
fn sparkle_native_delete_script_runs_while_shard_is_culled() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = native_definitions();
    rom[0x34db3..0x34db7].copy_from_slice(&[6, 0, 9, 0x99]);
    rom[0x34dc3..0x34dc5].copy_from_slice(&0x8154u16.to_le_bytes());
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997, 0xcf0a);
    r.set_word(0x1a03, 0x84fb);
    r.set_word(0x1b47, 0xcdc3);
    r.set_word(0x1b8f, 1);
    instructions::run_all(&mut r, &rom, 64).unwrap();
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1b8f), 0);
}

#[test]
fn malformed_initializers_reject_before_mutation() {
    let mut bytes = vec![0; WRAM_SIZE];
    let before = bytes.clone();
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = native_definitions();
    assert_eq!(glass_shattering::initialize(&mut r, &rom, 0xcdc5, 34, 6), Err("glass shard offset extent"));
    assert_eq!(glass_shattering::initialize(&mut r, &rom, 0xce6d, 34, 36), Err("glass sparkle source extent"));
    assert_eq!(glass_shattering::initialize(&mut r, &rom, 0xce6d, 35, 0), Err("invalid glass projectile slot"));
    assert_eq!(bytes, before);
    // The existing allocator remains the public owner of slot selection.
    assert!(enemy_projectiles::spawn_room(&mut Wram::new(&mut bytes).unwrap(), &rom, 0xcefc, 6).is_err());
}
