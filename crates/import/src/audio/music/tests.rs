use super::*;
use crate::audio::tests::blocks;

#[test]
fn opening_song_refs_are_bounded_and_unsupported_controls_are_rejected() {
    let mut track = [0; 46];
    track[0..2].copy_from_slice(&0x5830_u16.to_le_bytes());
    track[8..16].copy_from_slice(&[0x38, 0x58, 0xff, 0, 0x30, 0x58, 0, 0]);
    track[16..20].copy_from_slice(&[0x48, 0x58, 0x4e, 0x58]);
    track[32..38].copy_from_slice(&[0xf7, 2, 0, 0, 0x94, 0]);
    track[38..42].copy_from_slice(&[4, 0x32, 0x96, 0]);
    let import = |track: &[u8]| {
        let image = blocks(&[(0x5800, &[200; 24]), (0x5828, track)]);
        let song = Upload::parse(&image).unwrap();
        let common = Upload::parse(&[0; 4]).unwrap();
        decode(&song, 5, &mut Resources::new(&common, &song))
    };

    let (tracks, delay) = import(&track).unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(delay, 2);
    assert!(matches!(tracks[1].phrases[0][0], MusicEvent::Timing {
        duration: MusicTurns(length), articulation: Some(Articulation { ratio: 200, strength: 200 }),
        ..
    } if length.get() == 4));

    let mut malformed = track;
    malformed[18..20].copy_from_slice(&0x6800_u16.to_le_bytes());
    assert_eq!(import(&malformed), Err(ImportError::Pointer));

    let mut unsupported = track;
    unsupported[36] = 0xf1;
    assert_eq!(import(&unsupported), Err(ImportError::Music));

    let mut third_track = track;
    third_track[20] = 0x40;
    assert_eq!(import(&third_track), Err(ImportError::Arrangement));

    let mut finite = track;
    finite[10] = 2;
    assert_eq!(import(&finite), Err(ImportError::Arrangement));

    // These directory words happen to spell a complete arrangement and its target.
    let mut aliased = [0; 40];
    aliased[0..2].copy_from_slice(&0x582a_u16.to_le_bytes());
    aliased[2..10].copy_from_slice(&[0x32, 0x58, 0xff, 0, 0x2a, 0x58, 0, 0]);
    aliased[10..14].copy_from_slice(&[0x42, 0x58, 0x48, 0x58]);
    aliased[26..32].copy_from_slice(&track[32..38]);
    aliased[32..36].copy_from_slice(&track[38..42]);
    assert_eq!(import(&aliased), Err(ImportError::Pointer));
}
