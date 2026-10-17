use port::{WRAM_SIZE, Wram, sprite_objects};

#[test]
fn rendering_wraps_oam_and_preserves_extension_bits() {
    let mut rom = vec![0; 0x200000];
    rom[0x1a0002..0x1a0004].copy_from_slice(&0x8100u16.to_le_bytes());
    rom[0x1a0100..0x1a0102].copy_from_slice(&2u16.to_le_bytes());
    rom[0x1a0102..0x1a010c].copy_from_slice(&[0xff, 0xff, 3, 0xff, 0xff, 1, 0, 255, 0, 1]);
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0xef78 + 62, 0x8000);
        r.set_word(0xf0f8 + 62, 0);
        r.set_word(0xf1f8 + 62, 255);
        r.set_word(0xf078 + 62, 0xe01);
        r.set_word(0x590, 508);
        r.set_word(0x58e, 0x1234);
        r.set_word(0x570, 0x5555);
    }
    let mut expected = bytes.clone();
    {
        let mut r = Wram::new(&mut expected).unwrap();
        r.set_byte(0x370 + 508, 255);
        r.set_byte(0x371 + 508, 2);
        r.set_word(0x372 + 508, 0xe00);
        r.set_word(0x58e, 0xd234);
        r.set_byte(0x370, 1);
        r.set_byte(0x371, 254);
        r.set_word(0x372, 0xf01);
        r.set_word(0x590, 4);
    }
    sprite_objects::draw(&mut Wram::new(&mut bytes).unwrap(), &rom).unwrap();
    assert_eq!(bytes, expected);
}

#[test]
fn allocation_chooses_highest_free_slot_and_refuses_full_without_writes() {
    let mut rom = vec![0; 0x200000];
    rom[0x1a3da8..0x1a3daa].copy_from_slice(&0x8000u16.to_le_bytes());
    rom[0x1a0000..0x1a0002].copy_from_slice(&3u16.to_le_bytes());
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let before = bytes.clone();
    assert_eq!(sprite_objects::create(&mut Wram::new(&mut bytes).unwrap(), &[], 1, 2, 0, 3).unwrap(), 0xffff);
    assert_eq!(bytes, before);
    Wram::new(&mut bytes).unwrap().set_word(0xef78 + 60, 0);
    let mut expected = bytes.clone();
    {
        let mut r = Wram::new(&mut expected).unwrap();
        for (base, value) in [
            (0xf178, 0),
            (0xf278, 0),
            (0xf2f8, 0),
            (0xf0f8, 0xffff),
            (0xf1f8, 0x8000),
            (0xf078, 0xe05),
            (0xef78, 0x8000),
            (0xeff8, 3),
        ] {
            r.set_word(base + 60, value);
        }
    }
    assert_eq!(
        sprite_objects::dispatch_rom(
            &mut Wram::new(&mut bytes).unwrap(),
            &rom,
            "CreateSpriteAtPos",
            0x8000ffffu32 as i32,
            0x0e050000
        )
        .unwrap(),
        60
    );
    assert_eq!(bytes, expected);
}

#[test]
fn frame_advances_then_terminates_and_leaves_final_index_underflow() {
    let mut rom = vec![0; 0x200000];
    rom[0x1a0004..0x1a0006].copy_from_slice(&0xbd07u16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0xef78 + 62, 0x8000);
        r.set_word(0xeff8 + 62, 1);
        r.set_word(0xef78, 0x8100);
        r.set_word(0xeff8, 2);
    }
    sprite_objects::handle(&mut Wram::new(&mut bytes).unwrap(), &rom).unwrap();
    let r = Wram::new(&mut bytes).unwrap();
    assert_eq!(r.word(0xef78 + 62), 0);
    assert_eq!(r.word(0xeff8 + 62), 0);
    assert_eq!(r.word(0xef78), 0x8100);
    assert_eq!(r.word(0xeff8), 1);
    assert_eq!(r.word(0x1844), 0xfffe);
}

#[test]
fn clear_preserves_word_zero_and_clears_original_overlapping_range() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut expected = bytes.clone();
    expected[0xef7a..0xf378].fill(0);
    sprite_objects::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(), &[], "ClearSpriteObjects", 0, 0).unwrap();
    assert_eq!(bytes, expected);
}

#[test]
fn frozen_handler_preserves_all_wram_and_invalid_slot_is_rejected() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let before = bytes.clone();
    sprite_objects::handle(&mut Wram::new(&mut bytes).unwrap(), &[]).unwrap();
    assert_eq!(bytes, before);
    Wram::new(&mut bytes).unwrap().set_word(0x1844, 63);
    let before = bytes.clone();
    assert!(
        sprite_objects::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(), &[], "SpriteObject_Instr_Terminate", 0, 0)
            .is_err()
    );
    assert_eq!(bytes, before);
}
