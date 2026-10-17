use port::{WRAM_SIZE, Wram, enemy_ai};
struct NoEffects;
impl enemy_ai::Host for NoEffects {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16, _: &mut i32) -> Result<u8, &'static str> {
        Err("unexpected external allocation")
    }
}
fn ram(bytes: &mut [u8]) -> Wram<'_> {
    let mut r = Wram::new(bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r
}
#[test]
fn fractional_enemy_movement_and_wall_snap() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0xf7c, 0xc000);
    assert_eq!(enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Enemy_MoveRight_IgnoreSlopes", 0, 0x8000), Ok(0));
    assert_eq!(r.word(0xf7a), 129);
    assert_eq!(r.word(0xf7c), 0x4000);
    for y in [7, 8] {
        r.set_word(0x10002 + 2 * (16 * y + 8), 0x8000);
    }
    assert_eq!(enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Enemy_MoveRight_IgnoreSlopes", 0, 0x10000), Ok(1));
    assert_eq!(r.word(0xf7a), 129);
    assert_eq!(r.word(0xf7c), 65535);
}
#[test]
fn rio_swoop_transitions_into_hover_with_saved_horizontal_velocity() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0xf78 + 0x34, 24);
    r.set_word(0xf78 + 0x36, 256);
    enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Rio_3", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x34), 0);
    enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Rio_3", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x32), 0xbcff);
    assert_eq!(r.word(0xf78 + 0x30), 256);
    assert_eq!(r.word(0xf78 + 0x36), 0);
    assert_eq!(r.word(0xf78 + 0x34), 0);
}
#[test]
fn waver_left_initialization_preserves_nonstandard_sign_extension() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Waver_Init", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x30), 0x8000);
    assert_eq!(r.word(0xf78 + 0x32), 0xfffe);
    assert_eq!(r.word(0xf78 + 0x1a), 0x86a7);
}
#[test]
fn cyclic_extensions_and_invalid_ai_states_are_rejected() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0x10002 + 2 * 120, 0x5000);
    r.set_byte(0x16402 + 120, 1);
    r.set_word(0x10002 + 2 * 121, 0x5000);
    r.set_byte(0x16402 + 121, 255);
    assert!(enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Enemy_MoveRight_IgnoreSlopes", 0, 65536).is_err());
    r.set_word(0xf78 + 0x32, 0x1234);
    assert!(enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Rio_Main", 0, 0).is_err());
    assert!(enemy_ai::dispatch_host(&mut r, &[], &mut NoEffects, "Unknown", 0, 0).is_err());
}
