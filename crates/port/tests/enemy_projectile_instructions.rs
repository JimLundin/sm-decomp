use port::{WRAM_SIZE, Wram, enemy_projectile_instructions as e};
#[test]
fn fractional_velocity_preserves_low_subpixel_byte() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a27, 0xff37);
    r.set_word(0x1a4b, 10);
    r.set_word(0x1ab7, 0xff80);
    assert_eq!(e::move_x(&mut r, 0).unwrap(), 10);
    assert_eq!(r.word(0x1a27), 0x7f37);
}
#[test]
fn relative_branch_underflows_timer_and_sleep_keeps_opcode_address() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x31000] = 0xfe;
    assert_eq!(e::instruction(&mut r, &rom, 0x81ce, 0x9000, 0).unwrap(), 0x8ffe);
    assert_eq!(r.word(0x19df), 0xffff);
    assert_eq!(e::instruction(&mut r, &rom, 0x8159, 0x9002, 0).unwrap(), 0);
    assert_eq!(r.word(0x1b47), 0x9000);
}
#[test]
fn palette_count_is_inclusive_and_property_clear_uses_mask() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x31000..0x31005].copy_from_slice(&[0, 0x91, 2, 0, 1]);
    rom[0x31100..0x31104].copy_from_slice(&[0x34, 0x12, 0x78, 0x56]);
    assert_eq!(e::instruction(&mut r, &rom, 0x82d5, 0x9000, 0).unwrap(), 0x9005);
    assert_eq!(r.word(0xc002), 0x1234);
    assert_eq!(r.word(0xc004), 0x5678);
    r.set_word(0x1bd7, 0xffff);
    rom[0x31000..0x31002].copy_from_slice(&[0, 0x20]);
    e::instruction(&mut r, &rom, 0x823c, 0x9000, 0).unwrap();
    assert_eq!(r.word(0x1bd7), 0x2000);
}
#[test]
fn frame_runs_deleting_preinstruction_before_script_and_descends_slots() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0x9000);
    r.set_word(0x1a03 + 34, 0xe4fe);
    r.set_word(0x1a4b + 34, 256);
    r.set_word(0x1b8f + 34, 1);
    r.set_word(0x1b47 + 34, 0x9000);
    rom[0x31000..0x31004].copy_from_slice(&[7, 0, 0x34, 0x92]);
    e::run_all(&mut r, &rom, 64).unwrap();
    assert_eq!(r.word(0x1997 + 34), 0);
    assert_eq!(r.word(0x1b8f + 34), 7);
    assert_eq!(r.word(0x1b6b + 34), 0x9234);
    assert_eq!(r.word(0x1991), 0);
}
#[test]
fn shard_spawns_sparkle_into_lower_slot_that_runs_in_same_frame() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    // Pinned initializer and preinstruction, with a short independent sparkle script.
    rom[0x34f0a..0x34f14].copy_from_slice(&[0x6d, 0xce, 0x70, 0x81, 0, 0x91, 0, 0, 0, 0]);
    rom[0x31100..0x31104].copy_from_slice(&[9, 0, 0x34, 0x92]);
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0x9000);
    r.set_word(0x1a03 + 34, 0xce9b);
    r.set_word(0x1a4b + 34, 100);
    r.set_word(0x1a93 + 34, 100);
    r.set_word(0x1b8f + 34, 2);
    // NextRandom(0)=273, whose 0x420 bits are both clear.
    e::run_all(&mut r, &rom, 64).unwrap();
    assert_eq!(r.word(0x1997 + 32), 0xcf0a);
    assert_eq!(r.word(0x19bb + 32), 1600);
    assert_eq!(r.word(0x1b8f + 32), 9);
    assert_eq!(r.word(0x1b6b + 32), 0x9234);
    assert_eq!(r.word(0x1adb + 34), 32);
    assert_eq!(r.word(0x1b8f + 34), 1);
}
#[test]
fn script_cycle_is_rejected_and_disabled_scheduler_preserves_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    r.set_word(0x1991, 12);
    e::run_all(&mut r, &rom, 0).unwrap();
    assert_eq!(r.word(0x1991), 12);
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997, 0x9000);
    r.set_word(0x1a03, 0x8170);
    r.set_word(0x1b8f, 1);
    r.set_word(0x1b47, 0x9000);
    rom[0x31000..0x31004].copy_from_slice(&[0xab, 0x81, 0, 0x90]);
    assert_eq!(e::run_all(&mut r, &rom, 16), Err("enemy projectile instruction budget"));
}
