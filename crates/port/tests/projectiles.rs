use port::{WRAM_SIZE, Wram, projectiles};
fn offset(a: u32) -> usize {
    (((a >> 16) & 127) * 0x8000 + (a & 0x7fff)) as usize
}
fn word(rom: &mut [u8], a: u32, v: u16) {
    let i = offset(a);
    rom[i..i + 2].copy_from_slice(&v.to_le_bytes());
}
fn synthetic_rom() -> Vec<u8> {
    let mut rom = vec![0; 0x300000];
    word(&mut rom, 0x9383c1, 0x9000);
    word(&mut rom, 0x939000, 42);
    for direction in 0..10 {
        word(&mut rom, 0x939002 + 2 * direction, 0xa000);
    }
    word(&mut rom, 0x93a000, 7);
    word(&mut rom, 0x93a002, 0x9876);
    word(&mut rom, 0x93a004, 0x0905);
    word(&mut rom, 0x93a006, 3);
    word(&mut rom, 0x938683, 0xa100);
    rom
}
#[test]
fn initialization_and_instruction_data_are_distinct_stages() {
    let rom = synthetic_rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    projectiles::initialize(&mut ram, &rom, 0).unwrap();
    assert_eq!(ram.word(0xc2c), 42);
    assert_eq!(ram.word(0xbb4), 5);
    assert_eq!(ram.word(0xbc8), 9);
    assert_eq!(ram.word(0xc54), 1);
    projectiles::run_instructions(&mut ram, &rom).unwrap();
    assert_eq!(ram.word(0xcb8), 0x9876);
    assert_eq!(ram.word(0xc54), 7);
    assert_eq!(ram.word(0xc40), 0xa008);
}
#[test]
fn goto_then_delete_clears_bomb_and_preserves_other_slot() {
    let mut rom = synthetic_rom();
    word(&mut rom, 0x93a000, 0x8239);
    word(&mut rom, 0x93a002, 0xa100);
    word(&mut rom, 0x93a100, 0x822f);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xdde, 10);
    ram.set_word(0xc4a, 0xa000);
    ram.set_word(0xc5e, 1);
    ram.set_word(0xcd2, 1);
    ram.set_word(0xb64, 99);
    projectiles::run_instructions(&mut ram, &rom).unwrap();
    assert_eq!(ram.word(0xcd2), 0);
    assert_eq!(ram.word(0xc72), 0xb169);
    assert_eq!(ram.word(0xb64), 99);
}
#[test]
fn cyclic_or_unknown_instructions_reject_without_mutating_state() {
    let mut rom = synthetic_rom();
    word(&mut rom, 0x93a000, 0x8239);
    word(&mut rom, 0x93a002, 0xa000);
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xc40, 0xa000);
        ram.set_word(0xc54, 1);
    }
    let before = bytes.clone();
    assert!(projectiles::run_instructions(&mut Wram::new(&mut bytes).unwrap(), &rom).is_err());
    assert_eq!(bytes, before);
    word(&mut rom, 0x93a000, 0xffff);
    assert!(projectiles::run_instructions(&mut Wram::new(&mut bytes).unwrap(), &rom).is_err());
    assert_eq!(bytes, before);
}
#[test]
fn inherited_upward_speed_uses_low_byte_gate_and_unaligned_word() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xc04, 1);
    ram.set_word(0xdb0, 0x8000);
    ram.set_word(0xdb2, 1);
    ram.set_word(0xdac, 0x4000);
    ram.set_word(0xdae, 2);
    projectiles::initialize_speed(&mut ram, 0, 0x2ab).unwrap();
    assert_eq!(ram.word(0xbdc), 0x4eb);
    assert_eq!(ram.word(0xbf0), 0xc060 - 0x2ab);
    ram.set_word(0xdb2, 0x100);
    projectiles::initialize_speed(&mut ram, 0, 0x2ab).unwrap();
    assert_eq!(ram.word(0xbf0), 0u16.wrapping_sub(0x2ab));
}
#[test]
fn bomb_countdown_changes_fuse_then_explodes() {
    let rom = synthetic_rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xdde, 10);
    ram.set_word(0xc86, 16);
    ram.set_word(0xc4a, 0xa000);
    projectiles::dispatch_rom(&mut ram, &rom, "Bomb_Func2", 0, 0).unwrap();
    assert_eq!(ram.word(0xc86), 15);
    assert_eq!(ram.word(0xc4a), 0xa01c);
    ram.set_word(0xc86, 1);
    projectiles::dispatch_rom(&mut ram, &rom, "Bomb_Func2", 0, 0).unwrap();
    assert_eq!(ram.word(0xc86), 0);
    assert_eq!(ram.word(0xc4a), 0xa100);
    assert_eq!(ram.word(0xc5e), 1);
    assert_eq!(ram.byte(0x666), 8);
}
#[test]
fn cooldown_uses_low_byte_and_projectile_slot_validation_precedes_writes() {
    let rom = synthetic_rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xccc, 256);
    assert_eq!(projectiles::dispatch_rom(&mut ram, &rom, "Samus_CanFireBeam", 0, 0), Ok(1));
    assert_eq!(ram.word(0xcce), 1);
    let before = ram.word(0xcce);
    assert!(projectiles::clear(&mut ram, 19).is_err());
    assert_eq!(ram.word(0xcce), before);
}
#[test]
fn missile_acceleration_wraps_each_signed_component_independently() {
    let mut rom = synthetic_rom();
    word(&mut rom, 0x90c303 + 8, 0x0030);
    word(&mut rom, 0x90c305 + 8, 0xffc0);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xc04, 2);
    ram.set_word(0xc18, 0x100);
    ram.set_word(0xc7c, 0x100);
    ram.set_word(0xbdc, 0xfff0);
    ram.set_word(0xbf0, 0x0020);
    projectiles::missile_accelerate(&mut ram, &rom, 0).unwrap();
    assert_eq!(ram.word(0xbdc), 0x0020);
    assert_eq!(ram.word(0xbf0), 0xffe0);
    assert_eq!(ram.word(0xc7c), 0x100);
}
#[test]
fn super_missile_link_returns_without_mutation_when_slots_are_full() {
    let rom = synthetic_rom();
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut ram = Wram::new(&mut bytes).unwrap();
        for k in (0..10).step_by(2) {
            ram.set_word(0xc2c + k, 7);
        }
    }
    let before = bytes.clone();
    projectiles::missile_link(&mut Wram::new(&mut bytes).unwrap(), &rom).unwrap();
    assert_eq!(bytes, before);
}
#[test]
fn super_missile_link_uses_free_damage_slot_and_updates_parent_link() {
    let mut rom = synthetic_rom();
    word(&mut rom, 0x93842f, 0xa200);
    word(&mut rom, 0x93a200, 200);
    word(&mut rom, 0x93a202, 0xa300);
    let pose = offset(0x91b629 + 8);
    rom[pose + 3] = 2;
    rom[pose + 4] = 5;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_word(0xaf6, 100);
    ram.set_word(0xafa, 50);
    ram.set_word(0xc2c, 20);
    ram.set_word(0xc7c, 0x100);
    ram.set_word(0xcce, 1);
    projectiles::missile_link(&mut ram, &rom).unwrap();
    assert_eq!(ram.word(0xc1a), 0x8200);
    assert_eq!(ram.word(0xc2e), 200);
    assert_eq!(ram.word(0xc42), 0xa300);
    assert_eq!(ram.word(0xc6a), 0xb075);
    assert_eq!(ram.word(0xb66), 111);
    assert_eq!(ram.word(0xb7a), 46);
    assert_eq!(ram.word(0xc7c), 0x102);
    assert_eq!(ram.word(0xcce), 2);
}
