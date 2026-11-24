//! Health's compared fields.

use crate::replay::compare::Observation;

/// Health's State by part, so a difference names the part that differs.
pub(super) fn parts(health: &samus_health::State) -> [Observation; 6] {
    [
        ("health.energy.whole", health.energy.whole.get()),
        ("health.energy.fraction", health.energy.fraction),
        ("health.pending.whole", health.pending_loss.whole.get()),
        ("health.pending.fraction", health.pending_loss.fraction),
        (
            "health.warning",
            u16::from(health.warning == samus_health::Warning::On),
        ),
        ("health.protection", u16::from(health.protection.get())),
    ]
    .map(|(field, value)| Observation::quantity(field, i64::from(value)))
}
