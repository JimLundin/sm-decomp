#[path = "../src/enemy_instructions.rs"]
mod instructions;
use port::Wram;

#[test]
fn pre_instruction_updates_use_banked_rom_and_keep_other_record_bytes() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xf78 + 0x40 + 0x2e, 0xa2);
    let mut rom = vec![0; 0x400000];
    rom[0x110002..0x110004].copy_from_slice(&0x9234u16.to_le_bytes());
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_SetAiPreInstr", 0x40, 0xa28002), Ok(0xa28004));
    assert_eq!(ram.word(0xf78 + 0x40 + 0x3a), 0x9234);
    assert_eq!(ram.word(0xf78 + 0x3a), 0xa5a5);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_ClearAiPreInstr", 0x40, 0xa28002), Ok(0xa28002));
    assert_eq!(ram.word(0xf78 + 0x40 + 0x3a), 0x807b);
}

#[test]
fn timer_branches_keep_word_and_byte_widths_and_odd_relative_pointers() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xa2);
    let mut rom = vec![0; 0x400000];
    rom[0x110080..0x110082].copy_from_slice(&0x9081u16.to_le_bytes());
    ram.set_word(0xf90, 0);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_DecTimerAndGoto", 0, 0xa28080), Ok(0xa29081));
    assert_eq!(ram.word(0xf90), 65535);
    ram.set_word(0xf90, 1);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_DecTimerAndGoto", 0, 0xa28080), Ok(0xa28082));
    ram.set_word(0xf90, 0x1200);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_DecTimerAndGotoRel", 0, 0xa28080), Ok(0xa28001));
    assert_eq!(ram.word(0xf90), 0x12ff);
    ram.set_word(0xf90, 0x1201);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_DecTimerAndGotoRel", 0, 0xa28080), Ok(0xa28081));
    assert_eq!(ram.word(0xf90), 0x1200);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_SetTimer", 0, 0xa28080), Ok(0xa28082));
    assert_eq!(ram.word(0xf90), 0x9081);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_Skip2bytes", 0, 0xa28080), Ok(0xa28082));
}

#[test]
fn script_wait_sleep_deletion_and_offscreen_flags_keep_distinct_contracts() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xaa);
    ram.set_word(0xf86, 0x41);
    let mut rom = vec![0; 0x400000];
    rom[0x150004..0x150006].copy_from_slice(&7u16.to_le_bytes());
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_WaitNframes", 0, 0xaa8004), Ok(0));
    assert_eq!((ram.word(0xf94), ram.word(0xf92)), (7, 0x8006));
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_Sleep", 0, 0xaa8004), Ok(0));
    assert_eq!(ram.word(0xf92), 0x8002);
    instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_EnableOffScreenProcessing", 0, 0xaa8004).unwrap();
    assert_eq!(ram.word(0xf86), 0x841);
    instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_DisableOffScreenProcessing", 0, 0xaa8004).unwrap();
    assert_eq!(ram.word(0xf86), 0x41);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_StopScript", 0, 0xaa8004), Ok(0));
    assert_eq!(ram.word(0xf86), 0x241);
}

#[test]
fn vram_instruction_copies_packed_seven_byte_entry_at_unaligned_tail() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xaa);
    ram.set_word(0x330, 1);
    let mut rom = vec![0; 0x400000];
    rom[0x150001..0x150008].copy_from_slice(&[0x20, 0, 0x34, 0x92, 0x83, 0x00, 0x60]);
    assert_eq!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_CopyToVram", 0, 0xaa8001), Ok(0xaa8008));
    assert_eq!(ram.word(0x330), 8);
    assert_eq!(
        (0xd0..0xd9).map(|p| ram.byte(p)).collect::<Vec<_>>(),
        [0xa5, 0x20, 0, 0x34, 0x92, 0x83, 0x00, 0x60, 0xa5]
    );
}

#[test]
fn interpreter_runs_timer_loop_then_animation_and_waits_across_frames() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xfa6, 0xa2);
    ram.set_word(0xf92, 0x8000);
    ram.set_word(0xf94, 1);
    let mut rom = vec![0; 0x400000];
    // Set loop timer3, decrement+goto until1, then frame duration5/spritemap9234.
    for (i, word) in [0x8123u16, 3, 0x8110, 0x8004, 5, 0x9234].into_iter().enumerate() {
        rom[0x110000 + 2 * i..0x110002 + 2 * i].copy_from_slice(&word.to_le_bytes());
    }
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!(
        (ram.word(0xf90), ram.word(0xf94), ram.word(0xf8e), ram.word(0xf92), ram.word(0xf88)),
        (0, 5, 0x9234, 0x800c, 0x8000)
    );
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!((ram.word(0xf94), ram.word(0xf88)), (4, 0));
    ram.set_word(0xf8a, 4);
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!(ram.word(0xf94), 4);
}

#[test]
fn interpreter_rejects_real_bank_specific_dispatch_and_nonterminating_script() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xa2);
    ram.set_word(0xf92, 0x8000);
    ram.set_word(0xf94, 1);
    let mut rom = vec![0; 0x400000];
    rom[0x110000..0x110002].copy_from_slice(&0x88c6u16.to_le_bytes());
    assert_eq!(
        instructions::process_generic_instructions(&mut ram, &rom, 20),
        Err("unsupported bank-specific enemy instruction")
    );
    ram.set_word(0xf94, 1);
    rom[0x110000..0x110004].copy_from_slice(&[0xed, 0x80, 0, 0x80]);
    assert_eq!(
        instructions::process_generic_instructions(&mut ram, &rom, 20),
        Err("enemy instruction execution limit reached")
    );
}

#[test]
fn malformed_instruction_contracts_reject_before_memory_mutation() {
    let mut bytes = vec![0xa5; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xaa);
    let rom = vec![0; 0x400000];
    assert!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_SetAiPreInstr", 0, 0xaaffff).is_err());
    assert_eq!(ram.word(0xfb2), 0xa5a5);
    assert!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_Goto", 0, 0xaa8000).is_err());
    assert!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_ClearAiPreInstr", 64, 0xaa8000).is_err());
    assert!(instructions::dispatch_rom(&mut ram, &rom, "EnemyInstr_CopyToVram", 0, 0xaa8000).is_err());
    assert_eq!(ram.word(0x330), 0xa5a5);
    assert!(instructions::dispatch_rom(&mut ram, &[], "EnemyInstr_SetTimer", 0, 0xaa8000).is_err());
    assert_eq!(ram.word(0xf90), 0xa5a5);
}

#[test]
fn interpreter_wait_and_sleep_terminate_without_refreshing_spritemap_flags() {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0xfa6, 0xaa);
    ram.set_word(0xf92, 0x8000);
    ram.set_word(0xf94, 1);
    ram.set_word(0xf88, 0x8005);
    ram.set_word(0xf8e, 0x9234);
    let mut rom = vec![0; 0x400000];
    rom[0x150000..0x150004].copy_from_slice(&[0x3a, 0x81, 3, 0]);
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!((ram.word(0xf94), ram.word(0xf92), ram.word(0xf8e), ram.word(0xf88)), (3, 0x8004, 0x9234, 0x8005));
    ram.set_word(0xf94, 1);
    rom[0x150004..0x150006].copy_from_slice(&[0x2f, 0x81]);
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!((ram.word(0xf94), ram.word(0xf92), ram.word(0xf88)), (0, 0x8004, 0x8005));
    assert_eq!(instructions::process_generic_instructions(&mut ram, &rom, 20), Ok(()));
    assert_eq!((ram.word(0xf94), ram.word(0xf88)), (65535, 5));
}
