//! Verify-only lease counters (session-authoritative-tick.md, stage 0).
//!
//! They count frame ticks, Session leases (`Runtime::new`), the leases that
//! decoded the RAM image instead of taking up the kept or retained Session,
//! the leases opened inside a `foreign` call, and the `foreign` calls. The
//! running totals go to stderr as one `lease counters:` line at ticks 256, 512,
//! 1024 and 2048, then every 4096 ticks.
//! Nothing reads them back, so they cannot change behaviour.
use std::cell::Cell;

#[derive(Clone, Copy)]
struct Counts {
    ticks: u64,
    leases: u64,
    imports: u64,
    nested: u64,
    foreign: u64,
}

thread_local! {
    static COUNTS: Cell<Counts> = const {
        Cell::new(Counts {
            ticks: 0,
            leases: 0,
            imports: 0,
            nested: 0,
            foreign: 0,
        })
    };
}

fn update(change: impl FnOnce(&mut Counts)) {
    COUNTS.with(|counts| {
        let mut value = counts.get();
        change(&mut value);
        counts.set(value);
    });
}

/// A lease opened; `nested` when a foreign call is in progress.
pub(crate) fn lease(nested: bool) {
    update(|counts| {
        counts.leases += 1;
        counts.nested += u64::from(nested);
    });
}

/// A lease that decoded the RAM image.
pub(crate) fn import() {
    update(|counts| counts.imports += 1);
}

pub(crate) fn foreign() {
    update(|counts| counts.foreign += 1);
}

/// A frame tick ended.
pub(crate) fn tick() {
    update(|counts| {
        counts.ticks += 1;
        if (counts.ticks >= 256 && counts.ticks.is_power_of_two())
            || counts.ticks.is_multiple_of(4096)
        {
            let Counts {
                ticks,
                leases,
                imports,
                nested,
                foreign,
            } = *counts;
            eprintln!(
                "lease counters: ticks={ticks} leases={leases} imports={imports} nested={nested} foreign={foreign}"
            );
        }
    });
}
