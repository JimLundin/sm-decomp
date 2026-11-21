use super::*;

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    Some(Rom::new(std::fs::read(path).ok()?).unwrap())
}

#[test]
fn corridor_carries_the_arrangement_selected_at_its_entrance() {
    let Some(rom) = rom() else { return };
    let opening = resident_ceres(&rom, CERES_ELEVATOR_ROOM).unwrap();
    let carried = resident_ceres(&rom, MAGNET_STAIRS_ROOM).unwrap();
    let entered = resident_ceres(&rom, DEAD_SCIENTIST_ROOM).unwrap();
    assert_eq!(opening, carried);
    assert_eq!(opening, entered);
    assert_eq!(opening.music_reply.original_byte, 6);
    assert_eq!(opening.content.tracks.len(), 2);
    assert_eq!(opening.content.effects.len(), 8);
    assert_eq!(opening.instruments.len(), 8);
    assert_eq!(opening.samples.len(), 8);
    let outside = data::assets::world::rooms::MORPH_BALL_ROOM;
    assert_eq!(
        resident_ceres(&rom, outside),
        Err(ImportError::RoomSelection(outside))
    );
}

#[test]
fn upload_must_fit_without_truncation_or_conflicting_destinations() {
    use ImportError::Upload as Invalid;

    assert!(matches!(Upload::parse(&[1, 0, 0, 0]), Err(Invalid)));
    assert!(matches!(
        Upload::parse(&[2, 0, 255, 255, 0, 0, 0, 0, 0, 0]),
        Err(Invalid)
    ));
    assert!(matches!(
        Upload::parse(&[1, 0, 0, 0, 1, 1, 0, 0, 0, 2, 0, 0, 0, 0]),
        Err(Invalid)
    ));
    let blocks = Upload::parse(&[2, 0, 6, 0, 1, 2, 0, 0, 0, 0]).unwrap();
    let region = blocks.slice(6..8).unwrap();
    assert_eq!(region.word(6), Ok(513));
    assert_eq!(region.word(7), Err(ImportError::Pointer));
}

pub(super) fn blocks(entries: &[(u16, &[u8])]) -> Vec<u8> {
    let mut image = Vec::new();
    for (at, bytes) in entries {
        image.extend(u16::try_from(bytes.len()).unwrap().to_le_bytes());
        image.extend(at.to_le_bytes());
        image.extend_from_slice(bytes);
    }
    image.extend([0; 4]);
    image
}
