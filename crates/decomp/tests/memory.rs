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
fn destination_overflow_is_explicit() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut wram = WramView::new(&mut bytes).unwrap();
    let literal = RomView::new(&[1, 0x12, 0x34, 0xff]).unwrap();
    assert!(matches!(
        decompress_to_mem(&literal, &mut wram, 0x808000, WRAM_SIZE - 1),
        Err(DecompressError::WramBounds(WRAM_SIZE))
    ));
}

#[test]
fn fills_preserve_odd_words_and_wrapping_bytes() {
    let stream = [0x22, 0x42, 0x44, 0x12, 0x34, 0x63, 0xfe, 0xff];
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let result =
        decompress_to_mem(&RomView::new(&stream).unwrap(), &mut WramView::new(&mut bytes).unwrap(), 0x808000, 4096)
            .unwrap();
    assert_eq!(&bytes[4095..4109], &[0xa5, 0x42, 0x42, 0x42, 0x12, 0x34, 0x12, 0x34, 0x12, 0xfe, 0xff, 0, 1, 0xa5]);
    assert_eq!(result.written, 12);
}

#[test]
fn overlapping_and_preexisting_backreferences_are_sequential() {
    let stream = [0x00, 0x12, 0x83, 0, 0, 0xa1, 0, 0, 0xc2, 1, 0xfc, 2, 1, 0xff];
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let result =
        decompress_to_mem(&RomView::new(&stream).unwrap(), &mut WramView::new(&mut bytes).unwrap(), 0x808000, 4096)
            .unwrap();
    assert_eq!(&bytes[4096..4109], &[0x12, 0x12, 0x12, 0x12, 0x12, 0xed, 0xed, 0xed, 0xed, 0xed, 0x12, 0xed, 0x12]);
    assert_eq!(result.written, 13);
    let stream = [0xc1, 3, 0xff];
    let result =
        decompress_to_mem(&RomView::new(&stream).unwrap(), &mut WramView::new(&mut bytes).unwrap(), 0x808000, 8)
            .unwrap();
    assert_eq!(&bytes[8..10], &[0xa5, 0xa5]);
    assert!(result.cases.contains("preexisting-reference"));
}

#[test]
fn malformed_streams_fail_at_checked_memory_boundaries() {
    for (stream, destination, error) in [
        (vec![0xe0], 4096, DecompressError::InvalidSource(0x808001)),
        (vec![0x40, 0x12], 4096, DecompressError::InvalidSource(0x808002)),
        (vec![0xc0, 1, 0xff], 0, DecompressError::WramBounds(usize::MAX)),
        (vec![0x80, 1, 0, 0xff], WRAM_SIZE - 1, DecompressError::WramBounds(WRAM_SIZE)),
        (vec![0x00, 0x12], 4096, DecompressError::InvalidSource(0x808002)),
    ] {
        let mut bytes = vec![0xa5; WRAM_SIZE];
        assert_eq!(
            decompress_to_mem(
                &RomView::new(&stream).unwrap(),
                &mut WramView::new(&mut bytes).unwrap(),
                0x808000,
                destination
            )
            .unwrap_err(),
            error
        );
    }
}

#[test]
fn observer_retains_repeated_transient_stores() {
    let mut bytes = vec![0; decomp::WRAM_SIZE];
    let mut writes = Vec::new();
    {
        let mut observer = |offset, value| writes.push((offset, value));
        let mut ram = decomp::WramView::with_observer(&mut bytes, &mut observer).unwrap();
        ram.write(7, 42).unwrap();
        ram.write(7, 0).unwrap();
    }
    assert_eq!(writes, vec![(7, 42), (7, 0)]);
    assert_eq!(bytes[7], 0);
}
