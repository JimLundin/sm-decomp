//! The `verify` crate's folding rule (D21): typed streams fold only at quiescent cuts, so a
//! busy routine's partial progress on the oracle never has to match the engine.

use crate::data::Content;
use crate::game::{Session, State, TickInput, TimingSource};

/// Whether the oracle's main thread was at a wait site at a cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quiescence {
    /// At a wait site: every stream is compared.
    Quiescent,
    /// Inside a routine: typed streams are not compared.
    Busy,
}

/// The shared digest fold.
#[must_use]
pub const fn fold(running: u64, value: u64) -> u64 {
    (running ^ value).wrapping_mul(0x0100_0000_01B3)
}

/// The `samus` stream's projection: one function, whichever side the State came from.
#[must_use]
pub fn project_samus(state: &State) -> u64 {
    let (x, y) = state.samus;
    fold(fold(0xCBF2_9CE4_8422_2325, u64::from(x.unsigned_abs())), u64::from(y.unsigned_abs()))
}

/// A running typed digest that folds only at quiescent cuts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TypedDigest(pub u64);

impl TypedDigest {
    /// Observe one cut.
    pub const fn observe(&mut self, quiescence: Quiescence, projected: u64) {
        match quiescence {
            Quiescence::Quiescent => self.0 = fold(self.0, projected),
            Quiescence::Busy => {}
        }
    }
}

/// Run a session over `inputs` and fold its `samus` stream at the cuts the oracle's
/// record marks quiescent.
///
/// # Errors
///
/// The engine's fault, if it faults.
pub fn free_run(
    content: Content,
    start: State,
    inputs: &[(TickInput, Quiescence)],
) -> Result<TypedDigest, crate::game::SessionFault> {
    let mut session = Session::open(content, start, TimingSource::Recorded);
    let mut digest = TypedDigest::default();
    for (input, quiescence) in inputs {
        session.step(*input)?;
        digest.observe(*quiescence, project_samus(session.state()));
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Buttons, NmiTiming};
    use crate::{game, runtime};

    /// The oracle's view of the same run: at busy cuts its main thread is mid-load, so
    /// Samus hasn't been placed yet.
    fn oracle_digest(states: &[(State, Quiescence)]) -> TypedDigest {
        let mut digest = TypedDigest::default();
        for (state, quiescence) in states {
            let mut seen = state.clone();
            if *quiescence == Quiescence::Busy {
                seen.samus.0 = -1;
            }
            digest.observe(*quiescence, project_samus(&seen));
        }
        digest
    }

    #[test]
    fn typed_streams_agree_when_folded_only_at_quiescent_cuts() {
        let on_time = TickInput::default();
        let busy = TickInput { timing: NmiTiming::Overrun, ..TickInput::default() };
        let door = TickInput { pad: Buttons::ENTER_DOOR, ..TickInput::default() };
        let inputs = [
            (on_time, Quiescence::Quiescent),
            (door, Quiescence::Quiescent),
            (busy, Quiescence::Busy),
            (busy, Quiescence::Busy),
            (on_time, Quiescence::Quiescent),
            (on_time, Quiescence::Quiescent),
        ];
        let engine = free_run(Content::vanilla(), State::sample(), &inputs).unwrap();

        let mut session = Session::open(Content::vanilla(), State::sample(), TimingSource::Recorded);
        let states: Vec<(State, Quiescence)> = inputs
            .iter()
            .map(|(input, quiescence)| {
                session.step(*input).unwrap();
                (session.checkpoint(), *quiescence)
            })
            .collect();
        assert_eq!(engine, oracle_digest(&states));
    }

    #[test]
    fn intent_at_native_width_equals_the_exact_sprites() {
        let content = Content::vanilla();
        let mut session = Session::open(content, State::sample(), TimingSource::None);
        let right = TickInput { pad: Buttons::RIGHT, ..TickInput::default() };
        for _ in 0..120 {
            session.step(right).unwrap();
            let view = game::view::describe(session.state(), &content);
            assert_eq!(runtime::from_oam(&session.state().oam), runtime::at_width(&view, 0x100, false));
        }
        let view = game::view::describe(session.state(), &content);
        assert!(!view.margin.is_empty(), "an enemy outside the window is drawn only as a margin entry");
    }
}
