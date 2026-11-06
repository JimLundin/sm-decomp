use super::*;

fn hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        write!(text, "{byte:02x}").unwrap();
        text
    })
}

#[test]
fn sha256_matches_the_standard_vectors() {
    assert_eq!(
        hex(&sha256(b"")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        hex(&sha256(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // 56 bytes need a second padding block; 64 bytes exactly fill one.
    assert_eq!(
        hex(&sha256(&[b'a'; 64])),
        "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
    );
}

#[test]
fn the_wrong_size_is_rejected_before_hashing() {
    assert_eq!(Rom::new(vec![0; 0x200]), Err(RomError::Size(0x200)));
}

#[test]
fn the_right_size_with_other_bytes_is_rejected_by_hash() {
    assert!(matches!(
        Rom::new(vec![0; Rom::SIZE]),
        Err(RomError::Hash(_))
    ));
}

#[test]
fn lorom_banks_follow_one_another_in_the_file() {
    let image: Vec<u8> = (0..0x1_0000_u32).map(|n| n.to_le_bytes()[1]).collect();
    let view = Lorom::new(&image);
    assert_eq!(view.file_offset(RomAddress::new(0x80, 0x8000)), Ok(0));
    assert_eq!(view.file_offset(RomAddress::new(0x81, 0x8123)), Ok(0x8123));
    // The $00-$7D mirror maps to the same bytes.
    assert_eq!(view.file_offset(RomAddress::new(0x01, 0x8123)), Ok(0x8123));
    assert_eq!(
        view.file_offset(RomAddress::new(0x80, 0x7fff)),
        Err(AddressError::NotRom(RomAddress::new(0x80, 0x7fff)))
    );
    assert_eq!(
        view.file_offset(RomAddress::new(0x82, 0x8000)),
        Err(AddressError::PastEnd(RomAddress::new(0x82, 0x8000)))
    );
    // A read runs on from the end of one bank into the next.
    assert_eq!(
        view.read(RomAddress::new(0x80, 0xffff), 2),
        Ok(&[0x7f, 0x80][..])
    );
}

#[test]
fn long_pointers_round_trip() {
    let address = RomAddress::from_long([0xd5, 0xde, 0xc5]);
    assert_eq!(address, RomAddress::new(0xc5, 0xded5));
    assert_eq!(address.long(), 0x00c5_ded5);
    assert_eq!(address.advance(2), Some(RomAddress::new(0xc5, 0xded7)));
    assert_eq!(RomAddress::new(0x8f, 0xffff).advance(1), None);
}
