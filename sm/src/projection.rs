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
projected_views!(Runtime<'_>);
