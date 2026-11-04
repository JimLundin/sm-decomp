//! Proposed owned flag data only; no gameplay/reset implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct GameplayFlags {
    pub events: [u8; 8],
    pub bosses_by_area: [u8; 8],
    pub used_room_objects: [u8; 64],
    pub collected_locations: [u8; 64],
    pub opened_doors: [u8; 64],
    /// The unused bytes between the doors and the stations ($7E:D8F0..$7E:D8F7):
    /// new games clear them, the demos set them, the save file carries them,
    /// and the door rules read the first past the last door byte.
    pub unused: [u8; 8],
    pub used_stations: [u8; 16],
    pub map_stations: [u8; 8],
    /// The spare bytes after the map stations ($7E:D910..$7E:D913), which
    /// only the save file carries.
    pub spare: [u8; 4],
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
            unused: [0; 8],
            used_stations: [0; 16],
            map_stations: [0; 8],
            spare: [0; 4],
        }
    }
}

impl GameplayFlags {
    /// The bytes the flags take, from the events ($7E:D820..$7E:D913).
    pub const BYTES: usize = 0xf4;

    /// Byte `offset` of the flags, from the events', in RAM order.
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.parts()
            .into_iter()
            .try_fold(offset, |offset, part| match part.get(offset) {
                Some(&byte) => Err(byte),
                None => Ok(offset - part.len()),
            })
            .err()
    }

    /// Little-endian word `index` of the flags, from the events'.
    pub fn word(&self, index: usize) -> Option<u16> {
        Some(u16::from_le_bytes([
            self.byte(2 * index)?,
            self.byte(2 * index + 1)?,
        ]))
    }

    /// The flags' byte arrays, in RAM order.
    fn parts(&self) -> [&[u8]; 9] {
        [
            &self.events,
            &self.bosses_by_area,
            &self.used_room_objects,
            &self.collected_locations,
            &self.opened_doors,
            &self.unused,
            &self.used_stations,
            &self.map_stations,
            &self.spare,
        ]
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlagCompatibility {
    // LE scalar at 05E7. Boss helpers replace only its low byte.
    pub bitmask: u16,
}
