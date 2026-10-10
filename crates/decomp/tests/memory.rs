use decomp::{DecompressError, RomView, WRAM_SIZE, WramView, decompress_to_mem};

#[test]
fn bounded_views_preserve_unaligned_words_and_rom_bank_mirrors() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut wram = WramView::new(&mut bytes).unwrap();
    wram.write_u16_le(3, 0x1234).unwrap();
    assert_eq!(
        [wram.read(2).unwrap(), wram.read(3).unwrap(), wram.read(4).unwrap(), wram.read(5).unwrap()],
        [0xa5, 0x34, 0x12, 0xa5]
    );
    assert_eq!(wram.read_u16_le(3).unwrap(), 0x1234);
    assert_eq!(wram.write_u16_le(WRAM_SIZE - 1, 0x1234), Err(DecompressError::WramBounds(WRAM_SIZE)));
    assert_eq!(wram.read(WRAM_SIZE - 1).unwrap(), 0xa5);
    let rom = RomView::new(&[0x42]).unwrap();
    assert_eq!(rom.read(0x808000).unwrap(), 0x42);
    assert_eq!(rom.read(0x008000).unwrap(), 0x42);
    assert_eq!(rom.read(0x808001), Err(DecompressError::InvalidSource(0x808001)));
    assert_eq!(rom.read(0x807fff), Err(DecompressError::InvalidSource(0x807fff)));
}

#[test]
fn unsupported_commands_and_destination_overflow_are_explicit() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut wram = WramView::new(&mut bytes).unwrap();
    let rom = RomView::new(&[0x20, 0x42, 0xff]).unwrap();
    assert!(matches!(
        decompress_to_mem(&rom, &mut wram, 0x808000, 4096),
        Err(DecompressError::UnsupportedCommand(0x20))
    ));
    assert_eq!(wram.read(4096).unwrap(), 0xa5);
    let literal = RomView::new(&[1, 0x12, 0x34, 0xff]).unwrap();
    assert!(matches!(
        decompress_to_mem(&literal, &mut wram, 0x808000, WRAM_SIZE - 1),
        Err(DecompressError::WramBounds(WRAM_SIZE))
    ));
}
