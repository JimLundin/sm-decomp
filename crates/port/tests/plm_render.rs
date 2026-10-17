use port::{WRAM_SIZE, Wram, plm_render};
#[test]
fn coordinates_use_snes_eight_bit_divisor() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1c87, 222);
    r.set_word(0x7a5, 0x110);
    plm_render::calculate_block_coords(&mut r, 0).unwrap();
    assert_eq!((r.word(0x1c29), r.word(0x1c2b)), (15, 6));
    r.set_word(0x7a5, 0x100);
    plm_render::calculate_block_coords(&mut r, 0).unwrap();
    assert_eq!((r.word(0x1c29), r.word(0x1c2b)), (111, 0xffff));
}
fn scene(blocks: &[u16]) -> (Vec<u8>, Vec<u8>) {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xde6c, 0x9000);
    r.set_word(0xa008, 0x11);
    r.set_word(0xa00a, 0x22);
    r.set_word(0xa00c, 0x33);
    r.set_word(0xa00e, 0x44);
    let mut rom = vec![0; 0x28000];
    let off = 0x21000;
    for (i, w) in blocks.iter().enumerate() {
        rom[off + i * 2..off + i * 2 + 2].copy_from_slice(&w.to_le_bytes());
    }
    (bytes, rom)
}
#[test]
fn horizontal_draw_queues_top_and_bottom_rows_with_flips() {
    let (mut bytes, rom) = scene(&[4, 1, 0x401, 0x801, 0xc01, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!((r.word(0x1c25), r.word(0x330)), (32, 14));
    assert_eq!((r.word(0xd0), r.word(0xd2), r.byte(0xd4), r.word(0xd5)), (16, 0xc6c8, 0x7e, 0x5000));
    assert_eq!((r.word(0xd7), r.word(0xd9), r.word(0xdc)), (16, 0xc6d8, 0x5020));
    let row = (0..8).map(|i| r.word(0xc6c8 + i * 2)).collect::<Vec<_>>();
    assert_eq!(row, vec![0x11, 0x22, 0x4022, 0x4011, 0x8033, 0x8044, 0xc044, 0xc033]);
}
#[test]
fn column_clips_top_and_uses_vertical_vram_addressing() {
    let (mut bytes, rom) = scene(&[0x8004, 1, 0x401, 0x801, 0xc01, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x915, 16);
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!((r.word(0x1c25), r.word(0xd0), r.word(0xd5), r.word(0xdc)), (24, 12, 0xd040, 0xd041));
    assert_eq!((r.word(0xc6c8), r.word(0xc6ca), r.word(0xc6d4), r.word(0xc6d6)), (0x4022, 0x4044, 0x4011, 0x4033));
}
#[test]
fn row_crossing_screen_boundary_has_four_transfers() {
    let (mut bytes, rom) = scene(&[2, 1, 1, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1c29, 15);
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!(r.word(0x330), 28);
    let entries = (0..4)
        .map(|i| {
            let p = 0xd0 + i * 7;
            (r.word(p), r.word(p + 2), r.word(p + 5))
        })
        .collect::<Vec<_>>();
    assert_eq!(entries, vec![(4, 0xc6c8, 0x501e), (4, 0xc6cc, 0x5400), (4, 0xc6d0, 0x503e), (4, 0xc6d4, 0x5420)]);
}
#[test]
fn segment_offsets_are_relative_to_original_block() {
    let (mut bytes, rom) = scene(&[1, 1, 0x0101, 0x8001, 0x401, 0xff02, 1, 0x801, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1c29, 4);
    r.set_word(0x1c2b, 2);
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!((r.word(0x1c25), r.word(0x330)), (24, 42));
    assert_eq!((r.word(0xd5), r.word(0xe3), r.word(0xf1)), (0x5088, 0xd0ca, 0x504c));
}
#[test]
fn full_tilemap_stops_before_following_segment_and_full_queue_does_not_draw() {
    let (mut bytes, rom) = scene(&[1, 1, 0x0101, 0, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1c25, 504);
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!((r.word(0x1c25), r.word(0x330)), (512, 14));
    r.set_word(0x1c25, 0);
    r.set_word(0x330, 480);
    plm_render::draw(&mut r, &rom, 0).unwrap();
    assert_eq!((r.word(0x1c25), r.word(0x330)), (0, 480));
}
#[test]
fn malformed_draw_pointer_and_slot_reject_safely() {
    let (mut bytes, rom) = scene(&[1, 1, 0]);
    let mut r = Wram::new(&mut bytes).unwrap();
    assert!(plm_render::draw(&mut r, &rom, 1).is_err());
    r.set_word(0xde6c, 0x2000);
    assert!(plm_render::draw(&mut r, &rom, 0).is_err());
    r.set_word(0xde6c, 0xffff);
    assert!(plm_render::draw(&mut r, &rom, 0).is_err());
}
