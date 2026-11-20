//! Actual-origin fact codec development evidence; no producer behavior executes.
#[path = "../src/replay/frame_prelude_seed.rs"]
mod frame_prelude_seed;
use frame_prelude_seed::PreludeSeedFault;
use game::production::frame::borrowed::{ActorSlot, HazeSlot};

#[test]
fn incomplete_origin_is_rejected() {
    for length in [0, 544, 131_071, 131_073] {
        assert!(
            frame_prelude_seed::decode(&vec![0; length])
                .is_err_and(|fault| fault == PreludeSeedFault::Length)
        );
    }
}
#[test]
#[ignore = "requires independently qualified complete origin WRAM"]
fn actual_origin_preserves_current_facts_without_producing_results() {
    let path = std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap();
    let bytes = std::fs::read(path).unwrap();
    let seed = frame_prelude_seed::decode(&bytes).unwrap();
    let restored: game::production::frame::borrowed::PreludeSeed =
        canonical::from_bytes(&canonical::to_bytes(&seed)).unwrap();
    assert!(restored.eq(&seed));
    assert_eq!(seed.haze.slots.len(), 6);
    assert!(matches!(seed.haze.slots[0], HazeSlot::BackdropRamp { .. }));
    assert!(matches!(seed.haze.slots[5], HazeSlot::KeepTransfer { .. }));
    assert!(
        seed.haze
            .slots
            .iter()
            .skip(1)
            .take(4)
            .all(|slot| *slot == HazeSlot::Empty)
    );
    assert_eq!(seed.builder.backing.len(), 544);
    assert!(
        seed.builder
            .backing
            .as_slice()
            .eq(bytes.get(880..1424).unwrap())
    );
    assert!(matches!(
        seed.actors.slots[0],
        ActorSlot::Occupied {
            deleted: false,
            process_offscreen: true,
            intangible: true
        }
    ));
    assert!(matches!(
        seed.actors.slots[1],
        ActorSlot::Occupied {
            deleted: false,
            process_offscreen: true,
            intangible: false
        }
    ));
    assert!(
        seed.actors
            .slots
            .iter()
            .skip(2)
            .all(|slot| *slot == ActorSlot::Empty)
    );
    let mut altered_backing = bytes.clone();
    altered_backing[880] ^= 1;
    let altered = frame_prelude_seed::decode(&altered_backing).unwrap();
    assert!(
        altered
            .builder
            .backing
            .first()
            .ne(&seed.builder.backing.first())
    );
    for (at, value) in [
        (6324, 3),
        (6384, 0),
        (6396, 0),
        (6432, 0),
        (6204, 1),
        (2680, 1),
    ] {
        let mut malformed = bytes.clone();
        malformed[at] = value;
        assert!(
            frame_prelude_seed::decode(&malformed)
                .is_err_and(|fault| fault == PreludeSeedFault::Unsupported)
        );
    }
}
