//! Proposed owned flag data only; no gameplay/reset implementation.
#![forbid(unsafe_code)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GameplayFlags {
    pub(crate) events: [u8; 8],
    pub(crate) bosses_by_area: [u8; 8],
    pub(crate) used_room_objects: [u8; 64],
    pub(crate) collected_locations: [u8; 64],
    pub(crate) opened_doors: [u8; 64],
}
// Required by Session::default for import construction. This is NOT the behavior
// of NewSaveFile or demo initialization; neither may be replaced with Default.
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
pub(crate) struct FlagCompatibility {
    // LE scalar at 05E7. Boss helpers replace only its low byte.
    pub(crate) bitmask: u16,
}
