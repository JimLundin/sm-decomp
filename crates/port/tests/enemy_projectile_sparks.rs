use port::{WRAM_SIZE, Wram, enemy_projectile_sparks};

#[test]
fn initializer_copies_enemy_fractions_and_reads_the_eighth_pair_from_rom() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x373f0..0x373f4].copy_from_slice(&[0xbd, 0xdb, 0x1a, 0x30]);
    r.set_word(0xe54, 0x7c0);
    r.set_word(0xf7a + 0x7c0, 0xffff);
    r.set_word(0xf7c + 0x7c0, 0xfe37);
    r.set_word(0xf7e + 0x7c0, 0xfffc);
    r.set_word(0xf80 + 0x7c0, 0xfe73);
    r.set_word(0x5e5, 9); // NextRandom is318=0x13e; masked bits select pair7.
    enemy_projectile_sparks::initialize(&mut r, &rom, 34).unwrap();
    assert_eq!(r.word(0x5e5), 318);
    assert_eq!(r.word(0x1a4b + 34), 0xffff);
    assert_eq!(r.word(0x1a27 + 34), 0xfe37);
    assert_eq!(r.word(0x1a93 + 34), 4);
    assert_eq!(r.word(0x1a6f + 34), 0xfe73);
    assert_eq!(r.word(0x1b23 + 34), 0xdbbd);
    assert_eq!(r.word(0x1aff + 34), 0x301a);
}

#[test]
fn downward_probe_and_full_fractional_gravity_both_move_the_spark() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1991, 0);
    r.set_word(0x7a5, 64);
    r.set_word(0x1bb3, 0x1010);
    r.set_word(0x1a4b, 128);
    r.set_word(0x1a93, 128);
    r.set_word(0x1a6f, 0xff80);
    r.set_word(0x1adb, 1);
    r.set_word(0x1ab7, 0xffff);
    r.set_byte(0x5b5, 1);
    enemy_projectile_sparks::update(&mut r, &[], 0).unwrap();
    assert_eq!(r.word(0x1adb), 2);
    assert_eq!(r.word(0x1ab7), 0x3fff);
    assert_eq!(r.word(0x1a93), 131);
    assert_eq!(r.word(0x1a6f), 0x407f);
}

#[test]
fn landing_quadruples_full_horizontal_velocity_and_returns_before_trail() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1991, 0);
    r.set_word(0x7a5, 64);
    r.set_word(0x1bb3, 0x1010);
    r.set_word(0x1a4b, 128);
    r.set_word(0x1a27, 0xfe37);
    r.set_word(0x1a93, 128);
    r.set_word(0x1a6f, 0xfe73);
    r.set_word(0x1b23, 0xffff);
    r.set_word(0x1aff, 0xb800);
    for x in 0..64 {
        r.set_word(0x10002 + (8 * 64 + x) * 2, 0x8000);
    }
    enemy_projectile_sparks::update(&mut r, &[], 0).unwrap();
    assert_eq!(r.word(0x1b47), 0xf363);
    assert_eq!(r.word(0x1b8f), 1);
    assert_eq!(r.word(0x1b23), 0xfffe);
    assert_eq!(r.word(0x1aff), 0xe000);
    assert_eq!(r.word(0x1a4b), 128);
    assert_eq!(r.word(0x1a27), 0xfe37);
    assert_eq!(r.word(0x1a93), 126);
    assert_eq!(r.word(0x1a6f), 0);
    assert_eq!(r.word(0x1adb), 0xffff);
    assert_eq!(r.word(0x1ab7), 0x8000);
    assert_eq!(r.word(0xef78 + 62), 0);
}

#[test]
fn upward_trail_uses_byte_nmi_phase_final_position_and_highest_sprite_slot() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x1b0000];
    rom[0x1a3e08..0x1a3e0a].copy_from_slice(&0x9000u16.to_le_bytes());
    rom[0x1a1000..0x1a1002].copy_from_slice(&7u16.to_le_bytes());
    r.set_word(0x1adb, 0xffff);
    r.set_word(0x1ab7, 0x8000);
    r.set_word(0x1b23, 0);
    r.set_word(0x1aff, 0x8000);
    r.set_word(0x1a4b, 0xffff);
    r.set_word(0x1a27, 0x8000);
    r.set_word(0x1a93, 0);
    r.set_word(0x1a6f, 0);
    r.set_word(0x19df, 0x1234);
    r.set_word(0x19bb, 0xa6b5);
    r.set_word(0x5b4, 0xfc12);
    enemy_projectile_sparks::update(&mut r, &rom, 0).unwrap();
    assert_eq!(r.word(0xf0f8 + 62), 0);
    assert_eq!(r.word(0xf1f8 + 62), 0xffff);
    assert_eq!(r.word(0xf078 + 62), 0xa6b5);
    assert_eq!(r.word(0xef78 + 62), 0x9000);
    assert_eq!(r.word(0xeff8 + 62), 7);
    assert_eq!(r.word(0xef78 + 60), 0);
}
