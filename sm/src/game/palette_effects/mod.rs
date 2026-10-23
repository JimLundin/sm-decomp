//! Palette effect slots, two-stage scripts and ordered palette writes.
//! State is owned only during these operations; native effects release it first.
mod diagnostics;
mod execution;
mod state;
#[path = "../ownership_transport.rs"]
mod transport;
use crate::game_state::Game;
use state::PaletteEffects;
type Owner<'g> = transport::Port<'g, PaletteEffects>;
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    let mut owner = Owner::acquire(game);
    let flags = owner.read_le(0x1e79);
    owner.write_slice_le(
        0x1e79,
        if enabled {
            flags | 0x8000
        } else {
            flags & !0x8000
        },
    );
}
pub(crate) fn clear(game: &mut Game) {
    let mut owner = Owner::acquire(game);
    for slot in (0..8).rev() {
        owner.write_slice_le(0x1e7d + slot * 2, 0);
    }
}
/// Allocate and initialize one effect, preserving diagnostic write prefixes.
pub(crate) fn spawn(definition: u16, game: &mut Game) {
    execution::spawn(Owner::acquire(game), definition);
}
/// Advance enabled effects through their next script suspension.
pub(crate) fn advance(game: &mut Game) {
    execution::advance(Owner::acquire(game));
}

/// Private entry identities retained only for the actual-code comparison carrier.
pub(crate) mod checks {
    use super::*;
    pub(crate) fn setup(ea: u32, _k: u16, slot: u16, game: &mut Game) {
        execution::setup(Owner::acquire(game), ea, slot);
    }
    pub(crate) fn pre(ea: u32, k: u16, game: &mut Game) {
        execution::pre(Owner::acquire(game), ea, k);
    }
    pub(crate) fn process_one(k: u16, game: &mut Game) {
        execution::process(Owner::acquire(game), k);
    }
    pub(crate) fn command(ea: u32, k: u16, j: u16, game: &mut Game) -> crate::types::PairU16 {
        let (owner, k, j) = execution::command(Owner::acquire(game), ea, k, j);
        drop(owner);
        crate::types::PairU16 { k, j }
    }
}
