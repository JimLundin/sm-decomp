use port::{WRAM_SIZE, Wram, enemy_projectile_collision as c};
#[test]
fn solid_wall_stops_motion_at_projectile_radius() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x1a27, 0x1234);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    assert_eq!(c::horizontal(&mut r, &[], 0).unwrap(), 1);
    assert_eq!(r.word(0x1a4b), 30);
    assert_eq!(r.word(0x1a27), 0);
    assert_eq!(r.word(0xdc4), 18);
}
#[test]
fn extension_chain_resolves_target_and_leaves_final_block_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x10002 + 36, 0x5000);
    r.set_byte(0x16402 + 18, 1);
    r.set_word(0x10002 + 38, 0xd000);
    r.set_byte(0x16402 + 19, 1);
    r.set_word(0x10002 + 70, 0x8000);
    assert_eq!(c::horizontal(&mut r, &[], 0).unwrap(), 1);
    assert_eq!(r.word(0xdc4), 35);
    assert_eq!(r.word(0x1a4b), 30);
}
#[test]
fn floor_slope_uses_global_slot_and_outer_collision_clamp() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0xa1000];
    r.set_word(0x7a5, 16);
    r.set_word(0x1991, 2);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x0300);
    r.set_word(0x1a4b + 2, 24);
    r.set_word(0x1a93 + 2, 200);
    r.set_word(0x10002 + 33 * 2, 0x1000);
    r.set_byte(0x16402 + 33, 5);
    rom[0xa0b2b + 5 * 16 + 8] = 1;
    assert_eq!(c::vertical(&mut r, &rom, 0).unwrap(), 1);
    assert_eq!(r.word(0x1a93), 30);
    assert_eq!(r.word(0x1a93 + 2), 31);
    assert_eq!(r.word(0x1a6f + 2), 0xffff);
    assert_eq!(r.word(0x1a6f), 0);
}
#[test]
fn sweat_splash_and_robot_laser_delete_on_terrain() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x0300);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0xb714, 0).unwrap();
    assert_eq!(r.word(0x1a93), 26);
    assert_eq!(r.word(0x1b47), 0xb61d);
    assert_eq!(r.word(0x1b8f), 1);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    r.set_word(0x19bb, 1600);
    r.set_word(0x1997, 0x9000);
    e::preinstruction(&mut r, &[], 0xd3bf, 0).unwrap();
    assert_eq!(r.word(0x19bb), 0);
    assert_eq!(r.word(0x1997), 0);
}
