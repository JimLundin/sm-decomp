//! The game's content in a format of our own, named by what things are (docs/architecture/assets.md).
//!
//! Each asset's importer, which reads the ROM's layout once, sits beside its records.

pub mod actors;
pub mod art;
pub mod audio;
pub mod countdown;
pub mod discovery;
pub mod fingerprint;
pub mod flashback;
pub mod flight;
pub mod intro;
pub mod landing_site;
pub mod playback;
pub mod projectile;
pub mod ridley_flyby;
pub mod ridley_warning;
pub mod samus;
pub mod start;
pub mod tilesets;
pub mod world;

pub mod explosion;
pub mod zebes;
