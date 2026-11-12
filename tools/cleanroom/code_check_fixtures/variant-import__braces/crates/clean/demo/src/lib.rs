//! Variants imported as a group.
use types::Kind::{self, A, B};
pub const fn f(k: Kind) -> bool { matches!(k, A | B) }
