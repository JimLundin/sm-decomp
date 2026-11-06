use super::*;
use crate::rom::Rom;

#[test]
fn every_movement_type_round_trips_and_1c_is_an_error() {
    for value in 0..0x1C_u8 {
        assert_eq!(MovementType::try_from(value).map(u8::from), Ok(value));
    }
    assert_eq!(MovementType::try_from(0x1C), Err(0x1C));
}

#[test]
fn every_shot_byte_round_trips_and_a_to_f_are_errors() {
    for value in (0..10_u8).chain(0x10..=0xFF) {
        assert_eq!(Shot::try_from(value).map(u8::from), Ok(value));
    }
    for value in 10..0x10_u8 {
        assert_eq!(Shot::try_from(value), Err(value));
    }
    assert_eq!(Shot::try_from(0xFA), Ok(Shot::Withheld(0xFA)));
    assert_eq!(
        Shot::try_from(9),
        Ok(Shot::Fires(ShotDirection::UpFacingLeft))
    );
}

#[test]
fn every_facing_byte_round_trips() {
    for value in 0..=0xFF_u8 {
        assert_eq!(u8::from(PoseFacing::from(value)), value);
    }
    assert_eq!(PoseFacing::from(2), PoseFacing::Unknown(2));
}

#[test]
fn an_entry_decodes_field_by_field() {
    // Pose 2 and pose $A6's layout: left, movement, next, shot, y offset, -, radius, -.
    let poses = Poses::decode(&[
        0x04, 0x00, 0xFF, 0x07, 0x06, 0x00, 0x15, 0x00, // facing left, normal
        0x08, 0x0F, 0x01, 0xFA, 0xFC, 0x00, 0x15, 0x00, // a transition
        0x00, // a partial entry
    ])
    .unwrap();
    assert_eq!(poses.len(), 2);
    assert_eq!(
        poses.get(PoseId(0)),
        Some(&PoseDefinition {
            facing: PoseFacing::Left,
            movement: MovementType::Standing,
            next: None,
            shot: Shot::Fires(ShotDirection::Left),
            y_offset: 6,
            y_radius: 0x15,
        })
    );
    assert_eq!(
        poses.get(PoseId(1)),
        Some(&PoseDefinition {
            facing: PoseFacing::Right,
            movement: MovementType::CrouchingTransition,
            next: Some(PoseId(1)),
            shot: Shot::Withheld(0xFA),
            y_offset: -4,
            y_radius: 0x15,
        })
    );
    assert_eq!(poses.get(PoseId(2)), None);
}

#[test]
fn a_bad_movement_type_or_shot_names_its_pose() {
    let mut bytes = [0_u8; 16];
    bytes[9] = 0x1C;
    assert_eq!(
        Poses::decode(&bytes),
        Err(PoseError::MovementType {
            pose: PoseId(1),
            value: 0x1C
        })
    );
    bytes[9] = 0;
    bytes[3] = 0x0A;
    assert_eq!(
        Poses::decode(&bytes),
        Err(PoseError::Shot {
            pose: PoseId(0),
            value: 0x0A
        })
    );
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_pose_table_decodes_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let poses = Poses::load(cartridge.lorom()).unwrap();
    assert_eq!(poses.len(), POSE_COUNT);
    // $91:B629, pose 0: facing forward, power suit.
    assert_eq!(
        poses.get(PoseId(0)),
        Some(&PoseDefinition {
            facing: PoseFacing::Forward,
            movement: MovementType::Standing,
            next: None,
            shot: Shot::Withheld(0xFF),
            y_offset: 8,
            y_radius: 0x18,
        })
    );
    // $91:B641, pose 3: facing right, aiming up; becomes pose 1 without the button.
    assert_eq!(
        poses.get(PoseId(3)),
        Some(&PoseDefinition {
            facing: PoseFacing::Right,
            movement: MovementType::Standing,
            next: Some(PoseId(1)),
            shot: Shot::Fires(ShotDirection::UpFacingRight),
            y_offset: 6,
            y_radius: 0x15,
        })
    );
    // $91:B739, pose $22: unused, X direction 2.
    assert_eq!(
        poses.get(PoseId(0x22)).map(|pose| pose.facing),
        Some(PoseFacing::Unknown(2))
    );
    // $91:BE09, pose $FC: facing left, standing transition, aiming down-left.
    assert_eq!(
        poses.get(PoseId(0xFC)),
        Some(&PoseDefinition {
            facing: PoseFacing::Left,
            movement: MovementType::CrouchingTransition,
            next: None,
            shot: Shot::Fires(ShotDirection::DownLeft),
            y_offset: 3,
            y_radius: 0x15,
        })
    );
    let forward = (0..0xFD_u16)
        .filter(|&number| {
            poses.get(PoseId(number)).map(|pose| pose.facing) == Some(PoseFacing::Forward)
        })
        .collect::<Vec<_>>();
    assert_eq!(forward, [0, 0x9B]);
}
