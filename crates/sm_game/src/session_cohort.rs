//! Session-owned word groups as typed copies.
//!
//! A [`Cohort`] is a group of Session words with their RAM encoding. [`load`]
//! reads one through a host's projection. Code that holds a copy for its call
//! (the cinematic table's [`crate::cinematic_state::Cinematic::lend`]) must
//! not reach the group's words by any other path meanwhile; the verify build
//! checks that no projection access reaches a held copy's words.
use crate::projection::Views;
use crate::session::Session;

/// A group of Session-owned words with their RAM encoding.
pub trait Cohort: Clone + PartialEq + Default {
    /// Visit each word with its RAM address.
    fn words(&mut self, visit: impl FnMut(usize, &mut u16));
    /// Visit the group's words in `session`, in the order `words` visits the
    /// copy's.
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16));
    /// The group as `session` holds it: each word `words` visits holds the
    /// value `session_words` visits at the same position.
    fn session_copy(session: &Session) -> Self;
}

/// Two groups held together, in order.
impl<A: Cohort, B: Cohort> Cohort for (A, B) {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        self.0.words(&mut visit);
        self.1.words(&mut visit);
    }
    fn session_words(session: &mut Session, mut visit: impl FnMut(usize, &mut u16)) {
        A::session_words(session, &mut visit);
        B::session_words(session, &mut visit);
    }
    fn session_copy(session: &Session) -> Self {
        (A::session_copy(session), B::session_copy(session))
    }
}

/// The group as the host's projection holds it.
pub fn load<C: Cohort, H: Views + ?Sized>(game: &H) -> C {
    let mut cohort = C::default();
    game.load_words(&mut cohort);
    cohort
}

/// The words of every copy held, and their bytes as a bit set.
#[cfg(sm_verify_dll)]
#[derive(Default)]
struct Held {
    words: Vec<usize>,
    bytes: Vec<u64>,
}

#[cfg(sm_verify_dll)]
impl Held {
    fn mark(&mut self, at: usize, held: bool) {
        let (index, bit) = (at / 64, 1u64 << (at % 64));
        if self.bytes.len() <= index {
            self.bytes.resize(index + 1, 0);
        }
        if held {
            self.bytes[index] |= bit;
        } else {
            self.bytes[index] &= !bit;
        }
    }
    fn holds(&self, at: usize) -> bool {
        self.bytes
            .get(at / 64)
            .is_some_and(|bits| bits & (1u64 << (at % 64)) != 0)
    }
}

#[cfg(sm_verify_dll)]
std::thread_local! {
    static HELD: core::cell::RefCell<Held> = core::cell::RefCell::new(Held::default());
}

/// Mark `cohort`'s words held; returns the previous held count.
#[cfg(sm_verify_dll)]
pub(crate) fn hold<C: Cohort>(cohort: &mut C) -> usize {
    HELD.with(|held| {
        let mut held = held.borrow_mut();
        let previous = held.words.len();
        cohort.words(|at, _| {
            held.words.push(at);
            held.mark(at, true);
            held.mark(at + 1, true);
        });
        previous
    })
}

#[cfg(sm_verify_dll)]
pub(crate) fn release(previous: usize) {
    HELD.with(|held| {
        let mut held = held.borrow_mut();
        let released = held.words.split_off(previous);
        for at in released {
            held.mark(at, false);
            held.mark(at + 1, false);
        }
        // An outer copy keeps its own words held.
        for index in 0..held.words.len() {
            let at = held.words[index];
            held.mark(at, true);
            held.mark(at + 1, true);
        }
    });
}

/// Fail when byte `at` belongs to a word of a copy `with` holds.
#[cfg(sm_verify_dll)]
pub fn check_not_held(at: usize) {
    HELD.with(|held| {
        assert!(
            !held.borrow().holds(at),
            "byte ${at:04X} of a held Session copy was reached through the projection"
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_state::{
        DeathSequence, MapScreenState, MenuScreens, OptionsMenu, SaveSelection,
    };
    use crate::projection::SessionProjection as Projection;
    use crate::room_entry::world::WorldBinding;

    /// Each word a cohort visits is the one `session_encoding` projects there.
    fn check_encoding<C: Cohort + core::fmt::Debug>() {
        let mut session = Session::with_world(WorldBinding::canonical_legacy());
        let mut value = 0x1001u16;
        let mut expected = C::default();
        let mut addresses = Vec::new();
        C::session_words(&mut session, |at, word| {
            *word = value;
            addresses.push((at, value));
            value = value.wrapping_add(0x0101);
        });
        let mut index = 0;
        expected.words(|at, word| {
            assert_eq!(at, addresses[index].0);
            *word = addresses[index].1;
            index += 1;
        });
        let host = Projection(session);
        assert_eq!(load::<C, _>(&host), expected);
    }

    #[test]
    fn the_menu_screens_load_from_their_projected_words() {
        check_encoding::<MenuScreens>();
        check_encoding::<MapScreenState>();
        check_encoding::<SaveSelection>();
        check_encoding::<OptionsMenu>();
        check_encoding::<DeathSequence>();
    }

    #[test]
    fn the_session_words_load_from_their_projected_words() {
        check_encoding::<crate::session_words::SessionWords>();
        check_encoding::<(OptionsMenu, crate::session_words::SessionWords)>();
    }

    #[test]
    fn the_menu_state_and_inventory_load_from_their_projected_words() {
        check_encoding::<crate::menu_state::MenuState>();
        check_encoding::<crate::inventory::Inventory>();
        check_encoding::<crate::flow::Flow>();
    }

    #[test]
    fn the_cinematic_table_loads_from_its_projected_words() {
        check_encoding::<crate::cinematic_state::Cinematic>();
    }
}
