use port::{WRAM_SIZE, Wram, animated_tiles};

#[test]
fn palette_frame_skips_colors_and_waits_at_the_next_duration() {
    let mut rom = vec![0; 0x70000];
    // Set color byte index 4, duration 3, one color, skip two, another color, wait.
    let words: [u16; 9] = [0xc655, 4, 3, 0x1234, 0xc599, 0x5678, 0xc595, 7, 0xc5cf];
    for (i, word) in words.iter().enumerate() {
        rom[0x68000 + i * 2..0x68002 + i * 2].copy_from_slice(&word.to_le_bytes());
    }
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1e7b, 14);
    r.set_word(0x1ead + 14, 0xc526);
    r.set_word(0x1ecd + 14, 1);
    r.set_word(0x1ebd + 14, 0x8000);
    animated_tiles::process_palette(&mut r, &rom, 14).unwrap();
    assert_eq!(r.word(0xc004), 0x1234);
    assert_eq!(r.word(0xc00a), 0x5678);
    assert_eq!(r.word(0xc006), 0);
    assert_eq!(r.word(0x1ecd + 14), 3);
    assert_eq!(r.word(0x1ebd + 14), 0x800e);
    animated_tiles::process_palette(&mut r, &[], 14).unwrap();
    assert_eq!(r.word(0x1ecd + 14), 2);
}

#[test]
fn palette_instruction_pair_preserves_color_cursor_and_timer_high_byte() {
    let mut rom = vec![0; 0x70000];
    rom[0x68000] = 0xfe;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1edd + 14, 0xab00);
    assert_eq!(animated_tiles::palette_instruction(&mut r, &rom, 0xc648, 14, 0x8000).unwrap(), (14, 0x8001));
    assert_eq!(r.word(0x1edd + 14), 0xabfe);
    assert_eq!(animated_tiles::palette_instruction(&mut r, &[], 0xc5c6, 0xfffe, 0xffff).unwrap(), (28, 1));
    r.set_word(0x1e7b, 14);
    assert_eq!(animated_tiles::palette_instruction(&mut r, &[], 0xc595, 100, 0xfffe).unwrap(), (0, 0));
    assert_eq!(r.word(0x1ebd + 14), 2);
}

#[test]
fn one_byte_operand_at_bank_end_does_not_require_another_byte() {
    let mut rom = vec![0; 0x40000];
    rom[0x3ffff] = 0xfe;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    assert_eq!(animated_tiles::dispatch_rom(&mut r, &rom, "AnimtilesInstr_SetTimer", 0, 0xffff).unwrap(), 0);
    assert_eq!(r.word(0x1f01), 254);
    assert_eq!(animated_tiles::dispatch_rom(&mut r, &rom, "AnimtilesInstr_GotoRel", 0, 0xffff).unwrap(), 0xfffd);
}

#[test]
fn interpreter_runs_control_then_timed_frame_and_preserves_wait_pointer() {
    let mut rom = vec![0; 0x40000];
    rom[0x38000..0x38009].copy_from_slice(&[0xe3, 0x80, 7, 3, 0, 0x34, 0x92, 0xba, 0x81]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1ef3, 10);
    r.set_word(0x1f23, 1);
    r.set_word(0x1f17, 0x8000);
    r.set_word(0x1f0b, 0xab00);
    animated_tiles::process(&mut r, &rom).unwrap();
    assert_eq!(r.word(0x1f0b), 0xab07);
    assert_eq!(r.word(0x1f23), 3);
    assert_eq!(r.word(0x1f2f), 0x9234);
    assert_eq!(r.word(0x1f17), 0x8007);
    r.set_word(0x1f23, 1);
    animated_tiles::process(&mut r, &rom).unwrap();
    assert_eq!(r.word(0x1f23), 1);
    assert_eq!(r.word(0x1f17), 0x8007);
}

#[test]
fn byte_timer_preserves_high_byte_and_relative_branch_wraps() {
    let mut rom = vec![0; 0x40000];
    rom[0x38000] = 0xfe;
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    assert_eq!(animated_tiles::dispatch_rom(&mut r, &rom, "AnimtilesInstr_SetTimer", 10, 0x8000).unwrap(), 0x8001);
    assert_eq!(r.word(0x1f0b), 0xa5fe);
    assert_eq!(animated_tiles::dispatch_rom(&mut r, &rom, "AnimtilesInstr_GotoRel", 10, 0x8000).unwrap(), 0x7ffe);
    assert_eq!(r.word(0x1f49), 0x8000);
    r.set_word(0x1f0b, 0);
    assert_eq!(
        animated_tiles::dispatch_rom(&mut r, &rom, "AnimtilesInstr_DecrementTimerAndGotoRel", 10, 0x8000).unwrap(),
        0x7ffe
    );
    assert_eq!(r.word(0x1f0b), 0xffff);
    r.set_word(0x1f0b, 1);
    assert_eq!(
        animated_tiles::dispatch_rom(&mut r, &[], "AnimtilesInstr_DecrementTimerAndGoto", 10, 0xffff).unwrap(),
        1
    );
    assert_eq!(r.word(0x1f0b), 0);
}

#[test]
fn allocation_uses_highest_free_slot_and_preserves_every_other_byte() {
    let mut rom = vec![0; 0x40000];
    rom[0x38000..0x38006].copy_from_slice(&[0x34, 0x92, 0x80, 0, 0, 0x60]);
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let before = bytes.clone();
    animated_tiles::spawn(&mut Wram::new(&mut bytes).unwrap(), &[], 0x8000).unwrap();
    assert_eq!(bytes, before);
    Wram::new(&mut bytes).unwrap().set_word(0x1ef5 + 8, 0);
    let mut expected = bytes.clone();
    {
        let mut r = Wram::new(&mut expected).unwrap();
        for (base, value) in [
            (0x1ef5, 0x8000),
            (0x1f01, 0),
            (0x1f0d, 0x9234),
            (0x1f25, 0),
            (0x1f31, 0x80),
            (0x1f3d, 0x6000),
            (0x1f19, 1),
        ] {
            r.set_word(base + 8, value);
        }
    }
    animated_tiles::spawn(&mut Wram::new(&mut bytes).unwrap(), &rom, 0x8000).unwrap();
    assert_eq!(bytes, expected);
}
