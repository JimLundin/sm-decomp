//! Population creation and multipart search state, independent of RAM and effects.
#![forbid(unsafe_code)]

pub(crate) struct PopulationEntry {
    pub definition: u16,
    pub x: u16,
    pub y: u16,
    pub instruction: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub parameters: [u16; 2],
}

pub(crate) struct EnemyShape {
    pub width: u16,
    pub height: u16,
    pub health: u16,
    pub layer: u8,
    pub bank: u8,
    pub hurt_time: u8,
}

#[derive(Clone, Copy)]
pub(crate) enum Creation {
    Room,
    Dynamic,
}

impl Creation {
    pub fn clear_script_variables(self) -> bool {
        matches!(self, Self::Dynamic)
    }
    pub fn run_initializer(self, address: u16) -> bool {
        matches!(self, Self::Room) || address & 0x8000 != 0
    }
    pub fn initial_spritemap(
        self,
        properties: u16,
        extra: u16,
        empty: u16,
        extended: u16,
    ) -> Option<u16> {
        if properties & 0x2000 != 0 {
            Some(if matches!(self, Self::Room) && extra & 4 != 0 {
                extended
            } else {
                empty
            })
        } else if matches!(self, Self::Room) {
            Some(0)
        } else {
            None
        }
    }
}

pub(crate) enum SearchResult {
    Continue,
    Found(u16),
    Full,
}

/// This is the original search state, including its accumulated empty probes
/// across failed candidate windows. It is deliberately not a contiguous allocator.
pub(crate) struct SpawnSearch {
    pub probe: u16,
    candidate: u16,
    remaining: u16,
}

impl SpawnSearch {
    pub fn new(parts: u16) -> Self {
        Self {
            probe: 0,
            candidate: 0,
            remaining: Self::remaining_parts(parts),
        }
    }
    pub fn remaining_parts(parts: u16) -> u16 {
        let remaining = (parts as i16).wrapping_sub(1);
        if remaining < 0 { 0 } else { remaining as u16 }
    }
    pub fn observe(&mut self, occupied: bool) -> SearchResult {
        if occupied {
            self.candidate = self.candidate.wrapping_add(64);
            self.probe = self.candidate;
        } else if self.remaining == 0 {
            return SearchResult::Found(self.candidate);
        } else {
            self.remaining = self.remaining.wrapping_sub(1);
            self.probe = self.probe.wrapping_add(64);
        }
        if self.probe >= 2048 {
            SearchResult::Full
        } else {
            SearchResult::Continue
        }
    }
}

pub(crate) struct SpawnSequence {
    pub slot: u16,
    pub population: u16,
    remaining: u16,
}

impl SpawnSequence {
    pub fn new(slot: u16, population: u16, parts: u16) -> Self {
        Self {
            slot,
            population,
            remaining: SpawnSearch::remaining_parts(parts),
        }
    }
    // Creation returns the last initialized slot and uses a different stopping
    // rule from allocation. Keep that one-fewer-part behavior explicit.
    pub fn advance(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining = self.remaining.wrapping_sub(1);
        if self.remaining == 0 {
            return false;
        }
        self.slot = self.slot.wrapping_add(64);
        self.population = self.population.wrapping_add(16);
        true
    }
}
