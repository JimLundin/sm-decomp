//! Spec: CountdownContinuation. Retain a distinct qualified countdown opportunity.

use crate::countdown::{CountdownAnswer, CountdownOpportunity};
use crate::state::HudState;
use crate::work::Observation;

/// Spec: CountdownContinuation. Complete HUD State and one countdown observation's progress.
///
/// `new` merely offers unread work: it freezes no scene answer. On resumption the producer
/// must qualify fresh scene input before `choose`. Once chosen, `answer` exposes the actual
/// answer before `apply` completes it once. Only `publication` exposes a completed contribution.
/// Repeated applications return false, without producing another answer or picture.
/// Inspection and publication are retained values, not delivery; acknowledgements belong to
/// the caller, just as for HudWork events.
///
/// Serialize this whole value, discard the original, then deserialize a fresh CountdownWork.
/// Derived serde retains every HUD field, the selected opportunity, expiry/picture and unread,
/// chosen or completed progress. Do not restore State alone to resume consumed work.
///
/// ```
/// # use interface_hud::{CountdownWork, CountdownOpportunity, HudState, Observation};
/// # fn maintain(state: HudState, qualified: CountdownOpportunity) {
/// let mut work = CountdownWork::new(state); // an offer, not a scene decision
/// assert!(matches!(work.answer(), Observation::Unread));
/// // The scene producer supplies its actual qualified choice here.
/// assert!(work.choose(qualified));
/// assert!(matches!(work.answer(), Observation::Chosen(_))); // inspect before effects
/// assert!(work.publication().is_none());
/// assert!(work.apply());
/// let (complete_hud, answer) = work.publication().expect("maintenance completed");
/// // Carry complete_hud onward; answer contains the expiry and picture contribution.
/// assert!(!work.apply()); // no second contribution
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CountdownWork {
    state: HudState,
    answer: Observation<CountdownAnswer>,
}

impl CountdownWork {
    /// Spec: CountdownContinuation. Offer without choosing or contributing anything.
    #[must_use]
    pub const fn new(state: HudState) -> Self {
        Self {
            state,
            answer: Observation::Unread,
        }
    }

    /// Spec: EscapeCountdown. Receive the producer's actual qualified current choice and
    /// compute its answer from the owned countdown, never from earlier display inputs.
    /// Repeated choices cannot replace a chosen or consumed answer.
    pub const fn choose(&mut self, opportunity: CountdownOpportunity) -> bool {
        if !matches!(self.answer, Observation::Unread) {
            return false;
        }
        self.answer = Observation::Chosen(self.state.countdown.answer(opportunity));
        true
    }

    /// Spec: CountdownContinuation. Complete the chosen contribution once. Dormant maintenance
    /// changes no HUD field, including countdown, selection, caches, grid and map.
    pub const fn apply(&mut self) -> bool {
        if let Observation::Chosen(answer) = self.answer {
            self.answer = Observation::Completed(answer);
            return true;
        }
        false
    }

    /// Spec: CountdownContinuation. Unread offer, actual chosen answer or completed answer.
    #[must_use]
    pub const fn answer(&self) -> &Observation<CountdownAnswer> {
        &self.answer
    }

    /// Spec: CountdownContinuation. Complete current HUD, not inferred from display inputs.
    #[must_use]
    pub const fn state(&self) -> &HudState {
        &self.state
    }

    /// Spec: CountdownContinuation. The one completed answer/picture contribution, or none
    /// before completion. Reading this value does not deliver it again.
    #[must_use]
    pub const fn publication(&self) -> Option<(&HudState, &CountdownAnswer)> {
        match &self.answer {
            Observation::Completed(answer) => Some((&self.state, answer)),
            Observation::Unread | Observation::Chosen(_) => None,
        }
    }
}
