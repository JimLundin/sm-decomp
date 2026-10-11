use port::{WRAM_SIZE, Wram};

#[test]
fn death_explosion_retains_enemy_metadata_before_the_enemy_is_cleared() {
    let mut rom = vec![0; 0x38000];
    let offset = 0x30000 + (0xf345 & 0x7fff);
    rom[offset..offset + 2].copy_from_slice(&0xef89u16.to_le_bytes());
    let offset = 0x30000 + (0xefd9 & 0x7fff);
    rom[offset..offset + 2].copy_from_slice(&0xed4bu16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xe54, 64);
    r.set_word(0xf78 + 64, 0xe123);
    r.set_word(0xf7a + 64, 120);
    r.set_word(0xf7e + 64, 300);
    r.set_word(0xf86 + 64, 0x4000);
    r.set_word(0xf96 + 64, 0x1234);
    r.set_word(0xf98 + 64, 0x800);
    port::enemy_projectiles::spawn_with_gfx(&mut r, &rom, 2, 64, 0xf345).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 120);
    assert_eq!(r.word(0x1a93 + 34), 300);
    assert_eq!(r.word(0xf410 + 34), 0x8040);
    assert_eq!(r.word(0xf3c8 + 34), 0xe123);
    assert_eq!(r.word(0x19bb + 34), 0);
    assert_eq!(r.word(0x1b47 + 34), 0xed4b);
    assert_eq!(r.word(0x1b8f + 34), 1);
}
