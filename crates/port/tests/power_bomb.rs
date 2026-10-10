use port::{WRAM_SIZE, Wram, power_bomb};
struct Host(Vec<u16>);
impl port::projectile_special::Host for Host {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16) -> Result<u8, &'static str> {
        Err("unexpected PLM")
    }
}
impl power_bomb::Host for Host {
    fn spawn_hdma(&mut self, _: &mut Wram<'_>, script: u16) -> Result<u16, &'static str> {
        self.0.push(script);
        Ok(0xffff)
    }
}
#[test]
fn activation_allocates_two_channels_in_order_even_when_full() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xdde, 10);
    r.set_word(0xc86, 1);
    r.set_word(0xb6e, 100);
    r.set_word(0xb82, 120);
    let mut h = Host(vec![]);
    power_bomb::dispatch_host(&mut r, &[], &mut h, "PowerBomb_Func3", 0, 0).unwrap();
    assert_eq!(h.0, vec![0x8ace, 0x8b80]);
    assert_eq!(r.word(0xc86), 65535);
    assert_eq!(r.word(0xce2), 100);
    assert_eq!(r.word(0xce4), 120);
    assert_eq!(r.word(0x592), 0x8000);
}
#[test]
fn radius_walks_clipped_perimeter_including_repeated_corners() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0xce2, 100);
    r.set_word(0xce4, 100);
    r.set_word(0xcea, 0x2000);
    let mut h = Host(vec![]);
    power_bomb::dispatch_host(&mut r, &[], &mut h, "BlockFunc_A06A", 10, 0).unwrap();
    assert_eq!(r.word(0xdc4), 8 + 7 * 16);
}
#[test]
fn malformed_radius_is_rejected_before_any_tile_callback() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a5, 0);
    r.set_word(0x7a7, 16);
    let mut h = Host(vec![]);
    assert!(power_bomb::dispatch_host(&mut r, &[], &mut h, "BlockFunc_A06A", 10, 0).is_err());
    assert!(h.0.is_empty());
    assert!(power_bomb::dispatch_host(&mut r, &[], &mut h, "ProjPreInstr_PowerBomb", 11, 0).is_err());
}
#[test]
fn cleanup_still_performs_contact_with_the_cleared_type() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0xdde, 10);
    r.set_word(0xc22, 0x300);
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0x7a9, 1);
    r.set_word(0x7ab, 1);
    let mut h = Host(vec![]);
    power_bomb::dispatch_host(&mut r, &[], &mut h, "ProjPreInstr_PowerBomb", 10, 0).unwrap();
    assert_eq!(r.word(0xc22), 0);
    assert!(h.0.is_empty());
}
#[test]
fn wide_slot_arguments_reject_without_aliasing_state_or_callbacks() {
    for symbol in ["BlockFunc_A06A", "BombOrPowerBomb_Func1", "ProjPreInstr_PowerBomb"] {
        for argument in [65536, -65536] {
            let mut bytes = vec![0; WRAM_SIZE];
            let before = bytes.clone();
            let mut host = Host(vec![]);
            let result =
                power_bomb::dispatch_host(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, symbol, argument, 0);
            assert!(result.is_err(), "{symbol} accepted {argument}");
            assert_eq!(bytes, before, "{symbol} mutated state for {argument}");
            assert!(host.0.is_empty());
        }
    }
}
