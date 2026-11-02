//! Kraid lint collision shares the typed kernel.
#![forbid(unsafe_code)]
pub use crate::actors::kraid_lint_owned as owned;
pub use owned::lint_touch_samus;
