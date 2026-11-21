use super::*;
use crate::audio::tests::blocks;

fn instrument_source() -> Vec<u8> {
    blocks(&[
        (0x6c00, &[0, 0x80, 0, 0, 1, 0]),
        (0x6d00, &[0, 0x6e, 0, 0]),
        (0x6e00, &[1, 0, 0, 0, 0, 0, 0, 0, 0]),
    ])
}

#[test]
fn physical_parts_and_cue_aliases_belong_to_their_own_list() {
    let image = instrument_source();
    let common = Upload::parse(&image).unwrap();
    let area = Upload::parse(&[0; 4]).unwrap();
    let mut resources = Resources::new(&common, &area);
    let mut used = Vec::new();
    let definition = cue(SoundGroup::Contact, 1, CueId::Land, 2);
    let source = [
        0x04, 1, 0x04, 1, 0x08, 1, 0x0e, 1, 0, 30, 30, 0x93, 5, 0xff, 0, 30, 30, 0x99, 6, 0xff,
    ];
    let image = blocks(&[(0x100, &source)]);
    let upload = Upload::parse(&image).unwrap();
    let region = upload.block(0x100).unwrap();
    let content = clip(region, 0x104, definition, &mut resources, &mut used).unwrap();
    assert_eq!(content.parts.len(), 2);
    assert_eq!(content.parts[0].spans[0].duration.0.get(), 5);
    assert_eq!(content.parts[1].spans[0].duration.0.get(), 6);

    let duplicate = cue(SoundGroup::Contact, 2, CueId::Door, 2);
    assert_eq!(
        clip(region, 0x104, duplicate, &mut resources, &mut used),
        Err(ImportError::Cue)
    );

    let wrong_arity = cue(SoundGroup::Contact, 1, CueId::Land, 1);
    assert_eq!(
        clip(region, 0x104, wrong_arity, &mut resources, &mut Vec::new()),
        Err(ImportError::Cue)
    );

    let mut unsupported = source;
    unsupported[8] = 0xfc;
    let image = blocks(&[(0x100, &unsupported)]);
    let upload = Upload::parse(&image).unwrap();
    let broken = upload.block(0x100).unwrap();
    assert_eq!(
        clip(broken, 0x104, definition, &mut resources, &mut Vec::new()),
        Err(ImportError::Effect)
    );
}

#[test]
fn selected_library_cannot_be_read_from_the_common_upload_if_the_area_replaces_it() {
    let contact_directory = library(SoundGroup::Contact).range.start;
    let offset = contact_directory + 2 * 5;
    let area_image = blocks(&[(u16::try_from(offset).unwrap(), &[0])]);
    let area = Upload::parse(&area_image).unwrap();
    let common = Upload::parse(&[0; 4]).unwrap();
    let mut resources = Resources::new(&common, &area);
    assert_eq!(
        decode(&common, &area, &mut resources),
        Err(ImportError::Cue)
    );
}

#[test]
fn cue_directory_slots_cannot_supply_part_lists() {
    let source = [2, 1, 4, 1, 0, 30, 30, 0x93, 5, 0xff, 0, 0];
    let library = blocks(&[(0x100, &source)]);
    let upload = Upload::parse(&library).unwrap();
    let image = instrument_source();
    let common = Upload::parse(&image).unwrap();
    let area = Upload::parse(&[0; 4]).unwrap();
    let mut resources = Resources::new(&common, &area);
    let region = upload.block(0x100).unwrap();
    let cue = cue(SoundGroup::Action, 1, CueId::Beam, 1);
    assert_eq!(
        clip(region, 0x10c, cue, &mut resources, &mut Vec::new()),
        Err(ImportError::Pointer)
    );
}
