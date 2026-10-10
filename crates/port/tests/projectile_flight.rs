use port::{WRAM_SIZE, Wram, audio};
#[path = "../src/projectile_flight.rs"]
mod flight;
struct NoPlm;
impl flight::Host for NoPlm {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16) -> Result<u8, &'static str> {
        Err("unexpected PLM")
    }
}
fn setup(r: &mut Wram<'_>) {
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0x7a9, 1);
    r.set_word(0x7ab, 1);
    r.set_word(0x7b9, 512);
    r.set_word(0xb64, 100);
    r.set_word(0xb78, 100);
    r.set_word(0xbb4, 4);
    r.set_word(0xbc8, 4);
    r.set_word(0xcce, 1);
}
#[test]
fn signed_fixed_point_flight_preserves_fraction() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xbdc, 0xff80);
    assert_eq!(flight::collide(&mut r, &[], &mut NoPlm, 0, false, false, false), Ok(0));
    assert_eq!(r.word(0xb64), 99);
    assert_eq!(r.word(0xb8c), 0x8000);
}
#[test]
fn wave_reacts_to_solid_without_exploding() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xbdc, 0x100);
    for i in 0..256 {
        r.set_word(0x10002 + 2 * i, 0x8000);
    }
    assert_eq!(flight::collide(&mut r, &[], &mut NoPlm, 0, false, true, false), Ok(0));
    assert_eq!(r.word(0xb64), 101);
    assert_eq!(r.word(0xcce), 1);
}
#[test]
fn room_and_slot_reject_before_movement() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xbdc, 0x100);
    assert!(flight::collide(&mut r, &[], &mut NoPlm, 19, false, false, false).is_err());
    assert_eq!(r.word(0xb64), 100);
    r.set_word(0x7a5, 0);
    assert!(flight::collide(&mut r, &[], &mut NoPlm, 0, false, false, false).is_err());
    assert_eq!(r.word(0xb64), 100);
}

#[test]
fn dispatch_rejects_unknown_symbols() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    assert!(flight::dispatch_host(&mut r, &[], &mut NoPlm, "unknown", 0, 0).is_err());
}
fn roff(a: u32) -> usize {
    (((a >> 16) & 127) * 0x8000 + (a & 0x7fff)) as usize
}
fn rw(rom: &mut [u8], a: u32, v: u16) {
    let p = roff(a);
    rom[p..p + 2].copy_from_slice(&v.to_le_bytes());
}
#[test]
fn mode7_trail_rotation_and_signed_offsets_are_native() {
    let mut rom = vec![0; 0x300000];
    rw(&mut rom, 0x938000, 1);
    rw(&mut rom, 0x938006, 0);
    rw(&mut rom, 0x9ba4b3, 0x9000);
    rw(&mut rom, 0x9b9004, 0xa000);
    let i = roff(0x9ba000);
    rom[i..i + 4].copy_from_slice(&[252, 2, 6, 254]);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xc04, 2);
    r.set_word(0xc40, 0x8000);
    r.set_word(0xc54, 1);
    r.set_word(0x93f, 0x8000);
    r.set_word(0x80, 128);
    r.set_word(0x82, 128);
    r.set_word(0x7a, 0x100);
    r.set_word(0x7c, 0xff00);
    flight::place_trail(&mut r, &rom, 0, 34).unwrap();
    assert_eq!(r.word(0xd730 + 34), 148);
    assert_eq!(r.word(0xd778 + 34), 98);
    assert_eq!(r.word(0xd754 + 34), 158);
    assert_eq!(r.word(0xd79c + 34), 94);
}
#[test]
fn cyclic_tile_extensions_fail_before_any_host_callback() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xbdc, 0x100);
    r.set_word(0x10002 + 102 * 2, 0x5000);
    r.set_byte(0x16402 + 102, 1);
    r.set_word(0x10002 + 103 * 2, 0x5000);
    r.set_byte(0x16402 + 103, 255);
    assert_eq!(flight::collide(&mut r, &[], &mut NoPlm, 0, false, true, false), Err("cyclic shot tile extension"));
}
#[test]
fn no_amount_quicksand_setup_rejects_before_callback() {
    let mut rom = vec![0; 0x300000];
    rw(&mut rom, 0x949ea6, 0xb000);
    rw(&mut rom, 0x84b000, 0xb4c4);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xb02, 2);
    r.set_word(0xbdc, 0x100);
    r.set_word(0x10002 + 102 * 2, 0xc000);
    assert_eq!(
        flight::collide(&mut r, &rom, &mut NoPlm, 0, false, false, false),
        Err("quicksand setup requires actual movement context")
    );
}
#[test]
fn firing_uses_actual_initial_left_direction_minus_one_speed_quirk() {
    let mut rom = vec![0; 0x300000];
    let pose = roff(0x91b629 + 8);
    rom[pose + 3] = 7;
    rw(&mut rom, 0x9383c1, 0x9000);
    rw(&mut rom, 0x939000, 20);
    rw(&mut rom, 0x939010, 0xa000);
    rw(&mut rom, 0x93a004, 0x0404);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0xa1c, 1);
    r.set_word(0xaf6, 100);
    r.set_word(0xafa, 100);
    r.set_word(0xcce, 0);
    r.set_word(0xc2c, 0);
    r.set_word(0x9b2, 0x40);
    r.set_word(0x8f, 0x40);
    flight::fire_beam(&mut r, &rom, &mut NoPlm, false, false).unwrap();
    assert_eq!(r.word(0xb64), 88);
    assert_eq!(r.word(0xb8c), 0);
    assert_eq!(r.word(0xbdc), 0xfc00);
    assert_eq!(r.word(0xcce), 1);
}

#[test]
fn unloaded_room_zero_extent_preserves_reference_out_of_room_reaction() {
    let mut rom = vec![0; 0x300000];
    rw(&mut rom, 0x93867f, 0xa000);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    setup(&mut r);
    r.set_word(0x7b9, 0);
    r.set_word(0xc18, 0x100);
    r.set_word(0xc04, 7);
    r.set_word(0xbdc, 0xff00);
    assert_eq!(flight::collide(&mut r, &rom, &mut NoPlm, 0, false, false, true), Ok(1));
    assert_eq!(r.word(0xc18), 0x800);
    assert_eq!(r.word(0xc2c), 8);
}
