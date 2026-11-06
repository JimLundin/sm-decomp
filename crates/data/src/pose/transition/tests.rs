use super::*;
use crate::rom::Rom;

/// An entry's six bytes.
fn entry(pressed: u16, held: u16, pose: u16) -> Vec<u8> {
    [pressed, held, pose]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect()
}

#[test]
fn a_list_ends_at_its_end_word() {
    let mut bytes = entry(0x0080, 0x0200, 0x4D);
    bytes.extend(entry(0, 0x0100, 0x09));
    bytes.extend([0xFF, 0xFF]);
    bytes.extend(entry(0, 0, 0x01));
    assert_eq!(
        list(&bytes),
        Ok(vec![
            Transition {
                pressed: Controls::of(&[Control::Jump]),
                held: Controls::of(&[Control::Left]),
                pose: PoseId(0x4D),
            },
            Transition {
                pressed: Controls::NONE,
                held: Controls::of(&[Control::Right]),
                pose: PoseId(0x09),
            },
        ])
    );
    assert_eq!(list(&[0xFF, 0xFF]), Ok(Vec::new()));
}

#[test]
fn a_list_without_its_end_word_is_an_error() {
    assert!(list(&entry(0, 0, 1)).is_err());
}

#[test]
fn an_entry_needs_every_pressed_and_held_control() {
    let jump_left = Transition {
        pressed: Controls::of(&[Control::Jump]),
        held: Controls::of(&[Control::Left, Control::Run]),
        pose: PoseId(0x1A),
    };
    let held = Controls::of(&[Control::Left, Control::Run, Control::Jump]);
    assert!(jump_left.matches(Controls::of(&[Control::Jump]), held));
    assert!(!jump_left.matches(Controls::NONE, held));
    assert!(!jump_left.matches(
        Controls::of(&[Control::Jump]),
        Controls::of(&[Control::Left, Control::Jump])
    ));
}

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_transition_table_decodes_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let table = PoseTransitions::load(cartridge.lorom()).unwrap();
    // Every entry names a pose the pose table defines.
    for number in 0..u16::try_from(POSE_COUNT).unwrap() {
        for transition in table.of(PoseId(number)) {
            assert!(usize::from(transition.pose.0) < POSE_COUNT, "{number:#X}");
        }
    }
    // Pose 1, standing facing right: holding left turns her around ($25).
    assert!(
        table
            .of(PoseId(1))
            .iter()
            .any(|t| t.pose == PoseId(0x25) && t.held == Controls::of(&[Control::Left]))
    );
    // Holding right runs ($09).
    assert!(
        table
            .of(PoseId(1))
            .iter()
            .any(|t| t.pose == PoseId(0x09) && t.held == Controls::of(&[Control::Right]))
    );
}
