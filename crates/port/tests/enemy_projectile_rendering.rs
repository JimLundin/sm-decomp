use port::{WRAM_SIZE, Wram, enemy_projectile_rendering as rendering};

fn map(rom: &mut [u8], pointer: u16, entries: &[(u16, u8, u16)]) {
    let offset = 0x68000 + usize::from(pointer & 0x7fff);
    rom[offset..offset + 2].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    for (i, &(x, y, tile)) in entries.iter().enumerate() {
        let p = offset + 2 + i * 5;
        rom[p..p + 2].copy_from_slice(&x.to_le_bytes());
        rom[p + 2] = y;
        rom[p + 3..p + 5].copy_from_slice(&tile.to_le_bytes());
    }
}

#[test]
fn vertical_sign_and_carry_choose_opposite_onscreen_and_offscreen_entries() {
    let mut rom = vec![0; 0x70000];
    map(&mut rom, 0x8000, &[(0, 0xf8, 0), (0, 8, 1), (0, 0xf8, 2), (0, 8, 3)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    rendering::draw_spritemap(&mut r, &rom, 0x8d, 0x8000, 0, 4, 0, 0, false).unwrap();
    assert_eq!([r.byte(0x371), r.byte(0x375), r.byte(0x379), r.byte(0x37d)], [0xf0, 12, 0xf0, 12]);
    r.set_word(0x590, 0);
    rendering::draw_spritemap(&mut r, &rom, 0x8d, 0x8000, 0, 4, 0, 0, true).unwrap();
    assert_eq!([r.byte(0x371), r.byte(0x375), r.byte(0x379), r.byte(0x37d)], [252, 0xf0, 252, 0xf0]);
    r.set_word(0x590, 0);
    rendering::draw_spritemap(&mut r, &rom, 0x8d, 0x8000, 0, 252, 0, 0, false).unwrap();
    assert_eq!([r.byte(0x371), r.byte(0x375)], [244, 0xf0]);
}

#[test]
fn cursor_wrap_preserves_extension_bits_and_tile_addition_precedes_attributes() {
    let mut rom = vec![0; 0x70000];
    map(&mut rom, 0x8000, &[(0xfff8, 0, 0xffff), (0, 0, 1)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x590, 508);
    r.set_word(0x58e, 0x1234);
    r.set_word(0x570, 0x80);
    rendering::draw_spritemap(&mut r, &rom, 0x8d, 0x8000, 0, 0, 1, 0x2400, false).unwrap();
    assert_eq!(r.word(0x590), 4);
    assert_eq!(r.byte(0x56c), 248);
    assert_eq!(r.word(0x56e), 0x2400);
    assert_eq!(r.word(0x58e), 0xd234);
    assert_eq!(r.word(0x570), 0x80);
    assert_eq!(r.word(0x372), 0x2402);
}

#[test]
fn metadata_cursor_alias_obeys_c_write_order_and_captured_cursor() {
    let mut rom = vec![0; 0x70000];
    map(&mut rom, 0x8000, &[(0x8000, 0, 1)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    // Entry byte543 maps to 058F; extension word16 aliases 0590. Cursor
    // writes overwrite entry Y/character bytes in 0590/0591 at function end.
    r.set_word(0x590, 543);
    rendering::draw_spritemap(&mut r, &rom, 0x8d, 0x8000, 0, 7, 0xff, 0x2400, false).unwrap();
    assert_eq!(r.word(0x590), 35);
    assert_eq!(r.byte(0x58f), 0);
    assert_eq!(r.byte(0x592), 0x25);
}

#[test]
fn priority_pass_uses_reverse_pool_order_and_preserves_projectile_state() {
    let mut rom = vec![0; 0x70000];
    map(&mut rom, 0x8000, &[(0, 0, 0)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    for (k, x, low) in [(0, 10, true), (2, 20, false), (34, 30, true)] {
        r.set_word(0x1997 + k, 1);
        r.set_word(0x1bd7 + k, if low { 0x1000 } else { 0 });
        r.set_word(0x1b6b + k, 0x8000);
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, 100);
    }
    rendering::draw_priority(&mut r, &rom, true).unwrap();
    assert_eq!([r.byte(0x370), r.byte(0x374)], [30, 10]);
    assert_eq!(r.word(0x590), 8);
    rendering::draw_priority(&mut r, &rom, false).unwrap();
    assert_eq!(r.byte(0x378), 20);
    assert_eq!(r.word(0x1a4b + 34), 30);
}

#[test]
fn camera_clipping_short_circuits_invalid_maps_and_odd_slot_floors() {
    let mut rom = vec![0; 0x70000];
    map(&mut rom, 0x8000, &[(0, 0, 0)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b, 384);
    rendering::draw_projectile(&mut r, &[], 1, (0, 0)).unwrap();
    r.set_word(0x1a4b, 0);
    r.set_word(0x1a93, 384);
    rendering::draw_projectile(&mut r, &[], 1, (0, 0)).unwrap();
    r.set_word(0x1a4b, 0xffff);
    r.set_word(0x1a93, 100);
    r.set_word(0x1b6b, 0x8000);
    rendering::draw_projectile(&mut r, &rom, 1, (0, 0)).unwrap();
    assert_eq!(r.byte(0x370), 255);
    assert_eq!(r.word(0x570), 1);
    assert_eq!(r.word(0x590), 4);
}

#[test]
fn frozen_quake_needs_no_rom_and_phase_negates_wrapped_native_table_values() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1840, 3);
    r.set_word(0x183e, 2);
    r.set_word(0xa78, 1);
    assert_eq!(rendering::screen_shake(&r, &[]), Ok((0, 0)));
    r.set_word(0xa78, 0);
    let mut rom = vec![0; 0x40000];
    rom[0x30473..0x30477].copy_from_slice(&[0xff, 0xff, 0x00, 0x80]);
    assert_eq!(rendering::screen_shake(&r, &rom), Ok((1, 0x8000)));
    r.set_word(0x1840, 1);
    assert_eq!(rendering::screen_shake(&r, &rom), Ok((0xffff, 0x8000)));
    r.set_word(0x183e, 36);
    assert_eq!(rendering::screen_shake(&r, &[]), Ok((0, 0)));
    r.set_word(0x183e, 0xc000);
    rom[0x3046b..0x3046f].copy_from_slice(&[3, 0, 4, 0]);
    assert_eq!(rendering::screen_shake(&r, &rom), Ok((3, 4)));
}
