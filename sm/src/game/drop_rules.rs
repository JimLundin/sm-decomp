//! Drop patterns and weighted selection, independent of RAM and native effects.
#![forbid(unsafe_code)]

pub enum DropSource {
    MiniKraid,
    LowerNorfairPirate,
    Metroid,
    Ridley,
    Crocomire,
    Phantoon,
    Botwoon,
    Kraid,
    BombTorizo,
    GoldenTorizo,
    SporeSpawn,
    Draygon,
}

pub(crate) enum Spread {
    Nearby(u16),
    Area(u16, u16, u16),
}

impl DropSource {
    pub(crate) fn pattern(self) -> (u16, Spread) {
        match self {
            Self::MiniKraid => (0xe0ff, Spread::Nearby(4)),
            Self::LowerNorfairPirate => (0xf593, Spread::Nearby(5)),
            Self::Metroid => (0xdd7f, Spread::Nearby(5)),
            Self::Ridley => (0xe17f, Spread::Area(64, 320, 127)),
            Self::Crocomire => (0xddbf, Spread::Area(576, 96, 127)),
            Self::Phantoon => (0xe4bf, Spread::Area(64, 96, 127)),
            Self::Botwoon => (0xf293, Spread::Area(64, 128, 127)),
            Self::Kraid => (0xe2bf, Spread::Area(128, 352, 255)),
            Self::BombTorizo => (0xeeff, Spread::Area(64, 96, 127)),
            Self::GoldenTorizo => (0xeeff, Spread::Area(128, 288, 255)),
            Self::SporeSpawn => (0xdf3f, Spread::Area(64, 528, 127)),
            Self::Draygon => (0xde3f, Spread::Area(128, 352, 255)),
        }
    }
}

pub(crate) const PICKUPS: [u16; 6] = [1, 2, 4, 6, 5, 3];

pub(crate) struct Weights {
    total: u8,
    budget: u8,
    mask: u8,
}

impl Weights {
    pub(crate) fn new(bias: bool, needed: [bool; 4], mut read: impl FnMut(usize) -> u8) -> Self {
        let mut weights = Self {
            total: 0,
            budget: 255,
            mask: 0,
        };
        if bias {
            weights.total = read(1).wrapping_add(read(0));
            weights.mask = 3;
        } else {
            weights.total = read(3);
            weights.mask = 8;
            if needed[0] {
                weights.total = read(1).wrapping_add(read(0)).wrapping_add(weights.total);
                weights.mask |= 3;
            }
            if needed[1] {
                weights.total = read(2).wrapping_add(weights.total);
                weights.mask |= 4;
            }
            for index in 4..6 {
                if needed[index - 2] {
                    weights.budget = weights.budget.wrapping_sub(read(index));
                    weights.mask |= 1 << index;
                }
            }
        }
        weights
    }
    pub(crate) fn enabled(&self, index: usize) -> bool {
        (index >= 4 || self.total != 0) && self.mask & (1 << index) != 0
    }
    pub(crate) fn contribution(&self, index: usize, value: u8) -> u16 {
        if index < 4 {
            u16::from(self.budget) * u16::from(value) / u16::from(self.total)
        } else {
            u16::from(value)
        }
    }
}
