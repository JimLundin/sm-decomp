//! Compact copies of the thirty input words and 256-byte scroll allocation.
//! Constants index private words; INPUT_OFFSETS maps them to canonical WRAM.
pub(crate) const WIDTH: usize = 0;
pub(crate) const HEIGHT: usize = 1;
pub(crate) const UP_SCROLLER: usize = 2;
pub(crate) const DOWN_SCROLLER: usize = 3;
pub(crate) const FINISHED_HOOK: usize = 4;
pub(crate) const LAYER_X_FRACTION: usize = 5;
pub(crate) const LAYER_X: usize = 6;
pub(crate) const LAYER_Y_FRACTION: usize = 7;
pub(crate) const LAYER_Y: usize = 8;
pub(crate) const CAMERA_DISTANCE_INDEX: usize = 9;
pub(crate) const POSE_AND_MOVEMENT: usize = 10;
pub(crate) const KNOCKBACK_DIRECTION: usize = 11;
pub(crate) const FROZEN: usize = 12;
pub(crate) const X: usize = 13;
pub(crate) const X_FRACTION: usize = 14;
pub(crate) const Y: usize = 15;
pub(crate) const Y_FRACTION: usize = 16;
pub(crate) const IDEAL_X: usize = 17;
pub(crate) const IDEAL_Y: usize = 18;
pub(crate) const PREVIOUS_X: usize = 19;
pub(crate) const PREVIOUS_X_FRACTION: usize = 20;
pub(crate) const PREVIOUS_Y: usize = 21;
pub(crate) const PREVIOUS_Y_FRACTION: usize = 22;
pub(crate) const Y_DIRECTION: usize = 23;
pub(crate) const X_ACCELERATION_MODE: usize = 24;
pub(crate) const SLOW_GRAPPLE: usize = 25;
pub(crate) const DISTANCE_X: usize = 26;
pub(crate) const DISTANCE_X_FRACTION: usize = 27;
pub(crate) const DISTANCE_Y: usize = 28;
pub(crate) const DISTANCE_Y_FRACTION: usize = 29;

const INPUT_OFFSETS: [usize; 30] = [
    0x07a9, 0x07ab, 0x07ad, 0x07af, 0x07e9, 0x090f, 0x0911, 0x0913, 0x0915, 0x0941, 0x0a1e, 0x0a52, 0x0a78, 0x0af6,
    0x0af8, 0x0afa, 0x0afc, 0x0b0a, 0x0b0e, 0x0b10, 0x0b12, 0x0b14, 0x0b16, 0x0b36, 0x0b4a, 0x0cf8, 0x0da2, 0x0da4,
    0x0da6, 0x0da8,
];
const OUTPUT_FIELDS: [usize; 14] = [
    LAYER_X_FRACTION,
    LAYER_X,
    LAYER_Y_FRACTION,
    LAYER_Y,
    IDEAL_X,
    IDEAL_Y,
    PREVIOUS_X,
    PREVIOUS_X_FRACTION,
    PREVIOUS_Y,
    PREVIOUS_Y_FRACTION,
    DISTANCE_X,
    DISTANCE_X_FRACTION,
    DISTANCE_Y,
    DISTANCE_Y_FRACTION,
];

pub(crate) struct State {
    words: [u16; 30],
    scrolls: [u8; 256],
}

impl State {
    pub fn read(ram: &[u8]) -> Self {
        let words = INPUT_OFFSETS.map(|at| u16::from_le_bytes([ram[at], ram[at + 1]]));
        let mut scrolls = [0; 256];
        scrolls.copy_from_slice(&ram[0xcd20..0xce20]);
        Self { words, scrolls }
    }

    pub fn admitted(&self) -> bool {
        let width = self.word(WIDTH);
        let height = self.word(HEIGHT);
        matches!(self.word(FINISHED_HOOK), 0 | 0x9589)
            && (1..=255).contains(&width)
            && (1..=255).contains(&height)
            && u32::from(width) * u32::from(height) <= 256
    }

    pub fn word(&self, field: usize) -> u16 {
        self.words[field]
    }

    pub fn set_word(&mut self, field: usize, value: u16) {
        self.words[field] = value;
    }

    pub fn scroll(&self, index: u16) -> Result<u8, &'static str> {
        self.scrolls.get(usize::from(index)).copied().ok_or("camera scroll lookup outside 256-byte allocation")
    }

    pub fn commit(self, ram: &mut [u8]) {
        for field in OUTPUT_FIELDS {
            let at = INPUT_OFFSETS[field];
            ram[at..at + 2].copy_from_slice(&self.word(field).to_le_bytes());
        }
    }
}
