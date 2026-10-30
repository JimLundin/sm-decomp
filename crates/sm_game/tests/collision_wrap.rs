//! $94's level word reads past the room wrap like the original's long indexed read.
use sm_game::player_collision::geometry::{level_offset, level_word_bytes};

#[test]
fn out_of_room_level_words_wrap_like_the_long_indexed_read() {
    // Block 0xFFFF's X wraps to 0xFFFE in the 16-bit ASL, and $7F:0002 + X
    // reads $80:0000-$80:0001, the WRAM mirror of $7E:0000.
    let x = level_offset(0xffff);
    assert_eq!(x, 0xfffe);
    assert_eq!(level_word_bytes(x), [0, 1]);
    // The last word inside bank $7F stays there.
    assert_eq!(level_word_bytes(0xfffc), [0x1fffe, 0x1ffff]);
    // Block 0x8001 reads the same word as block 1.
    let x = level_offset(0x8001);
    assert_eq!(x, 2);
    assert_eq!(level_word_bytes(x), [0x10004, 0x10005]);
}
