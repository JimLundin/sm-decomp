//! The importers: the only code that knows the ROM's layout of the content it writes
//! (docs/architecture/assets.md §5). Each reads the player's ROM once and writes our own
//! formats, which `data` holds; `game` and `runtime` never depend on this crate.

pub mod cache;
pub mod projectile;
pub mod samus;
pub mod tileset;
pub mod world;
