//! The importers: the only code that knows the ROM's layout of the content it writes
//! (docs/architecture/assets.md §5). Each reads the player's ROM once and writes our own
//! formats, which `data` holds; `game` and `runtime` never depend on this crate.

pub mod actors;
pub mod audio;
pub mod cache;
pub mod countdown;
pub mod discovery;
pub mod flashback;
pub mod flight;
pub mod intro;
pub mod landing_site;
pub mod playback;
pub mod projectile;
pub mod ridley_flyby;
pub mod ridley_warning;
pub mod samus;
pub mod shaft;
pub mod start;
pub mod tileset;
pub mod world;

pub mod explosion;
pub mod zebes;
