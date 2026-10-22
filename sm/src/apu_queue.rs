//! Frame commands shared by audio playback, recording and oracle rewind.
#![forbid(unsafe_code)]

#[derive(Copy, Clone, Default)]
pub struct ApuQueue {
    pub pending: [u8; 4],
    pub entries: [[u8; 4]; 16],
    pub size: u8,
    pub position: u8,
    pub time_since_empty: u8,
}

impl ApuQueue {
    pub fn to_bytes(self) -> [u8; 71] {
        let mut bytes = [0; 71];
        bytes[..4].copy_from_slice(&self.pending);
        for (entry, ports) in self.entries.iter().zip(bytes[4..68].chunks_exact_mut(4)) {
            ports.copy_from_slice(entry);
        }
        bytes[68..].copy_from_slice(&[self.size, self.position, self.time_since_empty]);
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 71]) -> Self {
        assert!(bytes[68] <= 16, "invalid saved APU queue size");
        let mut entries = [[0; 4]; 16];
        for (entry, ports) in entries.iter_mut().zip(bytes[4..68].chunks_exact(4)) {
            entry.copy_from_slice(ports);
        }
        Self {
            pending: bytes[..4].try_into().unwrap(),
            entries,
            size: bytes[68],
            position: bytes[69],
            time_since_empty: bytes[70],
        }
    }

    pub fn first_difference(&self, other: &Self) -> Option<&'static str> {
        let Self {
            pending,
            entries,
            size,
            position,
            time_since_empty,
        } = self;
        if *pending != other.pending {
            return Some("pending.ports");
        }
        for (ports, other) in entries.iter().zip(&other.entries) {
            if ports != other {
                return Some("entries.ports");
            }
        }
        if *size != other.size {
            return Some("size");
        }
        if *position != other.position {
            return Some("position");
        }
        if *time_since_empty != other.time_since_empty {
            return Some("time_since_empty");
        }
        None
    }

    pub fn write(&mut self, port: usize, value: u8) {
        self.pending[port] = value;
    }

    pub fn reset(&mut self) {
        self.size = 0;
        self.time_since_empty = 0;
        self.position = 0;
        self.pending = [0xff; 4];
    }

    pub fn push(&mut self, uploading: bool) {
        if uploading {
            self.size = 0;
            return;
        }
        if self.size == 0 {
            self.time_since_empty = 0;
        } else {
            // Suppress an empty frame periodically when audio falls behind.
            if self.time_since_empty >= 32 && self.pending == [0xff; 4] {
                self.time_since_empty -= 4;
                return;
            }
            self.time_since_empty = self.time_since_empty.wrapping_add(1);
        }
        let old = usize::from(self.position & 15);
        self.position = self.position.wrapping_add(1);
        if self.size == 16 {
            // Carry overwritten commands into the next entry's empty ports.
            let next = usize::from(self.position & 15);
            for port in 0..4 {
                if self.entries[next][port] == 0xff {
                    self.entries[next][port] = self.entries[old][port];
                }
            }
        } else {
            self.size = self.size.wrapping_add(1);
        }
        self.entries[old] = self.pending;
        self.pending = [0xff; 4];
    }

    pub fn pop(&mut self) -> Option<[u8; 4]> {
        if self.size == 0 {
            return None;
        }
        let index = self.position.wrapping_sub(self.size) & 15;
        self.size -= 1;
        Some(self.entries[usize::from(index)])
    }

    pub fn apply_ports(writes: [u8; 4], ports: &mut [u8]) {
        for (port, value) in writes.into_iter().enumerate() {
            if value != 0xff {
                ports[port] = value;
            }
        }
    }

    pub fn apply_queued(&self, ports: &mut [u8]) {
        for remaining in (1..=self.size).rev() {
            let index = self.position.wrapping_sub(remaining) & 15;
            Self::apply_ports(self.entries[usize::from(index)], ports);
        }
    }
}
