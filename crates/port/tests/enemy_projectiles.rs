use port::{WRAM_SIZE, Wram, enemy_projectiles};
fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x300000];
    let p = 0x30000;
    rom[p..p + 14].copy_from_slice(&[0x9d, 0xb4, 0x11, 0x81, 0x22, 0x82, 8, 9, 0x34, 0x12, 0, 0, 0, 0]);
    rom
}
#[test]
fn highest_free_slot_gets_definition_and_native_wall_position() {
    let mut bytes = vec![0xaa; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997 + 32, 0);
    r.set_word(0x1997 + 30, 0);
    enemy_projectiles::spawn_room(&mut r, &rom(), 0x8000, 7).unwrap();
    assert_eq!(r.word(0x1997 + 32), 0x8000);
    assert_eq!(r.word(0x1993), 7);
    assert_eq!(r.word(0x1a4b + 32), 272);
    assert_eq!(r.word(0x1a93 + 32), 2184);
    assert_eq!(r.word(0x1b47 + 32), 0x8222);
    assert_eq!(r.word(0x1b8f + 32), 1);
    assert_eq!(r.word(0x1ab7 + 32), 0xaaaa);
    assert_eq!(r.word(0x1a27 + 32), 0);
    assert_eq!(r.word(0x1997 + 30), 0);
}
#[test]
fn tube_shard_uses_plm_coordinates_without_overwriting_stale_x_position() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = rom();
    rom[0x30000..0x30002].copy_from_slice(&0xd6c9u16.to_le_bytes());
    rom[0x35760..0x35762].copy_from_slice(&0x9876u16.to_le_bytes());
    r.set_byte(0x7a5, 16);
    r.set_word(0x1c87, 36);
    r.set_word(0x1a4b + 34, 0xabcd);
    enemy_projectiles::spawn_room(&mut r, &rom, 0x8000, 0).unwrap();
    assert_eq!(r.word(0x1b23 + 34), 72);
    assert_eq!(r.word(0x1a93 + 34), 72);
    assert_eq!(r.word(0x1ab7 + 34), 0xfe80);
    assert_eq!(r.word(0x1adb + 34), 320);
    assert_eq!(r.word(0x1a4b + 34), 0xabcd);
}
#[test]
fn full_pool_sets_parameter_without_reading_definition_and_rejections_are_atomic() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    enemy_projectiles::spawn_room(&mut r, &[], 0xffff, 0x1234).unwrap();
    assert_eq!(r.word(0x1993), 0x1234);
    r.set_word(0x1997, 0);
    assert!(enemy_projectiles::spawn_room(&mut r, &rom(), 0x8000, 0).is_ok());
    r.set_word(0x1997, 0);
    let mut bad = rom();
    bad[0x30000..0x30002].copy_from_slice(&0xd6c9u16.to_le_bytes());
    let before: Vec<_> = (0..WRAM_SIZE).map(|p| r.byte(p)).collect();
    assert!(enemy_projectiles::spawn_room(&mut r, &bad, 0x8000, 20).is_err());
    assert_eq!(before, (0..WRAM_SIZE).map(|p| r.byte(p)).collect::<Vec<_>>());
}
#[test]
fn enemy_graphics_combine_palette_and_tiles_from_selected_enemy() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf78 + 64 + 0x1e, 0xa00);
    r.set_word(0xf78 + 64 + 0x20, 0x123);
    r.set_word(0xf78 + 64 + 0x26, 0xffff);
    enemy_projectiles::spawn_with_gfx(&mut r, &rom(), 9, 64, 0x8000).unwrap();
    assert_eq!(r.word(0x19bb + 34), 0xb23);
}
