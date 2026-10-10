use port::{WRAM_SIZE, Wram, plm};
#[test]
fn quicksand_clamps_fractional_displacement_and_tracks_contact() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xb02, 3);
    let mut amount = 0x100aa;
    assert_eq!(plm::setup(&mut ram, 0xb4c4, 78, &mut amount), Ok(0));
    assert_eq!(amount, 0x30aa);
    assert_eq!(ram.word(0x1e71), 1);
    ram.set_word(0xa6e, 1);
    assert_eq!(plm::setup(&mut ram, 0xb4c4, 78, &mut amount), Ok(1));
    assert_eq!(amount, 0);
}
#[test]
fn native_spawn_uses_highest_free_slot_and_rejects_unknown_before_mutation() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x300000];
    let off = (4 << 15) + 0x372b;
    rom[off..off + 4].copy_from_slice(&[0xc4, 0xb4, 0xe3, 0xaa]);
    ram.set_word(0xdc4, 16);
    ram.set_word(0xb02, 3);
    let mut amount = 0x10000;
    assert_eq!(plm::spawn(&mut ram, &rom, 0xb72b, &mut amount), Ok(0));
    assert_eq!(amount, 0x3000);
    assert_eq!(ram.word(0x1c37 + 78), 0xb72b);
    assert_eq!(ram.word(0x1c87 + 78), 32);
    assert_eq!(ram.word(0x1d27 + 78), 0xaae3);
    let before: Vec<_> = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect();
    rom[off..off + 2].copy_from_slice(&[0xff, 0xff]);
    assert!(plm::spawn(&mut ram, &rom, 0xb72b, &mut amount).is_err());
    assert_eq!(before, (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>());
}
#[test]
fn occupied_pool_does_not_read_unknown_header_or_change_displacement() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for j in (0..80).step_by(2) {
        ram.set_word(0x1c37 + j, 0xffff);
    }
    let mut amount = -1;
    assert_eq!(plm::spawn(&mut ram, &[], 0, &mut amount), Ok(0));
    assert_eq!(amount, -1);
}
