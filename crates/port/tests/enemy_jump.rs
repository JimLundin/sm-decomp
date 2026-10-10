#[path = "../src/enemy_jump.rs"]
mod enemy_jump;
use port::{WRAM_SIZE, Wram, audio, enemy_ai};
struct NoEffects;
impl enemy_jump::Host for NoEffects {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16, _: &mut i32) -> Result<u8, &'static str> {
        Err("unexpected allocation")
    }
}
fn ram(b: &mut [u8]) -> Wram<'_> {
    let mut r = Wram::new(b).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 128);
    r.set_word(0xf82, 8);
    r.set_word(0xf84, 8);
    r
}
fn table_rom() -> Vec<u8> {
    vec![0; 0x120000]
}
#[test]
fn ascent_crosses_apex_and_switches_to_descent() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = ram(&mut b);
    r.set_word(0xf78 + 0x34, 2);
    r.set_word(0xf78 + 0x38, 3);
    r.set_word(0xf78 + 0x36, 3);
    enemy_jump::dispatch_host(&mut r, &table_rom(), &mut NoEffects, "Sidehopper_Func_19", 0, 0).unwrap();
    assert_eq!(r.word(0x7806), 1);
    assert_eq!(r.word(0xf78 + 0x34), 0);
    assert_eq!(r.word(0xf7a), 131);
}
#[test]
fn descent_clamps_speed_and_reverses_on_horizontal_wall() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = ram(&mut b);
    r.set_word(0xf78 + 0x34, 63);
    r.set_word(0xf78 + 0x38, 4);
    r.set_word(0xf78 + 0x36, 3);
    for by in [7, 8] {
        r.set_word(0x10002 + 2 * (16 * by + 8), 0x8000);
    }
    enemy_jump::dispatch_host(&mut r, &table_rom(), &mut NoEffects, "Sidehopper_Func_20", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x34), 64);
    assert_eq!(r.word(0xf78 + 0x36), 65533);
}
#[test]
fn facing_selection_and_animation_signal_preserve_state_order() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = ram(&mut b);
    r.set_word(0xaf6, 96);
    enemy_jump::dispatch_host(&mut r, &[], &mut NoEffects, "Sidehopper_Func_7", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x32), 0xac6c);
    r.set_word(0x7808, 1);
    enemy_jump::dispatch_host(&mut r, &[], &mut NoEffects, "Sidehopper_Func_18", 0, 0).unwrap();
    assert_eq!(r.word(0xf78 + 0x32), 0xabd6);
    assert_eq!(r.word(0x7808), 0);
}
#[test]
fn invalid_quadratic_integrals_and_unknown_states_fail_admission() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = ram(&mut b);
    assert!(enemy_jump::dispatch_host(&mut r, &table_rom(), &mut NoEffects, "Sidehopper_Func_2", 4096, 0).is_err());
    assert!(enemy_jump::dispatch_host(&mut r, &table_rom(), &mut NoEffects, "Sidehopper_Func_2", 4096, 3).is_err());
    r.set_word(0xf78 + 0x32, 0x1234);
    assert!(enemy_jump::dispatch_host(&mut r, &[], &mut NoEffects, "Sidehopper_Main", 0, 0).is_err());
}

#[test]
fn script_signal_keeps_pointer_and_current_enemy_quirk() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = ram(&mut b);
    r.set_word(0xe54, 64);
    assert_eq!(enemy_jump::dispatch_rom(&mut r, &[], "Sidehopper_Instr_1", 0, 0xa3aa6a), Ok(0xa3aa6a));
    assert_eq!(r.word(0x7808 + 64), 1);
    assert_eq!(r.word(0x7808), 0);
    assert!(enemy_jump::dispatch_rom(&mut r, &[], "Sidehopper_Instr_1", 0, 0xa2aa6a).is_err());
}
