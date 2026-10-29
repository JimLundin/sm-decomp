//! Real pure Scene dependency closure for detached Session presentation owners.
#![forbid(unsafe_code)]
pub(crate) use sm_game::scene::atmosphere;
#[allow(
    unused_imports,
    reason = "Detached Scene carrier retains the state facade without external slot reexport consumers."
)]
pub(crate) use sm_game::scene::state;
