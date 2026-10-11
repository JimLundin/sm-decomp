use port::{WRAM_SIZE, Wram, enemy_projectile_instructions::dispatch_rom};

fn rom(weights: [u8; 6]) -> Vec<u8> {
    let mut rom = vec![0; 0x1a0100];
    rom[0x10003a..0x10003c].copy_from_slice(&0x8000u16.to_le_bytes());
    rom[0x1a0000..0x1a0006].copy_from_slice(&weights);
    rom
}

#[test]
fn low_health_forces_health_drops_and_hysteresis_preserves_bias() {
    let rom = rom([0, 255, 0, 0, 0, 0]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf3c8, 0x8000);
    r.set_word(0x9c2, 29);
    assert_eq!(dispatch_rom(&mut r, &rom, "RandomDropRoutine", 0, 0).unwrap(), 2);
    assert_eq!(r.word(0xe1a), 1);
    r.set_word(0x9c2, 30);
    assert_eq!(dispatch_rom(&mut r, &rom, "RandomDropRoutine", 0, 0).unwrap(), 2);
    assert_eq!(r.word(0xe1a), 1);
    r.set_word(0x9c2, 50);
    dispatch_rom(&mut r, &rom, "RandomDropRoutine", 0, 0).unwrap();
    assert_eq!(r.word(0xe1a), 0);
}

#[test]
fn full_inventory_rejects_rare_ammo_and_missing_header_preserves_rng() {
    let rom = rom([0, 0, 0, 0, 255, 0]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf3c8, 0x8000);
    r.set_word(0x9c2, 99);
    r.set_word(0x9c4, 99);
    assert_eq!(dispatch_rom(&mut r, &rom, "RandomDropRoutine", 0, 0).unwrap(), 6);
    r.set_word(0x9cc, 1);
    assert_eq!(dispatch_rom(&mut r, &rom, "RandomDropRoutine", 0, 0).unwrap(), 5);
    r.set_word(0xf3c8, 0);
    let seed = r.word(0x5e5);
    assert_eq!(dispatch_rom(&mut r, &[], "RandomDropRoutine", 0, 0).unwrap(), 6);
    assert_eq!(r.word(0x5e5), seed);
}
