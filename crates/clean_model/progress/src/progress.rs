//! The whole of what the game keeps of a run besides where she is.

use crate::inventory::Inventory;
use crate::world::WorldMemory;

/// What the game remembers of her run: what she has, and what has happened in the world. It
/// carries from room to room and into the saved file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    /// What she has.
    pub inventory: Inventory,
    /// What has happened in the world, and what its places remember.
    pub world: WorldMemory,
}

impl Progress {
    /// A new file: 99 energy of a capacity of 99 and nothing else, and a world where nothing has
    /// happened yet.
    #[must_use]
    pub fn new_file() -> Self {
        Self {
            inventory: Inventory::new_file(),
            world: WorldMemory::default(),
        }
    }
}
