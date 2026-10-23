//! Legacy palette check entry, sharing the actual composed host and modules.
#![forbid(unsafe_code)]
#[path = "player_detached/main.rs"]
mod common;
pub(crate) use common::{game, ram_map, rom, types, verify_probe};
fn main() {
    common::run_legacy_palette();
}
