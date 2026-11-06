use super::*;

fn decoded(stream: &[u8]) -> Vec<u8> {
    let result = decompress(stream).unwrap();
    assert_eq!(
        result.consumed,
        stream.len(),
        "the terminator ends the stream"
    );
    result.bytes
}

#[test]
fn an_empty_stream_is_its_terminator() {
    assert_eq!(decoded(&[0xff]), Vec::<u8>::new());
}

#[test]
fn direct_copy_takes_length_plus_one_bytes() {
    assert_eq!(decoded(&[0x02, 1, 2, 3, 0xff]), [1, 2, 3]);
}

#[test]
fn byte_fill_repeats_one_byte() {
    assert_eq!(decoded(&[0x23, 0xaa, 0xff]), [0xaa; 4]);
}

#[test]
fn word_fill_alternates_and_may_end_on_the_first_byte() {
    assert_eq!(decoded(&[0x44, 1, 2, 0xff]), [1, 2, 1, 2, 1]);
}

#[test]
fn sigma_fill_counts_up_and_wraps() {
    assert_eq!(decoded(&[0x62, 0xfe, 0xff]), [0xfe, 0xff, 0x00]);
}

#[test]
fn dictionary_copy_reads_from_the_output_start() {
    // Write 1 2 3 4, then copy two bytes from offset 1.
    assert_eq!(
        decoded(&[0x03, 1, 2, 3, 4, 0x81, 0x01, 0x00, 0xff]),
        [1, 2, 3, 4, 2, 3]
    );
}

#[test]
fn inverted_dictionary_copy_inverts_each_byte() {
    assert_eq!(
        decoded(&[0x01, 0x0f, 0xf0, 0xa1, 0x00, 0x00, 0xff]),
        [0x0f, 0xf0, 0xf0, 0x0f]
    );
}

#[test]
fn sliding_copy_overlaps_its_own_output() {
    // One byte, then copy five from one back: the run repeats the byte just written.
    assert_eq!(decoded(&[0x00, 7, 0xc4, 0x01, 0xff]), [7; 6]);
}

#[test]
fn inverted_sliding_copy_inverts_from_a_distance() {
    // Code 7 has only the extended header: a short header of 111 is the extended marker.
    assert_eq!(
        decoded(&[0x01, 0x12, 0x34, 0xfc, 0x01, 0x02, 0xff]),
        [0x12, 0x34, 0xed, 0xcb]
    );
}

#[test]
fn the_extended_header_has_a_ten_bit_length() {
    // 111 001 01, 0x00: byte fill, length 0x100 + 1.
    let bytes = decoded(&[0xe5, 0x00, 0x55, 0xff]);
    assert_eq!(bytes, vec![0x55; 0x101]);
}

#[test]
fn the_extended_header_can_name_the_inverted_sliding_copy() {
    // 111 111 00, 0x01: inverted sliding copy of two bytes from one back.
    assert_eq!(
        decoded(&[0x00, 0x0f, 0xfc, 0x01, 0x01, 0xff]),
        [0x0f, 0xf0, 0x0f]
    );
}

#[test]
fn bytes_after_the_terminator_are_not_read() {
    let result = decompress(&[0x21, 9, 0xff, 1, 2, 3]).unwrap();
    assert_eq!(result.bytes, [9, 9]);
    assert_eq!(result.consumed, 3);
}

#[test]
fn a_missing_terminator_is_truncation() {
    assert_eq!(
        decompress(&[0x20, 9]),
        Err(DecompressError::Truncated { consumed: 2 })
    );
    assert_eq!(
        decompress(&[0x02, 1]),
        Err(DecompressError::Truncated { consumed: 2 })
    );
}

#[test]
fn a_copy_past_the_output_is_an_unwritten_reference() {
    assert_eq!(
        decompress(&[0x00, 1, 0x80, 0x05, 0x00, 0xff]),
        Err(DecompressError::UnwrittenReference {
            position: 1,
            source: Some(5)
        })
    );
    assert_eq!(
        decompress(&[0x00, 1, 0xc0, 0x02, 0xff]),
        Err(DecompressError::UnwrittenReference {
            position: 1,
            source: None
        })
    );
}

#[test]
fn output_past_a_bank_is_too_long() {
    // 65 fills of 0x400 bytes pass 0x10000.
    let mut stream: Vec<u8> = core::iter::repeat_n([0xe7, 0xff, 0x00], 65)
        .flatten()
        .collect();
    stream.push(0xff);
    assert_eq!(decompress(&stream), Err(DecompressError::TooLong));
}
