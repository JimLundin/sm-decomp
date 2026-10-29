//! Map discoveries: one explored bit per map cell in the area's 256 owned
//! bytes; positions outside them reach the host's legacy aliases.
use crate::player_update::state::State;

/// The Session state the exploration rules read and write.
pub trait ExplorationHost {
    fn minimap(&self) -> &State;
    fn minimap_mut(&mut self) -> &mut State;
    /// Exceptional positions relative to the exploration window. A preceding
    /// map-row write may use a negative displacement; never wrap or clamp it.
    fn imported_exploration_byte(&self, offset: isize) -> u8;
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8);
    /// The current room's map cell origin (map_x, map_y).
    fn map_origin(&self) -> (u16, u16);
}

pub struct MapWindow {
    pub half: u16,
    pub column: u16,
    pub y: u16,
    pub bit: usize,
    pub byte_index: u16,
}

pub fn explored_byte(h: &(impl ExplorationHost + ?Sized), offset: isize) -> u8 {
    if (0..256).contains(&offset) {
        h.minimap().explored[offset as usize]
    } else {
        h.imported_exploration_byte(offset)
    }
}
pub fn mark_byte(h: &mut (impl ExplorationHost + ?Sized), offset: isize, mask: u8) {
    let value = explored_byte(h, offset) | mask;
    if (0..256).contains(&offset) {
        h.minimap_mut().explored[offset as usize] = value;
    } else {
        h.set_imported_exploration_byte(offset, value);
    }
}
/// Mark the cell under room position (`x`, `y`) explored and return its window.
pub fn mark_explored(h: &mut (impl ExplorationHost + ?Sized), x: u16, y: u16) -> MapWindow {
    let (map_x, map_y) = h.map_origin();
    let x = map_x.wrapping_add(x >> 8);
    let y = map_y.wrapping_add(y >> 8).wrapping_add(1);
    let half = x & 0x20;
    let column = x & 0x1f;
    let bit = usize::from(x & 7);
    let byte_index = (column >> 3).wrapping_add(4u16.wrapping_mul(half.wrapping_add(y)));
    mark_byte(h, byte_index as isize, 0x80 >> bit);
    MapWindow {
        half,
        column,
        y,
        bit,
        byte_index,
    }
}
