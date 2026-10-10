use port::{WRAM_SIZE, Wram, room};
#[test]
fn parallax_manual_modes_and_signed_blocks() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x911, 0x1234);
    r.set_byte(0x91b, 128);
    assert_eq!(room::dispatch_rom(&mut r, &[], "CalculateLayer2Xpos", 0, 0), Ok(0));
    assert_eq!(r.word(0x917), 0x091a);
    r.set_byte(0x91b, 1);
    r.set_word(0x911, 0xfff0);
    assert_eq!(room::dispatch_rom(&mut r, &[], "CalculateLayer2Xpos", 0, 0), Ok(1));
    assert_eq!(r.word(0x917), 0x091a);
    room::dispatch_rom(&mut r, &[], "CalculateBgScrollAndLayerPositionBlocks", 0, 0).unwrap();
    assert_eq!(r.word(0x8f7), 0xffff);
}
#[test]
fn closed_scroll_cell_pushes_camera_toward_next_screen() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a9, 4);
    r.set_word(0x911, 127);
    r.set_word(0xb0a, 0);
    r.set_word(0xda2, 3);
    room::dispatch_rom(&mut r, &[], "HandleScrollingWhenTriggeringScrollLeft", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 132);
    assert!(room::dispatch_rom(&mut r, &[], "HardwareTransition", 0, 0).is_err());
}
#[test]
fn door_negative_distance_uses_direction_specific_default() {
    let mut rom = vec![0; 0x20000];
    let o = (0x83 & 127) * 32768;
    rom[o..o + 12].copy_from_slice(&[0, 0x90, 0x80, 2, 0, 0, 3, 4, 255, 255, 0, 0]);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x78d, 0x8000);
    room::dispatch_rom(&mut r, &rom, "LoadDoorHeader", 0, 0).unwrap();
    assert_eq!(r.word(0x79b), 0x9000);
    assert_eq!(r.word(0x92d), 1);
    assert_eq!(r.word(0x92b), 0x8000);
    assert_eq!(r.word(0xe16), 128);
    assert_eq!(r.word(0x927), 0x300);
}
#[test]
fn room_state_morph_gate_selects_actual_rom_destination() {
    let mut rom = vec![0; 0x80000];
    let o = (0x8f & 127) * 32768;
    rom[o..o + 4].copy_from_slice(&[0x52, 0xe6, 0x20, 0x80]);
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    assert_eq!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_MorphBallMissiles", 0x8002, 0), Ok(0x8004));
    r.set_word(0x9a4, 4);
    r.set_word(0x9c8, 5);
    assert_eq!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_MorphBallMissiles", 0x8002, 0), Ok(0));
    assert_eq!(r.word(0x7bb), 0x8020);
    assert!(room::dispatch_rom(&mut r, &rom, "RoomDefStateSelect_Finish", 0xfff0, 0).is_err());
}

#[test]
fn horizontal_ideal_reset_clears_its_own_fractional_word() {
    let mut b = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut b).unwrap();
    r.set_word(0x7a9, 4);
    r.set_word(0x911, 255);
    r.set_word(0xb0a, 0);
    r.set_word(0x90f, 0xa5a5);
    r.set_word(0x913, 0x1234);
    for i in 0..256 {
        r.set_byte(0xcd20 + i, 1);
    }
    room::dispatch_rom(&mut r, &[], "HandleScrollingWhenTriggeringScrollRight", 0, 0).unwrap();
    assert_eq!(r.word(0x911), 0);
    assert_eq!(r.word(0x90f), 0);
    assert_eq!(r.word(0x913), 0x1234);
}
