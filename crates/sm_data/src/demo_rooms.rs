//! Attract-mode demo rooms in bank $82.
//!
//! The set table at $82:876C holds each set's first scene address. Each set
//! is a run of 18-byte scenes ended by a $FFFF word; the four sets follow the
//! table back to back, up to $82:891A.

/// LoROM address of the set table.
pub const SETS_ADDRESS: u32 = 0x82_876c;
pub const SETS: usize = 4;
pub const SETS_BYTES: usize = 2 * SETS;
/// LoROM address of the first scene of the first set.
pub const SCENES_ADDRESS: u32 = 0x82_8774;
/// Bytes from $82:8774 up to $82:891A, past the last set's end marker.
pub const SCENES_BYTES: usize = 0x1a6;
const SCENE: usize = 18;
const END: u16 = 0xffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    Length(usize),
    /// A scene runs past the block before its set's end marker.
    Truncated {
        offset: usize,
    },
}

/// Each set's first scene, as a bank $82 address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoSets {
    starts: [u16; SETS],
}

impl DemoSets {
    pub fn import(bytes: &[u8]) -> Result<Self, ImportError> {
        if bytes.len() != SETS_BYTES {
            return Err(ImportError::Length(bytes.len()));
        }
        Ok(Self {
            starts: core::array::from_fn(|set| {
                u16::from_le_bytes([bytes[2 * set], bytes[2 * set + 1]])
            }),
        })
    }

    pub fn start(&self, set: usize) -> Option<u16> {
        self.starts.get(set).copied()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoScene {
    /// Bank $8F room header.
    pub room: u16,
    /// Bank $83 door the room is entered through.
    pub door: u16,
    pub door_slot: u16,
    pub layer1_x: u16,
    pub layer1_y: u16,
    /// Added to layer 1 Y for Samus's Y position. The original descriptor
    /// lists this word as the X offset.
    pub samus_y_offset: u16,
    /// Added to layer 1 X, plus 128, for Samus's X position.
    pub samus_x_offset: u16,
    /// Frames of demo input before the scene ends.
    pub timer: u16,
    /// Bank $82 setup routine run once the room is loaded.
    pub hook: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoEntry {
    Scene(DemoScene),
    /// The $FFFF word that ends a set.
    End,
}

/// Every scene and end marker in the block, keyed by bank $82 address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoScenes {
    entries: Vec<(u16, DemoEntry)>,
}

impl DemoScenes {
    /// Reads the block as consecutive scenes and end markers that end exactly
    /// at its last byte.
    pub fn import(bytes: &[u8]) -> Result<Self, ImportError> {
        if bytes.len() != SCENES_BYTES {
            return Err(ImportError::Length(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let mut entries = Vec::new();
        let mut offset = 0;
        while offset < SCENES_BYTES {
            let address = (SCENES_ADDRESS as u16).wrapping_add(offset as u16);
            if word(offset) == END {
                entries.push((address, DemoEntry::End));
                offset += 2;
                continue;
            }
            if offset + SCENE > SCENES_BYTES {
                return Err(ImportError::Truncated { offset });
            }
            let at = |index: usize| word(offset + 2 * index);
            entries.push((
                address,
                DemoEntry::Scene(DemoScene {
                    room: at(0),
                    door: at(1),
                    door_slot: at(2),
                    layer1_x: at(3),
                    layer1_y: at(4),
                    samus_y_offset: at(5),
                    samus_x_offset: at(6),
                    timer: at(7),
                    hook: at(8),
                }),
            ));
            offset += SCENE;
        }
        Ok(Self { entries })
    }

    /// The scene or end marker starting at a bank $82 address, or `None` for
    /// addresses that do not start one.
    pub fn entry(&self, address: u16) -> Option<DemoEntry> {
        self.entries
            .iter()
            .find(|(at, _)| *at == address)
            .map(|(_, entry)| *entry)
    }
}
