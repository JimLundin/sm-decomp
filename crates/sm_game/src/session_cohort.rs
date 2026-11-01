//! Session-owned word groups as typed copies, for code outside a lease.
//!
//! A function that runs on the compatibility host and owns a group's words
//! for its call runs on a [`Cohort`] copy through [`with`]: the copy is loaded
//! once from the host's projection (the retained Session's fields while it is
//! current, the RAM bytes otherwise), the function reads and stores its fields
//! directly, and the words it changed are stored back once. While a retained
//! Session is current they are stored on one lease, which keeps it current;
//! otherwise each changed word goes to the projection, low byte then high, as
//! the named word stores did.
//!
//! The function must not reach the group's words by any other path while it
//! holds the copy; code it calls takes the copy instead. The verify build
//! checks that no projection access reaches a held copy's words.
use crate::ram::CompatibilityRam;
use crate::runtime_access::RuntimeAccess;
use crate::session::Session;

/// A group of Session-owned words with their RAM encoding.
pub trait Cohort: Clone + PartialEq + Default {
    /// Visit each word with its RAM address.
    fn words(&mut self, visit: impl FnMut(usize, &mut u16));
    /// Visit the group's words in `session`, in the order `words` visits the
    /// copy's.
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16));
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
}

/// The group as the host's projection holds it.
pub fn load<C: Cohort, H: CompatibilityRam + ?Sized>(game: &H) -> C {
    let mut cohort = C::default();
    cohort.words(|at, word| *word = crate::projection::word(game, at));
    cohort
}

/// The words `after` changed from `before`: each one's index, RAM address
/// and value.
fn changes<C: Cohort>(before: &C, mut after: C) -> Vec<(usize, usize, u16)> {
    let mut previous = Vec::new();
    before.clone().words(|_, word| previous.push(*word));
    let mut changes = Vec::new();
    let mut index = 0;
    after.words(|at, word| {
        if previous[index] != *word {
            changes.push((index, at, *word));
        }
        index += 1;
    });
    changes
}

/// Store the words `after` changed from `before`.
pub fn store<C, H>(game: &mut H, before: &C, after: C)
where
    C: Cohort,
    H: CompatibilityRam + RuntimeAccess + ?Sized,
{
    if after == *before {
        return;
    }
    let changes = changes(before, after);
    if game.session_retained() {
        use crate::session_access::SessionAccess as _;
        let mut lease = game.runtime();
        let mut index = 0;
        C::session_words(lease.session_mut(), |_, word| {
            if let Some(&(_, _, value)) = changes.iter().find(|change| change.0 == index) {
                *word = value;
            }
            index += 1;
        });
    } else {
        for (_, at, value) in changes {
            crate::projection::set_word(game, at, value);
        }
    }
}

/// Run `action` on a copy of the group loaded from the host, then store the
/// words it changed.
pub fn with<C, H, T>(game: &mut H, action: impl FnOnce(&mut H, &mut C) -> T) -> T
where
    C: Cohort,
    H: CompatibilityRam + RuntimeAccess + ?Sized,
{
    let before = load::<C, H>(game);
    let mut cohort = before.clone();
    #[cfg(sm_verify_dll)]
    let held = hold(&mut cohort);
    let result = action(game, &mut cohort);
    #[cfg(sm_verify_dll)]
    release(held);
    store(game, &before, cohort);
    result
}

/// [`with`] for a host that cannot open a lease: the changed words go to the
/// projection, low byte then high, as the named word stores did.
pub fn with_projected<C, H, T>(game: &mut H, action: impl FnOnce(&mut H, &mut C) -> T) -> T
where
    C: Cohort,
    H: CompatibilityRam + ?Sized,
{
    let before = load::<C, H>(game);
    let mut cohort = before.clone();
    #[cfg(sm_verify_dll)]
    let held = hold(&mut cohort);
    let result = action(game, &mut cohort);
    #[cfg(sm_verify_dll)]
    release(held);
    if cohort != before {
        for (_, at, value) in changes(&before, cohort) {
            crate::projection::set_word(game, at, value);
        }
    }
    result
}

/// The words of every copy `with` holds, and their bytes as a bit set.
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
fn hold<C: Cohort>(cohort: &mut C) -> usize {
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
fn release(previous: usize) {
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
    fn the_cinematic_table_loads_from_its_projected_words() {
        check_encoding::<crate::cinematic_state::Cinematic>();
    }
}
