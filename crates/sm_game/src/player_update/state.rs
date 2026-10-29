//! Current area's exploration and the fifteen minimap HUD cells.
#![forbid(unsafe_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub explored: [u8; 256],
    pub tiles: [[u16; 5]; 3],
    pub disabled: u16,
}
impl Default for State {
    fn default() -> Self {
        Self {
            explored: [0; 256],
            tiles: [[0; 5]; 3],
            disabled: 0,
        }
    }
}
