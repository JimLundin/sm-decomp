//! Spec: `BabyPulses`. The baby's shape cycles through its pulse, even behind the egg.

use types::discovery::Pulse;

use crate::scene::{Animation, SpriteRef};

/// Spec: `BabyPulses`. One finished frame of the pulse: a newly placed baby shows its first
/// picture at age zero; after that the age advances one, modulo the cycle's length.
#[must_use]
pub fn advanced(pulse: Pulse, cycle: &Animation) -> Pulse {
    match pulse {
        Pulse::Unseen => Pulse::Age(0),
        Pulse::Age(age) => Pulse::Age(cycle.looped_after(age)),
    }
}

/// Spec: `BabyPulses`. The baby's picture: the one whose hold includes its pulse age; none
/// before its first finished frame.
#[must_use]
pub fn picture(pulse: Pulse, cycle: &Animation) -> Option<SpriteRef> {
    match pulse {
        Pulse::Unseen => None,
        Pulse::Age(age) => cycle.picture_at(age),
    }
}
