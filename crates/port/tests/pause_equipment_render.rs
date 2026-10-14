#[path = "../src/memory.rs"]
mod memory;
#[path = "../src/pause_equipment_render.rs"]
mod pause_equipment_render;
use memory::{WRAM_SIZE, Wram};
use pause_equipment_render as render;

fn word(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn bank_word(rom: &mut [u8], address: u16, value: u16) {
    word(rom, 0x10000 + usize::from(address & 0x7fff), value);
}

fn assets() -> Vec<u8> {
    let mut rom = vec![0; 0x18000];
    bank_word(&mut rom, 0xc100, 0x0c00);
    for id in 0..0x30u16 {
        let pointer = 0xc800 + id * 7;
        bank_word(&mut rom, 0xc569 + 2 * id, pointer);
        bank_word(&mut rom, pointer, 1);
        bank_word(&mut rom, pointer + 2, 0);
        bank_word(&mut rom, pointer + 5, id);
    }
    for (slot, timer, frame, variant) in [(1, 0x72b, 0x73f, 0x753), (2, 0x72d, 0x741, 0x755), (5, 0x733, 0x747, 0x75b)]
    {
        bank_word(&mut rom, 0xc0b2 + 2 * slot, timer);
        bank_word(&mut rom, 0xc0c4 + 2 * slot, frame);
        bank_word(&mut rom, 0xc0d6 + 2 * slot, variant);
        bank_word(&mut rom, 0xc0e8 + 2 * slot, 0xd000);
        bank_word(&mut rom, 0xc1e4 + 2 * slot, 0xd020);
    }
    rom[0x15000..0x15009].copy_from_slice(&[4, 7, 0, 2, 8, 1, 0xff, 0, 0]);
    for category in 0..4u16 {
        bank_word(&mut rom, 0xd020 + 2 * category, 0x10 + category);
        let pointer = 0xd100 + category * 0x20;
        bank_word(&mut rom, 0xc18e + 2 * category, pointer);
        for row in 0..6 {
            bank_word(&mut rom, pointer + row * 4, 50 + row);
            bank_word(&mut rom, pointer + row * 4 + 2, 80 + category);
        }
    }
    rom[0x14165..0x1416b].copy_from_slice(&[15, 0, 2, 0, 0xff, 0]);
    for (index, bits) in [0, 0x100, 1, 0x101].into_iter().enumerate() {
        bank_word(&mut rom, 0xb257 + index as u16 * 2, bits);
        let pointer = 0xd200 + index as u16 * 0x110;
        bank_word(&mut rom, 0xb25f + index as u16 * 2, pointer);
        for tile in 0..136u16 {
            bank_word(&mut rom, pointer + tile * 2, index as u16 * 0x1000 + tile);
        }
    }
    rom
}

#[test]
fn menu_wrap_and_extension_bits_preserve_unsigned_coordinates_and_tile_mask() {
    let mut rom = assets();
    bank_word(&mut rom, 0xc569, 0xce00);
    bank_word(&mut rom, 0xce00, 2);
    bank_word(&mut rom, 0xce02, 0xfff8);
    rom[0x14e04] = 0xfe;
    bank_word(&mut rom, 0xce05, 0xffff);
    bank_word(&mut rom, 0xce07, 0x100);
    rom[0x14e09] = 3;
    bank_word(&mut rom, 0xce0a, 0x1234);
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x590, 508);
    r.set_word(0x58e, 0x1234);
    r.set_word(0x570, 0x0040);
    render::draw_menu_spritemap(&mut r, &rom, 0, 0, 0xffff, 0x0a00).unwrap();
    assert_eq!((r.byte(0x56c), r.byte(0x56d), r.word(0x56e)), (248, 253, 0xfbff));
    assert_eq!(r.word(0x58e), 0xd234);
    assert_eq!((r.byte(0x370), r.byte(0x371), r.word(0x372)), (0, 2, 0x1a34));
    assert_eq!(r.word(0x570), 0x0041);
    assert_eq!(r.word(0x590), 4);
}

#[test]
fn menu_unaligned_cursor_preserves_the_extension_alias_write_order() {
    let mut rom = assets();
    bank_word(&mut rom, 0xc802, 0x8000);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x590, 511);
    render::draw_menu_spritemap(&mut r, &rom, 0, 0, 0x35, 0).unwrap();
    // X aliases extension byte 0x56f; Y subsequently replaces extension
    // byte 0x570. C reads that word before its later Y/character stores.
    assert_eq!(r.byte(0x56f), 0);
    assert_eq!(r.byte(0x570), 0x35);
    assert_eq!(r.word(0x571), 0);
    assert_eq!(r.word(0x590), 3);
}

#[test]
fn selector_uses_low_category_byte_even_when_item_row_is_nonzero() {
    let rom = assets();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x9a4, 1);
    r.set_word(0x755, 0x0502);
    r.set_word(0x72d, 2);
    render::equipment_screen_draw_item_selector(&mut r, &rom).unwrap();
    assert_eq!((r.byte(0x370), r.byte(0x371), r.word(0x372)), (54, 81, 0x0c12));
    assert_eq!(r.word(0x72d), 1);
    assert_eq!(r.word(0x741), 0);
}

#[test]
fn sprite_timer_signed_boundary_and_terminator_reset_match_sequential_frames() {
    let rom = assets();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x72d, 0x8000);
    render::draw_pause_screen_sprite_anim(&mut r, &rom, 3, 10, 20).unwrap();
    assert_eq!((r.word(0x72d), r.word(0x741)), (0x7fff, 0));
    r.set_word(0x72d, 0);
    render::draw_pause_screen_sprite_anim(&mut r, &rom, 3, 10, 20).unwrap();
    assert_eq!((r.word(0x72d), r.word(0x741)), (2, 1));
    render::draw_pause_screen_sprite_anim(&mut r, &rom, 3, 10, 20).unwrap();
    assert_eq!((r.word(0x72d), r.word(0x741)), (1, 1));
    render::draw_pause_screen_sprite_anim(&mut r, &rom, 3, 10, 20).unwrap();
    assert_eq!((r.word(0x72d), r.word(0x741)), (4, 0));
    assert_eq!(r.word(0x376), 0x0c11);
    assert_eq!(r.word(0x37e), 0x0c10);
}

#[test]
fn empty_selector_and_empty_palette_need_no_rom_and_preserve_state() {
    let mut bytes = vec![0; WRAM_SIZE];
    word(&mut bytes, 0x755, 0xffff);
    word(&mut bytes, 0x72f, 0xffff);
    word(&mut bytes, 0x743, 0xffff);
    let before = bytes.clone();
    let mut r = Wram::new(&mut bytes).unwrap();
    render::equipment_screen_draw_item_selector(&mut r, &[]).unwrap();
    assert_eq!(render::equipment_screen_display_reserves_palette_setup(&mut r, &[]), Ok(0x600));
    render::equipment_screen_display_reserve_tank_amount(&mut r, &[]).unwrap();
    assert_eq!(bytes, before);
}

#[test]
fn reserve_rounding_flickers_only_below_the_four_step_cutoff() {
    let rom = assets();
    for (health, phase, expected) in [
        (1, 0, 0x21),
        (1, 4, 0x20),
        (14, 0, 0x21),
        (15, 0, 0x22),
        (55, 0, 0x24),
        (56, 0, 0x24),
        (57, 0, 0x24),
        (99, 0, 0x27),
        (101, 0, 0x21),
    ] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x9d4, 400);
        r.set_word(0x9d6, health);
        r.set_word(0x72f, 2);
        r.set_byte(0x5b5, phase);
        render::equipment_screen_display_reserve_tank_amount(&mut r, &rom).unwrap();
        let partial = 0x372 + usize::from(health / 100) * 4;
        assert_eq!(r.word(partial), 0x0c00 | expected, "health {health}, phase {phase}");
        assert_eq!(r.word(0x3b10), 2052 + health / 100);
        assert_eq!(r.word(0x3b12), 2052 + health % 100 / 10);
        assert_eq!(r.word(0x3b14), 2052 + health % 10);
    }
}

#[test]
fn reserve_full_tanks_end_cap_and_palette_wrap_use_exact_outputs() {
    let rom = assets();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x9d4, 400);
    r.set_word(0x9d6, 400);
    r.set_word(0x72f, 1);
    r.set_word(0x743, 1);
    render::equipment_screen_display_reserve_tank_amount_(&mut r, &rom).unwrap();
    assert_eq!((r.word(0x72f), r.word(0x743)), (15, 0));
    assert_eq!((r.word(0x372), r.word(0x37e), r.word(0x382)), (0x0c1b, 0x0c1b, 0x0c1f));
    assert_eq!((r.byte(0x380), r.byte(0x381), r.word(0x590)), (56, 95, 20));
}

#[test]
fn wireframe_writes_only_seventeen_eight_word_rows_and_exact_seven_byte_queue() {
    let rom = assets();
    for (index, bits) in [0, 0x100, 1, 0x101].into_iter().enumerate() {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x9a2, bits | 0xeefe);
        r.set_word(0x330, 601);
        render::write_samus_wireframe_tilemap_and_queue(&mut r, &rom).unwrap();
        assert_eq!(r.word(0x39d8), index as u16 * 0x1000);
        assert_eq!(r.word(0x3de6), index as u16 * 0x1000 + 135);
        assert_eq!(r.word(0x39e8), 0xa5a5);
        assert_eq!(r.word(0x3dd6), 0xa5a5);
        assert_eq!(r.word(0x329), 1280);
        assert_eq!(r.word(0x32b), 0x3900);
        assert_eq!(r.byte(0x32d), 0x7e);
        assert_eq!(r.word(0x32e), 0x3080);
        assert_eq!(r.word(0x330), 608);
    }
}

#[test]
fn failed_preflight_leaves_full_wram_unchanged_even_after_late_selected_assets() {
    let rom = assets();
    for failure in 0..8 {
        let mut input_rom = rom.clone();
        let mut bytes = vec![0; WRAM_SIZE];
        word(&mut bytes, 0x9a4, 1);
        word(&mut bytes, 0x9d4, 400);
        word(&mut bytes, 0x9d6, 1);
        word(&mut bytes, 0x72f, 1);
        word(&mut bytes, 0x72d, 1);
        match failure {
            0 => word(&mut bytes, 0x590, 512),
            1 => word(&mut bytes, 0x755, 0x0602),
            2 => word(&mut bytes, 0x741, 2),
            3 => bank_word(&mut input_rom, 0xc569 + 2 * 0x1f, 0xffff),
            4 => word(&mut bytes, 0x9d6, 401),
            5 => word(&mut bytes, 0x743, 2),
            6 => word(&mut bytes, 0x330, 602),
            7 => bank_word(&mut input_rom, 0xb25f, 0xfffe),
            _ => unreachable!(),
        }
        let before = bytes.clone();
        let mut r = Wram::new(&mut bytes).unwrap();
        let result = match failure {
            0..=2 => render::equipment_screen_draw_item_selector(&mut r, &input_rom),
            3..=5 => render::equipment_screen_display_reserve_tank_amount(&mut r, &input_rom),
            _ => render::write_samus_wireframe_tilemap_and_queue(&mut r, &input_rom),
        };
        assert!(result.is_err(), "failure {failure}");
        assert_eq!(bytes, before, "failure {failure}");
    }
}

#[test]
fn dispatch_requires_a_known_routine_and_keeps_packed_sprite_words_intact() {
    let rom = assets();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    assert!(render::dispatch_rom(&mut r, &rom, "missing", 0, 0).is_err());
    assert_eq!(render::dispatch_rom(&mut r, &rom, "DrawMenuSpritemap", 0xffff0000u32 as i32, 0x0a00ffff), Ok(0));
    assert_eq!((r.byte(0x370), r.byte(0x371), r.word(0x372)), (255, 255, 0x0a00));
}
