use super::*;

fn put(bytes: &mut [u8], field: source::WramWord, value: u16) {
    let at = usize::from(field.offset);
    bytes
        .get_mut(at..at.saturating_add(2))
        .unwrap()
        .copy_from_slice(&value.to_le_bytes());
}

#[test]
fn current_external_inputs_do_not_consume_unqualified_pose_or_playhead() {
    let mut bytes = vec![0; 131_072];
    put(&mut bytes, source::JOYPAD1_LASTKEYS, 0x8100);
    put(&mut bytes, source::JOYPAD1_NEWKEYS, 0x0080);
    put(&mut bytes, source::EQUIPPED_ITEMS, 0x2000);
    put(&mut bytes, source::NMI_FRAME_COUNTER_WORD, 3);
    put(&mut bytes, source::CINEMATIC_ACTIVE, 1);
    put(&mut bytes, source::ELEVATOR_STATUS, 2);
    put(&mut bytes, source::SAMUS_POSE, u16::MAX);
    put(&mut bytes, source::SAMUS_ANIM_FRAME, u16::MAX);
    let external = current(&bytes).unwrap();
    assert!(external.pad.any(types::button::RIGHT) && external.pad.any(types::button::RUN));
    assert!(external.pressed.any(types::button::JUMP));
    assert!(external.items[Item::SpeedBooster]);
    assert_eq!(external.parity, Parity::Odd);
    assert!(external.cinematic_active && external.elevator_moving);
    let actual = external.movement(
        Sprite::Plain,
        samus_appearance::Ahead {
            end: Some(types::ClipEnd::End),
            turned_far: true,
        },
    );
    assert_eq!(actual.sprite_before_movement, Sprite::Plain);
    assert_eq!(actual.animation_ends, Some(types::ClipEnd::End));
    assert!(actual.spin_turned_far);
    assert_eq!(current(&bytes[..1]), Err(PlayerInputFault::Length));
}

#[test]
fn later_current_controls_are_distinct_from_the_retained_early_read() {
    let mut bytes = vec![0; 131_072];
    put(&mut bytes, source::JOYPAD1_LASTKEYS, 0x0100);
    let earlier = current(&bytes).unwrap();
    put(&mut bytes, source::JOYPAD1_LASTKEYS, 0x0200);
    let later = current(&bytes).unwrap();
    assert!(earlier.pad.any(types::button::RIGHT));
    assert!(later.pad.any(types::button::LEFT));
    assert!(!later.pad.any(types::button::RIGHT));
}
