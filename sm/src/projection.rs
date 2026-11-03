//! The compatibility hosts' projected bytes, behind sm_game's typed views.
//!
//! sm_game's views name what they read and store by field and layout
//! (`sm_game::projection::Views`). Here the compat `Game` and the Runtime
//! resolve them to their projected bytes: on a lease the Runtime's byte path
//! (the Session while it is current), on `Game` the installed Session's
//! projection or the RAM image. Words go low byte then high, as the named
//! word accesses did.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::external_inputs::ExternalInputs;
use sm_game::projection::{Decode, Field, Layout, Views};
use sm_game::ram::CompatibilityRam;

/// Byte `at` as the host's projection holds it.
pub(crate) fn byte<H: CompatibilityRam + ?Sized>(h: &H, at: usize) -> u8 {
    #[cfg(sm_verify_dll)]
    sm_game::session_cohort::check_not_held(at);
    h.compatibility_byte(at)
}

/// Store byte `at` through the host's projection.
pub(crate) fn set_byte<H: CompatibilityRam + ?Sized>(h: &mut H, at: usize, value: u8) {
    #[cfg(sm_verify_dll)]
    sm_game::session_cohort::check_not_held(at);
    h.compatibility_write_byte(at, value);
}

/// The little-endian word at `at`, low byte then high.
pub(crate) fn word<H: CompatibilityRam + ?Sized>(h: &H, at: usize) -> u16 {
    u16::from_le_bytes([byte(h, at), byte(h, at + 1)])
}

/// Store `value` as the little-endian word at `at`, low byte then high, as a
/// named word store does.
pub(crate) fn set_word<H: CompatibilityRam + ?Sized>(h: &mut H, at: usize, value: u16) {
    for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
        set_byte(h, at + lane, value);
    }
}

/// The external observations as the host's projection holds them.
pub(crate) fn observed_inputs<H: CompatibilityRam + ?Sized>(h: &H) -> ExternalInputs {
    sm_game::session_encoding::inputs(&|at| h.compatibility_byte(at))
}

fn store_words<H: CompatibilityRam + ?Sized>(
    h: &mut H,
    words: &mut impl Layout,
    mut select: impl FnMut(usize) -> bool,
) {
    let mut index = 0;
    words.visit(|at, value| {
        if select(index) {
            set_word(h, at, *value);
        }
        index += 1;
    });
}

macro_rules! projected_views {
    ($host:ty) => {
        impl Views for $host {
            fn view_word(&self, field: Field) -> u16 {
                word(self, field.address())
            }
            fn set_view_word(&mut self, field: Field, value: u16) {
                set_word(self, field.address(), value);
            }
            fn set_view_low_byte(&mut self, field: Field, value: u8) {
                set_byte(self, field.address(), value);
            }
            fn decode_view<T: Decode>(&self) -> T {
                T::decode(&|at| byte(self, at))
            }
            fn load_words(&self, words: &mut impl Layout) {
                words.visit(|at, value| *value = word(self, at));
            }
            fn store_words(&mut self, words: &mut impl Layout, select: impl FnMut(usize) -> bool) {
                store_words(self, words, select);
            }
        }
    };
}

projected_views!(Game);

/// The verify build's check that no access reaches a held copy's bytes,
/// for the bytes `field`'s word access covers.
#[cfg(sm_verify_dll)]
fn check_field(field: Field, bytes: usize) {
    let at = field.address();
    for lane in at..at + bytes {
        sm_game::session_cohort::check_not_held(lane);
    }
}

/// While the lease is current the Runtime reads and stores the Session's
/// fields directly; each store then refreshes the observations once, as the
/// projected byte stores did after each byte. Otherwise, and for a word the
/// Session has no field for, it takes the byte path as `Game` does.
impl Views for Runtime<'_> {
    fn view_word(&self, field: Field) -> u16 {
        if self.persistent_is_current() {
            if let Some(value) = field.session_word(self.session()) {
                #[cfg(sm_verify_dll)]
                check_field(field, 2);
                return value;
            }
        }
        word(self, field.address())
    }
    fn set_view_word(&mut self, field: Field, value: u16) {
        if self.persistent_is_current() {
            if let Some(word) = field.session_word_mut(self.session_mut()) {
                #[cfg(sm_verify_dll)]
                check_field(field, 2);
                *word = value;
                self.refresh_written_inputs();
                return;
            }
        }
        set_word(self, field.address(), value);
    }
    fn set_view_low_byte(&mut self, field: Field, value: u8) {
        if self.persistent_is_current() {
            if let Some(word) = field.session_word_mut(self.session_mut()) {
                #[cfg(sm_verify_dll)]
                check_field(field, 1);
                *word = (*word & 0xff00) | u16::from(value);
                self.refresh_written_inputs();
                return;
            }
        }
        set_byte(self, field.address(), value);
    }
    fn decode_view<T: Decode>(&self) -> T {
        if self.persistent_is_current() {
            #[cfg(sm_verify_dll)]
            T::decode(&|at| {
                sm_game::session_cohort::check_not_held(at);
                0
            });
            return T::from_session(self.session());
        }
        T::decode(&|at| byte(self, at))
    }
    fn load_words(&self, words: &mut impl Layout) {
        if self.persistent_is_current() {
            #[cfg(sm_verify_dll)]
            words.visit(|at, _| check_word(at));
            sm_game::projection::load_session_words(words, self.session());
            return;
        }
        words.visit(|at, value| *value = word(self, at));
    }
    fn store_words(&mut self, words: &mut impl Layout, mut select: impl FnMut(usize) -> bool) {
        if !self.persistent_is_current() {
            return store_words(self, words, select);
        }
        let mut selected = Vec::new();
        words.visit(|_at, _| {
            let store = select(selected.len());
            #[cfg(sm_verify_dll)]
            if store {
                check_word(_at);
            }
            selected.push(store);
        });
        if words.store_session(self.session_mut(), &selected) {
            if selected.contains(&true) {
                self.refresh_written_inputs();
            }
        } else {
            store_words(self, words, |index| selected[index]);
        }
    }
}

/// The verify build's held-copy check for the word at `at`.
#[cfg(sm_verify_dll)]
fn check_word(at: usize) {
    sm_game::session_cohort::check_not_held(at);
    sm_game::session_cohort::check_not_held(at + 1);
}
