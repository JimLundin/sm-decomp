//! Logical minimap sources; native locations belong only to the provider.
use crate::player_collision::effects::Content as CollisionContent;

pub trait Content: CollisionContent {
    /// An opaque selected coordinate sequence, not a mutable state address.
    type BossCoordinates: Copy;
    /// Capture the selected area exploration mask. Byte offsets are positions
    /// in that captured mask, including the compatibility exceptional tail.
    fn area_exploration_mask(&mut self, area: u16) -> Self::Record;
    /// Capture the selected map tile source, preserving lazy native word reads.
    fn area_map_tiles(&mut self, area: u16) -> Self::Record;
    /// Read only the identifier: unused coordinate pointers must remain unread.
    fn boss_map_id(&self, index: usize) -> u16;
    fn boss_coordinates(&self, index: usize) -> Self::BossCoordinates;
    /// Capture anew for each coordinate pair, before sentinel/x/y reads.
    fn boss_coordinate(&mut self, coordinates: Self::BossCoordinates, index: usize)
    -> Self::Record;
}
