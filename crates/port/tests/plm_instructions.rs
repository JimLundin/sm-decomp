use port::{WRAM_SIZE, Wram};
#[path = "../src/plm_instructions.rs"]
mod instructions;
fn rom_with(pc: u16, data: &[u8]) -> Vec<u8> {
    let mut rom = vec![0; 0x300000];
    let off = 0x20000 + (pc as usize & 0x7fff);
    rom[off..off + data.len()].copy_from_slice(data);
    rom
}
#[test]
fn byte_timer_preserves_high_byte_and_zero_underflows() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = rom_with(0x9000, &[3, 0x00, 0x98]);
    r.set_word(0x1d77, 0xab00);
    assert_eq!(instructions::instruction(&mut r, &rom, 0x874e, 0x9000, 0), Ok(Some(0x9001)));
    assert_eq!(r.word(0x1d77), 0xab03);
    r.set_word(0x1d77, 0);
    assert_eq!(instructions::instruction(&mut r, &rom, 0x873f, 0x9001, 0), Ok(Some(0x9800)));
    assert_eq!(r.word(0x1d77), 0xffff);
}
#[test]
fn call_return_and_sleep_use_single_link_register() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = rom_with(0x9000, &[0x00, 0x98]);
    assert_eq!(instructions::instruction(&mut r, &rom, 0x8a2e, 0x9000, 78), Ok(Some(0x9800)));
    assert_eq!(r.word(0xdebc + 78), 0x9002);
    assert_eq!(instructions::instruction(&mut r, &rom, 0x8a3a, 0x9800, 78), Ok(Some(0x9002)));
    assert_eq!(instructions::instruction(&mut r, &rom, 0x86b4, 0x9002, 78), Ok(None));
    assert_eq!(r.word(0x1d27 + 78), 0x9000);
}
#[test]
fn door_counter_copies_overlapping_two_byte_bit_array() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = rom_with(0x9000, &[1, 0x00, 0x98]);
    r.set_word(0x1dc7, 7);
    r.set_byte(0xd8b1, 0x12);
    assert_eq!(instructions::instruction(&mut r, &rom, 0x8a91, 0x9000, 0), Ok(Some(0x9800)));
    assert_eq!(r.word(0x1dc7), 0x9280);
    assert_eq!(r.word(0x1cd7), 0x8aa6);
    assert_eq!(r.word(0x5e7), 0x80);
}
#[test]
fn draw_rows_columns_and_negative_origin_offsets() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = rom_with(0x9000, &[2, 0, 0x11, 0x11, 0x22, 0x22, 0xff, 1, 2, 0x80, 0x33, 0x33, 0x44, 0x44, 0, 0]);
    r.set_word(0x1c87, 0x200);
    r.set_word(0xde6c, 0x9000);
    r.set_word(0x7a5, 16);
    instructions::process_draw_data(&mut r, &rom, 0).unwrap();
    assert_eq!(r.word(0x10202), 0x1111);
    assert_eq!(r.word(0x10204), 0x2222);
    assert_eq!(r.word(0x10220), 0x3333);
    assert_eq!(r.word(0x10240), 0x4444);
}
#[test]
fn frame_budget_and_rendering_gaps_reject() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let rom = rom_with(0x9000, &[0x24, 0x87, 0x00, 0x90]);
    r.set_byte(0x613, 1);
    assert_eq!(instructions::process_control_frame(&mut r, &rom, 8), Err("active PLM coroutine requires host"));
    r.set_byte(0x613, 0);
    r.set_word(0x1c23, 0x8000);
    r.set_word(0x1c37, 0xb703);
    r.set_word(0x1cd7, 0x853d);
    r.set_word(0x1d27, 0x9000);
    r.set_word(0xde1c, 1);
    assert_eq!(instructions::process_control_frame(&mut r, &rom, 8), Err("PLM instruction budget exceeded"));
    r.set_word(0xde1c, 1);
    let rom = rom_with(0x9000, &[1, 0, 0, 0]);
    assert_eq!(instructions::process_control_frame(&mut r, &rom, 8), Err("PLM frame rendering requires draw host"));
}
