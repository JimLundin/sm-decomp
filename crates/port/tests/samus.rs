use port::{Wram, WRAM_SIZE};
#[path = "../src/samus.rs"]
mod samus;
fn rom() -> Vec<u8> {
    // Small independent table fixture; routine tests do not require a game ROM.
    let mut image = vec![0; 0x100000];
    let mut write = |address: u32, value: u16| {
        let offset = (((address >> 16) << 15) | (address & 0x7fff)) as usize & 0x3fffff;
        image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    write(0x91b637, 21); // Standing radius.
    write(0x91b837, 7); // Morph-ball radius.
    write(0x91b61f, 1); // Initial speed-booster counter.
    for (n, value) in [0, 0x3000, 2, 0xc000, 0, 0x8000].into_iter().enumerate() {
        write(0x909f61 + n as u32 * 2, value);
    }
    image
}
#[test]
fn pose_radius_sets_bounds_and_preserves_the_bottom_across_pose_changes() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_word(0xafa, 100);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_GetBottom_R18", 0, 0), Ok(120));
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_GetTop_R20", 0, 0), Ok(79));
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_SetRadius", 0, 0), Ok(0));
    assert_eq!(ram.word(0xafe), 5);
    assert_eq!(ram.word(0xb00), 21);
    ram.set_word(0xa20, 1);
    ram.set_word(0xa1c, 0x41);
    ram.set_word(0xb14, 100);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_AlignBottomWithPrevPose", 0, 0), Ok(0));
    assert_eq!(ram.word(0xafa), 114);
    assert_eq!(ram.word(0xb14), 114);
}
#[test]
fn water_reduces_jump_and_acceleration_unless_gravity_suit_is_equipped() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_word(0xafa, 100);
    ram.set_word(0x195e, 110);
    ram.set_word(0x196e, 6);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "SetLiquidPhysicsType", 0, 0), Ok(0));
    assert_eq!(ram.word(0xad2), 1);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_InitJump", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb2e), 1);
    assert_eq!(ram.word(0xb2c), 0xc000);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_DetermineAccel_Y", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb32), 0x800);
    ram.set_word(0x9a2, 0x20);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_InitJump", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb2e), 4);
    assert_eq!(ram.word(0xb2c), 0xe000);
}
#[test]
fn running_builds_momentum_and_boosted_jumps_drop_fractional_carry() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x9a2, 0x2020);
    ram.set_byte(0xa1f, 1);
    ram.set_word(0x9b6, 0x8000);
    ram.set_word(0x8b, 0x8000);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_HandleExtraRunspeedX", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb3c), 1);
    assert_eq!(ram.word(0xb44), 0x1000);
    assert_eq!(ram.word(0xad0), 1);
    ram.set_word(0xb42, 3);
    ram.set_word(0xb44, 0xf000);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_InitJump", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb2c), 0xd000);
    assert_eq!(ram.word(0xb2e), 5);
}
#[test]
fn acceleration_clamps_at_run_speed_and_deceleration_stops_at_zero() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xb46, 2);
    ram.set_word(0xb48, 0xf000);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_CalcBaseSpeed_X", 0x9f61, 0), Ok(0x2c000));
    ram.set_word(0xb4a, 1);
    ram.set_word(0xb46, 0);
    ram.set_word(0xb48, 1);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_CalcBaseSpeed_X", 0x9f61, 0), Ok(0));
    assert_eq!(ram.word(0xb4a), 0);
}
#[test]
fn displacement_clamps_integer_speed_but_keeps_fraction_and_position_wraps() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_CalcDisplacementMoveRight", 0x00208000, 0), Ok(0x000f8000));
    assert_eq!(ram.word(0xdbc), 32);
    assert_eq!(ram.word(0xdbe), 0x8000);
    ram.set_word(0xaf6, 0xffff);
    ram.set_word(0xaf8, 0x8000);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_MoveRight_NoColl", 0x18000, 0), Ok(0));
    assert_eq!(ram.word(0xaf6), 1);
    assert_eq!(ram.word(0xaf8), 0);
    assert_eq!(ram.word(0xdae), 1);
    assert_eq!(ram.word(0xdb0), 0x8000);
}
#[test]
fn falling_transition_and_echo_cadence_update_the_observable_state() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xb36, 1);
    ram.set_word(0xb2e, 0xffff);
    ram.set_word(0xb2c, 1);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_CheckStartFalling", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb36), 2);
    assert_eq!(ram.word(0xb2c), 0);
    ram.set_word(0xa1c, 0x29);
    ram.set_word(0xb2e, 4);
    ram.set_word(0xb2c, 0xf000);
    ram.set_word(0xb32, 0x1c00);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_Func9", 0, 0), Ok(0));
    assert_eq!(ram.word(0xa94), 16);
    assert_eq!(ram.word(0xa96), 4);
    ram.set_word(0xb3e, 0x400);
    ram.set_word(0xaf6, 111);
    ram.set_word(0xafa, 222);
    assert_eq!(samus::dispatch_rom(&mut ram, &rom, "Samus_UpdateSpeedEchoPos", 0, 0), Ok(0));
    assert_eq!(ram.word(0xab0), 111);
    assert_eq!(ram.word(0xab8), 222);
    assert_eq!(ram.word(0xaae), 2);
}
