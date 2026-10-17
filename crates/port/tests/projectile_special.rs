use port::{WRAM_SIZE, Wram, projectile_special as special};
struct NoPlm;
impl special::Host for NoPlm {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16) -> Result<u8, &'static str> {
        Err("unexpected PLM")
    }
}
#[test]
fn sine_cardinal_points_use_exact_byte_products() {
    assert_eq!(special::sine(0, 32), Ok((0, 65504)));
    assert_eq!(special::sine(64, 32), Ok((32, 0)));
    assert_eq!(special::sine(128, 32), Ok((0, 32)));
    assert_eq!(special::sine(192, 32), Ok((65504, 0)));
    assert_eq!(special::sine(64, 256), Ok((0, 0)));
}
#[test]
fn plasma_phases_expand_contract_and_escape() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xbdc, 188);
    special::dispatch_host(&mut r, &[], &mut NoPlm, "ProjPreInstr_PlasmaSbaFunc_0", 0, 0).unwrap();
    assert_eq!(r.word(0xbdc), 192);
    assert_eq!(r.word(0xbf0), 1);
    r.set_word(0xbdc, 44);
    special::dispatch_host(&mut r, &[], &mut NoPlm, "ProjPreInstr_PlasmaSbaFunc_1", 0, 0).unwrap();
    assert_eq!(r.word(0xbdc), 40);
    assert_eq!(r.word(0xbf0), 2);
}
#[test]
fn bomb_fuse_does_not_react_before_explosion() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0x7a9, 1);
    r.set_word(0x7ab, 1);
    r.set_word(0xb6e, 100);
    r.set_word(0xb82, 100);
    r.set_word(0xc22, 0x500);
    r.set_word(0xc86, 5);
    special::bomb_contact(&mut r, &[], &mut NoPlm, 10).unwrap();
    assert_eq!(r.word(0xc22), 0x500);
    r.set_word(0xc86, 0);
    special::bomb_contact(&mut r, &[], &mut NoPlm, 10).unwrap();
    assert_eq!(r.word(0xc22), 0x501);
}
#[test]
fn spazer_orbit_switches_at_half_turn_and_pairs_cleanup_slots() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xc7c, 124);
    r.set_word(0xbf0, 4);
    r.set_word(0xafa, 128);
    special::dispatch_host(&mut r, &[], &mut NoPlm, "ProjPreInstr_SpazerSba_FuncB_0", 0, 10).unwrap();
    assert_eq!(r.word(0xc7c), 128);
    assert_eq!(r.word(0xbdc), 160);
    assert_eq!(r.word(0xbf0), 2);
    assert_eq!(r.word(0xca4), 2);
    assert_eq!(r.word(0xb78), 138);
    r.set_word(0xcce, 4);
    r.set_word(0xb64, 100);
    r.set_word(0xb68, 100);
    special::dispatch_host(&mut r, &[], &mut NoPlm, "ProjPreInstr_SpazerSba_FuncA_0", 0, 0).unwrap();
    assert_eq!(r.word(0xcce), 2);
    assert_eq!(r.word(0xb64), 0);
    assert_eq!(r.word(0xb68), 0);
}
#[test]
fn malformed_special_indices_reject_before_clear() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xcce, 4);
    assert!(special::dispatch_host(&mut r, &[], &mut NoPlm, "ProjPreInstr_SpazerSba", 8, 0).is_err());
    assert_eq!(r.word(0xcce), 4);
    r.set_word(0xbf0, 3);
    assert!(special::plasma(&mut r, &[], 0).is_err());
    assert_eq!(r.word(0xccc), 0);
}
#[test]
fn wave_acceleration_preserves_fraction_and_signed_terminal_speed() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xc90, 2);
    r.set_word(0xbf0, 100);
    r.set_word(0xb64, 200);
    r.set_word(0xb78, 200);
    r.set_word(0xaf6, 100);
    r.set_word(0xafa, 100);
    r.set_word(0xbdc, 0xffc0);
    r.set_word(0xc7c, 0xffc0);
    special::wave(&mut r, &[], 0).unwrap();
    assert_eq!(r.word(0xbdc), 0xff80);
    assert_eq!(r.word(0xb64), 199);
    assert_eq!(r.word(0xb8c), 0x8000);
    assert_eq!(r.word(0xc7c), 0xff80);
    assert_eq!(r.word(0xbf0), 99);
}
struct CountPlm(Vec<(u16, u16)>);
impl special::Host for CountPlm {
    fn spawn_plm(&mut self, r: &mut Wram<'_>, h: u16) -> Result<u8, &'static str> {
        self.0.push((r.word(0xdc4), h));
        Ok(0)
    }
}
#[test]
fn bomb_explosion_visits_five_tiles_once_in_reference_order() {
    let mut rom = vec![0; 0x300000];
    fn set(rom: &mut [u8], a: u32, v: u16) {
        let i = (((a >> 16) & 127) * 32768 + (a & 32767)) as usize;
        rom[i..i + 2].copy_from_slice(&v.to_le_bytes());
    }
    set(&mut rom, 0x949ea6, 0xb000);
    set(&mut rom, 0x84b000, 0xb3cf);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0x7a9, 1);
    r.set_word(0x7ab, 1);
    r.set_word(0xb6e, 100);
    r.set_word(0xb82, 100);
    r.set_word(0xc22, 0x500);
    for i in [102, 86, 103, 101, 118] {
        r.set_word(0x10002 + 2 * i, 0xc000);
    }
    let mut host = CountPlm(vec![]);
    special::bomb_contact(&mut r, &rom, &mut host, 10).unwrap();
    assert_eq!(host.0, vec![(102, 0xb000), (86, 0xb000), (103, 0xb000), (101, 0xb000), (118, 0xb000)]);
    special::bomb_contact(&mut r, &rom, &mut host, 10).unwrap();
    assert_eq!(host.0.len(), 5);
}
