use boss_ceres_ridley::{DoorKind, Figure, FigureLook, Phase, Quarter, Wait};
use types::Side;

use crate::enemy::ceres_door::{Unknown, figure, phase};

fn wait(frames: u16) -> Wait {
    Wait::new(frames).unwrap()
}

#[test]
fn the_decode_notes_pointers_name_each_phase() {
    let right = |at, timer| phase(DoorKind::Door, Side::Right, at, timer);
    let left = |at, timer| phase(DoorKind::Door, Side::Left, at, timer);
    let shaft = |at, timer| phase(DoorKind::ShaftDoor, Side::Left, at, timer);
    assert_eq!(right(0xF56C, 1), Ok(Phase::Placed));
    assert_eq!(left(0xF5BE, 1), Ok(Phase::Placed));
    assert_eq!(shaft(0xF61A, 1), Ok(Phase::Placed));
    assert_eq!(shaft(0xF626, 1), Ok(Phase::Wall));
    // Only the shaft's door is ever a wall.
    assert_eq!(left(0xF626, 1), Err(Unknown::Instruction(0xF626)));
    assert_eq!(right(0xF574, 2), Ok(Phase::Starting { wait: wait(2) }));
    assert_eq!(left(0xF5CE, 1), Ok(Phase::Open { wait: wait(1) }));
    assert_eq!(right(0xF5A0, 2), Ok(Phase::Closed { wait: wait(2) }));
    assert_eq!(
        right(0xF598, 3),
        Ok(Phase::Closing {
            quarter: Quarter::Fourth,
            wait: wait(3)
        })
    );
    assert_eq!(
        left(0xF600, 5),
        Ok(Phase::Opening {
            quarter: Quarter::Second,
            wait: wait(5)
        })
    );
    assert_eq!(right(0xF5A0, 0), Err(Unknown::Timer(0)));
    assert_eq!(right(0xF5A2, 1), Err(Unknown::Instruction(0xF5A2)));
}

#[test]
fn the_decode_notes_spritemaps_name_each_figure() {
    assert_eq!(figure(0x804D), Ok(None));
    assert_eq!(figure(0xFAC7), Ok(None));
    assert_eq!(
        figure(0xFA13),
        Ok(Some(Figure {
            look: FigureLook::Closed,
            facing: Side::Right
        }))
    );
    assert_eq!(
        figure(0xF9F3),
        Ok(Some(Figure {
            look: FigureLook::Open,
            facing: Side::Left
        }))
    );
    assert_eq!(figure(0x1234), Err(0x1234));
}
