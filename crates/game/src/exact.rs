//! The one encoder from intents to the original's bytes (MODEL §2), each encoding quirk a named
//! rule. It runs where the original wrote them, so the order of the batch is the original's.

pub mod art;
pub mod oam;
pub mod ppu;
pub mod ridley_flyby;
pub mod save_files;
pub mod tilemap;
