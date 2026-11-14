use super::*;
use data::rom::Rom;

const DIRECTIONS: [Heading; 10] = [
    Heading::UpFacingRight,
    Heading::UpRight,
    Heading::Right,
    Heading::DownRight,
    Heading::DownFacingRight,
    Heading::DownFacingLeft,
    Heading::DownLeft,
    Heading::Left,
    Heading::UpLeft,
    Heading::UpFacingLeft,
];

fn put(image: &mut [u8], bank: u8, address: u16, bytes: &[u8]) {
    let offset = usize::from(bank & 0x7f)
        .checked_mul(32768)
        .unwrap()
        .checked_add(usize::from(address & 0x7fff))
        .unwrap();
    image[offset..offset.checked_add(bytes.len()).unwrap()].copy_from_slice(bytes);
}

fn put_word(image: &mut [u8], bank: u8, address: u16, value: u16) {
    put(image, bank, address, &value.to_le_bytes());
}

fn fixture() -> Vec<u8> {
    let mut image = vec![0; Rom::SIZE];
    put_word(&mut image, 0x93, 0x83c1, 0x9000);
    put_word(&mut image, 0x93, 0x9000, 0);
    put_word(&mut image, 0x93, 0x867b, 0x9400);
    put_word(&mut image, 0x9b, 0xa4b3, 0x9100);
    for (index, _) in DIRECTIONS.iter().enumerate() {
        let index = u16::try_from(index).unwrap();
        let clip = advance(0x9200, index.checked_mul(16).unwrap()).unwrap();
        let picture = advance(0x9500, index.checked_mul(8).unwrap()).unwrap();
        put_word(
            &mut image,
            0x93,
            advance(0x9002, index.checked_mul(2).unwrap()).unwrap(),
            clip,
        );
        put(
            &mut image,
            0x93,
            clip,
            &[
                0,
                0,
                picture.to_le_bytes()[0],
                picture.to_le_bytes()[1],
                u8::try_from(index).unwrap(),
                255,
                0,
                0,
            ],
        );
        put_word(&mut image, 0x93, advance(clip, 8).unwrap(), 0x8239);
        put_word(&mut image, 0x93, advance(clip, 10).unwrap(), clip);
        put(
            &mut image,
            0x93,
            picture,
            &[1, 0, 0xf9, 0x81, 0xfe, 0x12, 0x6c],
        );
        put_word(
            &mut image,
            0x9b,
            advance(0x9100, index.checked_mul(2).unwrap()).unwrap(),
            advance(0x9200, index.checked_mul(4).unwrap()).unwrap(),
        );
        put(
            &mut image,
            0x9b,
            advance(0x9200, index.checked_mul(4).unwrap()).unwrap(),
            &[u8::try_from(index).unwrap(), 0x80, 0x7f, 0xff],
        );
        for (base, value) in [(0xc204, -23_i16), (0xc218, 31), (0xc22c, -41), (0xc240, 57)] {
            put(
                &mut image,
                0x90,
                advance(base, index.checked_mul(2).unwrap()).unwrap(),
                &value
                    .wrapping_add(i16::try_from(index).unwrap())
                    .to_le_bytes(),
            );
        }
    }
    // Two distinct pictures, including a maximum legal hold and an unsigned radius edge.
    put(
        &mut image,
        0x93,
        0x9400,
        &[0xff, 0x7f, 0, 0x95, 255, 0, 0, 0],
    );
    put(&mut image, 0x93, 0x9408, &[3, 0, 8, 0x95, 2, 7, 0, 0]);
    put_word(&mut image, 0x93, 0x9410, 0x822f);
    for entry in &map::MAP {
        put(
            &mut image,
            0x91,
            advance(
                POSE_DEFINITIONS.offset,
                entry.id.0.checked_mul(8).unwrap().checked_add(3).unwrap(),
            )
            .unwrap(),
            &[0x10, 0xf0],
        );
    }
    image
}

#[test]
fn invented_content_keeps_heading_offsets_holds_pictures_and_unsigned_lift() {
    let image = fixture();
    let imported = import(Lorom::new(&image)).unwrap();
    let power = &imported.content.power;
    assert!(power.damage.points() == 0, "invented field equality");
    assert!(
        power.impact.frames[0].hold.updates() == 32767,
        "invented field equality"
    );
    assert!(power.impact.frames.len() == 2, "invented field equality");
    assert!(
        power.impact.end == ClipEnd::Remove,
        "invented field equality"
    );
    for (index, heading) in DIRECTIONS.iter().copied().enumerate() {
        let clip = power.flight.get(heading);
        assert!(
            clip.frames[0].hold.updates() == 0,
            "invented field equality"
        );
        assert!(
            clip.frames[0].radius
                == Radius {
                    x: u8::try_from(index).unwrap(),
                    y: 255
                },
            "heading radius"
        );
        assert!(
            clip.end == ClipEnd::RepeatFrom(FrameNumber(0)),
            "invented field equality"
        );
        let appearance = power.trails[usize::from(clip.frames[0].trail_appearance.0)];
        assert!(
            appearance
                == TrailAppearance::PowerBeam(TrailOffsets {
                    left: Offset {
                        x: i16::try_from(index).unwrap() - 4,
                        y: -132
                    },
                    right: Offset { x: 123, y: -5 },
                }),
            "invented field equality"
        );
        let picture = power.sheet.sprite(clip.frames[0].picture).unwrap();
        assert!(picture.entries[0].x == -7, "invented field equality");
        assert!(picture.entries[0].y == 254, "invented field equality");
        for muzzle in &imported.content.muzzles {
            assert!(
                muzzle.aim == MuzzleAim::PreviousAim,
                "invented field equality"
            );
            assert!(muzzle.lift == 240, "invented field equality");
            assert!(
                muzzle.stationary.get(heading).x == -23 + i16::try_from(index).unwrap(),
                "invented field equality"
            );
            assert!(
                muzzle.moving.get(heading).y == 57 + i16::try_from(index).unwrap(),
                "invented field equality"
            );
        }
    }
}

#[test]
fn malformed_content_is_refused_by_kind() {
    let cases = [
        (0x93, 0x9000, 0x8000, ImportError::Damage),
        (0x93, 0x9200, 0x8001, ImportError::Control),
        (0x93, 0x9200, 0x822f, ImportError::EmptyClip),
        (0x93, 0x920a, 0x9201, ImportError::RepeatTarget),
        (0x93, 0x9206, 1, ImportError::Identifier),
        (0x93, 0x9210, 0x8239, ImportError::RepeatTarget),
    ];
    for (bank, address, value, error) in cases {
        let mut image = fixture();
        put_word(&mut image, bank, address, value);
        assert!(matches!(import(Lorom::new(&image)), Err(actual) if actual == error));
    }
    let mut image = fixture();
    put_word(&mut image, 0x93, 0x9410, 0x8239);
    put_word(&mut image, 0x93, 0x9412, 0x9400);
    assert!(matches!(
        import(Lorom::new(&image)),
        Err(ImportError::RepeatingImpact)
    ));
    image.truncate(
        0x13_usize
            .checked_mul(32768)
            .unwrap()
            .checked_add(0x9205 & 0x7fff)
            .unwrap(),
    );
    assert!(matches!(
        import(Lorom::new(&image)),
        Err(ImportError::Address(_))
    ));
}

#[test]
fn every_heading_and_aim_descriptor_is_named() {
    for (code, heading) in DIRECTIONS.iter().copied().enumerate() {
        assert!(
            muzzle_aim(u8::try_from(code).unwrap()) == Ok(MuzzleAim::Aimed(heading)),
            "invented field equality"
        );
    }
    assert!(
        muzzle_aim(0x10) == Ok(MuzzleAim::PreviousAim),
        "invented field equality"
    );
    assert!(
        muzzle_aim(0xff) == Ok(MuzzleAim::Forbidden),
        "invented field equality"
    );
    assert!(
        muzzle_aim(0x0a) == Err(ImportError::Aim),
        "invented field equality"
    );
}

#[test]
fn owned_content_validation_checks_references_and_complete_pose_coverage() {
    let mut content = import(Lorom::new(&fixture())).unwrap().content;
    content.power.flight.right.end = ClipEnd::RepeatFrom(FrameNumber(1));
    assert!(
        validate(&content) == Err(ImportError::RepeatTarget),
        "invented field equality"
    );
    content.power.flight.right.end = ClipEnd::RepeatFrom(FrameNumber(0));
    content.power.flight.right.frames[0].picture = SpriteId(u16::MAX);
    assert!(
        validate(&content) == Err(ImportError::Identifier),
        "invented field equality"
    );
    content.power.flight.right.frames[0].picture = SpriteId(0);
    content.power.flight.right.frames[0].trail_appearance = TrailAppearanceId(u16::MAX);
    assert!(
        validate(&content) == Err(ImportError::Identifier),
        "invented field equality"
    );
    content.power.flight.right.frames[0].trail_appearance = TrailAppearanceId(0);
    content.muzzles.push(content.muzzles[0].clone());
    assert!(
        validate(&content) == Err(ImportError::PoseConflict),
        "invented field equality"
    );
    content.muzzles.pop();
    content.muzzles.pop();
    assert!(
        validate(&content) == Err(ImportError::Identifier),
        "invented field equality"
    );
}

#[test]
fn impact_has_no_trail_selector_and_pose_dress_conflicts_are_refused() {
    let mut image = fixture();
    put_word(&mut image, 0x93, 0x9406, u16::MAX);
    assert!(import(Lorom::new(&image)).is_ok());
    let duplicate = map::MAP
        .iter()
        .find(|entry| {
            map::MAP
                .iter()
                .any(|other| other.id != entry.id && other.key == entry.key)
        })
        .unwrap();
    put(
        &mut image,
        0x91,
        advance(
            POSE_DEFINITIONS.offset,
            duplicate
                .id
                .0
                .checked_mul(8)
                .unwrap()
                .checked_add(4)
                .unwrap(),
        )
        .unwrap(),
        &[1],
    );
    assert!(matches!(
        import(Lorom::new(&image)),
        Err(ImportError::PoseConflict)
    ));
}

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    if !path.exists() {
        return None;
    }
    let image = std::fs::read(path).unwrap();
    assert!(Rom::new(image.clone()).is_ok(), "ROM identity");
    Some(Rom::new(image).unwrap())
}

#[test]
fn vanilla_import_reconstructs_every_imported_field_without_printing_content() {
    let Some(cartridge) = rom() else {
        return;
    };
    let rom = cartridge.lorom();
    let imported = import(rom).unwrap();
    let power = &imported.content.power;
    let descriptor = word(rom, 0x93, 0x83c1).unwrap();
    assert!(imported.places.clips.len() == 11, "complete clip coverage");
    assert!(
        power.damage.points() == word(rom, 0x93, descriptor).unwrap(),
        "damage"
    );
    check_clips(rom, &imported, descriptor);
    check_muzzles(rom, &imported.content);
    check_pictures(rom, &imported);
}

fn check_clips(rom: Lorom<'_>, imported: &Imported, descriptor: u16) {
    let power = &imported.content.power;
    for places in &imported.places.clips {
        let clip = power.clip(places.id);
        let expected_start = match places.id {
            ClipId::Impact => word(rom, 0x93, 0x867b).unwrap(),
            ClipId::Flight(heading) => {
                let index = DIRECTIONS
                    .iter()
                    .position(|&direction| direction == heading)
                    .unwrap();
                word(
                    rom,
                    0x93,
                    advance(
                        descriptor,
                        u16::try_from(index.checked_mul(2).unwrap())
                            .unwrap()
                            .checked_add(2)
                            .unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap()
            }
        };
        assert!(
            places.frames.first().copied() == Some(expected_start),
            "heading clip identity"
        );
        assert!(clip.frames.len() == places.frames.len(), "frame coverage");
        for (frame, &pointer) in clip.frames.iter().zip(&places.frames) {
            let source = rom.read(RomAddress::new(0x93, pointer), 8).unwrap();
            let picture = imported.places.pictures[usize::from(frame.picture.0)];
            let rebuilt = [
                frame.hold.updates().to_le_bytes()[0],
                frame.hold.updates().to_le_bytes()[1],
                picture.to_le_bytes()[0],
                picture.to_le_bytes()[1],
                frame.radius.x,
                frame.radius.y,
                0,
                0,
            ];
            let observed = if places.id == ClipId::Impact { 6 } else { 8 };
            assert!(
                source[..observed] == rebuilt[..observed],
                "clip frame reconstruction"
            );
            check_trail(
                rom,
                power.trails[usize::from(frame.trail_appearance.0)],
                places.id,
            );
        }
        match clip.end {
            ClipEnd::Remove => assert!(
                word(rom, 0x93, places.end).unwrap() == 0x822f,
                "remove ending"
            ),
            ClipEnd::RepeatFrom(from) => {
                assert!(
                    word(rom, 0x93, places.end).unwrap() == 0x8239,
                    "repeat ending"
                );
                assert!(
                    word(rom, 0x93, advance(places.end, 2).unwrap()).unwrap()
                        == places.frames[usize::from(from.0)],
                    "repeat target"
                );
            }
        }
    }
}

fn check_trail(rom: Lorom<'_>, trail: TrailAppearance, id: ClipId) {
    match id {
        ClipId::Impact => assert!(trail == TrailAppearance::Inactive, "impact appearance"),
        ClipId::Flight(heading) => {
            let table = word(rom, 0x9b, 0xa4b3).unwrap();
            let index = DIRECTIONS
                .iter()
                .position(|&direction| direction == heading)
                .unwrap();
            let pointer = word(
                rom,
                0x9b,
                advance(table, u16::try_from(index.checked_mul(2).unwrap()).unwrap()).unwrap(),
            )
            .unwrap();
            let bytes = rom.read(RomAddress::new(0x9b, pointer), 4).unwrap();
            let expected = TrailAppearance::PowerBeam(TrailOffsets {
                left: Offset {
                    x: i16::from(i8::from_le_bytes([bytes[0]]))
                        .checked_sub(4)
                        .unwrap(),
                    y: i16::from(i8::from_le_bytes([bytes[1]]))
                        .checked_sub(4)
                        .unwrap(),
                },
                right: Offset {
                    x: i16::from(i8::from_le_bytes([bytes[2]]))
                        .checked_sub(4)
                        .unwrap(),
                    y: i16::from(i8::from_le_bytes([bytes[3]]))
                        .checked_sub(4)
                        .unwrap(),
                },
            });
            assert!(trail == expected, "trail origins");
        }
    }
}

fn check_muzzles(rom: Lorom<'_>, content: &Content) {
    for entry in &map::MAP {
        let muzzle = content.muzzle(entry.key).unwrap();
        let source = rom
            .read(
                POSE_DEFINITIONS
                    .advance(usize::from(entry.id.0) * 8)
                    .unwrap(),
                8,
            )
            .unwrap();
        assert!(muzzle.aim == muzzle_aim(source[3]).unwrap(), "pose aim");
        assert!(muzzle.lift == source[4], "unsigned pose lift");
        for (index, heading) in DIRECTIONS.iter().copied().enumerate() {
            let offset = u16::try_from(index.checked_mul(2).unwrap()).unwrap();
            for (actual, x, y) in [
                (muzzle.stationary.get(heading), 0xc204, 0xc218),
                (muzzle.moving.get(heading), 0xc22c, 0xc240),
            ] {
                assert!(
                    actual.x.to_le_bytes()
                        == word(rom, 0x90, advance(x, offset).unwrap())
                            .unwrap()
                            .to_le_bytes(),
                    "muzzle x"
                );
                assert!(
                    actual.y.to_le_bytes()
                        == word(rom, 0x90, advance(y, offset).unwrap())
                            .unwrap()
                            .to_le_bytes(),
                    "muzzle y"
                );
            }
        }
    }
}

fn check_pictures(rom: Lorom<'_>, imported: &Imported) {
    for (&pointer, picture) in imported
        .places
        .pictures
        .iter()
        .zip(&imported.content.power.sheet.sprites)
    {
        let count = usize::from(word(rom, 0x93, pointer).unwrap());
        assert!(count == picture.entries.len(), "picture coverage");
        let source = rom
            .read(
                RomAddress::new(0x93, advance(pointer, 2).unwrap()),
                count.checked_mul(5).unwrap(),
            )
            .unwrap();
        for (sprite, bytes) in picture.entries.iter().zip(source.as_chunks::<5>().0) {
            let x = u16::from_le_bytes(sprite.x.to_le_bytes()) & 0x1ff;
            assert!(
                x == (u16::from_le_bytes([bytes[0], bytes[1]]) & 0x1ff),
                "picture x"
            );
            assert!(
                (sprite.size == data::spritemap::SpriteSize::Large) == (bytes[1] & 0x80 != 0),
                "picture size"
            );
            assert!(sprite.y == bytes[2], "picture y");
            assert!(
                sprite.attributes.word() == u16::from_le_bytes([bytes[3], bytes[4]]),
                "picture attributes"
            );
        }
    }
}
