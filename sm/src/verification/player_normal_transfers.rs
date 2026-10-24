//! Scoped observations of the actual normal/no-op-gamma beta4100 caller.
//! Shared by candidate and predecessor; each carrier declares its expected pairs.
use crate::game_state::Game;
use std::cell::RefCell;

thread_local! {
    static EVENTS: RefCell<Option<Vec<(u8, bool)>>> = RefCell::new(None);
}

// 0 = Runtime construction, 1 = Session import, 2 = guarded Session export.
pub(crate) fn record(game: &Game, stage: u8) {
    EVENTS.with(|events| {
        if let Some(events) = events.borrow_mut().as_mut() {
            events.push((stage, game.actor_identity_is_with_session()));
        }
    });
}

pub(crate) struct Scope {
    armed: bool,
    pairs: usize,
}
pub(crate) fn begin(id: u32, expected_pairs: usize) -> Scope {
    let armed = id == 4100;
    if armed {
        EVENTS.with(|events| {
            assert!(
                events.borrow().is_none(),
                "normal-transfer observation is not reentrant"
            );
            *events.borrow_mut() = Some(Vec::new());
        });
    }
    Scope {
        armed,
        pairs: expected_pairs,
    }
}
impl Scope {
    pub(crate) fn finish(self) {
        if !self.armed {
            return;
        }
        let observed = EVENTS.with(|events| events.borrow_mut().take().unwrap());
        let expected = [(0, false), (1, false), (2, true)].repeat(self.pairs);
        assert_eq!(
            observed, expected,
            "actual beta4100 normal/no-op-gamma transfer sequence"
        );
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if self.armed {
            EVENTS.with(|events| *events.borrow_mut() = None);
        }
    }
}
