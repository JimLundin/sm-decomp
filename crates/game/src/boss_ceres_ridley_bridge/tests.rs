use core::num::Wrapping;

use boss_ceres_ridley::{CeresDoor, DoorKind, Phase, Quarter, Stage, Wait};
use canonical::{Canonical, from_bytes, to_bytes};
use types::{Pair, Side};

use super::{Door, Station, phase_of, phase_parts, run};
use crate::state::room::{CeresStage, EnemyCommon, EnemyExtraProperty, EnemyProperty};

fn wait(frames: u16) -> Wait {
    Wait::new(frames).unwrap()
}

fn door(phase: Phase) -> Door {
    placed(DoorKind::Door, Side::Left, phase)
}

fn placed(kind: DoorKind, facing: Side, phase: Phase) -> Door {
    Door(CeresDoor {
        kind,
        facing,
        position: Pair {
            x: Wrapping(0x80),
            y: Wrapping(0x60),
        },
        phase,
    })
}

const FAR: Pair<Wrapping<u16>> = Pair {
    x: Wrapping(0x200),
    y: Wrapping(0x60),
};
const NEAR: Pair<Wrapping<u16>> = Pair {
    x: Wrapping(0x90),
    y: Wrapping(0x60),
};

#[test]
fn every_phase_round_trips_through_its_canonical_bytes() {
    let quarters = [
        Quarter::First,
        Quarter::Second,
        Quarter::Third,
        Quarter::Fourth,
    ];
    let mut phases = vec![
        Phase::Placed,
        Phase::Wall,
        Phase::Closed { wait: wait(2) },
        Phase::Open { wait: wait(1) },
        Phase::Starting { wait: wait(2) },
    ];
    for quarter in quarters {
        phases.push(Phase::Opening {
            quarter,
            wait: wait(5),
        });
        phases.push(Phase::Closing {
            quarter,
            wait: wait(3),
        });
    }
    let kinds = [
        (DoorKind::Door, Side::Left),
        (DoorKind::Door, Side::Right),
        (DoorKind::ShaftDoor, Side::Left),
    ];
    for phase in phases {
        let (tag, quarter, frames) = phase_parts(phase);
        assert_eq!(phase_of(tag, quarter, frames), Some(phase));
        for (kind, facing) in kinds {
            let bytes = to_bytes(&placed(kind, facing, phase));
            assert_eq!(from_bytes::<Door>(&bytes), Ok(placed(kind, facing, phase)));
        }
    }
    // A phase that carries a wait never has none, and one that carries none never has one.
    assert_eq!(phase_of(0, 0, 0), None);
    assert_eq!(phase_of(4, 0, 1), None);
    // A phase without a quarter refuses any but the first place, and no phase has a fifth.
    for tag in [0, 2, 5] {
        assert_eq!(phase_of(tag, 2, 2), None);
    }
    assert_eq!(phase_of(4, 1, 0), None);
    assert_eq!(phase_of(6, 3, 0), None);
    assert_eq!(phase_of(3, 4, 5), None);
    assert!(door(Phase::Placed).diff(&door(Phase::Wall)).is_some());
}

/// Bytes no door has are refused, so every door has one encoding.
#[test]
fn bytes_no_door_has_are_refused() {
    let closed = to_bytes(&door(Phase::Closed { wait: wait(2) }));
    let [kind, facing, _, _, _, _, tag, quarter, ..] = closed[..] else {
        panic!("a door's bytes: kind, facing, position, phase, quarter, wait");
    };
    assert_eq!((kind, facing, tag, quarter), (0, 0, 0, 0));
    for (at, byte) in [(0, 2), (1, 2), (6, 7), (7, 1)] {
        let mut refused = closed.clone();
        refused[at] = byte;
        assert!(from_bytes::<Door>(&refused).is_err(), "byte {at} = {byte}");
    }
}

#[test]
fn a_run_writes_what_the_next_selection_reads() {
    let mut common = EnemyCommon::default();
    let mut station = Station::new(CeresStage::Calm, 0);
    // A closed door whose wait runs out with her near starts to open: still shown and solid.
    let mut closed = door(Phase::Closed { wait: wait(1) });
    run(&mut common, &mut closed, NEAR, &mut station);
    assert!(matches!(closed.0.phase, Phase::Opening { .. }));
    assert!(!common.properties.contains(EnemyProperty::Invisible));
    assert!(!common.properties.contains(EnemyProperty::Intangible));
    assert!(
        common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
    // A run that only counts the wait off takes no figure.
    run(&mut common, &mut closed, NEAR, &mut station);
    assert!(
        !common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
    // An open door is hidden and passable.
    let mut open = door(Phase::Open { wait: wait(1) });
    run(&mut common, &mut open, NEAR, &mut station);
    assert!(common.properties.contains(EnemyProperty::Invisible));
    assert!(common.properties.contains(EnemyProperty::Intangible));
    run(&mut common, &mut open, FAR, &mut station);
    assert!(matches!(open.0.phase, Phase::Open { .. }));
}

#[test]
fn a_shake_one_door_starts_is_what_the_next_door_reads() {
    let mut common = EnemyCommon::default();
    let mut station = Station::new(CeresStage::Escape, 0);
    assert_eq!(station.stage, Stage::Escape);
    let mut first = door(Phase::Closed { wait: wait(2) });
    run(&mut common, &mut first, FAR, &mut station);
    // The random number's low 12 bits are 0: the strong shake, 4 frames.
    assert_eq!(station.shake_frames_left, 4);
    let mut second = door(Phase::Closed { wait: wait(2) });
    let before = station;
    run(&mut common, &mut second, FAR, &mut station);
    assert_eq!(station, before);
}
