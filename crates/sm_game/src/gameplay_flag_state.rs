//! Proposed owned flag data only; no gameplay/reset implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct GameplayFlags {
    pub events: [u8; 8],
    pub bosses_by_area: [u8; 8],
    pub used_room_objects: [u8; 64],
    pub collected_locations: [u8; 64],
    pub opened_doors: [u8; 64],
}
// Required by Session::default for import construction. This is NOT the behavior
// of save_slots::initialize or demo initialization; neither may be replaced with Default.
impl Default for GameplayFlags {
    fn default() -> Self {
        Self {
            events: [0; 8],
            bosses_by_area: [0; 8],
            used_room_objects: [0; 64],
            collected_locations: [0; 64],
            opened_doors: [0; 64],
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlagCompatibility {
    // LE scalar at 05E7. Boss helpers replace only its low byte.
    pub bitmask: u16,
}
