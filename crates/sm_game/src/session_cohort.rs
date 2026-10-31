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
//! holds the copy; code it calls takes the copy instead.
use crate::ram::CompatibilityRam;
use crate::runtime_access::RuntimeAccess;
use crate::session::Session;

/// A group of Session-owned words with their RAM encoding.
pub trait Cohort: Clone + PartialEq + Default {
    /// Visit each word with its RAM address.
    fn words(&mut self, visit: impl FnMut(usize, &mut u16));
    /// The group in `session`.
    fn of(session: &mut Session) -> &mut Self;
}

/// The group as the host's projection holds it.
pub fn load<C: Cohort, H: CompatibilityRam + ?Sized>(game: &H) -> C {
    let mut cohort = C::default();
    cohort.words(|at, word| *word = crate::projection::word(game, at));
    cohort
}

/// Store the words `after` changed from `before`.
pub fn store<C, H>(game: &mut H, before: &C, mut after: C)
where
    C: Cohort,
    H: CompatibilityRam + RuntimeAccess + ?Sized,
{
    if after == *before {
        return;
    }
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
    if game.session_retained() {
        use crate::session_access::SessionAccess as _;
        let mut lease = game.runtime();
        let mut index = 0;
        C::of(lease.session_mut()).words(|_, word| {
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
    let result = action(game, &mut cohort);
    store(game, &before, cohort);
    result
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
        C::of(&mut session).words(|_, word| {
            *word = value;
            value = value.wrapping_add(0x0101);
        });
        let expected = C::of(&mut session).clone();
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
}
