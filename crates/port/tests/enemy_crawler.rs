use port::{WRAM_SIZE, Wram, enemy_ai, enemy_crawler};
struct Host;
impl enemy_crawler::Host for Host {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16, _: &mut i32) -> Result<u8, &'static str> {
        Err("unexpected PLM")
    }
}
#[test]
fn gravity_carries_fraction_into_integer() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r.set_word(0x7802, 0xc000);
    r.set_word(0x7804, 2);
    enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "FireZoomer_Func_2", 0, 0).unwrap();
    assert_eq!(r.word(0x7802), 0x4000);
    assert_eq!(r.word(0x7804), 3);
    assert_eq!(r.word(0xf7e), 130);
    assert_eq!(r.word(0xf80), 0xc000);
}
#[test]
fn landing_restores_saved_orientation_and_sentinel_speed() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r.set_word(0xf78 + 0x3c, 255);
    r.set_word(0x7802, 0xffff);
    r.set_word(0x7804, 1);
    r.set_word(0x7806, 0xe7f2);
    r.set_word(0x7808, 4);
    for i in [135, 136] {
        r.set_word(0x10002 + 2 * i, 0x8000);
    }
    enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "FireZoomer_Func_2", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x3a), 0xe7f2);
    assert_eq!(r.word(0xf78 + 0x30), 128);
    assert_eq!(r.word(0xf78 + 0x32), 128);
    assert_eq!(r.word(0x7802), 0);
    assert_eq!(r.word(0x7804), 0);
    assert_eq!(r.word(0x7808), 0);
}
#[test]
fn sentinel_keeps_existing_velocities_and_orients_them() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf78 + 0x3c, 255);
    r.set_word(0xf78 + 0x30, 256);
    r.set_word(0xf78 + 0x32, 128);
    enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "StoneZoomer_E67A", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x30), 0xff00);
    assert_eq!(r.word(0xf78 + 0x32), 128);
    assert_eq!(r.word(0xf78 + 0x16), 0x804d);
    assert_eq!(r.word(0xf78 + 0x3a), 0xe6c1);
}
#[test]
fn malformed_dispatch_and_indexes_are_rejected() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf78 + 0x3a, 0xffff);
    assert!(enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "StoneZoomer_Main", 0, 0).is_err());
    assert!(enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "StoneZoomer_E67A", 1, 0).is_err());
    r.set_word(0xf78 + 0x3c, 32);
    assert!(enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "StoneZoomer_E67A", 0, 0).is_err());
}
#[test]
fn earthquake_records_orientation_before_fall_and_fourth_miss_starts_fall() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x180000];
    let table = ((0xa3 & 127) * 0x8000) + (0xe630 & 0x7fff);
    rom[table] = 0x34;
    rom[table + 1] = 0x12;
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r.set_word(0xf78 + 0x30, 128);
    r.set_word(0xf78 + 0x32, 128);
    r.set_word(0xf78 + 0x3a, 0xe6c8);
    r.set_word(0x1840, 30);
    r.set_word(0x183e, 20);
    enemy_crawler::dispatch_host(&mut r, &rom, &mut Host, "FireZoomer_Func_1", 0, 0).unwrap();
    assert_eq!(r.word(0x7806), 0xe6c8);
    assert_eq!(r.word(0xf78 + 0x3a), 0xe785);
    assert_eq!(r.word(0xf78 + 0x32), 0xff80);
    assert_eq!(r.word(0xf78 + 0x1a), 0x1234);
    r.set_word(0x1840, 0);
    r.set_word(0xf78 + 0x3a, 0xe6c8);
    r.set_word(0x7808, 3);
    enemy_crawler::dispatch_host(&mut r, &rom, &mut Host, "FireZoomer_Func_1", 0, 0).unwrap();
    assert_eq!(r.word(0x7808), 4);
    assert_eq!(r.word(0xf78 + 0x3a), 0xe785);
    assert_eq!(r.word(0x7806), 0xe6c8);
}
#[test]
fn horizontal_slope_scales_each_contact_before_displacement() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x180000];
    let table = (0xa0 & 127) * 0x8000 + (0xc49f & 0x7fff) + 5 * 4 + 2;
    rom[table] = 128;
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    for i in [120, 136] {
        r.set_word(0x10002 + 2 * i, 0x1000);
        r.set_byte(0x16402 + i, 5);
    }
    enemy_ai::dispatch_host(&mut r, &rom, &mut Host, "Enemy_MoveRight_ProcessSlopes", 0, 65536).unwrap();
    assert_eq!(r.word(0xf7a), 128);
    assert_eq!(r.word(0xf7c), 0x4000);
    assert_eq!(r.word(0x1e77), 5);
}
#[test]
fn instruction_preinstruction_aliases_crawler_state() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x180000];
    let p = 0x23 * 32768 + 0x6600;
    rom[p] = 0xf2;
    rom[p + 1] = 0xe7;
    assert_eq!(
        enemy_crawler::dispatch_host(&mut r, &rom, &mut Host, "Zoomer_Instr_SetPreinstr", 0, 0xa3e600).unwrap(),
        0xa3e602
    );
    assert_eq!(r.word(0xf78 + 0x3a), 0xe7f2);
    assert_eq!(r.word(0xf78 + 0x12), 0);
}
#[test]
fn every_property_orientation_has_independent_expected_speed() {
    for (property, x, y) in [(0, 0xff80, 128), (1, 128, 128), (2, 128, 0xff80), (3, 128, 128)] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0xf78 + 0xe, property);
        r.set_word(0xf78 + 0x3c, 255);
        r.set_word(0xf78 + 0x30, 128);
        r.set_word(0xf78 + 0x32, 128);
        enemy_crawler::dispatch_host(&mut r, &[], &mut Host, "StoneZoomer_E67A", 0, 0).unwrap();
        assert_eq!((r.word(0xf78 + 0x30), r.word(0xf78 + 0x32)), (x, y));
    }
}
#[test]
fn corner_contact_reverses_primary_speed_and_selects_floor_script() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x180000];
    let p = 0x23 * 32768 + 0x663c;
    rom[p] = 0x78;
    rom[p + 1] = 0x56;
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r.set_word(0xf78 + 0x30, 128);
    r.set_word(0xf78 + 0x32, 128);
    r.set_word(0x7808, 3);
    for i in [120, 136, 135] {
        r.set_word(0x10002 + 2 * i, 0x8000);
    }
    enemy_crawler::dispatch_host(&mut r, &rom, &mut Host, "FireZoomer_Func_1", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x30), 0xff80);
    assert_eq!(r.word(0xf78 + 0x32), 128);
    assert_eq!(r.word(0x7808), 0);
    assert_eq!(r.word(0xf78 + 0x1a), 0x5678);
    assert_eq!(r.word(0xf78 + 0x1c), 1);
}
