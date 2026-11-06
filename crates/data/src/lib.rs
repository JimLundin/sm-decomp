//! The game's content and the records shared across the engine: IDs and stable keys, the
//! ROM decoder and its tables, typed content, world records, profiles, the portable records
//! that `game` emits and `runtime` consumes, the address map and the shared digest.
//!
//! It holds no mutable state and no rules (ARCHITECTURE §4.1).

pub mod digest;
pub mod id;
