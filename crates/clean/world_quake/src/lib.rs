//! Room quake state, quiet opportunities and Drawing's quiet displacement query.

mod catalogue;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE, Check, Kind, Switch};
pub use types::quake::{QuakeDuration, QuakePhase, RoomQuakeState as State};
use types::{Fixed, Pair, ZERO};

/// Spec: `QuietQuakePhase`. What the scheduler established for this room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opportunity {
    /// One ordinary room quake opportunity.
    Ordinary,
    /// The scheduler established that there is no opportunity.
    Absent,
    /// An ordinary opportunity was not established for the requested update.
    Unestablished,
}

/// Spec: `QuietQuakePhase`. Why the model cannot perform a requested operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedReason {
    /// Nonzero remaining duration belongs to the active quake rules.
    ActiveQuake,
    /// The request lacks a scheduler-established ordinary opportunity.
    UnestablishedOpportunity,
}

/// Spec: `QuietQuakePhase`. Effects of a completed quiet opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuietEffects {
    /// The quake's offset for Drawing, not a composed position.
    pub displacement: Pair<Fixed>,
    /// Whether the quake requested actor shaking (never for a quiet opportunity).
    pub actor_shake_requested: bool,
}

/// Spec: `QuietQuakePhase`. Only a completed quiet opportunity publishes effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub enum Outcome {
    /// Phase advanced once and remaining duration stayed zero.
    Quiet(QuietEffects),
    /// No opportunity; the state was left alone.
    NoOpportunity,
    /// Outside this model's scope; the state was left alone.
    Unsupported(UnsupportedReason),
}

/// Spec: `QuietQuakePhase`. Drawing gets zero quake displacement at zero remaining duration,
/// regardless of phase or whether the scheduler just supplied an opportunity.
///
/// # Errors
///
/// Returns [`UnsupportedReason::ActiveQuake`] if the remaining duration is nonzero.
pub const fn displacement(state: &State) -> Result<Pair<Fixed>, UnsupportedReason> {
    if state.is_active() {
        Err(UnsupportedReason::ActiveQuake)
    } else {
        Ok(Pair { x: ZERO, y: ZERO })
    }
}

/// Spec: `QuietQuakePhase`. At an ordinary quiet opportunity, advance the phase modulo 65,536
/// and leave the duration untouched.
///
/// The caller's state holds the complete result for next time. Unsupported requests are
/// rejected before writing state or publishing effects.
pub const fn step(state: &mut State, opportunity: Opportunity) -> Outcome {
    match opportunity {
        Opportunity::Absent => return Outcome::NoOpportunity,
        Opportunity::Unestablished => {
            return Outcome::Unsupported(UnsupportedReason::UnestablishedOpportunity);
        }
        Opportunity::Ordinary => {}
    }

    let displacement = match displacement(state) {
        Ok(displacement) => displacement,
        Err(reason) => return Outcome::Unsupported(reason),
    };
    state.phase = QuakePhase(state.phase.0.wrapping_add(1));
    Outcome::Quiet(QuietEffects {
        displacement,
        actor_shake_requested: false,
    })
}
