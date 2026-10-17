use port::{Wram, enemy};

#[test]
fn enemy_distances_keep_record_offsets_and_signed_threshold_quirk() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xaf6, 10);
    ram.set_word(0xf78 + 0x40 + 2, 0xfffa);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "GetSamusEnemyDelta_X", 0x40, 0), Ok(16));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "IsSamusWithinEnemy_X", 0x40, 17), Ok(1));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CompareDistToSamus_X", 0x40, 16), Ok(1));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "IsSamusWithinEnemy_X", 0x40, 16), Ok(0));
    assert!(enemy::dispatch_rom(&mut ram, &[], "GetSamusEnemyDelta_X", 3, 0).is_err());
}

#[test]
fn screen_geometry_uses_signed_wrapping_and_reports_offscreen() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xf78 + 2, 256);
    ram.set_word(0xf78 + 6, 128);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CheckIfEnemyIsOnScreen", 0, 0), Ok(0));
    ram.set_word(0xf78 + 2, 257);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CheckIfEnemyIsOnScreen", 0, 0), Ok(1));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_ADA3", 1, 0), Ok(0));
    ram.set_word(0xf78 + 2, 0xffff);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "IsEnemyLeavingScreen", 0, 0), Ok(1));
    ram.set_word(0xf78 + 2, 0);
    ram.set_word(0xaf6, 0);
    ram.set_word(0xafa, 96);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "DetermineDirectionOfSamusFromEnemy", 0, 0), Ok(0));
    ram.set_word(0xaf6, 32);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "DetermineDirectionOfSamusFromEnemy", 0, 0), Ok(1));
}

#[test]
fn enemy_touch_checks_above_contact_and_forgiving_horizontal_radius() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in
        [(0xaf6, 100), (0xafa, 80), (0xafe, 8), (0xb00, 8), (0xf7a, 100), (0xf7e, 100), (0xf82, 4), (0xf84, 9)]
    {
        ram.set_word(offset, value);
    }
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CheckIfEnemyTouchesSamus", 0, 0), Ok(65535));
    ram.set_word(0xafa, 100);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CheckIfEnemyTouchesSamus", 0, 0), Ok(0));
    ram.set_word(0xaf6, 115);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_AC67", 0, 0), Ok(65535));
    ram.set_word(0xaf6, 116);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_AC67", 0, 0), Ok(0));
}

#[test]
fn angles_preserve_snes_byte_divisor_and_zero_vector_behavior() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (x, y, result) in [
        (1, 0, 64),
        (0, 1, 128),
        (65535, 0, 192),
        (0, 65535, 0),
        (0, 0, 129),
        (256, 1, 63),
        (1, 256, 129),
        (1, 1, 96),
        (65535, 65535, 224),
    ] {
        assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CalculateAngleFromXY", x, y), Ok(result));
    }
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "SignExtend8", 0x1280, 0), Ok(0xff80));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "SignExtend8", 0x127f, 0), Ok(0x127f));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "Abs16", 0x8000, 0), Ok(0x8000));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "Mult32", 0xffff, 0), Ok(0xffe0));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "SubtractThenAbs16", 0xffff, 0), Ok(1));
}

#[test]
fn sine_negative_fraction_negates_words_independently() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x400000];
    rom[0x103143 + 64] = 255;
    assert_eq!(enemy::dispatch_rom(&mut ram, &rom, "SineMult8bitFull", 64, 0x0101), Ok(0x00000100));
    assert_eq!(enemy::dispatch_rom(&mut ram, &rom, "CosineMult8bitFull", 0, 0x0101), Ok(0x0000ff00));
    assert_eq!(enemy::dispatch_rom(&mut ram, &rom, "SineMult8bit", 64, 0x0101), Ok(0));
    assert!(enemy::dispatch_rom(&mut ram, &[], "SineMult8bit", 64, 1).is_err());
}

#[test]
fn frozen_ai_keeps_original_handler_bits_as_thaw_timer() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xf78 + 0x12, 0x15);
    ram.set_word(0xf78 + 0x26, 2);
    ram.set_word(0xf78 + 0x24, 9);
    ram.set_word(0x9a6, 2);
    enemy::dispatch_rom(&mut ram, &[], "NormalEnemyFrozenAI", 0, 0).unwrap();
    assert_eq!(ram.word(0xf78 + 0x26), 1);
    assert_eq!(ram.word(0xf78 + 0x24), 0);
    enemy::dispatch_rom(&mut ram, &[], "NormalEnemyFrozenAI", 0, 0).unwrap();
    assert_eq!(ram.word(0xf78 + 0x26), 0);
    assert_eq!(ram.word(0xf78 + 0x12), 0x15);
    enemy::dispatch_rom(&mut ram, &[], "NormalEnemyFrozenAI", 0, 0).unwrap();
    assert_eq!(ram.word(0xf78 + 0x26), 0x11);
    assert_eq!(ram.word(0xf78 + 0x12), 0x11);
    enemy::dispatch_rom(&mut ram, &[], "SwitchEnemyAiToMainAi", 0, 0).unwrap();
    for field in [0x12, 0x26, 0x28, 0x2a] {
        assert_eq!(ram.word(0xf78 + field), 0);
    }
}

#[test]
fn samus_enemy_timers_and_previous_positions_follow_c_clamp() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in
        [(0x18a8, 0), (0x18aa, 1), (0x18ac, 65535), (0xaf6, 100), (0xafa, 100), (0xb10, 80), (0xb14, 120)]
    {
        ram.set_word(offset, value);
    }
    enemy::dispatch_rom(&mut ram, &[], "DecrementSamusTimers", 0, 0).unwrap();
    assert_eq!(ram.word(0x18a8), 0);
    assert_eq!(ram.word(0x18aa), 0);
    assert_eq!(ram.word(0x18ac), 65534);
    assert_eq!(ram.word(0x17ac), 65535);
    assert_eq!(ram.word(0x17ec), 65535);
    enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_B7A1", 0, 0).unwrap();
    assert_eq!(ram.word(0xb10), 112);
    assert_eq!(ram.word(0xb14), 88);
}

#[test]
fn enemy_block_probe_scans_full_height_with_fixed_point_motion() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in [(0x7a5, 16), (0xf7a, 32), (0xf7e, 32), (0xf82, 8), (0xf84, 8)] {
        ram.set_word(offset, value);
    }
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_BBBF", 0, 65536), Ok(0));
    ram.set_word(0x10002 + 2 * 34, 0x8000);
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_BBBF", 0, 65536), Ok(1));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_BBBF", 0, -65536), Ok(0));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_BC76", 0, 65536), Ok(1));
    assert_eq!(enemy::dispatch_rom(&mut ram, &[], "CalculateBlockContainingPixelPos", 0x1020, 0x1030), Ok(0));
    assert_eq!(ram.word(0xdc4), 306);
}

#[test]
fn vertical_probe_rejects_unwrapped_c_index_outside_level_allocation() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in [(0x7a5, 255), (0xf7a, 65520), (0xf7e, 4080), (0xf82, 1), (0xf84, 1)] {
        ram.set_word(offset, value);
    }
    assert!(enemy::dispatch_rom(&mut ram, &[], "EnemyFunc_BC76", 0, 0).is_err());
}
