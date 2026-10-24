//! Legacy animation checks using the shared host at its actual crate root.
#![forbid(unsafe_code)]
#![allow(dead_code)]
include!("../player_detached/shared.rs");

fn main() {
    run_legacy_animation();
}
