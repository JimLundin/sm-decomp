//! Comparison-only entry points, accounted separately from runtime.
#![forbid(unsafe_code)]
use super::*;
pub(crate) fn command(address: u32, index: u16, cursor: u16, game: &mut Game) -> u16 {
    let (owner, next) = execution::command(Owner::acquire(game), address, index, cursor);
    drop(owner);
    next
}
pub(crate) fn process_one(game: &mut Game) {
    execution::process(Owner::acquire(game));
}
