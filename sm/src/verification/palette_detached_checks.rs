//! Independent failure-state checks of the actual backend-free palette core.
#![forbid(unsafe_code)]
use super::{Detached, RomImage, Session, execution};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(super) fn run() {
    // A missing jump operand fails after the countdown write, not before it.
    let mut host = Detached::new();
    host.image = RomImage::zeroed(0x69000);
    host.session.effects.slots[0].counter = 2;
    let failed = catch_unwind(AssertUnwindSafe(|| {
        execution::command(&mut host, 9291321, 0, 0x9000);
    }));
    assert!(failed.is_err());
    let mut expected = Session::default();
    expected.effects.slots[0].counter = 1;
    assert_eq!(host.session, expected);
    assert!(host.events.is_empty());

    // A lazy completed countdown must skip the same missing operand entirely.
    let result = execution::command(&mut host, 9291321, 0, 0x9000);
    assert_eq!(result, (0, 0x9002));
    expected.effects.slots[0].counter = 0;
    assert_eq!(host.session, expected);

    // No sound event is delivered when the music operand itself is absent.
    host.session.set_enabled(true);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        execution::command(&mut host, 9291358, 0, 0x9000);
    }));
    assert!(failed.is_err());
    expected.effects.flags = 0x8000;
    assert_eq!(host.session, expected);
    assert!(host.events.is_empty());

    // Heat updates typed player damage and script state before a missing table
    // entry fails. The partial state must remain usable without any RAM import.
    host.session.effects.heat = 1;
    host.session.damage.fraction = 0xc000;
    host.session.damage.amount = 1;
    let failed = catch_unwind(AssertUnwindSafe(|| {
        execution::pre(&mut host, 9298809, 0);
    }));
    assert!(failed.is_err());
    expected.effects.heat = 1;
    expected.effects.previous_heat = 1;
    expected.effects.slots[0].delay = 1;
    expected.damage.amount = 2;
    assert_eq!(host.session, expected);
    assert!(host.events.is_empty());
}
