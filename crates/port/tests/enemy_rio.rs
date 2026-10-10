#[path = "../src/enemy_rio.rs"]
mod enemy_rio;
use port::{WRAM_SIZE, Wram, audio, enemy, enemy_ai};
struct NoEffects;
impl enemy_rio::Host for NoEffects {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16, _: &mut i32) -> Result<u8, &'static str> {
        Err("unexpected allocation")
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
fn rising_transition_preserves_horizontal_velocity_and_sets_audio() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0xf78 + 0x34, 0);
    r.set_word(0xf78 + 0x36, 128);
    enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "LowerNorfairRio_Func_4", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x34), 65535);
    assert_eq!(r.word(0xf78 + 0x36), 128);
    assert_eq!(r.word(0xf78 + 0x3a), 0xc82d);
    assert_eq!(r.word(0x7800), 0xc662);
    assert_eq!(r.word(0xf78 + 0x1c), 1);
}
#[test]
fn paired_child_uses_packed_spawn_stride_and_signed_y_offset() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0xe84 + 200, 1);
    r.set_word(0xe84 + 218, 0);
    r.set_word(0xe84 + 182, 77);
    r.set_word(0xe84 + 186, 90);
    r.set_word(0x7000 + 31 * 64, 0xc18f);
    r.set_word(0x7000 + 31 * 64 + 4, 0xfff4);
    enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "NorfairRio_Func_1", 0, 0).unwrap();
    assert_eq!(r.word(0xf7a), 77);
    assert_eq!(r.word(0xf7e), 78);
    assert_eq!(r.word(0x7800), 0xc1a3);
    assert_eq!(r.word(0xf78 + 0xe) & 0x100, 0);
}
#[test]
fn paired_child_deletes_when_parent_is_missing() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "LowerNorfairRio_Func_1", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0xe), 0x200);
}
#[test]
fn lower_recovery_waits_for_animation_signal() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = ram(&mut bytes);
    r.set_word(0xf78 + 0x3a, 0xc888);
    enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "LowerNorfairRio_Func_6", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x3a), 0xc888);
    r.set_word(0x7802, 1);
    enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "LowerNorfairRio_Func_6", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x3a), 0xc771);
    assert_eq!(r.word(0x7802), 0);
    r.set_word(0xf78 + 0x3a, 0x1234);
    assert!(enemy_rio::dispatch_host(&mut r, &[], &mut NoEffects, "NorfairRio_Main", 0, 0).is_err());
}
