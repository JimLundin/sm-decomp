use port::{WRAM_SIZE, Wram, enemy_projectile_instructions as instructions, enemy_projectiles};

#[test]
fn spawn_delay_tracks_the_head_then_stops_reading_it() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x30000..0x3000a].copy_from_slice(&[0xf3, 0xc2, 0x35, 0xc3, 0x32, 0xc4, 6, 6, 0x50, 0x30]);
    r.set_word(0xfba, 100);
    r.set_word(0xfbe, 100);
    r.set_word(0xaf6, 1000);
    r.set_word(0xafa, 1000);
    enemy_projectiles::spawn_room(&mut r, &rom, 0x8000, 0).unwrap();
    assert_eq!(r.word(0x1ab7 + 34), 0);
    assert_eq!(r.word(0x1adb + 34), 0x0450);
    assert_eq!(r.word(0x1aff + 34), 8);
    let mut host = enemy_projectiles::ProjectileHost::from_words([1, 2, 3, 4, 5, 6, 7, 8]);
    for frame in 0..8 {
        r.set_word(0xfba, 100 + frame);
        r.set_word(0xfbe, 100 + frame);
        instructions::preinstruction_with_host(&mut r, &rom, 0xc335, 34, &mut host).unwrap();
        assert_eq!(r.word(0x1a4b + 34), 110 + frame);
        assert_eq!(r.word(0x1a93 + 34), 116 + frame);
        assert_eq!(r.word(0x1aff + 34), 7 - frame);
    }
    r.set_word(0xfba, 1000);
    r.set_word(0xfbe, 1000);
    instructions::preinstruction_with_host(&mut r, &rom, 0xc335, 34, &mut host).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 117);
    assert_eq!(r.word(0x1a93 + 34), 127);
    assert_eq!(r.word(0x1a6f + 34), 0x5000);
    assert_eq!(host.words(), [1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn linked_enemy_collision_retains_signed_promotion_and_death_sentinel() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7854, 128);
    r.set_word(0xf8c + 128, 1);
    r.set_word(0xf7a + 128, 128);
    r.set_word(0xf7e + 128, 128);
    r.set_word(0xf82 + 128, 8);
    r.set_word(0xf84 + 128, 8);
    r.set_word(0x1a4b + 34, 128);
    r.set_word(0x1a93 + 34, 128);
    r.set_word(0x1bb3 + 34, 0x0606);
    assert_eq!(instructions::dispatch_rom(&mut r, &[], "CheckForCollisionWithShitroid_DoubleRet", 34, 0).unwrap(), 1);
    r.set_word(0xf7a + 128, 142);
    assert_eq!(instructions::dispatch_rom(&mut r, &[], "CheckForCollisionWithShitroid_DoubleRet", 34, 0).unwrap(), 0);
    r.set_word(0xf8c + 128, 0);
    assert_eq!(instructions::dispatch_rom(&mut r, &[], "CheckForCollisionWithShitroid_DoubleRet", 34, 0).unwrap(), 255);
    assert_eq!(r.word(0x1997 + 34), 0);
}

#[test]
fn missing_host_and_invalid_linked_slot_reject_before_mutation() {
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x198d, 0x8000);
        r.set_word(0x1997 + 34, 0xcb4b);
        r.set_word(0x1a03 + 34, 0xc335);
        r.set_word(0x7854, 1);
    }
    let before = bytes.clone();
    let mut r = Wram::new(&mut bytes).unwrap();
    assert!(instructions::preinstruction(&mut r, &[], 0xc335, 34).is_err());
    assert!(instructions::run_all(&mut r, &[], 10).is_err());
    assert!(instructions::preinstruction_with_host(&mut r, &[], 0xc335, 34, &mut Default::default()).is_err());
    assert_eq!(bytes, before);
}
