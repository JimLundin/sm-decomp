//! Development evidence for complete current sender origins and wrong-direction rejection.

#[path = "../src/replay/frame_sender_seed.rs"]
mod frame_sender_seed;

use frame_sender_seed::{SeedFault, decode};

#[test]
fn complete_lengths_and_current_unsupported_arms_are_checked() {
    assert_eq!(decode(&[], &[]), Err(SeedFault::Length));
    let mut wram = vec![0; 131_072];
    let mut apuram = vec![0; 65_536];
    for group in 0..3 {
        wram[0x0643 + group] = 15;
        wram[0x0646 + group] = 15;
    }
    let state = decode(&wram, &apuram).unwrap();
    let bytes = canonical::to_bytes(&state);
    drop(state);
    let restored: game::production::frame::Senders = canonical::from_bytes(&bytes).unwrap();
    assert_eq!(restored.validate(), Ok(()));
    for (at, value, fault) in [
        (0x0643, 16, SeedFault::Cursor),
        (0x0646, 14, SeedFault::Queue),
        (0x0649, 1, SeedFault::Progress),
    ] {
        let mut changed = wram.clone();
        changed[at] = value;
        assert_eq!(decode(&changed, &apuram), Err(fault));
    }
    apuram[0xf5] = 1;
    assert_eq!(decode(&wram, &apuram), Err(SeedFault::Reply));
    apuram[0xf5] = 0;
    // Native stale cue/delay/ring slots are irrelevant to these actual Ready/Empty arms.
    wram[0x064d..0x0686].fill(255);
    assert_eq!(decode(&wram, &apuram), Ok(restored));
}

#[test]
#[ignore = "requires independently qualified corrected origin and outgoing SMP backing"]
fn qualified_actual_origin_survives_whole_restore() {
    let wram = std::fs::read(std::env::var_os("SM_FRAME_SENDER_WRAM").unwrap()).unwrap();
    let apuram = std::fs::read(std::env::var_os("SM_FRAME_SENDER_APURAM").unwrap()).unwrap();
    let state = decode(&wram, &apuram).unwrap();
    let restored: game::production::frame::Senders =
        canonical::from_bytes(&canonical::to_bytes(&state)).unwrap();
    assert!(
        restored == state,
        "complete actual sender origin restoration"
    );
    assert_eq!(restored.validate(), Ok(()));
}
